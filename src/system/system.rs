use std::{future::Future, fmt::Debug};

use crate::{types::{Signal, SignalWaitersCollection, TypeInfo}, world::World};

#[async_trait::async_trait]
pub trait ISystem: Send + Debug {
    async fn before_tick(&mut self, _: &World, swc: &SignalWaitersCollection<()>) {
        swc.wait_all().await;
    }

    async fn tick(&mut self, _: &World) {}

    async fn after_tick(&mut self, _: &World, s: &Signal<()>) {
        s.signal(()).await;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SystemFabricError {
    #[error("System fabric error:[{0:?}]")]
    Other(anyhow::Error)
}

pub type SystemFabricResult<TSystem> = Result<TSystem, SystemFabricError>;

#[async_trait::async_trait]
pub trait ISystemFabric: Send + Sync {
    type TSystem: ISystem + Sized;

    async fn build(&self) -> SystemFabricResult<Self::TSystem>;
}

#[async_trait::async_trait]
impl<TFunc, TFuncFut, TSystem> ISystemFabric for TFunc
where
    TFunc: Fn() -> TFuncFut,
    TFuncFut: Future<Output = SystemFabricResult<TSystem>> + Send,
    TFunc: Send + Sync + 'static,
    TSystem: ISystem,
{
    type TSystem = TSystem;
    
    async fn build(&self) -> SystemFabricResult<Self::TSystem> {
        self().await
    }
}

impl<TSystem: 'static> std::fmt::Debug for dyn ISystemFabric<TSystem = TSystem> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("dyn ISystemFabric<TSystem>")
            .field("system", &TypeInfo::from_type::<TSystem>())
            .finish()
    }
}