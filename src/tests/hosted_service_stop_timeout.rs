use std::{sync::Arc, time::Duration};

use anthill_di::{
    DependencyContext,
    Constructor,
    types::BuildDependencyResult,
};
use tokio::{
    sync::{
        RwLock,
        oneshot::{
            Sender,
            Receiver,
        }
    },
    task::JoinHandle, time,
};

use crate::{services::IBaseService, life_time::ILifeTimeManager};

struct TestHostedService1 {
    task_handler: Option<JoinHandle<()>>,
    sender: Option<Sender<String>>,
}

#[async_trait_with_sync::async_trait(Sync)]
impl Constructor for TestHostedService1 {
    async fn ctor(ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self {
            task_handler: None,
            sender: Some(ctx.resolve::<Arc<RwLock<Option<Sender<String>>>>>().await?.write().await.take().unwrap()),
        })
    }
}

#[async_trait_with_sync::async_trait(Sync)]
impl IBaseService for TestHostedService1 {
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
    application_life_time: Arc<dyn ILifeTimeManager>,
}

#[async_trait_with_sync::async_trait(Sync)]
impl Constructor for TestHostedService2 {
    async fn ctor(ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self {
            task_handler: None,
            receiver: Some(ctx.resolve::<Arc<RwLock<Option<Receiver<String>>>>>().await?.write().await.take().unwrap()),
            application_life_time: ctx.resolve().await?,
        })
    }
}

#[async_trait_with_sync::async_trait(Sync)]
impl IBaseService for TestHostedService2 {
    async fn on_start(&mut self) {
        let receiver = self.receiver.take().unwrap();
        let lt = self.application_life_time.clone();
        self.task_handler = Some(tokio::spawn(async move {
            let receiver = receiver;
            assert_eq!("test".to_string(), receiver.await.unwrap());
            lt.stop().await;
        }));
    }

    async fn on_stop(&mut self) {
        self.task_handler.take().unwrap().await.unwrap();
        time::sleep(Duration::from_millis(6100)).await;
    }
}

#[tokio::test]
async fn hosted_service_stop_timeout() {
    use crate::{
        Application,
        types::AppRunError,
        life_time::InnerStateLifeTimeManager,
    };
    use anthill_di::types::TypeInfo;
    use tokio::{
        sync::{
            RwLock,
            oneshot
        },
    };

    let configuration_path = "hosted_service_stop_timeout.json".to_string();

    let mut app = Application::new(Some(configuration_path.clone())).await.unwrap();
    app.core_config.write().await.value.on_stop_timeout = Duration::from_secs(6);

    app.register_life_time_manager::<InnerStateLifeTimeManager>().await.unwrap();

    let (tx, rx) = oneshot::channel::<String>();
    app.root_ioc_context.register_instance(RwLock::new(Some(tx))).await.unwrap();
    app.root_ioc_context.register_instance(RwLock::new(Some(rx))).await.unwrap();

    app.register_service::<TestHostedService1>().await.unwrap();
    app.register_service::<TestHostedService2>().await.unwrap();

    let result = app.run().await;

    std::fs::remove_file(configuration_path).unwrap();

    assert_eq!(result.err(), Some(AppRunError::ServiceStopTimeoutExpired {
        timeout_duration: Duration::from_millis(6000),
        service_type_info: TypeInfo::from_type::<TestHostedService2>(),
    }))
}