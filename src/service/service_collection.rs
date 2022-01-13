use anthill_di::{Constructor, DependencyContext, types::BuildDependencyResult};
use async_trait::async_trait;
use crate::service::{
    ServiceInfo,
    HostedService
};

pub struct ServiceCollection {
    pub (crate) services: Vec<ServiceInfo>
}

impl ServiceCollection {
    pub fn add_service<T: HostedService + 'static>(&mut self, service: T) {
        self.services.push(ServiceInfo::new::<T>(service, None, Some("Hosted service".to_string())));
    }

    pub fn add_custom_service(&mut self, service_info: ServiceInfo) {
        self.services.push(service_info)
    }
}

#[async_trait]
impl Constructor for ServiceCollection {
    async fn ctor(_ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self { services: Vec::new(), })
    }
}