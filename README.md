# anthill-service-system
Rust runtime service system manager with ```anthill-di``` integration

## Warning

Library required Rust nightly for ```anthill-di```

---

## Basic concepts

First you need to create an application

``` rust
async fn _() {
    // create new ioc context
    let mut app = Application::new().await;

    // -- or -- //

    // from exist ioc context
    let root_context = DependencyContext::new_root();
    root_context.register_type::<Application>(DependencyLifeCycle::Transient).await.unwrap();

    // On create Application created new ioc context
    let mut app = root_context.resolve::<Application>().await.unwrap();
}

```

The service must implement the trait ```IBaseService``` + ```Constructor```

``` rust 
#[async_trait]
impl IBaseService for TestHostedService1 {
    async fn on_start(&mut self) {
        // you have (by default) 5s
        // to do something on start
    }

    async fn on_stop(&mut self) {
        // you have (by default) 5s
        // to do something on stop
    }
}
```

Or you can use the extension to work in a separate task

``` rust
#[async_trait]
impl IBackgroundService for TestBackgroundService1 {
    async fn execute(&self) {
        // you can do anything before app stoned
        // check state time by time and exit on stop (see next)
    }
}
```

To manipulate application state, you can request a dependency ```Arc<dyn ILifeTimeManager>```

``` rust
async fn _(ctx: DependencyContext) {
    let app_lifetime = ctx.resolve::<Arc<dyn ILifeTimeManager>>().await.unwrap();

    // check app state
    let is_running = app_lifetime.is_running().await;

    // stop app
    app_lifetime.stop().await;

    // wait until app stop request
    app_lifetime.wait_for_stop().await;
}
```

Then register dependencies and services

``` rust

async fn _() {
    // let mut app = Application::new().await;

    // for mor info about ioc see (https://crates.io/crates/anthill-di)
    app.root_ioc_context.register_type::<SomeType>(DependencyLifeCycle::Transient).await.unwrap();

    app.register_service::<BackgroundService<TestBackgroundService>>().await.unwrap();

    app.register_service::<TestBaseService>().await.unwrap();
}

```

For a more convenient setting of the service, use ```IStartup```

``` rust
#[async_trait]
impl IStartup for TestStartup {
async fn configure_dependency(&mut self, root_ioc_context: &mut DependencyContext) {
        root_ioc_context.register_type::<SomeComponent>().await.unwrap();
    }

    async fn configure_application(&mut self, _ : Arc<RwLock<CoreConfig>>, app: &mut Application) {
        app.register_service::<BackgroundService<TestBackgroundService1>>().await.unwrap();
        app.register_service::<BackgroundService<TestBackgroundService2>>().await.unwrap();
    }
}

async fn _() {
    // let mut app = Application::new().await;

    app.register_startup::<TestStartup>().await.unwrap();
}
```

Launch the app

``` rust 
async fn _() {
    // let mut app = Application::new().await;
    app.run().await.unwrap();
}
```

You can control start and stop timeout (by default 5s per service)

``` rust 
async fn _() {
    // let mut app = Application::new().await;

    let core_config = app.root_ioc_context.resolve::<Arc<RwLock<CoreConfig>>>().await.unwrap();

    core_config.write().await.on_start_timeout = Duration::from_millis(6000);
    core_config.write().await.on_stop_timeout = Duration::from_millis(6000);
}
```

Currently implemented ```CtrlCLifeTimeManager``` and ```InnerStateLifeTimeManager```    
* Use ```CtrlCLifeTimeManager``` for close app in ```ctrl+c``` press time    
* Use ```InnerStateLifeTimeManager``` for close app only by service request

Default lifetime manager is ```CtrlCLifeTimeManager```    

You can customize your app lifetime management    

``` rust
pub struct InnerStateLifeTimeManager {
    is_running: AtomicBool,
}

#[async_trait]
impl ILifeTimeManager for CtrlCLifeTimeManager {
    async fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst)
    }
    
    async fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    async fn wait_for_stop(&self) {
        while self.is_running.load(Ordering::SeqCst) {
            yield_now().await;
        }
    }
}

async fn _() {
    // let mut app = Application::new().await;

    // Use here you LifeTimeManager
    app.register_life_time_manager::<CtrlCLifeTimeManager>().await.unwrap();
}
```

---

## Example

```rust
use anthill_service_system::{configs::CoreConfig, Application, IStartup, life_time::{ILifeTimeManager, InnerStateLifeTimeManager}};
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

use crate::services::IBackgroundService;

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

        app.register_background_service::<TestBackgroundService1>().await.unwrap();
        app.register_background_service::<TestBackgroundService2>().await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let mut app = Application::new().await;
  
    app.register_startup::<TestStartup>().await.unwrap();
    
    app.run().await.unwrap();
}

```

#### Refs:
 - [crate.io](https://crates.io/crates/anthill-service-system)