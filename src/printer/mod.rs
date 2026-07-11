//! Printer module, ported from `internal/printer/`.
//!
//! Currently implements the `NameGenerator` for generating unique identifier
//! names during emit (temp variables, loop variables, unique names, node-based
//! generated names). The full printer (AST → text) is not yet ported; the
//! existing `emitter` module handles emit via source-text slicing.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::ast::{Node, SyntaxKind};

// ────────────────────────────────────────────────────────────────────────────
// GeneratedIdentifierFlags
// ────────────────────────────────────────────────────────────────────────────

/// Flags controlling generated identifier name generation.
/// Mirrors `printer.GeneratedIdentifierFlags` in Go.
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

    pub fn kind(self) -> Self {
        Self(self.0 & Self::KIND_MASK.0)
    }
    pub fn is_auto(self) -> bool {
        self.kind() == Self::AUTO
    }
    pub fn is_loop(self) -> bool {
        self.kind() == Self::LOOP
    }
    pub fn is_unique(self) -> bool {
        self.kind() == Self::UNIQUE
    }
    pub fn is_node(self) -> bool {
        self.kind() == Self::NODE
    }
    pub fn is_reserved_in_nested_scopes(self) -> bool {
        self.0 & Self::RESERVED_IN_NESTED_SCOPES.0 != 0
    }
    pub fn is_optimistic(self) -> bool {
        self.0 & Self::OPTIMISTIC.0 != 0
    }
    pub fn is_file_level(self) -> bool {
        self.0 & Self::FILE_LEVEL.0 != 0
    }
}

impl std::ops::BitOr for GeneratedIdentifierFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for GeneratedIdentifierFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitOrAssign for GeneratedIdentifierFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::Not for GeneratedIdentifierFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// AutoGenerateInfo
// ────────────────────────────────────────────────────────────────────────────

/// Options for creating a generated name.
/// Mirrors `printer.AutoGenerateOptions` in Go.
#[derive(Debug, Clone, Default)]
pub struct AutoGenerateOptions {
    pub flags: GeneratedIdentifierFlags,
    pub prefix: String,
    pub suffix: String,
}

/// Unique ID for tracking auto-generated names.
pub type AutoGenerateId = u32;

static NEXT_AUTO_GENERATE_ID: AtomicU32 = AtomicU32::new(0);

fn next_auto_generate_id() -> AutoGenerateId {
    NEXT_AUTO_GENERATE_ID.fetch_add(1, Ordering::Relaxed) + 1
}

/// Information about how to generate a name for an identifier.
/// Mirrors `printer.AutoGenerateInfo` in Go.
#[derive(Debug, Clone)]
pub struct AutoGenerateInfo {
    pub flags: GeneratedIdentifierFlags,
    pub id: AutoGenerateId,
    pub prefix: String,
    pub suffix: String,
    pub node: Option<Arc<Node>>,
}

// ────────────────────────────────────────────────────────────────────────────
// GeneratedName — stands in for Go's *ast.MemberName with autoGenerate info
// ────────────────────────────────────────────────────────────────────────────

/// A generated identifier name, carrying its text and auto-generation info.
///
/// In Go, this is an `*ast.IdentifierNode` or `*ast.PrivateIdentifierNode`
/// with an entry in the `EmitContext.autoGenerate` map. In Rust, we bundle
/// the text and auto-generate info together for simplicity.
#[derive(Debug, Clone)]
pub struct GeneratedName {
    pub text: String,
    pub auto_generate: AutoGenerateInfo,
    pub is_private: bool,
}

impl GeneratedName {
    pub fn new(text: String, is_private: bool, auto_generate: AutoGenerateInfo) -> Self {
        Self {
            text,
            is_private,
            auto_generate,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

// ────────────────────────────────────────────────────────────────────────────
// EmitContext
// ────────────────────────────────────────────────────────────────────────────

/// The context for emit operations, tracking auto-generation state.
/// Mirrors `printer.EmitContext` in Go (reduced form).
#[derive(Default)]
pub struct EmitContext {
    next_id: AtomicU32,
}

impl EmitContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_auto_generate_id(&self) -> AutoGenerateId {
        self.next_id.fetch_add(1, Ordering::Relaxed) + 1
    }
}

// ────────────────────────────────────────────────────────────────────────────
// NodeFactory
// ────────────────────────────────────────────────────────────────────────────

/// Factory for creating generated names.
/// Mirrors `printer.NodeFactory` (generated-name methods only) in Go.
pub struct NodeFactory<'a> {
    emit_context: &'a EmitContext,
}

impl<'a> NodeFactory<'a> {
    pub fn new(emit_context: &'a EmitContext) -> Self {
        Self { emit_context }
    }

