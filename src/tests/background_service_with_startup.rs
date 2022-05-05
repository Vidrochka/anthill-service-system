use crate::{configs::CoreConfig, Application};
use crate::{IStartup, life_time::{ILifeTimeManager, InnerStateLifeTimeManager}};
use std::sync::Arc;

use anthill_di::{
    DependencyContext,
};
use anthill_di_configuration_extension::ConfigurationSnapshot;
use anthill_di_configuration_extension::source::JsonFileConfiguration;
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

#[derive(constructor)]
struct TestStartup {}

#[async_trait_with_sync::async_trait(Sync)]
impl IStartup for TestStartup {
async fn configure_dependency(&mut self, root_ioc_context: &mut DependencyContext) {
        let (tx, rx) = oneshot::channel::<String>();
        
        root_ioc_context.register_instance(RwLock::new(Some(tx))).await.unwrap();
        root_ioc_context.register_instance(RwLock::new(Some(rx))).await.unwrap();
    }

    async fn configure_application(&mut self, _core_config: Arc<RwLock<ConfigurationSnapshot<CoreConfig, JsonFileConfiguration::<CoreConfig>>>>, app: &mut Application) {
        app.register_life_time_manager::<InnerStateLifeTimeManager>().await.unwrap();

        app.register_service::<BackgroundService<TestBackgroundService1>>().await.unwrap();
        app.register_service::<BackgroundService<TestBackgroundService2>>().await.unwrap();
    }
}

#[tokio::test]
async fn background_service_with_startup() {
    let configuration_path = "background_service_with_startup.json".to_string();

    let mut app = Application::new(Some(configuration_path.clone())).await.unwrap();
  
    app.register_startup::<TestStartup>().await.unwrap();
    
    app.run().await.unwrap();

    std::fs::remove_file(configuration_path).unwrap();
}