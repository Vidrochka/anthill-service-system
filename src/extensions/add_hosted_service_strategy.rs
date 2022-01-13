use std::any::Any;

use anthill_di::{
    DependencyContext,
    types::{
        AsyncCallback,
        BuildDependencyResult
    },
    TypeConstructor
};
use async_trait::async_trait;

use crate::{
    service::{
        HostedService,
        ServiceCollection,
        HostedServiceConstructor,
    },
    ApplicationLifeTime
};

#[async_trait]
pub trait AddHostedServiceStrategy {
    async fn add_hosted_service<T: HostedService + HostedServiceConstructor>(&self);
}

#[async_trait]
impl AddHostedServiceStrategy for DependencyContext {
    async fn add_hosted_service<T: HostedService + HostedServiceConstructor>(&self) {
        self.add_transient::<T>(Box::new(HostedServiceBuilder::new::<T>())).await.unwrap();
        let service = self.get_transient::<T>().await.unwrap();

        let service_collection = self.get_singleton::<ServiceCollection>().await.unwrap();
        service_collection.write().await.add_service::<T>(service);
    }
}

pub struct HostedServiceBuilder {
    async_ctor: AsyncCallback<DependencyContext, BuildDependencyResult<Box<dyn Any + Send + Sync>>>,
}

impl HostedServiceBuilder {
    pub fn new<T: HostedServiceConstructor>() -> Self {
        let ctor_wrapper: AsyncCallback<DependencyContext, BuildDependencyResult<Box<dyn Any + Send + Sync>>> = Box::new(
            move |ctx: DependencyContext| -> _{
                Box::pin(
                    async move {
                        let app_life_time = ctx.get_singleton::<ApplicationLifeTime>().await?;
                        let instance = T::ctor(app_life_time, ctx).await?;
                        Ok(Box::new(instance) as Box<dyn Any + Send + Sync>)
                    }
                )
            }
        );
        Self { async_ctor: ctor_wrapper }
    }
}

#[async_trait]
impl TypeConstructor for HostedServiceBuilder {
    async fn ctor(&self, ctx: DependencyContext) -> BuildDependencyResult<Box<dyn Any + Send + Sync>> {
        (self.async_ctor)(ctx).await
    }
}