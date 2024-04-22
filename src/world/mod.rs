use std::collections::HashMap;

use crate::types::ComponentId;


pub struct World {
    components: HashMap<ComponentId, u32>,
}