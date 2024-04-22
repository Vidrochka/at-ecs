use std::{fmt::Debug, sync::Arc};

use type_uuid::TypeUuid;

use crate::types::{SystemId, TypeInfo};

use super::system::{ISystem, ISystemFabric, SystemFabricResult};

#[derive(Debug)]
pub struct SystemConfiguration<TSystem: ISystem + TypeUuid + 'static> {
    id: SystemId,
    fabric: Box<dyn ISystemFabric<TSystem = TSystem>>,
    // /// prev systems
    // prev_systems: Vec<SystemId>,
    // /// next systems
    // next_systems: Vec<SystemId>,
}

impl<TSystem: ISystem + TypeUuid + 'static> SystemConfiguration<TSystem> {
    pub fn new<TFabric: ISystemFabric<TSystem = TSystem> + 'static>(fabric: TFabric) -> Self {
        Self {
            id: SystemId::from::<TSystem>(),
            fabric: Box::new(fabric),
            // prev_systems: Default::default(),
            // next_systems: Default::default(),
        }
    }
}

#[async_trait::async_trait]
pub trait ISystemConfiguration: Debug {
    fn system_id(&self) -> SystemId;
    fn system_type_info(&self) -> Arc<TypeInfo>;
    
    // fn prev_system(&mut self, system_id: SystemId);
    // fn next_system(&mut self, system_id: SystemId);
    
    // fn prev_systems(&self) -> &Vec<SystemId>;
    // fn next_systems(&self) -> &Vec<SystemId>;

    async fn build(&self) -> SystemFabricResult<Box<dyn ISystem>>;
}

#[async_trait::async_trait]
impl<TSystem: ISystem + TypeUuid + 'static> ISystemConfiguration for SystemConfiguration<TSystem> {
    fn system_id(&self) -> SystemId {
        self.id.clone()
    }

    fn system_type_info(&self) -> Arc<TypeInfo> {
        self.id.type_info()
    }

    async fn build(&self) -> SystemFabricResult<Box<dyn ISystem>> {
        self.fabric.build().await
            .map(|system| Box::new(system) as Box<dyn ISystem>)
    }

    // fn prev_system(&mut self, system_id: SystemId) {
    //     self.prev_systems.push(system_id);
    // }

    // fn next_system(&mut self, system_id: SystemId) {
    //     self.next_systems.push(system_id);
    // }

    // fn prev_systems(&self) -> &Vec<SystemId> {
    //     &self.prev_systems
    // }
    
    // fn next_systems(&self) -> &Vec<SystemId> {
    //     &self.next_systems
    // }
}