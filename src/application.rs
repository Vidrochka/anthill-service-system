use async_trait::async_trait;
use tokio::time::timeout;
use std::sync::{Arc, Weak};
use tokio::sync::RwLock;
use std::any::{type_name, TypeId};

use crate::{
    configs::CoreConfig,
    types::{
        AddServiceError,
        AddServiceResult,
        AppRunResult,
        AppRunError, AddStartupError, AddStartupResult, AddLifeTimeManagerResult, AddLifeTimeManagerError
    },
    services::IBaseService,
    IStartup,
    life_time::{
        ILifeTimeManager,
        CtrlCLifeTimeManager
    },
};

use anthill_di::{
    types::{BuildDependencyResult, BuildDependencyError},
    DependencyContext,
    DependencyLifeCycle,
    Constructor
};


pub struct Application {
    pub root_ioc_context: DependencyContext,
    core_config: Arc<RwLock<CoreConfig>>,
}

#[async_trait]
impl Constructor for Application {
    async fn ctor(ctx: DependencyContext) -> BuildDependencyResult<Self> {
        log::info!("Application creating ...");

        let mut ctx = ctx;
        ctx.set_empty_scope();

        ctx.register_type::<RwLock<CoreConfig>>(DependencyLifeCycle::Singleton).await
            .map_err(|e| BuildDependencyError::AddDependencyError {err: e})?;

        let core_config = ctx.resolve().await.unwrap();

        log::info!("Application created");

        Ok(Self {
            root_ioc_context: ctx,
            core_config,
        })
    }
}

impl Application {
    pub async fn new() -> Self {
        log::info!("Application creating ...");
        let root_ioc_context = DependencyContext::new_root();

        root_ioc_context.register_type::<RwLock<CoreConfig>>(DependencyLifeCycle::Singleton).await.unwrap();
        let core_config = root_ioc_context.resolve().await.unwrap();

        log::info!("Application created");

        Self {
            root_ioc_context,
            core_config,
        }
    }

    pub async fn register_service<TBaseService: IBaseService + Constructor>(&mut self) -> AddServiceResult {
        log::info!("Starting registration service, name:[{service_name}] type_id:[{type_id:?}]", service_name = type_name::<TBaseService>(), type_id = TypeId::of::<TBaseService>());

        self.root_ioc_context.register_type::<RwLock<TBaseService>>(DependencyLifeCycle::Singleton).await
            .map_err(|e| AddServiceError::IocAddDependencyError(e))?
            .map_as::<RwLock<dyn IBaseService>>().await.map_err(|e| AddServiceError::IocMapComponentError(e))?;

        log::info!("Service registered, name:[{service_name}] type_id:[{type_id:?}]", service_name = type_name::<TBaseService>(), type_id = TypeId::of::<TBaseService>());

        Ok(())
    }

    // pub async fn register_background_service<TBackgroundService: IBackgroundService + Constructor>(&mut self) -> AddServiceResult {
    //     log::info!("Starting registration background service, name:[{service_name}] type_id:[{type_id:?}]", service_name = type_name::<TBackgroundService>(), type_id = TypeId::of::<TBackgroundService>());

    //     self.root_ioc_context.register_type::<RwLock<TBackgroundService>>(DependencyLifeCycle::Singleton).await.map_err(|e| AddServiceError::IocAddDependencyError(e))?
    //         .map_as::<RwLock<dyn IBackgroundService>>().await.map_err(|e| AddServiceError::IocMapComponentError(e))?;

    //     self.root_ioc_context.register_type::<RwLock<BackgroundService<TBackgroundService>>>(DependencyLifeCycle::Singleton).await
    //         .map_err(|e| AddServiceError::IocAddDependencyError(e))?
    //         .map_as::<RwLock<dyn IBaseService>>().await.map_err(|e| AddServiceError::IocMapComponentError(e))?;

    //     log::info!("Background service registered, name:[{service_name}] type_id:[{type_id:?}]", service_name = type_name::<TBackgroundService>(), type_id = TypeId::of::<TBackgroundService>());

    //     Ok(())
    // }

    pub async fn register_startup<TStartup: IStartup + Constructor>(&mut self) -> AddStartupResult {
        self.root_ioc_context.register_type::<RwLock<TStartup>>(DependencyLifeCycle::Scoped).await
            .map_err(|e| AddStartupError::IocAddDependencyError(e))?
            .map_as::<RwLock<dyn IStartup>>().await
            .map_err(|e| AddStartupError::IocMapComponentError(e))?;

        Ok(())
    }

    pub async fn register_life_time_manager<TLifeTimeManager: ILifeTimeManager + Constructor>(&mut self) -> AddLifeTimeManagerResult {
        self.root_ioc_context.register_type::<TLifeTimeManager>(DependencyLifeCycle::Singleton).await
            .map_err(|e| AddLifeTimeManagerError::IocAddDependencyError(e))?
            .map_as::<dyn ILifeTimeManager>().await
            .map_err(|e| AddLifeTimeManagerError::IocMapComponentError(e))?;
        Ok(())
    }

