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
    task::JoinHandle,
};

use crate::{
    Startup,
    configs::CoreConfig,
    service::{
        HostedService,
        HostedServiceConstructor
    },
    ApplicationLifeTime,
    extensions::AddHostedServiceStrategy,
};

struct TestHostedService1 {
    task_handler: Option<JoinHandle<()>>,
    sender: Option<Sender<String>>,
}

#[async_trait]
impl HostedServiceConstructor for TestHostedService1 {
    async fn ctor(_application_life_time: Arc<RwLock<ApplicationLifeTime>>, ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self {
            task_handler: None,
            sender: Some(ctx.get_singleton::<Option<Sender<String>>>().await.unwrap().write().await.take().unwrap()),
        })
    }
}

#[async_trait]
impl HostedService for TestHostedService1 {
    async fn on_start(&mut self) {
        let sender = self.sender.take().unwrap();
        self.task_handler = Some(tokio::spawn(async move {
            let sender = sender;
            sender.send("test".to_string()).unwrap();
        }));
    }

    async fn on_stop(&mut self) {
        self.task_handler.take().unwrap().await.unwrap();
    }
}

struct TestHostedService2 {
    task_handler: Option<JoinHandle<()>>,
    receiver: Option<Receiver<String>>,
    application_life_time: Arc<RwLock<ApplicationLifeTime>>,
}

#[async_trait]
impl HostedServiceConstructor for TestHostedService2 {
    async fn ctor(application_life_time: Arc<RwLock<ApplicationLifeTime>>, ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self {
            task_handler: None,
            receiver: Some(ctx.get_singleton::<Option<Receiver<String>>>().await.unwrap().write().await.take().unwrap()),
            application_life_time,
        })
    }
}

#[async_trait]
impl HostedService for TestHostedService2 {
    async fn on_start(&mut self) {
        let receiver = self.receiver.take().unwrap();
        let lt = self.application_life_time.clone();
        self.task_handler = Some(tokio::spawn(async move {
            let receiver = receiver;
            assert_eq!("test".to_string(), receiver.await.unwrap());
            lt.write().await.running.cancel().await;
        }));
    }

    async fn on_stop(&mut self) {
        self.task_handler.take().unwrap().await.unwrap();
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

        dependency_context.add_hosted_service::<TestHostedService1>().await;
        dependency_context.add_hosted_service::<TestHostedService2>().await;
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