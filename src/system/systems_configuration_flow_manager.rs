use std::{collections::{HashMap, HashSet}, ops::Deref, sync::Arc};

use async_broadcast::{Receiver, Sender};
use async_lock::RwLock;
use petgraph::{algo, graph::NodeIndex, stable_graph::StableDiGraph, visit::IntoNodeIdentifiers};
use type_uuid::TypeUuid;

use crate::types::SystemId;

use super::{configuration::{ISystemConfiguration, SystemConfiguration}, systems_chain::{WorldSystemNode, SystemsChain}, system::{ISystem, ISystemFabric, SystemFabricError}};

#[derive(Debug)]
pub struct GlobalSystemNode{
    configuration: Box<dyn ISystemConfiguration>,
}

impl GlobalSystemNode {
    pub fn new(configuration: Box<dyn ISystemConfiguration>) -> Self {
        Self { configuration }
    }

    pub fn configuration(&self) -> &Box<dyn ISystemConfiguration> {
       &self.configuration
    }
}

/// Управляет всеми системами и строит наборы систем для миров
#[derive(Debug, Default)]
pub struct SystemsFlowConfigurationManager {
    system_idx: HashMap<SystemId, NodeIndex>,
    graph: StableDiGraph<GlobalSystemNode, ()>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SystemRegistrationError {
    #[error("System already exists, SystemId:[{0:?}]")]
    SystemAlreadyExists(SystemId),
}

pub type SystemRegistrationResult = Result<SystemId, SystemRegistrationError>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SystemAddLinkError {
    #[error("'System from' configuration not found, SystemId: [{0:?}]")]
    SystemFromConfigurationNotFound(SystemId),
    #[error("'System to' configuration not found, SystemId: [{0:?}]")]
    SystemToConfigurationNotFound(SystemId),
    #[error("Systems make cycle SystemId: [{0:?}] [{1:?}]")]
    Cycle(SystemId, SystemId),
}

pub type SystemAddLinkResult = Result<(), SystemAddLinkError>;

impl SystemsFlowConfigurationManager {
    pub fn system_node(&self, system_id: &SystemId) -> Option<&GlobalSystemNode> {
        self.system_idx.get(&system_id)
            .map(|x| self.graph.node_weight(*x)).flatten()
    }

    pub fn system_node_typed<TSystem: ISystem + TypeUuid + 'static>(&self) -> Option<&GlobalSystemNode> {
        self.system_node(&SystemId::from::<TSystem>())
    }

