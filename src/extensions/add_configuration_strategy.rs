use std::any::Any;

use anthill_di::{
    DependencyContext,
    types::{
        AsyncCallback,
        BuildDependencyResult, BuildDependencyError
    },
    TypeConstructor
};
use async_trait::async_trait;
use serde::Deserialize;

#[async_trait]
pub trait AddConfigurationStrategy {
    async fn add_transient_snapshot_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static;
    async fn add_singleton_snapshot_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static;
    async fn add_scoped_snapshot_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static;
}

#[async_trait]
impl AddConfigurationStrategy for DependencyContext {
    async fn add_transient_snapshot_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static {
        self.add_transient::<ConfigurationSnapshot<T>>(Box::new(SnapshotConfigurationBuilder::new::<T>(path))).await.unwrap();
    }

    async fn add_singleton_snapshot_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static {
        self.add_singleton::<ConfigurationSnapshot<T>>(Box::new(SnapshotConfigurationBuilder::new::<T>(path))).await.unwrap();
    }
    
    async fn add_scoped_snapshot_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static {
        self.add_scoped::<ConfigurationSnapshot<T>>(Box::new(SnapshotConfigurationBuilder::new::<T>(path))).await.unwrap();
    }
}

pub struct SnapshotConfigurationBuilder {
    async_ctor: AsyncCallback<DependencyContext, BuildDependencyResult<Box<dyn Any + Send + Sync>>>,
}

impl SnapshotConfigurationBuilder {
    pub fn new<T>(path: String) -> Self where for<'de> T: Deserialize<'de> + Sync + Send + 'static {
        let ctor: AsyncCallback<DependencyContext, BuildDependencyResult<Box<dyn Any + Send + Sync>>> = Box::new(
            move |_ctx: DependencyContext| -> _{
                let path = path.clone();
                Box::pin(
                    async move {
                        let configuration_snapshot = ConfigurationSnapshot::<T>::new(path).await
                            .map_err(|err| BuildDependencyError::Custom { message: format!("{:?}", err) })?;

                        Ok(Box::new(configuration_snapshot) as Box<dyn Any + Send + Sync>)
                    }
                )
            }
        );
        Self { async_ctor: ctor }
    }
}

#[async_trait]
impl TypeConstructor for SnapshotConfigurationBuilder {
    async fn ctor(&self, ctx: DependencyContext) -> BuildDependencyResult<Box<dyn Any + Send + Sync>> {
        (self.async_ctor)(ctx).await
    }
}

pub struct ConfigurationSnapshot<T> where for<'de> T: Deserialize<'de> + Sync + Send + 'static {
    path: String,
    pub snapshot: T,
}

#[derive(Debug)]
pub enum ConfigurationSnapshotError {
    ReadError { err: tokio::io::Error },
    DeserializeError { err: serde_json::Error }
}

type ConfigurationSnapshotResult<T> = Result<T, ConfigurationSnapshotError>;

impl<T> ConfigurationSnapshot<T> where for<'de> T: Deserialize<'de> + Sync + Send + 'static {
    pub async fn new(path: String) -> ConfigurationSnapshotResult<Self> {
        let data = tokio::fs::read_to_string(path.clone()).await
            .map_err(|err| ConfigurationSnapshotError::ReadError { err })?;

        let configuration: T = serde_json::from_str(&*data)
            .map_err(|err| ConfigurationSnapshotError::DeserializeError { err })?;
        
        Ok(Self {
            path,
            snapshot: configuration,
        })
    }

    pub async fn update(&mut self) -> ConfigurationSnapshotResult<()> {
        let data = tokio::fs::read_to_string(self.path.clone()).await
            .map_err(|err| ConfigurationSnapshotError::ReadError { err })?;

        let mut configuration: T = serde_json::from_str(&*data)
            .map_err(|err| ConfigurationSnapshotError::DeserializeError { err })?;

        std::mem::swap(&mut self.snapshot, &mut configuration);

        Ok(())
    }
}