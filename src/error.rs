use std::time::Duration;

use anthill_di::DiError;


#[derive(Debug)]
pub enum ServiceSystemError {
    PrematureTermination,
    DiError{di_error: DiError},
    ServiceStartTimeoutExpire{timeout: Duration, service_name: String},
    ServiceEndTimeoutExpire{timeout: Duration, service_name: String},
    ConfigureServicesError{description: String},
    ConfigureInjectionsError{description: String},
}