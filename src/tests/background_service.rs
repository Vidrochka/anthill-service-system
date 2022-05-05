use std::sync::Arc;

use anthill_di::{
    DependencyContext,
};
use tokio::{
    sync::{
        RwLock,
        oneshot::{
            Sender,
            Receiver,
        }
    },
};

use crate::{services::IBackgroundService, life_time::ILifeTimeManager};

use anthill_di_derive::constructor;

#[derive(constructor)]
struct TestBackgroundService1 {
    #[ioc_context] ctx: DependencyContext,
}

#[async_trait_with_sync::async_trait(Sync)]
impl IBackgroundService for TestBackgroundService1 {
    async fn execute(&self) {
        let sender = self.ctx.resolve::<Arc<RwLock<Option<Sender<String>>>>>().await.unwrap()
            .write().await
            .take().unwrap();
        sender.send("test".to_string()).unwrap();
    }
}

#[derive(constructor)]
struct TestBackgroundService2 {
    application_life_time: Arc<dyn ILifeTimeManager>,
    #[ioc_context] ctx: DependencyContext,
}

#[async_trait_with_sync::async_trait(Sync)]
impl IBackgroundService for TestBackgroundService2 {
    async fn execute(&self) {
        let receiver = self.ctx.resolve::<Arc<RwLock<Option<Receiver<String>>>>>().await.unwrap().write().await.take().unwrap();
        assert_eq!("test".to_string(), receiver.await.unwrap());
        self.application_life_time.stop().await;
    }
}

#[tokio::test]
async fn background_service() {
    use tokio::sync::oneshot;
    use crate::{life_time::InnerStateLifeTimeManager, Application, services::BackgroundService};

    let configuration_path = "background_service.json".to_string();

    let mut app = Application::new(Some(configuration_path.clone())).await.unwrap();

    app.register_life_time_manager::<InnerStateLifeTimeManager>().await.unwrap();
    
    let (tx, rx) = oneshot::channel::<String>();
    app.root_ioc_context.register_instance(RwLock::new(Some(tx))).await.unwrap();
    app.root_ioc_context.register_instance(RwLock::new(Some(rx))).await.unwrap();

    app.register_service::<BackgroundService<TestBackgroundService1>>().await.unwrap();
    app.register_service::<BackgroundService<TestBackgroundService2>>().await.unwrap();
    
    app.run().await.unwrap();

    std::fs::remove_file(configuration_path).unwrap();
}