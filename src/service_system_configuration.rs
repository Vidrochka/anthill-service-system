use std::sync::Arc;

use anthill_di::Injector;
use tokio::sync::RwLock;

#[async_trait::async_trait]
pub trait ServiceSystemCofiguration: Sync + Send {
    async fn configure_injections(&mut self, injector: Arc<RwLock<Injector>>) -> Result<(), String>;
    async fn configure_services(&mut self, injector: Arc<RwLock<Injector>>) -> Result<(), String>;
}