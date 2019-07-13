use super::error::{Error, ErrorKind};
use super::pipeline::{Job, Pipeline};
use failure::ResultExt;
use log::{error, trace};
use std::process::{Child, Command, Stdio};
use std::str;

pub struct JobProcess<'a>(&'a Job, Child);

pub fn execute(pipeline: &Pipeline) -> Result<&Pipeline, Error> {
    for stage in &pipeline.stages {
        trace!("Running stage: {}/{}", pipeline.id, stage);

        let status = execute_stage(&pipeline, &stage);

        match status {
            Ok(_) => {
                trace!("Stage completed: {}/{}", pipeline.id, stage);
            }
            Err(err) => {
                error!("{}", err);

                return Err(ErrorKind::PipelineExecutionFailed(pipeline.id.to_string()))?;
            }
        }
    }

    Ok(pipeline)
}

pub fn execute_stage(pipeline: &Pipeline, stage: &str) -> Result<String, Error> {
    let jobs: Vec<&Job> = pipeline
        .jobs
        .iter()
        .filter(|job| job.stage == stage)
        .collect();

    let jobs_count = jobs.len();

    let started = start_jobs(jobs);

    let completed = wait_jobs(started);

    let successful_count = completed
        .into_iter()
        .filter_map(|process| process.ok())
        .count();

    if successful_count == jobs_count {
        Ok(stage.to_string())
    } else {
        Err(ErrorKind::StageExecutionFailed(stage.to_string()))?
    }
}

pub fn start_jobs(jobs: Vec<&Job>) -> Vec<Result<JobProcess, Error>> {
    let started_jobs: Vec<Result<JobProcess, Error>> =
        jobs.iter().map(|job| start_job(job)).collect();

    started_jobs
        .iter()
        .filter_map(|process| process.as_ref().err())
        .for_each(|err| error!("{}", err));

    started_jobs
        .iter()
        .filter_map(|process| process.as_ref().ok())
        .for_each(|JobProcess(job, _)| trace!("Running job: {}", job.breadcrumb));

    started_jobs
}

pub fn wait_jobs(jobs: Vec<Result<JobProcess, Error>>) -> Vec<Result<&Job, Error>> {
    let completed_jobs: Vec<Result<&Job, Error>> = jobs
        .into_iter()
        .filter_map(|process| process.ok())
        .map(|process| wait_job(process))
        .collect();

    completed_jobs
        .iter()
        .filter_map(|process| process.as_ref().err())
        .for_each(|err| error!("{}", err));

    completed_jobs
        .iter()
        .filter_map(|process| process.as_ref().ok())
        .for_each(|job| trace!("Job completed: {}", job.breadcrumb));

    completed_jobs
}

pub fn start_job(job: &Job) -> Result<JobProcess, Error> {
    let child = Command::new("sh")
        .arg(&job.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context(ErrorKind::JobStartFailed(job.breadcrumb.to_string()))?;

    Ok(JobProcess(job, child))
}

pub fn wait_job(process: JobProcess) -> Result<&Job, Error> {
    let JobProcess(job, child) = process;

    let output = child
        .wait_with_output()
        .context(ErrorKind::JobWaitFailed(job.breadcrumb.to_string()))?;

    if output.status.success() {
        Ok(job)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);

        Err(ErrorKind::JobExecutionFailed(
            job.breadcrumb.to_string(),
            stderr.to_string(),
        ))?
    }
}