    pub async fn run(&mut self) -> AppRunResult {
        self.apply_life_time_manager().await?;
        self.apply_startups().await?;

        log::info!("Resolving services...");
        let mut services = self.root_ioc_context.resolve_collection::<Arc<RwLock<dyn IBaseService>>>().await
            .map_err(|e| AppRunError::IocBuildDependencyError(e))?;
        log::info!("Services resolved [{count}]", count = services.len());

        self.start(&mut services).await?;

        let lifetime_time_manager = self.root_ioc_context.resolve::<Arc<dyn ILifeTimeManager>>().await
            .expect("LifeTimeManager not found");

        lifetime_time_manager.wait_for_stop().await;

        self.stop(&mut services).await?;

        Ok(())
    }

    async fn apply_life_time_manager(&mut self) -> AppRunResult {
        if let Err(err) = self.root_ioc_context.resolve::<Arc<dyn ILifeTimeManager>>().await {
            log::info!("Life time manager not found, use default [CtrlCLifeTimeManager] [{err:?}]");

            self.register_life_time_manager::<CtrlCLifeTimeManager>().await.map_err(|e| {
                return match e {
                    AddLifeTimeManagerError::IocAddDependencyError(err) => AppRunError::IocAddDependencyError(err),
                    AddLifeTimeManagerError::IocMapComponentError(err) => AppRunError::IocMapComponentError(err),
                }
            })?;
        }

        Ok(())
    }

    async fn apply_startups(&mut self) -> AppRunResult {
        log::info!("Apply startups...");

        let startups = self.root_ioc_context.resolve_collection::<Weak<RwLock<dyn IStartup>>>().await;

        let mut startups = if let Err(BuildDependencyError::NotFound { .. }) = startups {
            Vec::new()
        } else {
            let startups = startups.map_err(|e| AppRunError::IocBuildDependencyError(e))?;
            startups
        };

        for startup in startups.iter_mut() {
            let sturtup = startup.upgrade().expect("Startup not exist in scope");
            let mut startup_write_guard = sturtup.write().await;

            startup_write_guard.configure_dependency(&mut self.root_ioc_context).await;
            startup_write_guard.configure_application(self.core_config.clone(), self).await;

            log::info!("Startups applied [{service_info:?}]", service_info = startup_write_guard.get_type_info());
        }

        log::info!("Startups applied [{count}]", count = startups.len());

        Ok(())
    }

    async fn start(&mut self, services: &mut Vec<Arc<RwLock<dyn IBaseService>>>) -> AppRunResult {
        log::info!("Application starting ...");

        let on_start_timeout = self.core_config.read().await.on_start_timeout.clone();

        let mut service_start_tasks = Vec::new();
        for service in services.iter() {
            let service = service.clone();

            let service_type_info = service.read().await.get_type_info();
            log::info!("Starting service ... [{service_type_info:?}]");

            let on_start_task = timeout(on_start_timeout.clone(), tokio::spawn(async move {
                let mut service_write_guard = service.write().await;
                service_write_guard.on_start().await;
            }));
            
            service_start_tasks.push((on_start_task, service_type_info))
        }

        for (task_handler, service_type_info) in service_start_tasks.into_iter() {
            if task_handler.await.is_err() {
                log::error!("Service start error [{service_type_info:?}]");

                return Err(AppRunError::ServiceStartTimeoutExpired { timeout_duration: on_start_timeout, service_type_info });
            }

            log::info!("Service started [{service_type_info:?}]");
        }

        log::info!("Application started");

        Ok(())
    }

    async fn stop(&mut self, services: &mut Vec<Arc<RwLock<dyn IBaseService>>>) -> AppRunResult {
        log::info!("Application stopping...");

        let on_stop_timeout = self.core_config.read().await.on_start_timeout.clone();

        let mut service_stop_tasks = Vec::new();
        for service in services.iter() {
            let service = service.clone();
            
            let service_type_info = service.write().await.get_type_info(); 
            log::info!("Stopping service ... [{service_type_info:?}]");

            let on_stop_task = timeout(on_stop_timeout.clone(), tokio::spawn(async move {
                let mut service_write_guard = service.write().await;
                service_write_guard.on_stop().await;
            }));
            
            service_stop_tasks.push((on_stop_task, service_type_info))
        }

        for (task_handler, service_type_info) in service_stop_tasks.into_iter() {
            if task_handler.await.is_err() {
                log::error!("Service stop error [{service_type_info:?}]");

                return Err(AppRunError::ServiceStopTimeoutExpired { timeout_duration: on_stop_timeout, service_type_info });
            }

            log::info!("Service stopped [{service_type_info:?}]");
        }

        log::info!("Application stopped");

        Ok(())
    }
}