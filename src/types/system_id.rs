use std::sync::Arc;

use petgraph::csr::IndexType;
use type_uuid::TypeUuid;
use uuid::Uuid;

use crate::system::system::ISystem;

use super::TypeInfo;

/// Максимально убираем влияние информации о типе для оптимизации, но храним для удобства
#[derive(Debug, Clone, Eq, Ord)]
pub struct SystemId(Uuid, Arc<TypeInfo>);

impl std::hash::Hash for SystemId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialOrd for SystemId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq for SystemId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl SystemId {
    pub fn new(id: Uuid, type_info: Arc<TypeInfo>) -> SystemId {
        Self(id, type_info)
    }

    pub fn from<TSystem: ISystem + TypeUuid + 'static>() -> Self {
        Self(Uuid::from_bytes(TSystem::UUID), Arc::new(TypeInfo::from_type::<TSystem>()))
    }

    pub fn type_info(&self) -> Arc<TypeInfo> {
        self.1.clone()
    }

    pub fn uuid(&self) -> Uuid {
        self.0.clone()
    }
}