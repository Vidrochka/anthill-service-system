use std::sync::Arc;

use anthill_di::{DependencyContext, types::TypeInfo};
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{configs::CoreConfig, Application};

#[async_trait]
pub trait IStartup: Sync + Send + 'static {
    async fn configure_application(&mut self, core_config: Arc<RwLock<CoreConfig>>, app: &mut Application);
    async fn configure_dependency(&mut self, root_ioc_context: &mut DependencyContext);

    fn get_type_info(&self) -> TypeInfo {
        TypeInfo::from_type::<Self>()
    }    
}