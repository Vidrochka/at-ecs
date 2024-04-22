use std::fmt::Debug;

use anthill_di::{from_dependency_context::IFromDependencyContext, query::*, DependencyContext};

use super::system::{ISystem, ISystemFabric, SystemFabricError};

pub fn system_from_ioc_context<TSystem: ISystem + Debug + 'static>(ctx: DependencyContext) -> impl ISystemFabric<TSystem = TSystem> {
    move || {
        let ctx = ctx.clone();

        Box::pin(async move {
            Query::<FetchFirstRequired::<TSystem>>::get(&ctx).await
                .map(|x| x.result)
                .map_err(|e| SystemFabricError::Other(e.into()))
        })
    }
}