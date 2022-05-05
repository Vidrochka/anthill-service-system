use anthill_di::{Constructor, DependencyContext, types::BuildDependencyResult};
use tokio::task::yield_now;
use super::ILifeTimeManager;
use std::sync::atomic::{AtomicBool, Ordering};
use anthill_di_derive::constructor;

#[derive(constructor)]
pub struct InnerStateLifeTimeManager {
    #[custom_resolve(value = "AtomicBool::new(true)")] is_running: AtomicBool,
}

// #[async_trait_with_sync::async_trait(Sync)]
// impl Constructor for InnerStateLifeTimeManager {
//     async fn ctor(_: DependencyContext) -> BuildDependencyResult<Self> {
//         let is_running = AtomicBool::new(true);
//         Ok(Self { is_running })
//     }
// }

#[async_trait_with_sync::async_trait(Sync)]
impl ILifeTimeManager for InnerStateLifeTimeManager {
    async fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst)
    }
    
    async fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    async fn wait_for_stop(&self) {
        while self.is_running.load(Ordering::SeqCst) {
            yield_now().await;
        }
    }
}