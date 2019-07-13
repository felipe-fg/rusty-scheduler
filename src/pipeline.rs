use super::error::{Error, ErrorKind};
use super::interval::Interval;
use failure::ResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Pipeline {
    #[serde(default)]
    pub id: String,

    #[serde(default)]
    pub path: String,

    #[serde(default)]
    pub expression: String,

    #[serde(skip_deserializing)]
    #[serde(skip_serializing)]
    #[serde(default)]
    pub interval: Interval,

    #[serde(default)]
    pub stages: Vec<String>,

    #[serde(default)]
    pub jobs: Vec<Job>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Job {
    #[serde(default)]
    pub id: String,

    #[serde(skip_deserializing)]
    #[serde(skip_serializing)]
    #[serde(default)]
    pub breadcrumb: String,

    #[serde(default)]
    pub stage: String,

    #[serde(default)]
    pub script: String,

    #[serde(default)]
    pub path: String,
}

impl Pipeline {
    pub fn read_dir(pipelines_path: &str) -> Result<Vec<Result<Pipeline, Error>>, Error> {
        let mut pipelines = Vec::new();

        let dirs = fs::read_dir(pipelines_path)
            .context(ErrorKind::InvalidPipelineFolder(pipelines_path.to_string()))?;

        for entry in dirs {
            let mut entry = entry
                .context(ErrorKind::InvalidPipelineFolder(pipelines_path.to_string()))?
                .path();

            if entry.is_dir() {
                entry.push("pipeline.json");

                if entry.is_file() {
                    let entry = entry.to_string_lossy().to_string();

                    let pipeline = Pipeline::read_file(&entry);

                    pipelines.push(pipeline);
                }
            }
        }

        Ok(pipelines)
    }

    pub fn read_file(pipeline_path: &str) -> Result<Pipeline, Error> {
        let pipeline_data = fs::read_to_string(pipeline_path)
            .context(ErrorKind::InvalidPipelineFile(pipeline_path.to_string()))?;

        let mut pipeline: Pipeline = serde_json::from_str(&pipeline_data)
            .context(ErrorKind::InvalidPipelineFile(pipeline_path.to_string()))?;

        pipeline.path = pipeline_path.to_string();

        pipeline.interval = Interval::new(&pipeline.expression)
            .map_err(|_| ErrorKind::InvalidPipelineFile(pipeline_path.to_string()))?;

        for job in &mut pipeline.jobs {
            let mut script_file = PathBuf::from(pipeline_path);
            script_file.pop();
            script_file.push(&job.script);

            job.breadcrumb = format!("{}/{}/{}", &pipeline.id, &job.stage, &job.id);
            job.path = script_file.to_string_lossy().to_string();
        }

        Ok(pipeline)
    }
}
