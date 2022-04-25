use std::sync::Arc;

use anthill_di::{
    DependencyContext,
    Constructor,
    types::BuildDependencyResult,
};
use async_trait::async_trait;
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

struct TestBackgroundService1 {
    ctx: DependencyContext,
}

#[async_trait]
impl Constructor for TestBackgroundService1 {
    async fn ctor(ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self { ctx })
    }
}

#[async_trait]
impl IBackgroundService for TestBackgroundService1 {
    async fn execute(&self) {
        let sender = self.ctx.resolve::<Arc<RwLock<Option<Sender<String>>>>>().await.unwrap()
            .write().await
            .take().unwrap();
        sender.send("test".to_string()).unwrap();
    }
}

struct TestBackgroundService2 {
    application_life_time: Arc<dyn ILifeTimeManager>,
    ctx: DependencyContext,
}

#[async_trait]
impl Constructor for TestBackgroundService2 {
    async fn ctor(ctx: DependencyContext) -> BuildDependencyResult<Self> {
        let application_life_time = ctx.resolve().await?;

        Ok(Self {
            application_life_time,
            ctx,
        })
    }
}

#[async_trait]
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

    let mut app = Application::new().await;

    app.register_life_time_manager::<InnerStateLifeTimeManager>().await.unwrap();
    
    let (tx, rx) = oneshot::channel::<String>();
    app.root_ioc_context.register_instance(RwLock::new(Some(tx))).await.unwrap();
    app.root_ioc_context.register_instance(RwLock::new(Some(rx))).await.unwrap();

    app.register_service::<BackgroundService<TestBackgroundService1>>().await.unwrap();
    app.register_service::<BackgroundService<TestBackgroundService2>>().await.unwrap();
    
    app.run().await.unwrap();
}