use std::sync::Arc;

use anthill_di::DependencyContext;
use tokio::{
    sync::RwLock,
    time::timeout
};

use crate::{
    configs::CoreConfig,
    ApplicationLifeTime,
    service::ServiceCollection, types::{AppRunResult, AppRunError}
};

pub struct Application {
    _root_dependency_ctx: DependencyContext,
    core_config: Arc<RwLock<CoreConfig>>,
    application_life_time: Arc<RwLock<ApplicationLifeTime>>,
    service_collection: Arc<RwLock<ServiceCollection>>,
}

impl Application {
    pub async fn new(root_dependency_ctx: DependencyContext) -> Self {
        let core_config= root_dependency_ctx.get_singleton::<CoreConfig>().await.unwrap();
        let application_life_time = root_dependency_ctx.get_singleton::<ApplicationLifeTime>().await.unwrap();
        let service_collection = root_dependency_ctx.get_singleton::<ServiceCollection>().await.unwrap();
        
        Self {
            _root_dependency_ctx: root_dependency_ctx,
            core_config,
            application_life_time,
            service_collection,
        }
    }
    
    pub async fn run(&mut self) -> AppRunResult<()> {
        log::info!("Application starting");

        {
            let mut service_start_tasks = Vec::new();
            let on_start_timeout = self.core_config.read().await.on_start_timeout.clone();

            let mut service_collection_guard = self.service_collection.write().await;

            let mut i = 0;
            for service_info in service_collection_guard.services.iter_mut() {
                log::info!("Service running: [{}] [{}]", service_info.payload, service_info.name);

                let service = service_info.service.clone();
                let on_start_task = timeout(on_start_timeout.clone(), tokio::spawn(async move {
                    let mut servce_guard = service.write().await;
                    servce_guard.on_start().await;
                }));

                service_start_tasks.push((on_start_task, i));
                i+=1;
            }

            for service_start_task in service_start_tasks.into_iter() {
                if service_start_task.0.await.is_err() {
                    return Err(AppRunError::ServiceStartTimeoutExpired {
                        timeout_duration: on_start_timeout.clone(),
                        service_name: service_collection_guard.services[service_start_task.1].name.clone(),
                        payload: service_collection_guard.services[service_start_task.1].payload.clone(),
                    });
                }
            }
        }

        log::info!("Application started");

        let stop_waiter = self.application_life_time.write().await.running.get_cancel_waiter().await;
        stop_waiter.await.unwrap();

        log::info!("Application stopping");

        {
            let mut service_stop_tasks = Vec::new();
            let on_stop_timeout = self.core_config.read().await.on_stop_timeout.clone();

            let mut service_collection_guard = self.service_collection.write().await;

            let mut i = 0;
            for service_info in service_collection_guard.services.iter_mut() {
                log::info!("Service stopping: [{}] [{}]", service_info.payload, service_info.name);

                let service = service_info.service.clone();
                let on_start_task = timeout(on_stop_timeout.clone(), tokio::spawn(async move {
                    let mut servce_guard = service.write().await;
                    servce_guard.on_stop().await;
                }));

                service_stop_tasks.push((on_start_task, i));
                i+=1;
            }

            for service_start_task in service_stop_tasks.into_iter() {
                if service_start_task.0.await.is_err() {
                    return Err(AppRunError::ServiceStopTimeoutExpired {
                        timeout_duration: on_stop_timeout.clone(),
                        service_name: service_collection_guard.services[service_start_task.1].name.clone(),
                        payload: service_collection_guard.services[service_start_task.1].payload.clone(),
                    });
                }
            }
        }

        log::info!("Application stopped");

        Ok(())
    }
}