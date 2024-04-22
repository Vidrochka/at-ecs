use std::collections::HashSet;

use crate::types::ComponentId;

use super::IQuery;

impl IQuery for () {
    fn get_dependencies(&mut self) -> HashSet<ComponentId> {
        HashSet::new()
    }

    fn check(&mut self, _: &HashSet<ComponentId>) -> bool {
        true
    }
}

impl<TQueryPart: IQuery + 'static> IQuery for (TQueryPart, ) {
    fn get_dependencies(&mut self) -> HashSet<ComponentId> {
        self.0.get_dependencies()
    }

    fn check(&mut self, components: &HashSet<ComponentId>) -> bool {
        self.0.check(&components)
    }
}

impl<
    TQueryPart1: IQuery + 'static,
    TQueryPart2: IQuery + 'static,
> IQuery for (
    TQueryPart1,
    TQueryPart2,
) {
    fn get_dependencies(&mut self) -> HashSet<ComponentId> {
        let mut deps = self.0.get_dependencies();
        deps.extend(&mut self.1.get_dependencies().into_iter());
        deps
    }

    fn check(&mut self, components: &HashSet<ComponentId>) -> bool {
        self.0.check(&components) &&
        self.1.check(&components)
    }
}


impl<
    TQueryPart1: IQuery + 'static,
    TQueryPart2: IQuery + 'static,
    TQueryPart3: IQuery + 'static,
> IQuery for (
    TQueryPart1,
    TQueryPart2,
    TQueryPart3,
) {
    fn get_dependencies(&mut self) -> HashSet<ComponentId> {
        let mut deps = self.0.get_dependencies();
        deps.extend(&mut self.1.get_dependencies().into_iter());
        deps.extend(&mut self.2.get_dependencies().into_iter());
        deps
    }

    fn check(&mut self, components: &HashSet<ComponentId>) -> bool {
        self.0.check(&components) &&
        self.1.check(&components) &&
        self.2.check(&components)
    }
}

impl<
    TQueryPart1: IQuery + 'static,
    TQueryPart2: IQuery + 'static,
    TQueryPart3: IQuery + 'static,
    TQueryPart4: IQuery + 'static,
> IQuery for (
    TQueryPart1,
    TQueryPart2,
    TQueryPart3,
    TQueryPart4,
) {
    fn get_dependencies(&mut self) -> HashSet<ComponentId> {
        let mut deps = self.0.get_dependencies();
        deps.extend(&mut self.1.get_dependencies().into_iter());
        deps.extend(&mut self.2.get_dependencies().into_iter());
        deps.extend(&mut self.3.get_dependencies().into_iter());
        deps
    }

    fn check(&mut self, components: &HashSet<ComponentId>) -> bool {
        self.0.check(&components) &&
        self.1.check(&components) &&
        self.2.check(&components) &&
        self.3.check(&components)
    }
}