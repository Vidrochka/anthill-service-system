use anthill_di::{
    DependencyContext,
    extensions::{ConstructedDependencySetStrategy, InterfaceDependencySetStrategy}, Constructor
};

use crate::{
    configs::CoreConfig,
    utils::CancellationHandler,
    ApplicationLifeTime,
    Application, Startup,
    service::ServiceCollection
};


pub struct ApplicationBuilder {
    root_dependency_ctx: DependencyContext,
}

impl ApplicationBuilder {
    pub async fn new() -> Self {
        let mut root_dependency_ctx = DependencyContext::new_root();
        add_shared_dependencies(&mut root_dependency_ctx).await;

        Self { root_dependency_ctx, }
    }

    pub async fn with_startup<T: Startup + Constructor>(mut self) -> Self {
        self.root_dependency_ctx.set_transient_interface::<dyn Startup, T>().await.unwrap();
        let mut startup = self.root_dependency_ctx.get_transient::<Box<dyn Startup>>().await.unwrap();

        let core_config = self.root_dependency_ctx.get_singleton::<CoreConfig>().await.unwrap();
        startup.configure_system(&mut self.root_dependency_ctx, core_config.clone()).await;
        startup.configure_dependency(&mut self.root_dependency_ctx).await;

        self
    }

    pub async fn build(self) -> Application {
        Application::new( self.root_dependency_ctx ).await
    }    
}

async fn add_shared_dependencies(root_dependency_ctx: &DependencyContext) {
    root_dependency_ctx.set_singleton::<CoreConfig>().await.unwrap();
    root_dependency_ctx.set_transient::<CancellationHandler>().await.unwrap();
    root_dependency_ctx.set_singleton::<ApplicationLifeTime>().await.unwrap();
    root_dependency_ctx.set_singleton::<ServiceCollection>().await.unwrap();
}