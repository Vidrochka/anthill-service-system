use std::any::type_name;

use anthill_di::{DependencyContext, extensions::ConstructedDependencySetStrategy};
use async_trait::async_trait;

use crate::service::{
    ServiceCollection,
    HostedServiceConstructor,
    BackgroundService,
    BackgroundServiceWrapper,
    ServiceInfo
};

use super::HostedServiceBuilder;

#[async_trait]
pub trait AddBackgroundServiceStrategy {
    async fn add_background_service<T: BackgroundService + HostedServiceConstructor>(&self);
}

#[async_trait]
impl AddBackgroundServiceStrategy for DependencyContext {
    async fn add_background_service<T: BackgroundService + HostedServiceConstructor>(&self) {
        self.add_transient::<T>(Box::new(HostedServiceBuilder::new::<T>())).await.unwrap();
        self.set_transient::<BackgroundServiceWrapper<T>>().await.unwrap();

        let service = self.get_transient::<BackgroundServiceWrapper<T>>().await.unwrap();

        let service_collection = self.get_singleton::<ServiceCollection>().await.unwrap();

        let service_info = ServiceInfo::new::<BackgroundServiceWrapper<T>>(
            service,
            Some(type_name::<T>().to_string()),
            Some("Background service".to_string())
        );

        service_collection.write().await.add_custom_service(service_info);
    }
}