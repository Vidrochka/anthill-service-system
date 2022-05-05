use std::sync::Arc;

use anthill_di::{DependencyContext, types::TypeInfo};
use anthill_di_configuration_extension::{ConfigurationSnapshot, source::JsonFileConfiguration};
use tokio::sync::RwLock;
use crate::{configs::CoreConfig, Application};


#[async_trait_with_sync::async_trait(Sync)]
pub trait IStartup: Sync + Send + 'static {
    async fn configure_application(&mut self, core_config: Arc<RwLock<ConfigurationSnapshot<CoreConfig, JsonFileConfiguration::<CoreConfig>>>>, app: &mut Application);
    async fn configure_dependency(&mut self, root_ioc_context: &mut DependencyContext);

    fn get_type_info(&self) -> TypeInfo {
        TypeInfo::from_type::<Self>()
    }    
}