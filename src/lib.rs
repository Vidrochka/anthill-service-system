#![feature(type_name_of_val)]
#![feature(unsize)]

mod service_system_configuration;
pub use service_system_configuration::*;

mod service_system_manager;
pub use service_system_manager::*;

pub mod service;
pub use service::*;

pub mod utils;
pub mod extensions;
pub mod error;

pub mod tests;