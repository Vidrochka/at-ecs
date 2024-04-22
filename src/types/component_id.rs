use std::sync::Arc;

use type_uuid::TypeUuid;
use uuid::Uuid;

use super::TypeInfo;

/// Максимально убираем влияние информации о типе для оптимизации, но храним для удобства
#[derive(Debug, Clone, Eq, Ord)]
pub struct ComponentId(Uuid, Arc<TypeInfo>);

impl std::hash::Hash for ComponentId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialOrd for ComponentId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq for ComponentId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl ComponentId {
    pub fn new(id: Uuid, type_info: Arc<TypeInfo>) -> ComponentId {
        Self(id, type_info)
    }

    pub fn from<TComponent: TypeUuid + 'static>() -> Self {
        Self(Uuid::from_bytes(TComponent::UUID), Arc::new(TypeInfo::from_type::<TComponent>()))
    }
    
    pub fn type_info(&self) -> Arc<TypeInfo> {
        self.1.clone()
    }

    pub fn uuid(&self) -> Uuid {
        self.0.clone()
    }
}