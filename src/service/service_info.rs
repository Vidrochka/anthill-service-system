use std::{any::type_name, sync::Arc};

use tokio::sync::RwLock;

use crate::service::HostedService;

pub struct ServiceInfo {
    pub service: Arc<RwLock<Box<dyn HostedService>>>,
    pub (crate) name: String,
    pub (crate) payload: String,
}

impl ServiceInfo {
    pub fn new<T: HostedService + Send + 'static>(service: T, name: Option<String>, payload: Option<String>) -> Self {
        Self {
            service: Arc::new(RwLock::new(Box::new(service))),
            name: name.unwrap_or(type_name::<T>().to_string()),
            payload: payload.unwrap_or("".to_string()),
        }
    }
}