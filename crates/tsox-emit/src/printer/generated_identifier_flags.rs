#![allow(unused_imports)]

use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct GeneratedIdentifierFlags(pub i32);

impl GeneratedIdentifierFlags {
    pub const NONE: Self = Self(0);
    pub const AUTO: Self = Self(1);
    pub const LOOP: Self = Self(2);
    pub const UNIQUE: Self = Self(3);
    pub const NODE: Self = Self(4);
    pub const KIND_MASK: Self = Self(7);

    pub const RESERVED_IN_NESTED_SCOPES: Self = Self(1 << 3);
    pub const OPTIMISTIC: Self = Self(1 << 4);
    pub const FILE_LEVEL: Self = Self(1 << 5);
    pub const ALLOW_NAME_SUBSTITUTION: Self = Self(1 << 6);

    pub fn kind(self) -> Self { ::tsox_core::fntrace::enter("kind"); 
        Self(self.0 & Self::KIND_MASK.0)
    }
    pub fn is_auto(self) -> bool { ::tsox_core::fntrace::enter("is_auto"); 
        self.kind() == Self::AUTO
    }
    pub fn is_loop(self) -> bool { ::tsox_core::fntrace::enter("is_loop"); 
        self.kind() == Self::LOOP
    }
    pub fn is_unique(self) -> bool { ::tsox_core::fntrace::enter("is_unique"); 
        self.kind() == Self::UNIQUE
    }
    pub fn is_node(self) -> bool { ::tsox_core::fntrace::enter("is_node"); 
        self.kind() == Self::NODE
    }
    pub fn is_reserved_in_nested_scopes(self) -> bool { ::tsox_core::fntrace::enter("is_reserved_in_nested_scopes"); 
        self.0 & Self::RESERVED_IN_NESTED_SCOPES.0 != 0
    }
    pub fn is_optimistic(self) -> bool { ::tsox_core::fntrace::enter("is_optimistic"); 
        self.0 & Self::OPTIMISTIC.0 != 0
    }
    pub fn is_file_level(self) -> bool { ::tsox_core::fntrace::enter("is_file_level"); 
        self.0 & Self::FILE_LEVEL.0 != 0
    }
}

impl std::ops::BitOr for GeneratedIdentifierFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { ::tsox_core::fntrace::enter("bitor"); 
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for GeneratedIdentifierFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self { ::tsox_core::fntrace::enter("bitand"); 
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitOrAssign for GeneratedIdentifierFlags {
    fn bitor_assign(&mut self, rhs: Self) { ::tsox_core::fntrace::enter("bitor_assign"); 
        self.0 |= rhs.0;
    }
}

impl std::ops::Not for GeneratedIdentifierFlags {
    type Output = Self;
    fn not(self) -> Self { ::tsox_core::fntrace::enter("not"); 
        Self(!self.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AutoGenerateOptions {
    pub flags: GeneratedIdentifierFlags,
    pub prefix: String,
    pub suffix: String,
}

pub type AutoGenerateId = u32;

#[allow(dead_code)]
pub(crate) static NEXT_AUTO_GENERATE_ID: AtomicU32 = AtomicU32::new(0);

#[allow(dead_code)]
pub(crate) fn next_auto_generate_id() -> AutoGenerateId { ::tsox_core::fntrace::enter("next_auto_generate_id"); 
    NEXT_AUTO_GENERATE_ID.fetch_add(1, Ordering::Relaxed) + 1
}

#[derive(Debug, Clone)]
pub struct AutoGenerateInfo {
    pub flags: GeneratedIdentifierFlags,
    pub id: AutoGenerateId,
    pub prefix: String,
    pub suffix: String,
    pub node: Option<Arc<Node>>,
}

#[derive(Debug, Clone)]
pub struct GeneratedName {
    pub text: String,
    pub auto_generate: AutoGenerateInfo,
    pub is_private: bool,
}

impl GeneratedName {
    pub fn new(text: String, is_private: bool, auto_generate: AutoGenerateInfo) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            text,
            is_private,
            auto_generate,
        }
    }

    pub fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        &self.text
    }
}

#[derive(Default)]
pub struct EmitContext {
    pub(crate) next_id: AtomicU32,
    pub emit_nodes:
        std::cell::RefCell<HashMap<*const Node, crate::printer::mig::m4m_2::EmitNode>>,
    pub auto_generate: HashMap<*const Node, AutoGenerateInfo>,
    pub var_scope_stack: Vec<crate::printer::mig::m4m_3::VarScope>,
    pub let_scope_stack: Vec<crate::printer::mig::m4m_3::VarScope>,
}

impl EmitContext {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self::default()
    }

    pub fn next_auto_generate_id(&self) -> AutoGenerateId { ::tsox_core::fntrace::enter("next_auto_generate_id"); 
        self.next_id.fetch_add(1, Ordering::Relaxed) + 1
    }
}

pub struct NodeFactory<'a> {
    pub(crate) emit_context: &'a EmitContext,
}

impl<'a> NodeFactory<'a> {
    pub fn new(emit_context: &'a EmitContext) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self { emit_context }
    }

    pub(crate) fn new_generated_identifier(
        &self,
        kind: GeneratedIdentifierFlags,
        text: &str,
        node: Option<Arc<Node>>,
        options: AutoGenerateOptions,
    ) -> GeneratedName { ::tsox_core::fntrace::enter("new_generated_identifier"); 
        let id = self.emit_context.next_auto_generate_id();
        let display_text = if text.is_empty() && node.is_some() {
            node.as_ref().unwrap().text().to_string()
        } else {
            text.to_string()
        };
        let auto_generate = AutoGenerateInfo {
            flags: kind | (options.flags & !GeneratedIdentifierFlags::KIND_MASK),
            id,
            prefix: options.prefix,
            suffix: options.suffix,
            node,
        };
        GeneratedName::new(display_text, false, auto_generate)
    }