    pub fn register<TSystem: ISystem + TypeUuid + 'static>(&mut self, system_fabric: impl ISystemFabric<TSystem = TSystem> + 'static) -> SystemRegistrationResult {
        let system_id = self.system_idx.get(&SystemId::from::<TSystem>())
            .map(|x| self.graph.node_weight(*x).map(|x| x.configuration.system_id()))
            .flatten();

        if let Some(system_id) = system_id {
            return Err(SystemRegistrationError::SystemAlreadyExists(system_id));
        }

        let system_configuration = GlobalSystemNode::new(Box::new(SystemConfiguration::new(system_fabric)) as Box<dyn ISystemConfiguration>);
        let id = system_configuration.configuration.system_id();

        let idx = self.graph.add_node(system_configuration);
        self.system_idx.insert(id.clone(), idx);

        Ok(id)
    }

    pub fn add_link(&mut self, from_system_id: &SystemId, to_system_id: &SystemId) -> SystemAddLinkResult {
        let Some(from_system_idx) = self.system_idx.get(from_system_id) else {
            return Err(SystemAddLinkError::SystemFromConfigurationNotFound(
                from_system_id.clone(),
            ));
        };

        let Some(to_system_idx) = self.system_idx.get(to_system_id) else {
            return Err(SystemAddLinkError::SystemToConfigurationNotFound(
                to_system_id.clone(),
            ));
        };

        let edge_idx = self.graph.add_edge(*from_system_idx, *to_system_idx, ());

        if algo::is_cyclic_directed(&self.graph) {
            self.graph.remove_edge(edge_idx);
            
            return Err(SystemAddLinkError::Cycle(from_system_id.clone(), to_system_id.clone()));
        }

        Ok(())
    }

    pub fn add_link_typed<TSystemFrom: ISystem + TypeUuid + 'static, TSystemTo: ISystem + TypeUuid + 'static>(&mut self) -> SystemAddLinkResult {
        self.add_link(&SystemId::from::<TSystemFrom>(), &SystemId::from::<TSystemTo>())
    }

    pub async fn build_world_systems_manager(&self, expected_systemd_ids: HashSet<SystemId>) -> SystemsChain {
        let mut not_found_systems = Vec::new();
        let mut systems_with_build_error = HashMap::new();

        let mut world_systems_graph = StableDiGraph::<WorldSystemNode, (), _>::new();

        let mut world_systems_node_idx = HashMap::new();
        
        for system_id in expected_systemd_ids.iter() {
            let Some(system_idx) = self.system_idx.get(system_id) else {
                tracing::warn!("Expected system not registered in global manager: system_id:[{system_id:?}]");

                not_found_systems.push(system_id.clone());

                continue;
            };

            let system_node = self.graph.node_weight(*system_idx).unwrap();

            let system = match system_node.configuration.build().await {
                Ok(system) => system,
                Err(error) => {
                    tracing::error!("System build error: system_id:[{:?}], error:[{:?}]", system_id, error);

                    systems_with_build_error.insert(system_id.clone(), error);

                    continue;
                }
            };

            let world_system_node = WorldSystemNode::new(system_id.clone(), Arc::new(RwLock::new(system)));

            let world_system_idx = world_systems_graph.add_node(world_system_node);

            world_systems_node_idx.insert(system_id.clone(), world_system_idx);
        }

        for world_system_idx in world_systems_graph.node_indices().into_iter().collect::<Vec<_>>() {
            let system_id = world_systems_graph.node_weight(world_system_idx).unwrap().system_id.clone();

            let global_system_idx = self.system_idx.get(&system_id).unwrap().to_owned();

            let global_prev_systems_node_idx = self.graph.neighbors_directed(global_system_idx, petgraph::Direction::Incoming)
                .collect::<Vec<_>>();

            let mut waiters = Vec::new();

            for global_prev_system_node_idx in global_prev_systems_node_idx {
                let prev_system_id = self.graph.node_weight(global_prev_system_node_idx).unwrap().configuration.system_id();

                let Some(world_prev_system_node_idx) = world_systems_node_idx.get(&prev_system_id) else {
                    continue;
                };

                let world_prev_system_node = world_systems_graph.node_weight(*world_prev_system_node_idx).unwrap();

                waiters.push(world_prev_system_node.signal.signal_waiter());

                world_systems_graph.add_edge(*world_prev_system_node_idx, world_system_idx, ());
            }

            let world_system_node = world_systems_graph.node_weight_mut(world_system_idx).unwrap();
            world_system_node.waiters.subscribe_many(waiters).await;
        }

        SystemsChain::new(
            world_systems_graph,
            not_found_systems,
            systems_with_build_error,
            world_systems_node_idx,
        )
    }

    // pub async fn build_world_system_manager2(&self, expected_systemd_ids: HashSet<SystemId>) -> WorldSystemManager {
        

    //     let mut not_found_systems = Vec::new();
    //     let mut systems_with_build_error = HashMap::new();

    //     let mut graph = StableDiGraph::<WorldSystemNode, (), _>::new();
    //     let mut system_idx = HashMap::new();

    //     let mut systems = HashMap::new();

    //     for system_id in expected_systemd_ids.iter() {
    //         let Some(system_configuration) = self.system_configurations.get(system_id) else {
    //             tracing::warn!("Expected system not registered in manager: system_id:[{system_id:?}]");

    //             not_found_systems.push(system_id.clone());

    //             continue;
    //         };

    //         let system = match system_configuration.build().await {
    //             Ok(system) => system,
    //             Err(error) => {
    //                 tracing::error!("System build error: system_id:[{:?}], error:[{:?}]", system_id, error);

    //                 systems_with_build_error.insert(system_id.clone(), error);

    //                 continue;
    //             }
    //         };

    //         systems.insert(system_id.clone(), system);

    //         let (s, r) = async_broadcast::broadcast::<()>(1);
            
    //         let idx = graph.add_node(WorldSystemNode {
    //             system_id: system_id.clone(),
    //             sender: s,
    //             reciever: r,
    //         });

    //         system_idx.insert(system_id.clone(), idx);
    //     }

    //     for node_idx in graph.node_identifiers().into_iter() {
    //         let node = graph.node_weight(node_idx)
    //             .expect(&format!("[WTF] Node by index not found: node_idx:[{node_idx:?}]"));

    //         let system_configuration = self.system_configurations.get(&node.system_id)
    //             .expect(&format!("[WTF] Required system configuration not found in collection: system_id:[{:?}]", node.system_id));

    //         for prev_system_id in system_configuration.prev_systems() {
    //             // Если индекс не найден, значит зависимость не была добавлена в системы мира
    //             let Some(prev_system_idx) = system_idx.get(prev_system_id).map(|x| *x) else {
    //                 tracing::warn!("Prev system skipped: system_id:[{prev_system_id:?}]");
    //                 continue;
    //             };

    //             if !graph.contains_edge(prev_system_idx, node_idx) {
    //                 graph.add_edge(prev_system_idx, node_idx, ());
    //             }
    //         }

    //         for next_system_id in system_configuration.next_systems() {
    //             // Если индекс не найден, значит зависимость не была добавлена в системы мира
    //             let Some(next_system_idx) = system_idx.get(next_system_id).map(|x| *x) else {
    //                 tracing::warn!("Next system skipped: system_id:[{next_system_id:?}]");
    //                 continue;
    //             };

    //             if !graph.contains_edge(node_idx, next_system_idx) {
    //                 graph.add_edge(node_idx, next_system_idx, ());
    //             }
    //         }
    //     }

    //     let mut system_compilations = HashMap::new();
        
    //     for node_idx in graph.node_indices().into_iter() {
    //         let node = graph.node_weight(node_idx).expect(&format!("[WTF] Node by index not found: node_idx:[{node_idx:?}]"));

    //         let system = systems.remove(&node.system_id).expect("[WTF] System not found");

    //         let mut prev_dep = Vec::new();

    //         for prev_system_idx in graph.neighbors_directed(node_idx, petgraph::Direction::Incoming).into_iter() {
    //             let prev_system_node = graph.node_weight(prev_system_idx)
    //                 .expect(&format!("[WTF] Node by index not found: node_idx:[{prev_system_idx:?}]"));

    //             prev_dep.push(PrevDependency::new(prev_system_node.reciever.clone(), prev_system_node.system_id.clone()));
    //         }

    //         let mut next_dep = Vec::new();

    //         for next_system_idx in graph.neighbors_directed(node_idx, petgraph::Direction::Outgoing).into_iter() {
    //             let next_system_node = graph.node_weight(next_system_idx)
    //                 .expect(&format!("[WTF] Node by index not found: node_idx:[{next_system_idx:?}]"));

    //             next_dep.push(next_system_node.system_id.clone());
    //         }

    //         system_compilations.insert(node.system_id.clone(), Arc::new(RwLock::new(SystemCompilation::new(
    //             system,
    //             prev_dep,
    //             NextDependencies::new(node.sender.clone(), next_dep),
    //         ))));
    //     }

    //     WorldSystemManager::new(
    //         system_compilations,
    //         graph,
    //         not_found_systems,
    //         systems_with_build_error,
    //         system_idx,
    //         None,
    //     )
    // }
}