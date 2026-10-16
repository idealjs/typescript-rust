pub(crate) mod deep_clone_node;
#[allow(unused_imports)]
pub use deep_clone_node::*;
pub mod diagnostic;
pub mod module_pattern;
pub mod node;
pub mod node_data_generated;
pub mod node_flags;
pub mod positionmap;
pub mod symbol;
pub mod syntax_kind_generated;
pub mod utilities;

pub use diagnostic::*;
pub use module_pattern::*;
pub mod dynamic_imports;
pub use dynamic_imports::*;
pub use node::*;
pub use node_data_generated::*;
pub use node_flags::*;
pub use symbol::*;
pub use syntax_kind_generated::SyntaxKind;
pub use utilities::*;
#[cfg(test)]
pub(crate) mod deepclone_tests;
pub(crate) mod node_line_map;
pub mod node_node;
pub mod node_node_list;
pub mod node_source_file;
#[cfg(test)]
pub(crate) mod node_tests;
#[cfg(test)]
pub(crate) mod positionmap_tests;
pub(crate) mod symbol_flags;
pub(crate) mod symbol_flow;
pub(crate) mod symbol_internal_names;
pub(crate) mod symbol_map;
pub(crate) mod symbol_symbol;
#[cfg(test)]
pub(crate) mod symbol_tests;
pub(crate) mod utilities_declarations;
pub(crate) mod utilities_expressions;
pub(crate) mod utilities_functions;
pub(crate) mod utilities_heritage;
pub(crate) mod utilities_misc;
pub(crate) mod utilities_modifiers;
pub(crate) mod utilities_module_state;
pub(crate) mod utilities_modules;
pub(crate) mod utilities_navigation;
pub(crate) mod utilities_predicates;
pub(crate) mod utilities_sourcefile;
pub(crate) mod utilities_statements;
pub(crate) mod utilities_synthesized;
pub(crate) mod utilities_types;

// r 轮接线:迁移批次模块
pub mod mig;
pub mod subtree_facts;
pub mod visitor;

// r16a 接线:checker mig 所需 ast 符号提权
pub use mig::m3f::get_next_jsdoc_comment_location;
pub use mig::m3f::get_symbol_id;
pub use mig::m3f_2::has_abstract_modifier;
pub use mig::m3g_2::is_object_literal_method;
pub use mig::m3g_2::is_string_literal_like_type;
pub use mig::m3g_2::is_this_in_type_query;
pub use mig::m3g_3::node_kind_is;
pub use mig::m3g_3::skip_parentheses;
pub use utilities_heritage::get_extends_heritage_clause_element as get_class_extends_heritage_element;
pub mod utilities_r16a;
pub use utilities_r16a::*;
