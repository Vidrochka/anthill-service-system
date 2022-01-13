use std::sync::Arc;

use anthill_di::{DependencyContext, types::BuildDependencyResult};
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::ApplicationLifeTime;

#[async_trait]
pub trait HostedServiceConstructor where Self: Sync + Send + Sized + 'static {
    async fn ctor(application_life_time: Arc<RwLock<ApplicationLifeTime>>, ctx: DependencyContext) -> BuildDependencyResult<Self>;
}

#[async_trait]
pub trait HostedService where Self: Sync + Send + 'static {
    async fn on_start(&mut self);
    async fn on_stop(&mut self);
}