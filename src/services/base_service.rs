use anthill_di::types::TypeInfo;

#[async_trait_with_sync::async_trait(Sync)]
pub trait IBaseService where Self: Sync + Send + 'static {
    async fn on_start(&mut self);
    async fn on_stop(&mut self);

    fn get_type_info(&self) -> TypeInfo {
        TypeInfo::from_type::<Self>()
    }
}