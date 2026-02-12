use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ConfigInvalid,
    ConfigIo,
    PlanCycle,
    PlanUnknownDependency,
    ExecutionFailed,
    Internal,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConfigInvalid => "CONFIG_INVALID",
            Self::ConfigIo => "CONFIG_IO",
            Self::PlanCycle => "PLAN_CYCLE",
            Self::PlanUnknownDependency => "PLAN_UNKNOWN_DEPENDENCY",
            Self::ExecutionFailed => "EXECUTION_FAILED",
            Self::Internal => "INTERNAL",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("configuration error [{code}]: {message}")]
    Config { code: ErrorCode, message: String },
    #[error("planning error [{code}]: {message}")]
    Plan { code: ErrorCode, message: String },
    #[error("execution error [{code}]: {message}")]
    Execution { code: ErrorCode, message: String },
    #[error("internal error [{code}]: {message}")]
    Internal { code: ErrorCode, message: String },
}

impl RunnerError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Config { code, .. }
            | Self::Plan { code, .. }
            | Self::Execution { code, .. }
            | Self::Internal { code, .. } => *code,
        }
    }
}
