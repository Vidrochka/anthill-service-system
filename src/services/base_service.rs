use anthill_di::types::TypeInfo;
use async_trait::async_trait;


#[async_trait]
pub trait IBaseService where Self: Sync + Send + 'static {
    async fn on_start(&mut self);
    async fn on_stop(&mut self);

    fn get_type_info(&self) -> TypeInfo {
        TypeInfo::from_type::<Self>()
    }
}