    fn new_generated_identifier(
        &self,
        kind: GeneratedIdentifierFlags,
        text: &str,
        node: Option<Arc<Node>>,
        options: AutoGenerateOptions,
    ) -> GeneratedName {
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

    fn new_generated_private_identifier(
        &self,
        kind: GeneratedIdentifierFlags,
        text: &str,
        node: Option<Arc<Node>>,
        options: AutoGenerateOptions,
    ) -> GeneratedName {
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

    pub fn new_temp_variable(&self) -> GeneratedName {
        self.new_temp_variable_ex(AutoGenerateOptions::default())
    }

    pub fn new_temp_variable_ex(&self, options: AutoGenerateOptions) -> GeneratedName {
        self.new_generated_identifier(GeneratedIdentifierFlags::AUTO, "", None, options)
    }

    pub fn new_loop_variable(&self) -> GeneratedName {
        self.new_loop_variable_ex(AutoGenerateOptions::default())
    }

    pub fn new_loop_variable_ex(&self, options: AutoGenerateOptions) -> GeneratedName {
        self.new_generated_identifier(GeneratedIdentifierFlags::LOOP, "", None, options)
    }

    pub fn new_unique_name(&self, text: &str) -> GeneratedName {
        self.new_unique_name_ex(text, AutoGenerateOptions::default())
    }

    pub fn new_unique_name_ex(&self, text: &str, options: AutoGenerateOptions) -> GeneratedName {
        self.new_generated_identifier(GeneratedIdentifierFlags::UNIQUE, text, None, options)
    }

    pub fn new_generated_name_for_node(&self, node: &Arc<Node>) -> GeneratedName {
        self.new_generated_name_for_node_ex(node, AutoGenerateOptions::default())
    }

    pub fn new_generated_name_for_node_ex(
        &self,
        node: &Arc<Node>,
        mut options: AutoGenerateOptions,
    ) -> GeneratedName {
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

    pub fn new_unique_private_name(&self, text: &str) -> GeneratedName {
        self.new_unique_private_name_ex(text, AutoGenerateOptions::default())
    }

    pub fn new_unique_private_name_ex(
        &self,
        text: &str,
        options: AutoGenerateOptions,
    ) -> GeneratedName {
        self.new_generated_private_identifier(GeneratedIdentifierFlags::UNIQUE, text, None, options)
    }

    pub fn new_generated_private_name_for_node(&self, node: &Arc<Node>) -> GeneratedName {
        self.new_generated_private_name_for_node_ex(node, AutoGenerateOptions::default())
    }

    pub fn new_generated_private_name_for_node_ex(
        &self,
        node: &Arc<Node>,
        mut options: AutoGenerateOptions,
    ) -> GeneratedName {
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

// ────────────────────────────────────────────────────────────────────────────
// NameGenerator
// ────────────────────────────────────────────────────────────────────────────

const TEMP_FLAGS_AUTO: i32 = 0x0000_0000;
const TEMP_FLAGS_COUNT_MASK: i32 = 0x0FFF_FFFF;
const TEMP_FLAGS_I: i32 = 0x1000_0000;

struct NameGenerationScope {
    next: Option<Box<NameGenerationScope>>,
    temp_flags: i32,
    formatted_name_temp_flags: HashMap<String, i32>,
    reserved_names: HashSet<String>,
}

impl NameGenerationScope {
    fn new() -> Self {
        Self {
            next: None,
            temp_flags: TEMP_FLAGS_AUTO,
            formatted_name_temp_flags: HashMap::new(),
            reserved_names: HashSet::new(),
        }
    }
}

/// Generates unique identifier names during emit.
/// Mirrors `printer.NameGenerator` in Go.
pub struct NameGenerator {
    node_id_to_generated_name: HashMap<u64, String>,
    node_id_to_generated_private_name: HashMap<u64, String>,
    auto_generated_id_to_generated_name: HashMap<AutoGenerateId, String>,
    name_generation_scope: Option<Box<NameGenerationScope>>,
    private_name_generation_scope: Option<Box<NameGenerationScope>>,
    generated_names: HashSet<String>,
    get_text_of_node: Box<dyn Fn(&Node) -> String>,
    is_unique_local_name: Option<Box<dyn Fn(&str, &Node) -> bool>>,
}

impl NameGenerator {
    pub fn new() -> Self {
        Self {
            node_id_to_generated_name: HashMap::new(),
            node_id_to_generated_private_name: HashMap::new(),
            auto_generated_id_to_generated_name: HashMap::new(),
            name_generation_scope: None,
            private_name_generation_scope: None,
            generated_names: HashSet::new(),
            get_text_of_node: Box::new(|n: &Node| n.text().to_string()),
            is_unique_local_name: None,
        }
    }

    pub fn with_get_text_of_node<F>(mut self, f: F) -> Self
    where
        F: Fn(&Node) -> String + 'static,
    {
        self.get_text_of_node = Box::new(f);
        self
    }

    pub fn with_is_unique_local_name<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &Node) -> bool + 'static,
    {
        self.is_unique_local_name = Some(Box::new(f));
        self
    }

    pub fn push_scope(&mut self, reuse_temp_variable_scope: bool) {
        self.private_name_generation_scope = Some(Box::new(NameGenerationScope {
            next: self.private_name_generation_scope.take(),
            ..NameGenerationScope::new()
        }));
        if !reuse_temp_variable_scope {
            self.name_generation_scope = Some(Box::new(NameGenerationScope {
                next: self.name_generation_scope.take(),
                ..NameGenerationScope::new()
            }));
        }
    }

    pub fn pop_scope(&mut self, reuse_temp_variable_scope: bool) {
        if let Some(scope) = self.private_name_generation_scope.take() {
            self.private_name_generation_scope = scope.next;
        }
        if !reuse_temp_variable_scope {
            if let Some(scope) = self.name_generation_scope.take() {
                self.name_generation_scope = scope.next;
            }
        }
    }

    fn get_scope_mut(&mut self, private_name: bool) -> &mut Option<Box<NameGenerationScope>> {
        if private_name {
            &mut self.private_name_generation_scope
        } else {
            &mut self.name_generation_scope
        }
    }

    fn get_temp_flags(&self, private_name: bool) -> i32 {
        let scope = if private_name {
            &self.private_name_generation_scope
        } else {
            &self.name_generation_scope
        };
        scope.as_ref().map_or(TEMP_FLAGS_AUTO, |s| s.temp_flags)
    }

    fn set_temp_flags(&mut self, private_name: bool, flags: i32) {
        let scope = self.get_scope_mut(private_name);
        if scope.is_none() {
            *scope = Some(Box::new(NameGenerationScope::new()));
        }
        scope.as_mut().unwrap().temp_flags = flags;
    }

    fn get_temp_flags_for_formatted_name(&self, private_name: bool, key: &str) -> i32 {
        let scope = if private_name {
            &self.private_name_generation_scope
        } else {
            &self.name_generation_scope
        };
        scope
            .as_ref()
            .and_then(|s| s.formatted_name_temp_flags.get(key).copied())
            .unwrap_or(TEMP_FLAGS_AUTO)
    }

    fn set_temp_flags_for_formatted_name(&mut self, private_name: bool, key: String, flags: i32) {
        let scope = self.get_scope_mut(private_name);
        if scope.is_none() {
            *scope = Some(Box::new(NameGenerationScope::new()));
        }
        scope
            .as_mut()
            .unwrap()
            .formatted_name_temp_flags
            .insert(key, flags);
    }

    fn reserve_name(&mut self, name: &str, private_name: bool, scoped: bool, temp: bool) {
        if private_name || scoped {
            let scope = self.get_scope_mut(private_name);
            if scope.is_none() {
                *scope = Some(Box::new(NameGenerationScope::new()));
            }
            scope
                .as_mut()
                .unwrap()
                .reserved_names
                .insert(name.to_string());
        } else if !temp {
            self.generated_names.insert(name.to_string());
        }
    }

    fn is_reserved_name(&self, name: &str, private_name: bool) -> bool {
        if self.generated_names.contains(name) {
            return true;
        }
        let mut scope = if private_name {
            &self.private_name_generation_scope
        } else {
            &self.name_generation_scope
        };
        while let Some(s) = scope {
            if s.reserved_names.contains(name) {
                return true;
            }
            scope = &s.next;
        }
        false
    }

    fn is_unique_name(&self, name: &str, private_name: bool) -> bool {
        !self.is_reserved_name(name, private_name)
    }

    fn check_unique_name(&self, name: &str, private_name: bool) -> bool {
        self.is_unique_name(name, private_name)
    }

    /// Generate the text for a generated identifier.
    pub fn generate_name(&mut self, name: &GeneratedName) -> String {
        let auto_generate = &name.auto_generate;
        if auto_generate.flags.is_node() {
            self.generate_name_for_node_cached(
                auto_generate.node.as_ref(),
                name.is_private,
                auto_generate.flags,
                &auto_generate.prefix,
                &auto_generate.suffix,
            )
        } else {
            if let Some(cached) = self
                .auto_generated_id_to_generated_name
                .get(&auto_generate.id)
            {
                return cached.clone();
            }
            let generated = self.make_name(name);
            self.auto_generated_id_to_generated_name
                .insert(auto_generate.id, generated.clone());
            generated
        }
    }

    fn generate_name_for_node_cached(
        &mut self,
        node: Option<&Arc<Node>>,
        private_name: bool,
        flags: GeneratedIdentifierFlags,
        prefix: &str,
        suffix: &str,
    ) -> String {
        let node = node.expect("node-based name requires a node");
        let node_id = node.id();
        if private_name {
            if let Some(cached) = self.node_id_to_generated_private_name.get(&node_id) {
                return cached.clone();
            }
        } else {
            if let Some(cached) = self.node_id_to_generated_name.get(&node_id) {
                return cached.clone();
            }
        }
        let generated = self.generate_name_for_node(node, private_name, flags, prefix, suffix);
        if private_name {
            self.node_id_to_generated_private_name
                .insert(node_id, generated.clone());
        } else {
            self.node_id_to_generated_name
                .insert(node_id, generated.clone());
        }
        generated
    }

    fn generate_name_for_node(
        &mut self,
        node: &Arc<Node>,
        private_name: bool,
        flags: GeneratedIdentifierFlags,
        prefix: &str,
        suffix: &str,
    ) -> String {
        match node.kind {
            SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier => {
                let text = (self.get_text_of_node)(node);
                self.make_unique_name(
                    &text,
                    flags.is_optimistic(),
                    flags.is_reserved_in_nested_scopes(),
                    private_name,
                    prefix,
                    suffix,
                )
            }
            SyntaxKind::ModuleDeclaration | SyntaxKind::EnumDeclaration => {
                assert!(
                    !private_name && prefix.is_empty() && suffix.is_empty(),
                    "Generated name for a module or enum cannot be private and may have neither a prefix nor suffix"
                );
                self.generate_name_for_module_or_enum(node)
            }
            SyntaxKind::ImportDeclaration | SyntaxKind::ExportDeclaration => {
                assert!(
                    !private_name && prefix.is_empty() && suffix.is_empty(),
                    "Generated name for an import or export cannot be private and may have neither a prefix nor suffix"
                );
                self.generate_name_for_import_or_export_declaration(node)
            }
            SyntaxKind::FunctionDeclaration | SyntaxKind::ClassDeclaration => {
                assert!(
                    !private_name && prefix.is_empty() && suffix.is_empty(),
                    "Generated name for a class or function declaration cannot be private and may have neither a prefix nor suffix"
                );
                if let Some(name) = node.name() {
                    return self.generate_name_for_node(name, false, flags, "", "");
                }
                self.generate_name_for_export_default()
            }
            SyntaxKind::ExportAssignment => {
                assert!(
                    !private_name && prefix.is_empty() && suffix.is_empty(),
                    "Generated name for an export assignment cannot be private and may have neither a prefix nor suffix"
                );
                self.generate_name_for_export_default()
            }
            SyntaxKind::ClassExpression => {
                assert!(
                    !private_name && prefix.is_empty() && suffix.is_empty(),
                    "Generated name for a class expression cannot be private and may have neither a prefix nor suffix"
                );
                self.make_unique_name("class", false, false, false, "", "")
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                self.generate_name_for_method_or_accessor(node, private_name, prefix, suffix)
            }
            SyntaxKind::ComputedPropertyName => {
                self.make_temp_variable_name(TEMP_FLAGS_AUTO, true, private_name, prefix, suffix)
            }
            _ => self.make_temp_variable_name(TEMP_FLAGS_AUTO, false, private_name, prefix, suffix),
        }
    }

    fn generate_name_for_module_or_enum(&mut self, node: &Arc<Node>) -> String {
        let name_node = node.name().expect("module/enum must have a name");
        let name = (self.get_text_of_node)(name_node);
        if let Some(ref check) = self.is_unique_local_name {
            if check(&name, node) {
                self.reserve_name(&name, false, false, false);
                return name;
            }
        }
        self.make_unique_name(&name, false, false, false, "", "")
    }

    fn generate_name_for_import_or_export_declaration(&mut self, node: &Arc<Node>) -> String {
        let base_name = get_external_module_name(node)
            .map(|s| make_identifier_from_module_name(&s))
            .unwrap_or_else(|| "module".to_string());
        self.make_unique_name(&base_name, false, false, false, "", "")
    }

    fn generate_name_for_export_default(&mut self) -> String {
        self.make_unique_name("default", false, false, false, "", "")
    }

    fn generate_name_for_method_or_accessor(
        &mut self,
        node: &Arc<Node>,
        private_name: bool,
        prefix: &str,
        suffix: &str,
    ) -> String {
        if let Some(name) = node.name() {
            if name.kind == SyntaxKind::Identifier {
                return self.generate_name_for_node_cached(
                    Some(name),
                    private_name,
                    GeneratedIdentifierFlags::NONE,
                    prefix,
                    suffix,
                );
            }
        }
        self.make_temp_variable_name(TEMP_FLAGS_AUTO, false, private_name, prefix, suffix)
    }

    fn make_name(&mut self, name: &GeneratedName) -> String {
        let auto_generate = &name.auto_generate;
        match auto_generate.flags.kind() {
            GeneratedIdentifierFlags::AUTO => self.make_temp_variable_name(
                TEMP_FLAGS_AUTO,
                auto_generate.flags.is_reserved_in_nested_scopes(),
                name.is_private,
                &auto_generate.prefix,
                &auto_generate.suffix,
            ),
            GeneratedIdentifierFlags::LOOP => self.make_temp_variable_name(
                TEMP_FLAGS_I,
                auto_generate.flags.is_reserved_in_nested_scopes(),
                false,
                &auto_generate.prefix,
                &auto_generate.suffix,
            ),
            GeneratedIdentifierFlags::UNIQUE => self.make_unique_name(
                name.text(),
                auto_generate.flags.is_optimistic(),
                auto_generate.flags.is_reserved_in_nested_scopes(),
                name.is_private,
                &auto_generate.prefix,
                &auto_generate.suffix,
            ),
            _ => name.text().to_string(),
        }
    }

    fn make_temp_variable_name(
        &mut self,
        flags: i32,
        reserved_in_nested_scopes: bool,
        private_name: bool,
        prefix: &str,
        suffix: &str,
    ) -> String {
        let simple = prefix.is_empty() && suffix.is_empty();
        let key = if simple {
            String::new()
        } else {
            let k = format_generated_name(private_name, prefix, "", suffix);
            if private_name {
                ensure_leading_hash(&k)
            } else {
                k
            }
        };

        let mut temp_flags = if simple {
            self.get_temp_flags(private_name)
        } else {
            self.get_temp_flags_for_formatted_name(private_name, &key)
        };

        if flags != 0 && temp_flags & flags == 0 {
            let full_name = format_generated_name(private_name, prefix, "_i", suffix);
            if self.is_unique_name(&full_name, private_name) {
                temp_flags |= flags;
                self.reserve_name(&full_name, private_name, reserved_in_nested_scopes, true);
                if simple {
                    self.set_temp_flags(private_name, temp_flags);
                } else {
                    self.set_temp_flags_for_formatted_name(private_name, key, temp_flags);
                }
                return full_name;
            }
        }

        loop {
            let count = temp_flags & TEMP_FLAGS_COUNT_MASK;
            temp_flags += 1;
            if count != 8 && count != 13 {
                let name = if count < 26 {
                    format!("_{}", (b'a' + count as u8) as char)
                } else {
                    format!("_{}", count - 26)
                };
                let full_name = format_generated_name(private_name, prefix, &name, suffix);
                if self.is_unique_name(&full_name, private_name) {
                    self.reserve_name(&full_name, private_name, reserved_in_nested_scopes, true);
                    if simple {
                        self.set_temp_flags(private_name, temp_flags);
                    } else {
                        self.set_temp_flags_for_formatted_name(private_name, key, temp_flags);
                    }
                    return full_name;
                }
            }
        }
    }

    fn make_unique_name(
        &mut self,
        base_name: &str,
        optimistic: bool,
        scoped: bool,
        private_name: bool,
        prefix: &str,
        suffix: &str,
    ) -> String {
        let base_name = remove_leading_hash(base_name);
        if optimistic {
            let full_name = format_generated_name(private_name, prefix, &base_name, suffix);
            if self.check_unique_name(&full_name, private_name) {
                self.reserve_name(&full_name, private_name, scoped, false);
                return full_name;
            }
        }

        let mut base_name = base_name.to_string();
        if !base_name.is_empty() && !base_name.ends_with('_') {
            base_name.push('_');
        }

        let mut i = 1;
        loop {
            let full_name =
                format_generated_name(private_name, prefix, &format!("{base_name}{i}"), suffix);
            if self.check_unique_name(&full_name, private_name) {
                self.reserve_name(&full_name, private_name, scoped, false);
                return full_name;
            }
            i += 1;
        }
    }
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Helper functions
// ────────────────────────────────────────────────────────────────────────────

fn has_leading_hash(text: &str) -> bool {
    text.starts_with('#')
}

fn remove_leading_hash(text: &str) -> &str {
    if has_leading_hash(text) {
        &text[1..]
    } else {
        text
    }
}

fn ensure_leading_hash(text: &str) -> String {
    if has_leading_hash(text) {
        text.to_string()
    } else {
        format!("#{text}")
    }
}

fn format_generated_name(private_name: bool, prefix: &str, base: &str, suffix: &str) -> String {
    let name = format!(
        "{}{}{}",
        remove_leading_hash(prefix),
        remove_leading_hash(base),
        remove_leading_hash(suffix)
    );
    if private_name {
        ensure_leading_hash(&name)
    } else {
        name
    }
}

fn make_identifier_from_module_name(module_name: &str) -> String {
    let base = crate::tspath::get_base_file_name(module_name);
    let mut result = String::new();
    let bytes = base.as_bytes();
    let mut start = 0;
    let mut pos = 0;
    while pos < bytes.len() {
        let ch = bytes[pos] as char;
        if pos == 0 && ch.is_ascii_digit() {
            result.push('_');
        } else if !is_ascii_word_character(ch) {
            if start < pos {
                result.push_str(&base[start..pos]);
            }
            result.push('_');
            start = pos + 1;
        }
        pos += 1;
    }
    if start < pos {
        result.push_str(&base[start..pos]);
    }
    if result.chars().last().map(|c| c == '_').unwrap_or(false) {
        result.pop();
    }
    result
}

fn is_ascii_word_character(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch.is_ascii_digit() || ch == '_'
}

/// Get the module specifier (string literal) from an import/export declaration.
fn get_external_module_name(node: &Arc<Node>) -> Option<String> {
    match &node.data {
        crate::ast::node_data_generated::NodeData::ImportDeclaration(d) => {
            Some(d.module_specifier.text().to_string())
        }
        crate::ast::node_data_generated::NodeData::ExportDeclaration(d) => {
            d.module_specifier.as_ref().map(|n| n.text().to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::node_data_generated::{IdentifierData, NodeData};
    use crate::ast::symbol::{NodeSymbolMap, SymbolFlags};
    use crate::binder::Binder;
    use crate::parser::Parser;

    fn parse(source: &str) -> Arc<crate::ast::SourceFile> {
        let (file, _diags) =
            Parser::parse_source_file_text_with_diagnostics("test.ts", source.to_string());
        Arc::new(file)
    }

    fn parse_and_bind(source: &str) -> (Arc<crate::ast::SourceFile>, NodeSymbolMap) {
        let file = parse(source);
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        let symbol_map = std::mem::take(&mut binder.symbol_map);
        (file, symbol_map)
    }

    /// Create an `is_unique_local_name` callback that checks the binder's locals.
    fn make_is_unique_local_name(symbol_map: Arc<NodeSymbolMap>) -> impl Fn(&str, &Node) -> bool {
        move |name: &str, node: &Node| -> bool {
            let mask = SymbolFlags::VALUE | SymbolFlags::ExportValue | SymbolFlags::Alias;
            if let Some(locals) = symbol_map.locals_of(node) {
                if let Some(sym) = locals.get(name) {
                    if sym.flags & mask != SymbolFlags::empty() {
                        return false;
                    }
                }
            }
            true
        }
    }

    /// Create a simple Identifier node with the given text.
    fn make_identifier(text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    // ── TempVariable tests ─────────────────────────────────────────────────

    #[test]
    fn temp_variable_1() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_temp_variable();
        let name2 = factory.new_temp_variable();
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "_a");
        assert_eq!(g.generate_name(&name2), "_b");
    }

    #[test]
    fn temp_variable_2() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_temp_variable_ex(AutoGenerateOptions {
            prefix: "A".to_string(),
            suffix: "B".to_string(),
            ..Default::default()
        });
        let name2 = factory.new_temp_variable_ex(AutoGenerateOptions {
            prefix: "A".to_string(),
            suffix: "B".to_string(),
            ..Default::default()
        });
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "A_aB");
        assert_eq!(g.generate_name(&name2), "A_bB");
    }

    #[test]
    fn temp_variable_3() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_temp_variable();
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "_a");
        assert_eq!(g.generate_name(&name1), "_a");
    }

    #[test]
    fn temp_variable_scoped() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_temp_variable();
        let name2 = factory.new_temp_variable();
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "_a");
        g.push_scope(false);
        assert_eq!(g.generate_name(&name2), "_a");
        g.pop_scope(false);
    }

    #[test]
    fn temp_variable_scoped_reserved() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_temp_variable_ex(AutoGenerateOptions {
            flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
            ..Default::default()
        });
        let name2 = factory.new_temp_variable();
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "_a");
        g.push_scope(false);
        assert_eq!(g.generate_name(&name2), "_b");
        g.pop_scope(false);
    }

