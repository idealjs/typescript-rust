#![allow(unused_imports)]

use super::*;

impl NameGenerationScope {
    pub(crate) fn new() -> Self {
        Self {
            next: None,
            temp_flags: TEMP_FLAGS_AUTO,
            formatted_name_temp_flags: HashMap::new(),
            reserved_names: HashSet::new(),
        }
    }
}

pub struct NameGenerator {
    pub(crate) node_id_to_generated_name: HashMap<u64, String>,
    pub(crate) node_id_to_generated_private_name: HashMap<u64, String>,
    pub(crate) auto_generated_id_to_generated_name: HashMap<AutoGenerateId, String>,
    pub(crate) name_generation_scope: Option<Box<NameGenerationScope>>,
    pub(crate) private_name_generation_scope: Option<Box<NameGenerationScope>>,
    pub(crate) generated_names: HashSet<String>,
    pub(crate) get_text_of_node: Box<dyn Fn(&Node) -> String>,
    pub(crate) is_unique_local_name: Option<Box<dyn Fn(&str, &Node) -> bool>>,
}
