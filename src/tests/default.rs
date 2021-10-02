use std::sync::Arc;
use std::time::Duration;

use anthill_di::Injector;
use anthill_di::{Injection, builders::ContainerBuilder};
use tokio::sync::RwLock;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::utils::SystemState;
use crate::{Service, ServiceSystemCofiguration};

use crate::extensions::AddServiceExtension;

struct TextWrapper1 {
    pub text: String
}

impl Injection for TextWrapper1 {
    fn build_injection(_: &mut anthill_di::Injector) -> Result<Self, anthill_di::DiError> {
        Ok(Self{text: "test1".to_string()})
    }
}

struct TextWrapper2 {
    pub text: String
}

impl Injection for TextWrapper2 {
    fn build_injection(_: &mut anthill_di::Injector) -> Result<Self, anthill_di::DiError> {
        Ok(Self{text: "test2".to_string()})
    }
}

struct TestService1 {
    service_task: Option<JoinHandle<()>>,
}

impl Injection for TestService1 {
    fn build_injection(_: &mut anthill_di::Injector) -> Result<Self, anthill_di::DiError> {
        Ok(Self{
            service_task: None,
        })
    }
}

#[async_trait::async_trait]
impl Service for TestService1 {
    async fn on_start(&mut self, injector: Arc<RwLock<Injector>>) {
        let sender: Arc<RwLock<Sender<String>>> = injector.write().await.get_singletone().unwrap();
        let text_wrapper_1: TextWrapper1 = injector.write().await.get_new_instance().unwrap();
        let text_wrapper_2: TextWrapper2 = injector.write().await.get_new_instance().unwrap();

        self.service_task = Some(tokio::spawn(async move {
            use tokio::time::sleep;
            sleep(Duration::from_millis(1)).await;
            sender.write().await.send(format!("{}_{}", text_wrapper_1.text, text_wrapper_2.text)).await.unwrap();
        }));
    }

    async fn on_end(&mut self, _: Arc<RwLock<Injector>>) {
        if let Some(service_task) = self.service_task.take() {
            service_task.await.unwrap();
        }
    }
}

struct TestService2 {
    service_task: Option<JoinHandle<()>>,
}

impl Injection for TestService2 {
    fn build_injection(_: &mut anthill_di::Injector) -> Result<Self, anthill_di::DiError> {
        Ok(Self{
            service_task: None,
        })
    }
}

#[async_trait::async_trait]
impl Service for TestService2 {
    async fn on_start(&mut self, injector: Arc<RwLock<Injector>>) {
        let receiver: Arc<RwLock<Receiver<String>>> = injector.write().await.get_singletone().unwrap();
        let system_state: Arc<RwLock<SystemState>> = injector.write().await.get_singletone().unwrap();

        self.service_task = Some(tokio::spawn(async move {
            assert_eq!(receiver.write().await.recv().await.unwrap(), "test1_test2");
            system_state.write().await.stop();
        }));
    }

    async fn on_end(&mut self, _: Arc<RwLock<Injector>>) {
        if let Some(service_task) = self.service_task.take() {
            service_task.await.unwrap();
        }
    }
}

struct TestConfiguration {
}

#[async_trait::async_trait]
impl ServiceSystemCofiguration for TestConfiguration {
    async fn configure_injections(&mut self, injector: Arc<RwLock<anthill_di::Injector>>) -> Result<(), String> {
        use tokio::sync::mpsc;
        let (sender, receiver) = mpsc::channel::<String>(1);

        injector.write().await.add_container(ContainerBuilder::bind_type::<TextWrapper1>().build());
        injector.write().await.add_container(ContainerBuilder::bind_type::<TextWrapper2>().build());
        injector.write().await.add_container(ContainerBuilder::bind_unconfigured_type().build_with_value(sender));
        injector.write().await.add_container(ContainerBuilder::bind_unconfigured_type().build_with_value(receiver));
        
        Ok(())
    }

    async fn configure_services(&mut self, injector: Arc<RwLock<anthill_di::Injector>>) -> Result<(), String> {
        injector.write().await.add_service::<TestService1>().await.map_err(|e| format!("{:?}", e))?;
        injector.write().await.add_service::<TestService2>().await.map_err(|e| format!("{:?}", e))?;
        Ok(())
    }
}

#[tokio::test]
async fn default() {
    use crate::ServiceSystemManager;
    ServiceSystemManager::new(Box::new(TestConfiguration{})).await.unwrap().run().await.unwrap();
}