#![feature(trait_alias)]
#![feature(map_many_mut)]

pub mod archetype;
pub mod component;
pub mod system;
pub mod query;
pub mod world;

pub mod types;

#[cfg(test)]
pub mod tests;