    pub(crate) fn new_generated_private_identifier(
        &self,
        kind: GeneratedIdentifierFlags,
        text: &str,
        node: Option<Arc<Node>>,
        options: AutoGenerateOptions,
    ) -> GeneratedName { ::tsox_core::fntrace::enter("new_generated_private_identifier"); 
        let id = self.emit_context.next_auto_generate_id();
        let display_text = if text.is_empty() {
            if let Some(ref n) = node {
                n.text().to_string()
            } else {
                format!("(auto@{id})")
            }
        } else if !text.starts_with('#') {
            panic!("First character of private identifier must be #: {text}");
        } else {
            text.to_string()
        };
        let formatted =
            format_generated_name(true, &options.prefix, &display_text, &options.suffix);
        let auto_generate = AutoGenerateInfo {
            flags: kind | (options.flags & !GeneratedIdentifierFlags::KIND_MASK),
            id,
            prefix: options.prefix,
            suffix: options.suffix,
            node,
        };
        GeneratedName::new(formatted, true, auto_generate)
    }

    pub fn new_temp_variable(&self) -> GeneratedName { ::tsox_core::fntrace::enter("new_temp_variable"); 
        self.new_temp_variable_ex(AutoGenerateOptions::default())
    }

    pub fn new_temp_variable_ex(&self, options: AutoGenerateOptions) -> GeneratedName { ::tsox_core::fntrace::enter("new_temp_variable_ex"); 
        self.new_generated_identifier(GeneratedIdentifierFlags::AUTO, "", None, options)
    }

    pub fn new_loop_variable(&self) -> GeneratedName { ::tsox_core::fntrace::enter("new_loop_variable"); 
        self.new_loop_variable_ex(AutoGenerateOptions::default())
    }

    pub fn new_loop_variable_ex(&self, options: AutoGenerateOptions) -> GeneratedName { ::tsox_core::fntrace::enter("new_loop_variable_ex"); 
        self.new_generated_identifier(GeneratedIdentifierFlags::LOOP, "", None, options)
    }

    pub fn new_unique_name(&self, text: &str) -> GeneratedName { ::tsox_core::fntrace::enter("new_unique_name"); 
        self.new_unique_name_ex(text, AutoGenerateOptions::default())
    }

    pub fn new_unique_name_ex(&self, text: &str, options: AutoGenerateOptions) -> GeneratedName { ::tsox_core::fntrace::enter("new_unique_name_ex"); 
        self.new_generated_identifier(GeneratedIdentifierFlags::UNIQUE, text, None, options)
    }

    pub fn new_generated_name_for_node(&self, node: &Arc<Node>) -> GeneratedName { ::tsox_core::fntrace::enter("new_generated_name_for_node"); 
        self.new_generated_name_for_node_ex(node, AutoGenerateOptions::default())
    }

    pub fn new_generated_name_for_node_ex(
        &self,
        node: &Arc<Node>,
        mut options: AutoGenerateOptions,
    ) -> GeneratedName { ::tsox_core::fntrace::enter("new_generated_name_for_node_ex"); 
        if !options.prefix.is_empty() || !options.suffix.is_empty() {
            options.flags |= GeneratedIdentifierFlags::OPTIMISTIC;
        }
        self.new_generated_identifier(
            GeneratedIdentifierFlags::NODE,
            "",
            Some(Arc::clone(node)),
            options,
        )
    }

    pub fn new_unique_private_name(&self, text: &str) -> GeneratedName { ::tsox_core::fntrace::enter("new_unique_private_name"); 
        self.new_unique_private_name_ex(text, AutoGenerateOptions::default())
    }

    pub fn new_unique_private_name_ex(
        &self,
        text: &str,
        options: AutoGenerateOptions,
    ) -> GeneratedName { ::tsox_core::fntrace::enter("new_unique_private_name_ex"); 
        self.new_generated_private_identifier(GeneratedIdentifierFlags::UNIQUE, text, None, options)
    }

    pub fn new_generated_private_name_for_node(&self, node: &Arc<Node>) -> GeneratedName { ::tsox_core::fntrace::enter("new_generated_private_name_for_node"); 
        self.new_generated_private_name_for_node_ex(node, AutoGenerateOptions::default())
    }

    pub fn new_generated_private_name_for_node_ex(
        &self,
        node: &Arc<Node>,
        mut options: AutoGenerateOptions,
    ) -> GeneratedName { ::tsox_core::fntrace::enter("new_generated_private_name_for_node_ex"); 
        if !options.prefix.is_empty() || !options.suffix.is_empty() {
            options.flags |= GeneratedIdentifierFlags::OPTIMISTIC;
        }
        self.new_generated_private_identifier(
            GeneratedIdentifierFlags::NODE,
            "",
            Some(Arc::clone(node)),
            options,
        )
    }
}

pub(crate) const TEMP_FLAGS_AUTO: i32 = 0x0000_0000;
pub(crate) const TEMP_FLAGS_COUNT_MASK: i32 = 0x0FFF_FFFF;
pub(crate) const TEMP_FLAGS_I: i32 = 0x1000_0000;

pub(crate) struct NameGenerationScope {
    pub(crate) next: Option<Box<NameGenerationScope>>,
    pub(crate) temp_flags: i32,
    pub(crate) formatted_name_temp_flags: HashMap<String, i32>,
    pub(crate) reserved_names: HashSet<String>,
}
