use std::time::Duration;

pub type AppRunResult<T> = Result<T, AppRunError>;

#[derive(Debug, PartialEq)]
pub enum AppRunError {
    ServiceStartTimeoutExpired { timeout_duration: Duration, service_name: String, payload: String, },
    ServiceStopTimeoutExpired { timeout_duration: Duration, service_name: String, payload: String, },
}