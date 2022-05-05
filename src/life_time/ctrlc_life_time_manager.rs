use anthill_di::{Constructor, DependencyContext, types::BuildDependencyResult};
use tokio::task::yield_now;
use super::ILifeTimeManager;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

pub struct CtrlCLifeTimeManager {
    is_running: Arc<AtomicBool>,
}

#[async_trait_with_sync::async_trait(Sync)]
impl Constructor for CtrlCLifeTimeManager {
    async fn ctor(_: DependencyContext) -> BuildDependencyResult<Self> {
        let is_running = Arc::new(AtomicBool::new(true));

        let is_running_clone = is_running.clone();
        ctrlc::set_handler(move || {
            is_running_clone.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");
        
        Ok(Self { is_running })
    }
}

#[async_trait_with_sync::async_trait(Sync)]
impl ILifeTimeManager for CtrlCLifeTimeManager {
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