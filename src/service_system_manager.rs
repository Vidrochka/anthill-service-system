use crate::{ServiceSystemCofiguration, error::ServiceSystemError, service::Service, utils::{State, SystemState}};

use anthill_di::{Injector, builders::ContainerBuilder};

use std::sync::Arc;

use tokio::sync::RwLock;

pub struct ServiceSystemManager {
    service_collection: Arc<RwLock<Vec<Arc<RwLock<dyn Service + Sync + Send>>>>>,
    system_state: Arc<RwLock<SystemState>>,
    injector: Arc<RwLock<Injector>>,
}

impl ServiceSystemManager {
    pub async fn new(configuration: Box<dyn ServiceSystemCofiguration>) -> Result<Self, ServiceSystemError> {
        let containers = vec![
            ContainerBuilder::bind_type::<SystemState>().build(),
            ContainerBuilder::bind_unconfigured_type::<Vec<Arc<RwLock<dyn Service + Sync + Send>>>>().build_with_value(Vec::new()),
        ];

        let injector = Injector::new(containers).await;

        let mut configuration = configuration;
        configuration.configure_injections(injector.clone()).await
            .map_err(|e| ServiceSystemError::ConfigureServicesError{description: e})?;
        configuration.configure_services(injector.clone()).await
            .map_err(|e| ServiceSystemError::ConfigureInjectionsError{description: e})?;

        let service_collection: Arc<RwLock<Vec<Arc<RwLock<dyn Service + Sync + Send>>>>>
            = injector.write().await.get_singletone().map_err(|e| ServiceSystemError::DiError{di_error: e})?;

        let system_state: Arc<RwLock<SystemState>>
            = injector.write().await.get_singletone().map_err(|e| ServiceSystemError::DiError{di_error: e})?;

        let service_system_manager = Self{
            service_collection: service_collection,
            system_state: system_state,
            injector: injector,
        };

        Ok(service_system_manager)
    }

    pub async fn run(&mut self) -> Result<(), ServiceSystemError> {
        use tokio::time::{timeout, Duration};

        let mut service_task_collection = Vec::new();

        for service in self.service_collection.read().await.iter() {
            let service_clone = Arc::clone(service);
            let service_name = service_clone.read().await.get_service_name();

            let injector_ref = self.injector.clone();

            service_task_collection.push((
                service_name,
                timeout(Duration::from_millis(5000), tokio::spawn(async move { service_clone.write().await.on_start(injector_ref).await; })),
            ));
        }

        for (service_name, task) in service_task_collection.into_iter() {
            if task.await.is_err() {
                return Err(ServiceSystemError::ServiceStartTimeoutExpire{timeout: Duration::from_millis(5000), service_name: service_name})
            }
        }

        if self.system_state.read().await.get_state() != State::Run {
            return Err(ServiceSystemError::PrematureTermination);
        }

        let waiter = self.system_state.write().await.wait_stop();
        waiter.await.unwrap();

        let mut service_task_collection = Vec::new();

        for service in self.service_collection.read().await.iter() {
            let service_clone = Arc::clone(service);
            let service_name = service_clone.read().await.get_service_name();

            let injector_ref = self.injector.clone();

            service_task_collection.push((
                service_name,
                timeout(Duration::from_millis(5000), tokio::spawn(async move { service_clone.write().await.on_end(injector_ref).await; })),
            ));
        }

        for (service_name, task) in service_task_collection.into_iter() {
            if task.await.is_err() {
                return Err(ServiceSystemError::ServiceEndTimeoutExpire{timeout: Duration::from_millis(5000), service_name: service_name})
            }
        }
        
        Ok(())
    }
}