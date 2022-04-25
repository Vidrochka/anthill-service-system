use async_trait::async_trait;

#[async_trait]
pub trait ILifeTimeManager : Sync + Send {
    async fn stop(&self);
    async fn is_running(&self) -> bool;
    async fn wait_for_stop(&self);
}