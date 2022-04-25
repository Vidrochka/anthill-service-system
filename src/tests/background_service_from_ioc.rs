use crate::{configs::CoreConfig, Application};
use crate::{IStartup, life_time::{ILifeTimeManager, InnerStateLifeTimeManager}};
use std::sync::Arc;

use anthill_di::{
    DependencyContext,
    Constructor,
    types::BuildDependencyResult,
};
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::{
    sync::{
        RwLock,
        oneshot::{
            Sender,
            Receiver,
        }
    },
};

use crate::services::{IBackgroundService, BackgroundService};

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

struct TestStartup {}

#[async_trait]
impl Constructor for TestStartup {
    async fn ctor(_ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl IStartup for TestStartup {
    async fn configure_dependency(&mut self, root_ioc_context: &mut DependencyContext) {
        let (tx, rx) = oneshot::channel::<String>();
        
        root_ioc_context.register_instance(RwLock::new(Some(tx))).await.unwrap();
        root_ioc_context.register_instance(RwLock::new(Some(rx))).await.unwrap();
    }

    async fn configure_application(&mut self, _ : Arc<RwLock<CoreConfig>>, app: &mut Application) {
        app.register_life_time_manager::<InnerStateLifeTimeManager>().await.unwrap();

        app.register_service::<BackgroundService<TestBackgroundService1>>().await.unwrap();
        app.register_service::<BackgroundService<TestBackgroundService2>>().await.unwrap();
    }
}

#[tokio::test]
async fn background_service_from_ioc() {
    use anthill_di::DependencyLifeCycle;

    let root_context = DependencyContext::new_root();
    root_context.register_type::<Application>(DependencyLifeCycle::Transient).await.unwrap();

    let mut app = root_context.resolve::<Application>().await.unwrap();
  
    app.register_startup::<TestStartup>().await.unwrap();
    
    app.run().await.unwrap();
}