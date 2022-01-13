use std::sync::Arc;

use anthill_di::DependencyContext;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::configs::CoreConfig;

#[async_trait]
pub trait Startup: Sync + Send {
    async fn configure_system(&mut self, dependency_context: &mut DependencyContext, core_config: Arc<RwLock<CoreConfig>>);
    async fn configure_dependency(&mut self, dependency_context: &mut DependencyContext);
}