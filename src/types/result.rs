use std::time::Duration;
use anthill_di::types::{AddDependencyError, BuildDependencyError, TypeInfo, MapComponentError};
use thiserror::Error;
use anthill_di_configuration_extension::types::LoadConfigurationError;

#[derive(Error, Debug)]
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
    #[error("Load configuration error: [{0:?}]")]
    LoadConfigurationError(LoadConfigurationError),
}

impl PartialEq for AppRunError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ServiceStartTimeoutExpired { timeout_duration: l_timeout_duration, service_type_info: l_service_type_info }, Self::ServiceStartTimeoutExpired { timeout_duration: r_timeout_duration, service_type_info: r_service_type_info }) => l_timeout_duration == r_timeout_duration && l_service_type_info == r_service_type_info,
            (Self::ServiceStopTimeoutExpired { timeout_duration: l_timeout_duration, service_type_info: l_service_type_info }, Self::ServiceStopTimeoutExpired { timeout_duration: r_timeout_duration, service_type_info: r_service_type_info }) => l_timeout_duration == r_timeout_duration && l_service_type_info == r_service_type_info,
            (Self::IocAddDependencyError(l0), Self::IocAddDependencyError(r0)) => l0 == r0,
            (Self::IocBuildDependencyError(l0), Self::IocBuildDependencyError(r0)) => l0 == r0,
            (Self::IocMapComponentError(l0), Self::IocMapComponentError(r0)) => l0 == r0,
            (Self::LoadConfigurationError(..), Self::LoadConfigurationError(..)) => true,
            _ => false,
        }
    }
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

#[derive(Error, Debug)]
pub enum RegisterDefaultConfigurationError {
    #[error("Ioc add dependency error: [{0:?}]")]
    IocAddDependencyError(AddDependencyError),

    #[error("Io error: [{0:?}]")]
    IoError(std::io::Error),
}

pub type RegisterDefaultConfigurationResult = Result<(), RegisterDefaultConfigurationError>;