use std::{collections::HashSet, marker::PhantomData, fmt::Debug};

use type_uuid::TypeUuid;

use crate::types::ComponentId;

use super::IQuery;

#[derive(Debug, Default)]
pub struct Not<TComponent: Debug + TypeUuid> {
    pd: PhantomData<TComponent>
}

impl<TComponent: Debug + TypeUuid> Not<TComponent> {
    pub fn new() -> Self {
        Self { pd: PhantomData }
    }
}

impl<TComponent: Debug + TypeUuid + 'static> IQuery for Not<TComponent> {
    fn get_dependencies(&mut self) ->HashSet<ComponentId> {
        HashSet::new()
    }

    fn check(&mut self, components: &HashSet<ComponentId>) -> bool {
        !components.contains(&ComponentId::from::<TComponent>())
    }
}