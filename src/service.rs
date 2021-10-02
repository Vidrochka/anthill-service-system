use std::{any::{TypeId, type_name}, sync::Arc};

use anthill_di::Injector;
use tokio::sync::RwLock;


#[async_trait::async_trait]
pub trait Service : Sync + Send {
    async fn on_start(&mut self, injector: Arc<RwLock<Injector>>);
    async fn on_end(&mut self, injector: Arc<RwLock<Injector>>);
    
    fn get_service_name(&self) -> String {
        type_name::<Self>().to_string()
    }

    fn get_service_type_id(&self) -> TypeId where Self: 'static {
        TypeId::of::<Self>()
    }
}