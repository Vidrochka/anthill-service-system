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
    async fn add_transient_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static;
    async fn add_singleton_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static;
    async fn add_scoped_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static;
}

#[async_trait]
impl AddConfigurationStrategy for DependencyContext {
    async fn add_transient_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static {
        self.add_transient::<T>(Box::new(ConfigurationBuilder::new::<T>(path))).await.unwrap();
    }

    async fn add_singleton_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static {
        self.add_singleton::<T>(Box::new(ConfigurationBuilder::new::<T>(path))).await.unwrap();
    }
    
    async fn add_scoped_configuration<T>(&self, path: String) where for<'de> T: Deserialize<'de> + Sync + Send + 'static {
        self.add_scoped::<T>(Box::new(ConfigurationBuilder::new::<T>(path))).await.unwrap();
    }
}

pub struct ConfigurationBuilder {
    async_ctor: AsyncCallback<DependencyContext, BuildDependencyResult<Box<dyn Any + Send + Sync>>>,
}

impl ConfigurationBuilder {
    pub fn new<T>(path: String) -> Self where for<'de> T: Deserialize<'de> + Sync + Send + 'static {
        let ctor: AsyncCallback<DependencyContext, BuildDependencyResult<Box<dyn Any + Send + Sync>>> = Box::new(
            move |_ctx: DependencyContext| -> _{
                let path = path.clone();
                Box::pin(
                    async move {
                        let data = tokio::fs::read_to_string(path.clone()).await
                            .map_err(|err| BuildDependencyError::Custom { message: format!("{:?}", ConfigurationError::ReadError { err })})?;

                        let configuration: T = serde_json::from_str(&*data)
                            .map_err(|err| BuildDependencyError::Custom { message: format!("{:?}", ConfigurationError::DeserializeError { err })})?;

                        Ok(Box::new(configuration) as Box<dyn Any + Send + Sync>)
                    }
                )
            }
        );
        Self { async_ctor: ctor }
    }
}

#[async_trait]
impl TypeConstructor for ConfigurationBuilder {
    async fn ctor(&self, ctx: DependencyContext) -> BuildDependencyResult<Box<dyn Any + Send + Sync>> {
        (self.async_ctor)(ctx).await
    }
}

#[derive(Debug)]
pub enum ConfigurationError {
    ReadError { err: tokio::io::Error },
    DeserializeError { err: serde_json::Error }
}