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
            self,
            Sender,
            Receiver,
        }
    },
};

use crate::{
    Startup,
    configs::CoreConfig,
    service::{
        HostedServiceConstructor, BackgroundService
    },
    ApplicationLifeTime,
    extensions::AddBackgroundServiceStrategy,
};

struct TestBackgroundService1 {
    ctx: DependencyContext,
}

#[async_trait]
impl HostedServiceConstructor for TestBackgroundService1 {
    async fn ctor(_application_life_time: Arc<RwLock<ApplicationLifeTime>>, ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self { ctx })
    }
}

#[async_trait]
impl BackgroundService for TestBackgroundService1 {
    async fn execute(&mut self) {
        let sender = self.ctx.get_singleton::<Option<Sender<String>>>().await.unwrap()
            .write().await
            .take().unwrap();
        sender.send("test".to_string()).unwrap();
    }
}

struct TestBackgroundService2 {
    application_life_time: Arc<RwLock<ApplicationLifeTime>>,
    ctx: DependencyContext,
}

#[async_trait]
impl HostedServiceConstructor for TestBackgroundService2 {
    async fn ctor(application_life_time: Arc<RwLock<ApplicationLifeTime>>, ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self {
            application_life_time,
            ctx,
        })
    }
}

#[async_trait]
impl BackgroundService for TestBackgroundService2 {
    async fn execute(&mut self) {
        let receiver = self.ctx.get_singleton::<Option<Receiver<String>>>().await.unwrap().write().await.take().unwrap();
        assert_eq!("test".to_string(), receiver.await.unwrap());
        self.application_life_time.write().await.running.cancel().await;
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
impl Startup for TestStartup {
    async fn configure_system(&mut self, _dependency_context: &mut DependencyContext, _core_config: Arc<RwLock<CoreConfig>>) {

    }

    async fn configure_dependency(&mut self, dependency_context: &mut DependencyContext) {
        let (tx, rx) = oneshot::channel::<String>();

        dependency_context.add_singleton_instance(Some(tx)).await.unwrap();
        dependency_context.add_singleton_instance(Some(rx)).await.unwrap();

        dependency_context.add_background_service::<TestBackgroundService1>().await;
        dependency_context.add_background_service::<TestBackgroundService2>().await;
    }
}

#[tokio::test]
async fn single_transient() {
    use crate::ApplicationBuilder;
    
    ApplicationBuilder::new().await
        .with_startup::<TestStartup>().await
        .build().await
        .run().await
        .unwrap();
}