use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("execution error: {0}")]
    Execution(String),
}
