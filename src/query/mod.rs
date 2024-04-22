pub mod has;
pub mod typle;
pub use has::*;

pub mod not;
pub use not::*;

use std::{
    collections::HashSet,
    fmt::Debug,
};

use crate::types::ComponentId;



pub trait IQuery: Debug {
    fn get_dependencies(&mut self) -> HashSet<ComponentId>;
    fn check(&mut self, components: &HashSet<ComponentId>) -> bool;
}