    // ── LoopVariable tests ─────────────────────────────────────────────────

    #[test]
    fn loop_variable_1() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_loop_variable();
        let name2 = factory.new_loop_variable();
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "_i");
        assert_eq!(g.generate_name(&name2), "_a");
    }

    #[test]
    fn loop_variable_2() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_loop_variable_ex(AutoGenerateOptions {
            prefix: "A".to_string(),
            suffix: "B".to_string(),
            ..Default::default()
        });
        let name2 = factory.new_loop_variable_ex(AutoGenerateOptions {
            prefix: "A".to_string(),
            suffix: "B".to_string(),
            ..Default::default()
        });
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "A_iB");
        assert_eq!(g.generate_name(&name2), "A_aB");
    }

    #[test]
    fn loop_variable_3() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_loop_variable();
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "_i");
        assert_eq!(g.generate_name(&name1), "_i");
    }

    #[test]
    fn loop_variable_scoped() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_loop_variable();
        let name2 = factory.new_loop_variable();
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "_i");
        g.push_scope(false);
        assert_eq!(g.generate_name(&name2), "_i");
        g.pop_scope(false);
    }

    // ── UniqueName tests ───────────────────────────────────────────────────

    #[test]
    fn unique_name_1() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_unique_name("foo");
        let name2 = factory.new_unique_name("foo");
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "foo_1");
        assert_eq!(g.generate_name(&name2), "foo_2");
    }

    #[test]
    fn unique_name_2() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_unique_name("foo");
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "foo_1");
        assert_eq!(g.generate_name(&name1), "foo_1");
    }

    #[test]
    fn unique_name_scoped() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_unique_name("foo");
        let name2 = factory.new_unique_name("foo");
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "foo_1");
        g.push_scope(false);
        assert_eq!(g.generate_name(&name2), "foo_2");
        g.pop_scope(false);
    }

    // ── UniquePrivateName tests ────────────────────────────────────────────

    #[test]
    fn unique_private_name_1() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_unique_private_name("#foo");
        let name2 = factory.new_unique_private_name("#foo");
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "#foo_1");
        assert_eq!(g.generate_name(&name2), "#foo_2");
    }

    #[test]
    fn unique_private_name_2() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_unique_private_name("#foo");
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "#foo_1");
        assert_eq!(g.generate_name(&name1), "#foo_1");
    }

    #[test]
    fn unique_private_name_scoped() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let name1 = factory.new_unique_private_name("#foo");
        let name2 = factory.new_unique_private_name("#foo");
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "#foo_1");
        g.push_scope(false);
        assert_eq!(g.generate_name(&name2), "#foo_2");
        g.pop_scope(false);
    }

    // ── GeneratedNameForIdentifier tests ───────────────────────────────────

    #[test]
    fn generated_name_for_identifier_1() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("function f() {}");
        let stmt = &file.node.data;
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = stmt else {
            panic!("expected SourceFile");
        };
        let func = &d.statements.nodes[0];
        let n = func.name().unwrap();
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "f_1");
    }

    #[test]
    fn generated_name_for_identifier_2() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("function f() {}");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let func = &d.statements.nodes[0];
        let n = func.name().unwrap();
        let name1 = factory.new_generated_name_for_node_ex(
            n,
            AutoGenerateOptions {
                prefix: "a".to_string(),
                suffix: "b".to_string(),
                ..Default::default()
            },
        );
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "afb");
    }

    #[test]
    fn generated_name_for_identifier_3() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("function f() {}");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let func = &d.statements.nodes[0];
        let n = func.name().unwrap();
        let _name1 = factory.new_generated_name_for_node_ex(
            n,
            AutoGenerateOptions {
                prefix: "a".to_string(),
                suffix: "b".to_string(),
                ..Default::default()
            },
        );
        // In Go, name2 is created from name1 (a generated Identifier node with text "afb").
        // In Rust, GeneratedName is not a Node, so we create an Identifier node with text "afb".
        let afb_node = make_identifier("afb");
        let name2 = factory.new_generated_name_for_node(&afb_node);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name2), "afb_1");
    }

    // ── GeneratedNameForNamespace tests ────────────────────────────────────

    #[test]
    fn generated_name_for_namespace_1() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, symbol_map) = parse_and_bind("namespace foo { }");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let ns1 = &d.statements.nodes[0];
        let name1 = factory.new_generated_name_for_node(ns1);
        let mut g = NameGenerator::new()
            .with_is_unique_local_name(make_is_unique_local_name(Arc::new(symbol_map)));
        assert_eq!(g.generate_name(&name1), "foo");
    }

    #[test]
    fn generated_name_for_namespace_2() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, symbol_map) = parse_and_bind("namespace foo { var foo; }");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let ns1 = &d.statements.nodes[0];
        let name1 = factory.new_generated_name_for_node(ns1);
        let mut g = NameGenerator::new()
            .with_is_unique_local_name(make_is_unique_local_name(Arc::new(symbol_map)));
        assert_eq!(g.generate_name(&name1), "foo_1");
    }

    #[test]
    fn generated_name_for_namespace_3() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, symbol_map) = parse_and_bind(
            "namespace ns1 { namespace foo { var foo; } } namespace ns2 { namespace foo { var foo; } }",
        );
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let ns1_outer = &d.statements.nodes[0];
        let ns2_outer = &d.statements.nodes[1];
        let crate::ast::node_data_generated::NodeData::ModuleDeclaration(ns1_data) =
            &ns1_outer.data
        else {
            panic!("expected ModuleDeclaration");
        };
        let ns1_body = ns1_data.body.as_ref().unwrap();
        let inner_ns1 = get_module_block_statements(ns1_body).unwrap()[0].clone();
        let name1 = factory.new_generated_name_for_node(&inner_ns1);

        let crate::ast::node_data_generated::NodeData::ModuleDeclaration(ns2_data) =
            &ns2_outer.data
        else {
            panic!("expected ModuleDeclaration");
        };
        let ns2_body = ns2_data.body.as_ref().unwrap();
        let inner_ns2 = get_module_block_statements(ns2_body).unwrap()[0].clone();
        let name2 = factory.new_generated_name_for_node(&inner_ns2);

        let mut g = NameGenerator::new()
            .with_is_unique_local_name(make_is_unique_local_name(Arc::new(symbol_map)));
        assert_eq!(g.generate_name(&name1), "foo_1");
        assert_eq!(g.generate_name(&name2), "foo_2");
    }

    #[test]
    fn generated_name_for_namespace_4() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, symbol_map) = parse_and_bind(
            "namespace ns1 { namespace foo { var foo; } } namespace ns2 { namespace foo { var foo; } }",
        );
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let ns1_outer = &d.statements.nodes[0];
        let ns2_outer = &d.statements.nodes[1];
        let crate::ast::node_data_generated::NodeData::ModuleDeclaration(ns1_data) =
            &ns1_outer.data
        else {
            panic!("expected ModuleDeclaration");
        };
        let ns1_body = ns1_data.body.as_ref().unwrap();
        let inner_ns1 = get_module_block_statements(ns1_body).unwrap()[0].clone();
        let name1 = factory.new_generated_name_for_node(&inner_ns1);

        let crate::ast::node_data_generated::NodeData::ModuleDeclaration(ns2_data) =
            &ns2_outer.data
        else {
            panic!("expected ModuleDeclaration");
        };
        let ns2_body = ns2_data.body.as_ref().unwrap();
        let inner_ns2 = get_module_block_statements(ns2_body).unwrap()[0].clone();
        let name2 = factory.new_generated_name_for_node(&inner_ns2);

        let mut g = NameGenerator::new()
            .with_is_unique_local_name(make_is_unique_local_name(Arc::new(symbol_map)));
        g.push_scope(false);
        let text1 = g.generate_name(&name1);
        g.pop_scope(false);
        g.push_scope(false);
        let text2 = g.generate_name(&name2);
        g.pop_scope(false);
        assert_eq!(text1, "foo_1");
        assert_eq!(text2, "foo_2");
    }

    // ── GeneratedNameForNodeCached ─────────────────────────────────────────

    #[test]
    fn generated_name_for_node_cached() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, symbol_map) = parse_and_bind("namespace foo { var foo; }");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let ns1 = &d.statements.nodes[0];
        let name1 = factory.new_generated_name_for_node(ns1);
        let name2 = factory.new_generated_name_for_node(ns1);
        let mut g = NameGenerator::new()
            .with_is_unique_local_name(make_is_unique_local_name(Arc::new(symbol_map)));
        assert_eq!(g.generate_name(&name1), "foo_1");
        assert_eq!(g.generate_name(&name2), "foo_1");
    }

    // ── GeneratedNameForImport/Export ──────────────────────────────────────

    #[test]
    fn generated_name_for_import() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("import * as foo from 'foo'");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let n = &d.statements.nodes[0];
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "foo_1");
    }

    #[test]
    fn generated_name_for_export() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("export * as foo from 'foo'");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let n = &d.statements.nodes[0];
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "foo_1");
    }

    // ── GeneratedNameForFunctionDeclaration ────────────────────────────────

    #[test]
    fn generated_name_for_function_declaration_1() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("export function f() {}");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let n = &d.statements.nodes[0];
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "f_1");
    }

    #[test]
    fn generated_name_for_function_declaration_2() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("export default function () {}");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let n = &d.statements.nodes[0];
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "default_1");
    }

    // ── GeneratedNameForClassDeclaration ───────────────────────────────────

    #[test]
    fn generated_name_for_class_declaration_1() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("export class C {}");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let n = &d.statements.nodes[0];
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "C_1");
    }

    #[test]
    fn generated_name_for_class_declaration_2() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("export default class {}");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let n = &d.statements.nodes[0];
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "default_1");
    }

    // ── GeneratedNameForExportAssignment ───────────────────────────────────

    #[test]
    fn generated_name_for_export_assignment() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("export default 0");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let n = &d.statements.nodes[0];
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "default_1");
    }

    // ── GeneratedNameForClassExpression ────────────────────────────────────

    #[test]
    fn generated_name_for_class_expression() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("(class {})");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let stmt = &d.statements.nodes[0];
        let expr = stmt.expression().unwrap();
        let inner = expr.expression().unwrap();
        let name1 = factory.new_generated_name_for_node(inner);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "class_1");
    }

    // ── GeneratedNameForMethod tests ───────────────────────────────────────

    #[test]
    fn generated_name_for_method_1() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("class C { m() {} }");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let class_node = &d.statements.nodes[0];
        let crate::ast::node_data_generated::NodeData::ClassDeclaration(class_data) =
            &class_node.data
        else {
            panic!("expected ClassDeclaration");
        };
        let n = &class_data.members.nodes[0];
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "m_1");
    }

    #[test]
    fn generated_name_for_method_2() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("class C { 0() {} }");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let class_node = &d.statements.nodes[0];
        let crate::ast::node_data_generated::NodeData::ClassDeclaration(class_data) =
            &class_node.data
        else {
            panic!("expected ClassDeclaration");
        };
        let n = &class_data.members.nodes[0];
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "_a");
    }

    // ── GeneratedPrivateNameForMethod ──────────────────────────────────────

    #[test]
    fn generated_private_name_for_method() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("class C { m() {} }");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let class_node = &d.statements.nodes[0];
        let crate::ast::node_data_generated::NodeData::ClassDeclaration(class_data) =
            &class_node.data
        else {
            panic!("expected ClassDeclaration");
        };
        let n = &class_data.members.nodes[0];
        let name1 = factory.new_generated_private_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "#m_1");
    }

    // ── GeneratedNameForComputedPropertyName ───────────────────────────────

    #[test]
    fn generated_name_for_computed_property_name() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("class C { [x] }");
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let class_node = &d.statements.nodes[0];
        let crate::ast::node_data_generated::NodeData::ClassDeclaration(class_data) =
            &class_node.data
        else {
            panic!("expected ClassDeclaration");
        };
        let member = &class_data.members.nodes[0];
        let n = member.name().unwrap();
        let name1 = factory.new_generated_name_for_node(n);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "_a");
    }

    // ── GeneratedNameForOther ──────────────────────────────────────────────

    #[test]
    fn generated_name_for_other() {
        let ec = EmitContext::new();
        let factory = NodeFactory::new(&ec);
        let (file, _) = parse_and_bind("class C { [x] }");
        // Use a member node as a stand-in for a non-specific node
        let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        let class_node = &d.statements.nodes[0];
        let crate::ast::node_data_generated::NodeData::ClassDeclaration(class_data) =
            &class_node.data
        else {
            panic!("expected ClassDeclaration");
        };
        let member = &class_data.members.nodes[0];
        let name1 = factory.new_generated_name_for_node(member);
        let mut g = NameGenerator::new();
        assert_eq!(g.generate_name(&name1), "_a");
    }
}

/// Helper: get the statements from a ModuleBlock node.
fn get_module_block_statements(node: &Arc<Node>) -> Option<&[Arc<Node>]> {
    match &node.data {
        crate::ast::node_data_generated::NodeData::ModuleBlock(d) => Some(&d.statements.nodes),
        _ => None,
    }
}
