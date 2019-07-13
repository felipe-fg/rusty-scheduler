use failure::{Context, Fail};

#[derive(Debug, Display)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Invalid pipeline folder: {}", _0)]
    InvalidPipelineFolder(String),
    #[fail(display = "Invalid pipeline file: {}", _0)]
    InvalidPipelineFile(String),

    #[fail(display = "Invalid state file: {}", _0)]
    InvalidStateFile(String),

    #[fail(display = "Error executing pipeline: {}", _0)]
    PipelineExecutionFailed(String),

    #[fail(display = "Error executing stage: {}", _0)]
    StageExecutionFailed(String),

    #[fail(display = "Error starting job: {}", _0)]
    JobStartFailed(String),
    #[fail(display = "Error waiting job: {}", _0)]
    JobWaitFailed(String),
    #[fail(display = "Error executing job: {}\nError:\n{}", _0, _1)]
    JobExecutionFailed(String, String),

    #[fail(display = "Invalid interval expression: {}", _0)]
    InvalidIntervalExpression(String),
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}
