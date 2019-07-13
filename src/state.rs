use super::error::{Error, ErrorKind};
use super::pipeline::Pipeline;
use chrono::{DateTime, TimeZone, Utc};
use failure::ResultExt;
use log::{trace, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct State {
    #[serde(default)]
    pub id: String,

    #[serde(skip_serializing)]
    #[serde(default)]
    pub path: String,

    #[serde(default)]
    pub active: bool,

    #[serde(default = Utc::now())]
    pub timestamp: DateTime<Utc>,
}

impl State {
    pub fn read_from_pipeline(pipeline: &Pipeline) -> State {
        let mut state_path = PathBuf::from(&pipeline.path);
        state_path.pop();
        state_path.push("state.json");

        let state_path = state_path.to_string_lossy().to_string();

        let state = State::read_file(&state_path);

        match state {
            Ok(state) => {
                trace!("State loaded: {}", pipeline.id);

                state
            }
            Err(err) => {
                warn!("{}", err);
                warn!("State created: {}", pipeline.id);

                State {
                    id: pipeline.id.to_string(),
                    path: state_path.to_string(),
                    active: false,
                    timestamp: Utc.timestamp(0, 0),
                }
            }
        }
    }

    pub fn read_file(state_path: &str) -> Result<State, Error> {
        let state_data = fs::read_to_string(state_path)
            .context(ErrorKind::InvalidStateFile(state_path.to_string()))?;

        let mut state: State = serde_json::from_str(&state_data)
            .context(ErrorKind::InvalidStateFile(state_path.to_string()))?;

        state.path = state_path.to_string();

        Ok(state)
    }

    pub fn write_file(&self) -> Result<(), Error> {
        let state_data = serde_json::to_string_pretty(&self)
            .context(ErrorKind::InvalidStateFile(self.path.to_string()))?;

        fs::write(self.path.to_string(), state_data)
            .context(ErrorKind::InvalidStateFile(self.path.to_string()))?;

        Ok(())
    }
}
