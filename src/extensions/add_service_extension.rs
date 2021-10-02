use std::sync::Arc;

use anthill_di::{DiError, Injection, Injector, builders::ContainerBuilder};
use tokio::sync::RwLock;

use crate::service::Service;

#[async_trait::async_trait]
pub trait AddServiceExtension: Send {
    async fn add_service<TType>(&mut self) -> Result<(), DiError> where TType: Injection + Service + Sync + Send + 'static;
}

#[async_trait::async_trait]
impl AddServiceExtension for Injector {
    async fn add_service<TType>(&mut self) -> Result<(), DiError> where TType: Injection + Service + Sync + Send + 'static {
        self.add_container(ContainerBuilder::bind_type::<TType>().build());
        
        let service_instance = self.get_singletone::<TType>()?;
        let service_collection = self.get_singletone::<Vec<Arc<RwLock<dyn Service + Sync + Send>>>>()?;
        service_collection.write().await.push(service_instance.clone() as Arc<RwLock<dyn Service + Sync + Send>>);
        Ok(())
    }
}