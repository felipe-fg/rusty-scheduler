use super::error::Error;
use super::executor;
use super::pipeline::Pipeline;
use super::state::State;
use chrono::Utc;
use log::{error, info, trace};
use std::thread;
use std::time::Duration;

pub fn run(pipelines_path: &str, refresh_interval: Duration) {
    info!("Scheduler started");

    let mut ignore_active = true;

    loop {
        trace!("Reloading pipelines");

        let pipelines = Pipeline::read_dir(pipelines_path);

        let pipelines = unwrap_pipelines(pipelines);

        if pipelines.is_empty() {
            trace!("No pipeline loaded");
        } else {
            for pipeline in &pipelines {
                trace!("Pipeline loaded: {}", pipeline.id);
            }

            for pipeline in pipelines {
                run_pipeline(pipeline, ignore_active);
            }
        }

        ignore_active = false;

        thread::sleep(refresh_interval);
    }
}

pub fn run_pipeline(pipeline: Pipeline, ignore_active: bool) {
    let mut state = match import_state(&pipeline, ignore_active) {
        None => return,
        Some(state) => state,
    };

    thread::spawn(move || {
        trace!("Running pipeline: {}", pipeline.id);

        let timestamp = Utc::now();

        let status = executor::execute(&pipeline);

        match status {
            Ok(_) => {
                trace!("Pipeline completed: {}", pipeline.id);

                state.timestamp = timestamp;
            }
            Err(err) => {
                error!("{}", err);
            }
        }

        state.active = false;

        export_state(&state);
    });
}

pub fn import_state(pipeline: &Pipeline, ignore_active: bool) -> Option<State> {
    let mut state = State::read_from_pipeline(&pipeline);

    if !pipeline.interval.should_run(state.timestamp, Utc::now()) {
        return None;
    }

    if state.active && !ignore_active {
        trace!("Pipeline is already running: {}", pipeline.id);

        return None;
    }

    state.active = true;

    export_state(&state);

    Some(state)
}

pub fn export_state(state: &State) {
    match state.write_file() {
        Ok(_) => {
            trace!("State exported: {}", state.id);
        }
        Err(err) => {
            error!("{}", err);
        }
    };
}

pub fn unwrap_pipelines(pipelines: Result<Vec<Result<Pipeline, Error>>, Error>) -> Vec<Pipeline> {
    match pipelines {
        Err(err) => {
            error!("{}", err);

            Vec::new()
        }
        Ok(pipelines) => {
            pipelines
                .iter()
                .filter_map(|pipeline| pipeline.as_ref().err())
                .for_each(|err| error!("{}", err));

            pipelines
                .into_iter()
                .filter_map(|pipeline| pipeline.ok())
                .collect()
        }
    }
}
