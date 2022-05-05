#[async_trait_with_sync::async_trait(Sync)]
pub trait ILifeTimeManager : Sync + Send {
    async fn stop(&self);
    async fn is_running(&self) -> bool;
    async fn wait_for_stop(&self);
}