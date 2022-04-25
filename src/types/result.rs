use std::time::Duration;
use anthill_di::types::{AddDependencyError, BuildDependencyError, TypeInfo, MapComponentError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AppRunError {
    #[error("Service start timeout expired: [{timeout_duration:?}] [{service_type_info:?}]")]
    ServiceStartTimeoutExpired { timeout_duration: Duration, service_type_info: TypeInfo, },
    #[error("Service end timeout expired: [{timeout_duration:?}] [{service_type_info:?}]")]
    ServiceStopTimeoutExpired { timeout_duration: Duration, service_type_info: TypeInfo, },
    #[error("Ioc add dependency error: [{0:?}]")]
    IocAddDependencyError(AddDependencyError),
    #[error("Ioc build dependency error: [{0:?}]")]
    IocBuildDependencyError(BuildDependencyError),
    #[error("Ioc map dependency error: [{0:?}]")]
    IocMapComponentError(MapComponentError),
}

pub type AppRunResult = Result<(), AppRunError>;

#[derive(Error, Debug, PartialEq)]
pub enum AddServiceError {
    #[error("Ioc add dependency error: [{0:?}]")]
    IocAddDependencyError(AddDependencyError),
    #[error("Ioc build dependency error: [{0:?}]")]
    IocBuildDependencyError(BuildDependencyError),
    #[error("Ioc map dependency error: [{0:?}]")]
    IocMapComponentError(MapComponentError),
}

pub type AddServiceResult = Result<(), AddServiceError>;

#[derive(Error, Debug, PartialEq)]
pub enum AddStartupError {
    #[error("Ioc add dependency error: [{0:?}]")]
    IocAddDependencyError(AddDependencyError),
    // #[error("Ioc build dependency error: [{0:?}]")]
    // IocBuildDependencyError(BuildDependencyError),
    #[error("Ioc map dependency error: [{0:?}]")]
    IocMapComponentError(MapComponentError),
}

pub type AddStartupResult = Result<(), AddStartupError>;

#[derive(Error, Debug, PartialEq)]
pub enum AddLifeTimeManagerError {
    #[error("Ioc add dependency error: [{0:?}]")]
    IocAddDependencyError(AddDependencyError),
    // #[error("Ioc build dependency error: [{0:?}]")]
    // IocBuildDependencyError(BuildDependencyError),
    #[error("Ioc map dependency error: [{0:?}]")]
    IocMapComponentError(MapComponentError),
}

pub type AddLifeTimeManagerResult = Result<(), AddLifeTimeManagerError>;