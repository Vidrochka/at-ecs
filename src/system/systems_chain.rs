use std::{collections::HashMap, sync::Arc};

use async_lock::RwLock;
use petgraph::{graph::NodeIndex, stable_graph::StableDiGraph};

use crate::types::{Signal, SignalWaitersCollection, SystemId};

use super::system::{ISystem, SystemFabricError};

#[derive(Debug)]
pub struct WorldSystemNode{
    pub (crate) system_id: SystemId,
    pub (crate) system: Arc<RwLock<Box<dyn ISystem>>>,

    
    pub (crate) signal: Signal<()>,
    pub (crate) waiters: SignalWaitersCollection<()>,
}

impl WorldSystemNode {
    pub fn new(
        system_id: SystemId,
        system: Arc<RwLock<Box<dyn ISystem>>>,
    ) -> Self {
        Self {
            system_id,
            system,
            signal: Signal::new(),
            waiters: SignalWaitersCollection::new(),
        }
    }

    pub async fn waiters(&self) -> &SignalWaitersCollection<()> {
        &self.waiters
    }
}

#[derive(Debug)]
pub struct SystemsChain {
    graph: StableDiGraph<WorldSystemNode, ()>,
    system_idx: HashMap<SystemId, NodeIndex>,

    not_found_systems: Vec<SystemId>,
    systems_with_build_error: HashMap<SystemId, SystemFabricError>,
}


impl SystemsChain {
    pub fn new(
        graph: StableDiGraph<WorldSystemNode, ()>,
        not_found_systems: Vec<SystemId>,
        systems_with_build_error: HashMap<SystemId, SystemFabricError>,
        system_idx: HashMap<SystemId, NodeIndex>,
    ) -> Self {
        Self {
            graph,
            not_found_systems,
            systems_with_build_error,
            system_idx,
        }
    }
    
    // pub fn recalculate_order(&mut self) -> CalculateOrderResult<()> {
        //     self.systems_order = recalculate_order(&self.graph)?;
        //     Ok(())
        // }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CalculateOrderError {
    #[error("System dependency sycle detected, SystemId:[{0:?}]")]
    Cycle(SystemId),
}

type CalculateOrderResult<T> = Result<T, CalculateOrderError>;

// pub fn recalculate_order(graph: &StableDiGraph<WorldSystemNode, ()>) -> CalculateOrderResult<Vec<SystemId>> {
//     let sorted_idxes = petgraph::algo::toposort(graph, None)
//         .map_err(|e| CalculateOrderError::Cycle(graph.node_weight(e.node_id()).expect("Node not found by returned id").system_id.clone()))?;

//     let systems_order = sorted_idxes.into_iter()
//         .map(|idx| graph.node_weight(idx).expect("Node not found by returned id").system_id.clone())
//         .collect::<Vec<SystemId>>();
        
//     Ok(systems_order)
// }