use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Internal Error: {0}")]
    InternalError(String),
    #[error("Aborted!")]
    Abort,
    #[error("Aborted: {0}")]
    AbortMessage(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
