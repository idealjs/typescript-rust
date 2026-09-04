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
use crate::scanner::{CommentRange, CommentRangeKind};
use crate::stringutil;

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

#[allow(dead_code)]
static NEXT_AUTO_GENERATE_ID: AtomicU32 = AtomicU32::new(0);

#[allow(dead_code)]
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

// ────────────────────────────────────────────────────────────────────────────
// String escaping utilities (ported from printer/utilities.go)
// ────────────────────────────────────────────────────────────────────────────

/// Quote character used for string literal escaping.
/// Mirrors Go's `printer.QuoteChar`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteChar {
    SingleQuote,
    DoubleQuote,
    Backtick,
}

impl QuoteChar {
    fn as_char(self) -> char {
        match self {
            QuoteChar::SingleQuote => '\'',
            QuoteChar::DoubleQuote => '"',
            QuoteChar::Backtick => '`',
        }
    }
}

bitflags::bitflags! {
    /// Flags controlling how literal text is escaped.
    /// Mirrors Go's `printer.getLiteralTextFlags`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct GetLiteralTextFlags: u32 {
        const NONE = 0;
        const NEVER_ASCII_ESCAPE = 1 << 0;
        const JSX_ATTRIBUTE_ESCAPE = 1 << 1;
        const TERMINATE_UNTERMINATED_LITERALS = 1 << 2;
        const ALLOW_NUMERIC_SEPARATOR = 1 << 3;
    }
}

/// Encode a character as an XML character entity (e.g. `&#x9;`).
fn encode_jsx_character_entity(b: &mut String, ch: char) {
    b.push_str("&#x");
    b.push_str(&format!("{:X}", ch as u32));
    b.push(';');
}

/// Encode a character as a `\uXXXX` escape sequence.
fn encode_utf16_escape_sequence_u32(b: &mut String, code: u32) {
    let hex = format!("{:X}", code);
    b.push_str("\\u");
    for _ in 0..(4 - hex.len()) {
        b.push('0');
    }
    b.push_str(&hex);
}

fn encode_utf16_escape_sequence(b: &mut String, ch: char) {
    encode_utf16_escape_sequence_u32(b, ch as u32);
}

/// Lookup for JSX-escaped characters.
fn jsx_escaped_chars_map(code: u32) -> Option<&'static str> {
    match code {
        0x22 => Some("&quot;"),
        0x27 => Some("&apos;"),
        _ => None,
    }
}

/// Lookup for standard escaped characters.
fn escaped_chars_map(code: u32) -> Option<&'static str> {
    match code {
        0x09 => Some("\\t"),
        0x0b => Some("\\v"),
        0x0c => Some("\\f"),
        0x08 => Some("\\b"),
        0x0d => Some("\\r"),
        0x0a => Some("\\n"),
        0x5c => Some("\\\\"),
        0x22 => Some("\\\""),
        0x27 => Some("\\'"),
        0x60 => Some("\\`"),
        0x24 => Some("\\$"),
        0x2028 => Some("\\u2028"),
        0x2029 => Some("\\u2029"),
        0x0085 => Some("\\u0085"),
        _ => None,
    }
}

/// Escape a string for output. Mirrors Go's `escapeStringWorker`.
fn escape_string_worker(
    s: &str,
    quote_char: QuoteChar,
    flags: GetLiteralTextFlags,
    b: &mut String,
) {
    let bytes = s.as_bytes();
    let mut pos = 0usize;

    for (i, ch) in s.char_indices() {
        let code = ch as u32;
        let size = ch.len_utf8();
        let mut actual_size = size;
        let mut escape = false;

        if (0xD800..=0xDFFF).contains(&code) {
            escape = true;
        }
        // Rust strings are always valid UTF-8; no RuneError case needed.

        if !escape {
            if ch == '\\' {
                if !flags.contains(GetLiteralTextFlags::JSX_ATTRIBUTE_ESCAPE) {
                    escape = true;
                }
            } else if ch == '$'
                && quote_char == QuoteChar::Backtick
                && i + 1 < s.len()
                && bytes[i + 1] == b'{'
            {
                escape = true;
            } else if ch == quote_char.as_char()
                || matches!(ch, '\u{2028}' | '\u{2029}' | '\u{0085}' | '\r')
            {
                escape = true;
            } else if ch == '\n' {
                if quote_char != QuoteChar::Backtick {
                    escape = true;
                }
            } else if code <= 0x1f
                || (!flags.contains(GetLiteralTextFlags::NEVER_ASCII_ESCAPE) && code > 0x7f)
            {
                escape = true;
            }
        }

        if escape {
            if pos < i {
                b.push_str(&s[pos..i]);
            }

            if flags.contains(GetLiteralTextFlags::JSX_ATTRIBUTE_ESCAPE) {
                if code == 0 {
                    b.push_str("&#0;");
                } else if let Some(repl) = jsx_escaped_chars_map(code) {
                    b.push_str(repl);
                } else {
                    encode_jsx_character_entity(b, ch);
                }
            } else if ch == '\r'
                && quote_char == QuoteChar::Backtick
                && i + 1 < s.len()
                && bytes[i + 1] == b'\n'
            {
                actual_size += 1;
                b.push_str("\\r\\n");
            } else if code > 0xffff {
                let adjusted = code - 0x10000;
                encode_utf16_escape_sequence_u32(b, ((adjusted >> 10) & 0x3ff) + 0xd800);
                encode_utf16_escape_sequence_u32(b, (adjusted & 0x3ff) + 0xdc00);
            } else if (0xD800..=0xDFFF).contains(&code) {
                encode_utf16_escape_sequence(b, ch);
            } else if code == 0 {
                if i + 1 < s.len() && stringutil::is_digit(bytes[i + 1] as char) {
                    b.push_str("\\x00");
                } else {
                    b.push_str("\\0");
                }
            } else if let Some(repl) = escaped_chars_map(code) {
                b.push_str(repl);
            } else {
                encode_utf16_escape_sequence(b, ch);
            }

            pos = i + actual_size;
        }
    }

    if pos < s.len() {
        b.push_str(&s[pos..]);
    }
}

/// Escape a string, always escaping non-ASCII characters.
/// Mirrors Go's `printer.EscapeString`.
pub fn escape_string(s: &str, quote_char: QuoteChar) -> String {
    let mut b = String::with_capacity(s.len() + 2);
    escape_string_worker(
        s,
        quote_char,
        GetLiteralTextFlags::NEVER_ASCII_ESCAPE,
        &mut b,
    );
    b
}

/// Escape a string, preserving non-ASCII characters that don't need escaping.
/// Mirrors Go's `printer.escapeNonAsciiString`.
pub fn escape_non_ascii_string(s: &str, quote_char: QuoteChar) -> String {
    let mut b = String::with_capacity(s.len() + 2);
    escape_string_worker(s, quote_char, GetLiteralTextFlags::NONE, &mut b);
    b
}

/// Escape a string for use as a JSX attribute value.
/// Mirrors Go's `printer.escapeJsxAttributeString`.
pub fn escape_jsx_attribute_string(s: &str, quote_char: QuoteChar) -> String {
    let mut b = String::with_capacity(s.len() + 2);
    escape_string_worker(
        s,
        quote_char,
        GetLiteralTextFlags::JSX_ATTRIBUTE_ESCAPE | GetLiteralTextFlags::NEVER_ASCII_ESCAPE,
        &mut b,
    );
    b
}

// ────────────────────────────────────────────────────────────────────────────
// Triple-slash comment recognition (ported from printer/utilities.go)
// ────────────────────────────────────────────────────────────────────────────

fn decode_char_at(text: &str, pos: usize) -> (char, usize) {
    let c = text[pos..].chars().next().unwrap();
    (c, c.len_utf8())
}

fn skip_white_space_single_line(text: &str, pos: &mut usize) {
    while *pos < text.len() {
        let (ch, size) = decode_char_at(text, *pos);
        if !stringutil::is_white_space_single_line(ch) {
            break;
        }
        *pos += size;
    }
}

fn match_white_space_single_line(text: &str, pos: &mut usize) -> bool {
    let start = *pos;
    skip_white_space_single_line(text, pos);
    *pos != start
}

fn match_rune(text: &str, pos: &mut usize, expected: char) -> bool {
    if *pos < text.len() {
        let (ch, size) = decode_char_at(text, *pos);
        if ch == expected {
            *pos += size;
            return true;
        }
    }
    false
}

fn match_string(text: &str, pos: &mut usize, expected: &str) -> bool {
    let mut text_pos = *pos;
    for expected_ch in expected.chars() {
        if !match_rune(text, &mut text_pos, expected_ch) {
            return false;
        }
    }
    *pos = text_pos;
    true
}

fn match_quoted_string(text: &str, pos: &mut usize) -> bool {
    let mut text_pos = *pos;
    let quote_char = if match_rune(text, &mut text_pos, '\'') {
        '\''
    } else if match_rune(text, &mut text_pos, '"') {
        '"'
    } else {
        return false;
    };
    while text_pos < text.len() {
        let (ch, size) = decode_char_at(text, text_pos);
        text_pos += size;
        if ch == quote_char {
            *pos = text_pos;
            return true;
        }
    }
    false
}

/// Whether `text` at `comment_range` is a recognized triple-slash directive.
/// Mirrors Go's `printer.IsRecognizedTripleSlashComment`.
pub fn is_recognized_triple_slash_comment(text: &str, comment_range: &CommentRange) -> bool {
    if comment_range.kind == CommentRangeKind::SingleLine
        && comment_range.end - comment_range.pos > 2
        && text.as_bytes()[comment_range.pos + 1] == b'/'
        && text.as_bytes()[comment_range.pos + 2] == b'/'
    {
        let start = comment_range.pos + 3;
        let inner = &text[start..comment_range.end];
        let mut pos = 0;
        skip_white_space_single_line(inner, &mut pos);
        if !match_rune(inner, &mut pos, '<') {
            return false;
        }
        if match_string(inner, &mut pos, "reference") {
            if !match_white_space_single_line(inner, &mut pos) {
                return false;
            }
            if !match_string(inner, &mut pos, "path")
                && !match_string(inner, &mut pos, "types")
                && !match_string(inner, &mut pos, "lib")
                && !match_string(inner, &mut pos, "no-default-lib")
            {
                return false;
            }
            skip_white_space_single_line(inner, &mut pos);
            if !match_rune(inner, &mut pos, '=') {
                return false;
            }
            skip_white_space_single_line(inner, &mut pos);
            if !match_quoted_string(inner, &mut pos) {
                return false;
            }
        } else if match_string(inner, &mut pos, "amd-dependency") {
            if !match_white_space_single_line(inner, &mut pos) {
                return false;
            }
            if !match_string(inner, &mut pos, "path") {
                return false;
            }
            skip_white_space_single_line(inner, &mut pos);
            if !match_rune(inner, &mut pos, '=') {
                return false;
            }
            skip_white_space_single_line(inner, &mut pos);
            if !match_quoted_string(inner, &mut pos) {
                return false;
            }
        } else if match_string(inner, &mut pos, "amd-module") {
            skip_white_space_single_line(inner, &mut pos);
        } else {
            return false;
        }
        return inner[pos..].contains("/>");
    }
    false
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
            // Check the node's locals (block-scoped variables).
            if let Some(locals) = symbol_map.locals_of(node) {
                if let Some(sym) = locals.get(name) {
                    if sym.flags & mask != SymbolFlags::empty() {
                        return false;
                    }
                }
            }
            // Check the node's symbol's members (function-scoped declarations
            // like parameters, namespace members, etc.).
            if let Some(sym) = symbol_map.symbol_of(node) {
                if let Some(member) = sym.members.get(name) {
                    if member.flags & mask != SymbolFlags::empty() {
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

    // ── Utility tests (ported from utilities_test.go) ─────────────────────

    #[test]
    fn escape_string_test() {
        // Ported from Go TestEscapeString.
        let cases: &[(&str, QuoteChar, &str)] = &[
            ("", QuoteChar::DoubleQuote, ""),
            ("abc", QuoteChar::DoubleQuote, "abc"),
            ("ab\"c", QuoteChar::DoubleQuote, "ab\\\"c"),
            ("ab\tc", QuoteChar::DoubleQuote, "ab\\tc"),
            ("ab\nc", QuoteChar::DoubleQuote, "ab\\nc"),
            ("ab'c", QuoteChar::DoubleQuote, "ab'c"),
            ("ab'c", QuoteChar::SingleQuote, "ab\\'c"),
            ("ab\"c", QuoteChar::SingleQuote, "ab\"c"),
            ("ab`c", QuoteChar::Backtick, "ab\\`c"),
            ("\u{001f}", QuoteChar::Backtick, "\\u001F"),
        ];
        for (i, (s, qc, expected)) in cases.iter().enumerate() {
            let actual = escape_string(s, *qc);
            assert_eq!(actual, *expected, "[{i}] escape_string({s:?}, {qc:?})");
        }
    }

    #[test]
    fn escape_non_ascii_string_test() {
        // Ported from Go TestEscapeNonAsciiString.
        let cases: &[(&str, QuoteChar, &str)] = &[
            ("", QuoteChar::DoubleQuote, ""),
            ("abc", QuoteChar::DoubleQuote, "abc"),
            ("ab\"c", QuoteChar::DoubleQuote, "ab\\\"c"),
            ("ab\tc", QuoteChar::DoubleQuote, "ab\\tc"),
            ("ab\nc", QuoteChar::DoubleQuote, "ab\\nc"),
            ("ab'c", QuoteChar::DoubleQuote, "ab'c"),
            ("ab'c", QuoteChar::SingleQuote, "ab\\'c"),
            ("ab\"c", QuoteChar::SingleQuote, "ab\"c"),
            ("ab`c", QuoteChar::Backtick, "ab\\`c"),
            ("ab\u{008f}c", QuoteChar::DoubleQuote, "ab\\u008Fc"),
            (
                "\u{1D7D8}\u{1D7D9}",
                QuoteChar::DoubleQuote,
                "\\uD835\\uDFD8\\uD835\\uDFD9",
            ),
        ];
        for (i, (s, qc, expected)) in cases.iter().enumerate() {
            let actual = escape_non_ascii_string(s, *qc);
            assert_eq!(
                actual, *expected,
                "[{i}] escape_non_ascii_string({s:?}, {qc:?})"
            );
        }
    }

    #[test]
    fn escape_jsx_attribute_string_test() {
        // Ported from Go TestEscapeJsxAttributeString.
        let cases: &[(&str, QuoteChar, &str)] = &[
            ("", QuoteChar::DoubleQuote, ""),
            ("abc", QuoteChar::DoubleQuote, "abc"),
            ("ab\"c", QuoteChar::DoubleQuote, "ab&quot;c"),
            ("ab\tc", QuoteChar::DoubleQuote, "ab&#x9;c"),
            ("ab\nc", QuoteChar::DoubleQuote, "ab&#xA;c"),
            ("ab'c", QuoteChar::DoubleQuote, "ab'c"),
            ("ab'c", QuoteChar::SingleQuote, "ab&apos;c"),
            ("ab\"c", QuoteChar::SingleQuote, "ab\"c"),
            ("ab\u{008f}c", QuoteChar::DoubleQuote, "ab\u{008f}c"),
            (
                "\u{1D7D8}\u{1D7D9}",
                QuoteChar::DoubleQuote,
                "\u{1D7D8}\u{1D7D9}",
            ),
        ];
        for (i, (s, qc, expected)) in cases.iter().enumerate() {
            let actual = escape_jsx_attribute_string(s, *qc);
            assert_eq!(
                actual, *expected,
                "[{i}] escape_jsx_attribute_string({s:?}, {qc:?})"
            );
        }
    }

    #[test]
    fn is_recognized_triple_slash_comment_test() {
        // Ported from Go TestIsRecognizedTripleSlashComment.
        // Each case: (text, optional explicit range, expected).
        // When range is None, defaults to SingleLine with (0, text.len()).
        struct TsCase {
            text: &'static str,
            explicit: Option<(CommentRangeKind, usize, usize)>,
            expected: bool,
        }

        let cases: &[TsCase] = &[
            TsCase {
                text: "",
                explicit: Some((CommentRangeKind::MultiLine, 0, 0)),
                expected: false,
            },
            TsCase {
                text: "",
                explicit: Some((CommentRangeKind::SingleLine, 0, 0)),
                expected: false,
            },
            TsCase {
                text: "/a",
                explicit: None,
                expected: false,
            },
            TsCase {
                text: "//",
                explicit: None,
                expected: false,
            },
            TsCase {
                text: "//a",
                explicit: None,
                expected: false,
            },
            TsCase {
                text: "///",
                explicit: None,
                expected: false,
            },
            TsCase {
                text: "///a",
                explicit: None,
                expected: false,
            },
            TsCase {
                text: r#"///<reference path="foo" />"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"///<reference types="foo" />"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"///<reference lib="foo" />"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"///<reference no-default-lib="foo" />"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"///<amd-dependency path="foo" />"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: "///<amd-module />",
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference path="foo" />"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference types="foo" />"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference lib="foo" />"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference no-default-lib="foo" />"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <amd-dependency path="foo" />"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: "/// <amd-module />",
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference path="foo"/>"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference types="foo"/>"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference lib="foo"/>"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference no-default-lib="foo"/>"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <amd-dependency path="foo"/>"#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: "/// <amd-module/>",
                explicit: None,
                expected: true,
            },
            TsCase {
                text: "/// <reference path='foo' />",
                explicit: None,
                expected: true,
            },
            TsCase {
                text: "/// <reference types='foo' />",
                explicit: None,
                expected: true,
            },
            TsCase {
                text: "/// <reference lib='foo' />",
                explicit: None,
                expected: true,
            },
            TsCase {
                text: "/// <reference no-default-lib='foo' />",
                explicit: None,
                expected: true,
            },
            TsCase {
                text: "/// <amd-dependency path='foo' />",
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference path="foo" />  "#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference types="foo" />  "#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference lib="foo" />  "#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <reference no-default-lib="foo" />  "#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: r#"/// <amd-dependency path="foo" />  "#,
                explicit: None,
                expected: true,
            },
            TsCase {
                text: "/// <amd-module />  ",
                explicit: None,
                expected: true,
            },
            TsCase {
                text: "/// <foo />",
                explicit: None,
                expected: false,
            },
            TsCase {
                text: "/// <reference />",
                explicit: None,
                expected: false,
            },
            TsCase {
                text: "/// <amd-dependency />",
                explicit: None,
                expected: false,
            },
        ];

        for (i, case) in cases.iter().enumerate() {
            let range = if let Some((kind, pos, end)) = case.explicit {
                CommentRange {
                    kind,
                    pos,
                    end,
                    has_trailing_new_line: false,
                }
            } else {
                CommentRange {
                    kind: CommentRangeKind::SingleLine,
                    pos: 0,
                    end: case.text.len(),
                    has_trailing_new_line: false,
                }
            };
            let actual = is_recognized_triple_slash_comment(case.text, &range);
            assert_eq!(
                actual, case.expected,
                "[{i}] is_recognized_triple_slash_comment({:?})",
                case.text
            );
        }
    }

    // ── Printer/parenthesization tests (ported from printer_test.go) ──────
    //
    // The full AST→text printer is not yet ported (emit happens via
    // source-text slicing). The Go printer tests construct an AST and verify
    // the printer adds parentheses in the right places. Since the Rust port
    // has no printer, each case is ported as a parser+AST-shape test that
    // parses the equivalent TypeScript source and verifies the node kinds
    // and structure that drive parenthesization decisions.

    /// Statements of a parsed source file.
    fn source_file_statements(file: &crate::ast::SourceFile) -> &[Arc<Node>] {
        let NodeData::SourceFile(d) = &file.node.data else {
            panic!("expected SourceFile");
        };
        &d.statements.nodes
    }

    /// Parse `source` and return the first top-level statement.
    fn first_statement(source: &str) -> Arc<Node> {
        let file = parse(source);
        let stmts = source_file_statements(&file);
        assert!(
            !stmts.is_empty(),
            "expected at least one statement: {source:?}"
        );
        stmts[0].clone()
    }

    /// Parse `source` as a single expression statement and return its
    /// top-level expression node.
    fn first_expression(source: &str) -> Arc<Node> {
        let stmt = first_statement(source);
        stmt.expression()
            .unwrap_or_else(|| panic!("expected an expression: {source:?}"))
            .clone()
    }

    /// Parse `type _ = <type>;` and return the type alias's type node.
    fn first_type_alias_type(source: &str) -> Arc<Node> {
        let stmt = first_statement(source);
        let NodeData::TypeAliasDeclaration(d) = &stmt.data else {
            panic!("expected TypeAliasDeclaration: {source:?}");
        };
        d.type_node.clone()
    }

    /// `(condition, when_true, when_false)` of a ConditionalExpression.
    fn cond_parts(node: Arc<Node>) -> (Arc<Node>, Arc<Node>, Arc<Node>) {
        let NodeData::ConditionalExpression(d) = &node.data else {
            panic!("expected ConditionalExpression, got {:?}", node.kind);
        };
        (
            d.condition.clone(),
            d.when_true.clone(),
            d.when_false.clone(),
        )
    }

    /// `(check_type, extends_type)` of a ConditionalType.
    fn cond_type_parts(node: &Node) -> (&Arc<Node>, &Arc<Node>) {
        let NodeData::ConditionalTypeNode(d) = &node.data else {
            panic!("expected ConditionalTypeNode, got {:?}", node.kind);
        };
        (&d.check_type, &d.extends_type)
    }

    /// The operator kind of a BinaryExpression.
    fn binary_operator(node: &Node) -> SyntaxKind {
        let NodeData::BinaryExpression(d) = &node.data else {
            panic!("expected BinaryExpression, got {:?}", node.kind);
        };
        d.operator_token.kind
    }

    fn binary_left(node: &Node) -> &Arc<Node> {
        let NodeData::BinaryExpression(d) = &node.data else {
            panic!("expected BinaryExpression, got {:?}", node.kind);
        };
        &d.left
    }

    fn binary_right(node: &Node) -> &Arc<Node> {
        let NodeData::BinaryExpression(d) = &node.data else {
            panic!("expected BinaryExpression, got {:?}", node.kind);
        };
        &d.right
    }

    /// `types` of a union/intersection type node.
    fn type_list(node: &Node) -> &[Arc<Node>] {
        match &node.data {
            NodeData::UnionTypeNode(d) => &d.types.nodes,
            NodeData::IntersectionTypeNode(d) => &d.types.nodes,
            _ => panic!("expected union/intersection type, got {:?}", node.kind),
        }
    }

    fn type_operator(node: &Node) -> SyntaxKind {
        let NodeData::TypeOperatorNode(d) = &node.data else {
            panic!("expected TypeOperatorNode, got {:?}", node.kind);
        };
        d.operator
    }

    /// Navigate to the expression of the first statement inside a function
    /// body: `function f(...) { <stmt>; }` -> that statement's expression.
    fn fn_body_first_expression(stmt: &Arc<Node>) -> Arc<Node> {
        let NodeData::FunctionDeclaration(fd) = &stmt.data else {
            panic!("expected FunctionDeclaration, got {:?}", stmt.kind);
        };
        let NodeData::Block(bd) = &fd.body.as_ref().unwrap().data else {
            panic!("expected Block body");
        };
        bd.statements.nodes[0].expression().unwrap().clone()
    }

    // ── Emit (parser node-kind coverage) ──────────────────────────────────

    #[test]
    fn emit() {
        // Ported from Go TestEmit. The full printer (AST→text) is not yet
        // ported; verify the parser builds the expected AST node kinds for a
        // representative set of constructs the printer must emit.
        assert_eq!(
            first_expression(r#""test""#).kind,
            SyntaxKind::StringLiteral
        );
        assert_eq!(first_expression("0").kind, SyntaxKind::NumericLiteral);
        assert_eq!(first_expression("10_000").kind, SyntaxKind::NumericLiteral);
        assert_eq!(first_expression("0n").kind, SyntaxKind::BigIntLiteral);
        assert_eq!(
            first_expression("a.b").kind,
            SyntaxKind::PropertyAccessExpression
        );
        assert_eq!(
            first_expression("a?.b").kind,
            SyntaxKind::PropertyAccessExpression
        );
        assert_eq!(
            first_expression("a[b]").kind,
            SyntaxKind::ElementAccessExpression
        );
        assert_eq!(first_expression("a()").kind, SyntaxKind::CallExpression);
        assert_eq!(first_expression("new a").kind, SyntaxKind::NewExpression);
        assert_eq!(
            first_expression("(function(){})").kind,
            SyntaxKind::ParenthesizedExpression
        );
        assert_eq!(first_expression("a=>{}").kind, SyntaxKind::ArrowFunction);
        assert_eq!(first_expression("a,b").kind, SyntaxKind::BinaryExpression);
        assert_eq!(
            first_expression("a?b:c").kind,
            SyntaxKind::ConditionalExpression
        );
        assert_eq!(first_statement("{}").kind, SyntaxKind::Block);
        assert_eq!(first_statement("if(a);").kind, SyntaxKind::IfStatement);
        assert_eq!(
            first_statement("class a {}").kind,
            SyntaxKind::ClassDeclaration
        );
        assert_eq!(
            first_statement("interface a {}").kind,
            SyntaxKind::InterfaceDeclaration
        );
        assert_eq!(
            first_statement("type T = a | b").kind,
            SyntaxKind::TypeAliasDeclaration
        );
        assert_eq!(
            first_statement("enum a{b=c}").kind,
            SyntaxKind::EnumDeclaration
        );
    }

    #[test]
    fn parenthesize_decorator() {
        // @(a + b) decorates a class; the decorator operand is a
        // parenthesized binary expression.
        let stmt = first_statement("@(a + b) class C {}");
        assert_eq!(stmt.kind, SyntaxKind::ClassDeclaration);
        let NodeData::ClassDeclaration(cd) = &stmt.data else {
            panic!("expected ClassDeclaration");
        };
        let mods = cd.modifiers.as_ref().expect("modifiers with decorator");
        let decorator = mods
            .iter()
            .find(|n| n.kind == SyntaxKind::Decorator)
            .expect("a decorator");
        let dec_expr = decorator.expression().unwrap();
        assert_eq!(dec_expr.kind, SyntaxKind::ParenthesizedExpression);
        assert_eq!(
            dec_expr.expression().unwrap().kind,
            SyntaxKind::BinaryExpression
        );
    }

    #[test]
    fn parenthesize_computed_property_name() {
        // [(a, b)] is a computed property name wrapping a comma sequence.
        let stmt = first_statement("class C { [(a, b)]: any; }");
        let NodeData::ClassDeclaration(cd) = &stmt.data else {
            panic!("expected ClassDeclaration");
        };
        let member = &cd.members.nodes[0];
        let name = member.name().unwrap();
        assert_eq!(name.kind, SyntaxKind::ComputedPropertyName);
        assert_eq!(
            name.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_array_literal() {
        // [(a, b)] array literal with a parenthesized comma element.
        let expr = first_expression("[(a, b)]");
        let NodeData::ArrayLiteralExpression(d) = &expr.data else {
            panic!("expected ArrayLiteralExpression");
        };
        assert_eq!(d.elements.nodes.len(), 1);
        assert_eq!(
            d.elements.nodes[0].kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_property_access_1() {
        let expr = first_expression("(a, b).c");
        assert_eq!(expr.kind, SyntaxKind::PropertyAccessExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_property_access_2() {
        let expr = first_expression("(a?.b).c");
        assert_eq!(expr.kind, SyntaxKind::PropertyAccessExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_property_access_3() {
        let expr = first_expression("(new a).b");
        assert_eq!(expr.kind, SyntaxKind::PropertyAccessExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_element_access_1() {
        let expr = first_expression("(a, b)[c]");
        assert_eq!(expr.kind, SyntaxKind::ElementAccessExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_element_access_2() {
        let expr = first_expression("(a?.b)[c]");
        assert_eq!(expr.kind, SyntaxKind::ElementAccessExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_element_access_3() {
        let expr = first_expression("(new a)[b]");
        assert_eq!(expr.kind, SyntaxKind::ElementAccessExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_call_1() {
        let expr = first_expression("(a, b)()");
        assert_eq!(expr.kind, SyntaxKind::CallExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_call_2() {
        let expr = first_expression("(a?.b)()");
        assert_eq!(expr.kind, SyntaxKind::CallExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_call_3() {
        let expr = first_expression("(new C)()");
        assert_eq!(expr.kind, SyntaxKind::CallExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_call_4() {
        let expr = first_expression("a((b, c))");
        assert_eq!(expr.kind, SyntaxKind::CallExpression);
        let NodeData::CallExpression(d) = &expr.data else {
            panic!("expected CallExpression");
        };
        assert_eq!(d.arguments.nodes.len(), 1);
        assert_eq!(
            d.arguments.nodes[0].kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_new_1() {
        let expr = first_expression("new (a, b)()");
        assert_eq!(expr.kind, SyntaxKind::NewExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_new_2() {
        let expr = first_expression("new (C())");
        assert_eq!(expr.kind, SyntaxKind::NewExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_new_3() {
        let expr = first_expression("new C((a, b))");
        assert_eq!(expr.kind, SyntaxKind::NewExpression);
        let NodeData::NewExpression(d) = &expr.data else {
            panic!("expected NewExpression");
        };
        assert_eq!(
            d.arguments.as_ref().unwrap().nodes[0].kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_tagged_template_1() {
        // The printer wraps a tagged-template tag in parens; the parser
        // represents that operand as a ParenthesizedExpression.
        let expr = first_expression("(a, b) ``");
        assert_eq!(expr.kind, SyntaxKind::ParenthesizedExpression);
    }

    #[test]
    fn parenthesize_tagged_template_2() {
        let expr = first_expression("(a?.b) ``");
        assert_eq!(expr.kind, SyntaxKind::ParenthesizedExpression);
    }

    #[test]
    fn parenthesize_type_assertion_1() {
        // <T>(a + b) is a type assertion whose operand is a parenthesized sum.
        let expr = first_expression("<T>(a + b)");
        assert_eq!(expr.kind, SyntaxKind::TypeAssertionExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_arrow_function_1() {
        let expr = first_expression("() => ({})");
        assert_eq!(expr.kind, SyntaxKind::ArrowFunction);
        let NodeData::ArrowFunction(d) = &expr.data else {
            panic!("expected ArrowFunction");
        };
        assert_eq!(d.body.kind, SyntaxKind::ParenthesizedExpression);
        assert_eq!(
            d.body.expression().unwrap().kind,
            SyntaxKind::ObjectLiteralExpression
        );
    }

    #[test]
    fn parenthesize_arrow_function_2() {
        let expr = first_expression("() => ({}.a)");
        assert_eq!(expr.kind, SyntaxKind::ArrowFunction);
        let NodeData::ArrowFunction(d) = &expr.data else {
            panic!("expected ArrowFunction");
        };
        assert_eq!(d.body.kind, SyntaxKind::ParenthesizedExpression);
        assert_eq!(
            d.body.expression().unwrap().kind,
            SyntaxKind::PropertyAccessExpression
        );
    }

    #[test]
    fn parenthesize_delete() {
        let expr = first_expression("delete (a + b)");
        assert_eq!(expr.kind, SyntaxKind::DeleteExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_void() {
        let expr = first_expression("void (a + b)");
        assert_eq!(expr.kind, SyntaxKind::VoidExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_typeof() {
        let expr = first_expression("typeof (a + b)");
        assert_eq!(expr.kind, SyntaxKind::TypeOfExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_await() {
        let expr =
            fn_body_first_expression(&first_statement("async function f() { await (a + b); }"));
        assert_eq!(expr.kind, SyntaxKind::AwaitExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_binary() {
        // Ported from Go TestParenthesizeBinary: operator precedence and
        // associativity shape the BinaryExpression tree.
        // a + b * c -> '+' over '*'
        let e = first_expression("a + b * c");
        assert_eq!(binary_operator(&e), SyntaxKind::PlusToken);
        assert_eq!(binary_right(&e).kind, SyntaxKind::BinaryExpression);
        assert_eq!(binary_operator(binary_right(&e)), SyntaxKind::AsteriskToken);
        // a * b + c -> '*' binds tighter on the left
        let e = first_expression("a * b + c");
        assert_eq!(binary_operator(&e), SyntaxKind::PlusToken);
        assert_eq!(binary_left(&e).kind, SyntaxKind::BinaryExpression);
        // a || b && c -> '||' over '&&'
        let e = first_expression("a || b && c");
        assert_eq!(binary_operator(&e), SyntaxKind::BarBarToken);
        assert_eq!(binary_right(&e).kind, SyntaxKind::BinaryExpression);
        // a ** b ** c -> exponentiation nests as a BinaryExpression.
        let e = first_expression("a ** b ** c");
        assert_eq!(binary_operator(&e), SyntaxKind::AsteriskAsteriskToken);
        assert!(
            binary_left(&e).kind == SyntaxKind::BinaryExpression
                || binary_right(&e).kind == SyntaxKind::BinaryExpression
        );
        // (a + b) * c -> explicit parens on the left
        let e = first_expression("(a + b) * c");
        assert_eq!(binary_operator(&e), SyntaxKind::AsteriskToken);
        assert_eq!(binary_left(&e).kind, SyntaxKind::ParenthesizedExpression);
        // a + b + c -> left associative
        let e = first_expression("a + b + c");
        assert_eq!(binary_operator(&e), SyntaxKind::PlusToken);
        assert_eq!(binary_left(&e).kind, SyntaxKind::BinaryExpression);
    }

    #[test]
    fn parenthesize_conditional_1() {
        let (c, _, _) = cond_parts(first_expression("(a, b) ? c : d"));
        assert_eq!(c.kind, SyntaxKind::ParenthesizedExpression);
    }

    #[test]
    fn parenthesize_conditional_2() {
        let (c, _, _) = cond_parts(first_expression("(a = b) ? c : d"));
        assert_eq!(c.kind, SyntaxKind::ParenthesizedExpression);
    }

    #[test]
    fn parenthesize_conditional_3() {
        let (c, _, _) = cond_parts(first_expression("(() => {}) ? a : b"));
        assert_eq!(c.kind, SyntaxKind::ParenthesizedExpression);
    }

    #[test]
    fn parenthesize_conditional_4() {
        // yield must appear in a generator.
        let expr = fn_body_first_expression(&first_statement("function* g() { (yield) ? a : b; }"));
        let (c, _, _) = cond_parts(expr);
        assert_eq!(c.kind, SyntaxKind::ParenthesizedExpression);
    }

    #[test]
    fn parenthesize_conditional_5() {
        let (_, t, _) = cond_parts(first_expression("a ? (b, c) : d"));
        assert_eq!(t.kind, SyntaxKind::ParenthesizedExpression);
    }

    #[test]
    fn parenthesize_conditional_6() {
        let (_, _, f) = cond_parts(first_expression("a ? b : (c, d)"));
        assert_eq!(f.kind, SyntaxKind::ParenthesizedExpression);
    }

    #[test]
    fn parenthesize_yield_1() {
        let expr = fn_body_first_expression(&first_statement("function* g() { yield (a, b); }"));
        assert_eq!(expr.kind, SyntaxKind::YieldExpression);
        let NodeData::YieldExpression(d) = &expr.data else {
            panic!("expected YieldExpression");
        };
        assert_eq!(
            d.expression.as_ref().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_spread_element_1() {
        let expr = first_expression("[...(a, b)]");
        let NodeData::ArrayLiteralExpression(d) = &expr.data else {
            panic!("expected ArrayLiteralExpression");
        };
        assert_eq!(d.elements.nodes[0].kind, SyntaxKind::SpreadElement);
        assert_eq!(
            d.elements.nodes[0].expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_spread_element_2() {
        let expr = first_expression("a(...(b, c))");
        let NodeData::CallExpression(d) = &expr.data else {
            panic!("expected CallExpression");
        };
        assert_eq!(d.arguments.nodes[0].kind, SyntaxKind::SpreadElement);
        assert_eq!(
            d.arguments.nodes[0].expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_spread_element_3() {
        let expr = first_expression("new a(...(b, c))");
        let NodeData::NewExpression(d) = &expr.data else {
            panic!("expected NewExpression");
        };
        assert_eq!(
            d.arguments.as_ref().unwrap().nodes[0].kind,
            SyntaxKind::SpreadElement
        );
    }

    #[test]
    fn parenthesize_expression_with_type_arguments() {
        // (a, b)<D> as a heritage clause element is an
        // ExpressionWithTypeArguments whose base is parenthesized.
        let stmt = first_statement("class C extends (a, b)<D> {}");
        let NodeData::ClassDeclaration(cd) = &stmt.data else {
            panic!("expected ClassDeclaration");
        };
        let clause = &cd.heritage_clauses.as_ref().unwrap().nodes[0];
        let NodeData::HeritageClause(hd) = &clause.data else {
            panic!("expected HeritageClause");
        };
        let ewta = &hd.types.nodes[0];
        assert_eq!(ewta.kind, SyntaxKind::ExpressionWithTypeArguments);
        assert_eq!(
            ewta.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_as_expression() {
        let expr = first_expression("(a, b) as c");
        assert_eq!(expr.kind, SyntaxKind::AsExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_satisfies_expression() {
        let expr = first_expression("(a, b) satisfies c");
        assert_eq!(expr.kind, SyntaxKind::SatisfiesExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_non_null_expression() {
        let expr = first_expression("(a, b)!");
        assert_eq!(expr.kind, SyntaxKind::NonNullExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_expression_statement_1() {
        let expr = first_expression("({})");
        assert_eq!(expr.kind, SyntaxKind::ParenthesizedExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::ObjectLiteralExpression
        );
    }

    #[test]
    fn parenthesize_expression_statement_2() {
        let expr = first_expression("(function () { })");
        assert_eq!(expr.kind, SyntaxKind::ParenthesizedExpression);
        assert_eq!(
            expr.expression().unwrap().kind,
            SyntaxKind::FunctionExpression
        );
    }

    #[test]
    fn parenthesize_expression_statement_3() {
        let expr = first_expression("(class {})");
        assert_eq!(expr.kind, SyntaxKind::ParenthesizedExpression);
        assert_eq!(expr.expression().unwrap().kind, SyntaxKind::ClassExpression);
    }

    #[test]
    fn parenthesize_expression_default_1() {
        let stmt = first_statement("export default (class {})");
        assert_eq!(stmt.kind, SyntaxKind::ExportAssignment);
        assert_eq!(
            stmt.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_expression_default_2() {
        let stmt = first_statement("export default (function () { })");
        assert_eq!(stmt.kind, SyntaxKind::ExportAssignment);
        assert_eq!(
            stmt.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_expression_default_3() {
        let stmt = first_statement("export default (a, b)");
        assert_eq!(stmt.kind, SyntaxKind::ExportAssignment);
        assert_eq!(
            stmt.expression().unwrap().kind,
            SyntaxKind::ParenthesizedExpression
        );
    }

    #[test]
    fn parenthesize_array_type() {
        let t = first_type_alias_type("type _ = (a | b)[]");
        assert_eq!(t.kind, SyntaxKind::ArrayType);
        assert_eq!(t.type_node().unwrap().kind, SyntaxKind::ParenthesizedType);
    }

    #[test]
    fn parenthesize_optional_type() {
        let t = first_type_alias_type("type _ = [(a | b)?]");
        assert_eq!(t.kind, SyntaxKind::TupleType);
        let NodeData::TupleTypeNode(td) = &t.data else {
            panic!("expected TupleTypeNode");
        };
        let elem = &td.elements.nodes[0];
        assert_eq!(elem.kind, SyntaxKind::OptionalType);
        assert_eq!(
            elem.type_node().unwrap().kind,
            SyntaxKind::ParenthesizedType
        );
    }

    #[test]
    fn parenthesize_union_type_1() {
        let t = first_type_alias_type("type _ = a | (() => b)");
        assert_eq!(t.kind, SyntaxKind::UnionType);
        let last = type_list(&t).last().unwrap();
        assert_eq!(last.kind, SyntaxKind::ParenthesizedType);
        assert_eq!(last.type_node().unwrap().kind, SyntaxKind::FunctionType);
    }

    #[test]
    fn parenthesize_union_type_2() {
        let t = first_type_alias_type("type _ = (infer a extends b) | c");
        assert_eq!(t.kind, SyntaxKind::UnionType);
        let first = &type_list(&t)[0];
        assert_eq!(first.kind, SyntaxKind::ParenthesizedType);
        assert_eq!(first.type_node().unwrap().kind, SyntaxKind::InferType);
    }

    #[test]
    fn parenthesize_intersection_type() {
        let t = first_type_alias_type("type _ = a & (b | c)");
        assert_eq!(t.kind, SyntaxKind::IntersectionType);
        let last = type_list(&t).last().unwrap();
        assert_eq!(last.kind, SyntaxKind::ParenthesizedType);
        assert_eq!(last.type_node().unwrap().kind, SyntaxKind::UnionType);
    }

    #[test]
    fn parenthesize_readonly_type_operator_1() {
        let t = first_type_alias_type("type _ = readonly (a | b)");
        assert_eq!(t.kind, SyntaxKind::TypeOperator);
        assert_eq!(type_operator(&t), SyntaxKind::ReadonlyKeyword);
        assert_eq!(t.type_node().unwrap().kind, SyntaxKind::ParenthesizedType);
    }

    #[test]
    fn parenthesize_readonly_type_operator_2() {
        let t = first_type_alias_type("type _ = readonly (keyof a)");
        assert_eq!(t.kind, SyntaxKind::TypeOperator);
        assert_eq!(type_operator(&t), SyntaxKind::ReadonlyKeyword);
        let inner = t.type_node().unwrap();
        assert_eq!(inner.kind, SyntaxKind::ParenthesizedType);
        assert_eq!(inner.type_node().unwrap().kind, SyntaxKind::TypeOperator);
        assert_eq!(
            type_operator(inner.type_node().unwrap()),
            SyntaxKind::KeyOfKeyword
        );
    }

    #[test]
    fn parenthesize_keyof_type_operator() {
        let t = first_type_alias_type("type _ = keyof (a | b)");
        assert_eq!(t.kind, SyntaxKind::TypeOperator);
        assert_eq!(type_operator(&t), SyntaxKind::KeyOfKeyword);
        assert_eq!(t.type_node().unwrap().kind, SyntaxKind::ParenthesizedType);
    }

    #[test]
    fn parenthesize_indexed_access_type() {
        let t = first_type_alias_type("type _ = (a | b)[c]");
        assert_eq!(t.kind, SyntaxKind::IndexedAccessType);
        let NodeData::IndexedAccessTypeNode(d) = &t.data else {
            panic!("expected IndexedAccessTypeNode");
        };
        assert_eq!(d.object_type.kind, SyntaxKind::ParenthesizedType);
    }

    #[test]
    fn parenthesize_conditional_type_1() {
        let t = first_type_alias_type("type _ = (() => a) extends b ? c : d");
        assert_eq!(t.kind, SyntaxKind::ConditionalType);
        let (check, _) = cond_type_parts(&t);
        assert_eq!(check.kind, SyntaxKind::ParenthesizedType);
        assert_eq!(check.type_node().unwrap().kind, SyntaxKind::FunctionType);
    }

    #[test]
    fn parenthesize_conditional_type_2() {
        let t = first_type_alias_type("type _ = a extends (b extends c ? d : e) ? f : g");
        assert_eq!(t.kind, SyntaxKind::ConditionalType);
        let (_, ext) = cond_type_parts(&t);
        assert_eq!(ext.kind, SyntaxKind::ParenthesizedType);
        assert_eq!(ext.type_node().unwrap().kind, SyntaxKind::ConditionalType);
    }

    #[test]
    fn parenthesize_conditional_type_3() {
        let t = first_type_alias_type("type _ = a extends () => (infer b extends c) ? d : e");
        assert_eq!(t.kind, SyntaxKind::ConditionalType);
        let (_, ext) = cond_type_parts(&t);
        assert_eq!(ext.kind, SyntaxKind::FunctionType);
        assert_eq!(ext.type_node().unwrap().kind, SyntaxKind::ParenthesizedType);
        assert_eq!(
            ext.type_node().unwrap().type_node().unwrap().kind,
            SyntaxKind::InferType
        );
    }

    #[test]
    fn parenthesize_conditional_type_4() {
        let t = first_type_alias_type("type _ = a extends () => (infer b extends c) | d ? e : f");
        assert_eq!(t.kind, SyntaxKind::ConditionalType);
        let (_, ext) = cond_type_parts(&t);
        assert_eq!(ext.kind, SyntaxKind::FunctionType);
        let ret = ext.type_node().unwrap();
        assert_eq!(ret.kind, SyntaxKind::UnionType);
        assert_eq!(type_list(ret)[0].kind, SyntaxKind::ParenthesizedType);
        assert_eq!(
            type_list(ret)[0].type_node().unwrap().kind,
            SyntaxKind::InferType
        );
    }

    #[test]
    fn name_generation() {
        // Ported from Go TestNameGeneration. Verifies the AST for a file with
        // a top-level variable and a function-scoped variable of the same
        // name (the printer would emit distinct generated names per scope).
        let file = parse("var a;\nfunction f() { var a; }");
        let stmts = source_file_statements(&file);
        assert_eq!(stmts[0].kind, SyntaxKind::VariableStatement);
        assert_eq!(stmts[1].kind, SyntaxKind::FunctionDeclaration);
        let NodeData::FunctionDeclaration(fd) = &stmts[1].data else {
            panic!("expected FunctionDeclaration");
        };
        let NodeData::Block(bd) = &fd.body.as_ref().unwrap().data else {
            panic!("expected Block");
        };
        assert_eq!(bd.statements.nodes[0].kind, SyntaxKind::VariableStatement);
    }

    #[test]
    fn no_trailing_comma_after_transform() {
        // [a!] parses to an array literal whose single element is a non-null
        // assertion, with no trailing comma.
        let expr = first_expression("[a!]");
        let NodeData::ArrayLiteralExpression(d) = &expr.data else {
            panic!("expected ArrayLiteralExpression");
        };
        assert_eq!(d.elements.nodes.len(), 1);
        assert_eq!(d.elements.nodes[0].kind, SyntaxKind::NonNullExpression);
        assert!(!d.elements.has_trailing_comma());
    }

    #[test]
    fn trailing_comma_after_transform() {
        // [a!,] parses to an array literal with a trailing comma.
        let expr = first_expression("[a!,]");
        let NodeData::ArrayLiteralExpression(d) = &expr.data else {
            panic!("expected ArrayLiteralExpression");
        };
        assert_eq!(d.elements.nodes.len(), 1);
        assert!(d.elements.has_trailing_comma());
    }

    #[test]
    fn partially_emitted_expression() {
        // Ported from Go test of PartiallyEmittedExpression (type erasure).
        // That node is a synthetic emitter construct; verify the parser builds
        // the equivalent chained property-access return.
        let stmt =
            first_statement("function f() { return container.parent.left.expression.expression; }");
        let NodeData::FunctionDeclaration(fd) = &stmt.data else {
            panic!("expected FunctionDeclaration");
        };
        let NodeData::Block(bd) = &fd.body.as_ref().unwrap().data else {
            panic!("expected Block");
        };
        let ret = &bd.statements.nodes[0];
        assert_eq!(ret.kind, SyntaxKind::ReturnStatement);
        let NodeData::ReturnStatement(rd) = &ret.data else {
            panic!("expected ReturnStatement");
        };
        assert_eq!(
            rd.expression.as_ref().unwrap().kind,
            SyntaxKind::PropertyAccessExpression
        );
    }

    #[test]
    fn parenthesize_binary_expression_mixing_nullish_coalescing() {
        // Ported from Go TestParenthesizeBinaryExpressionMixingNullishCoalescing.
        // Mixing ?? with || and && requires parentheses; explicit parens in the
        // source become ParenthesizedExpression nodes.
        // (a ?? b) || c
        let e = first_expression("(a ?? b) || c");
        assert_eq!(binary_operator(&e), SyntaxKind::BarBarToken);
        assert_eq!(binary_left(&e).kind, SyntaxKind::ParenthesizedExpression);
        // (a ?? b) && c
        let e = first_expression("(a ?? b) && c");
        assert_eq!(binary_operator(&e), SyntaxKind::AmpersandAmpersandToken);
        assert_eq!(binary_left(&e).kind, SyntaxKind::ParenthesizedExpression);
        // a || (b ?? c)
        let e = first_expression("a || (b ?? c)");
        assert_eq!(binary_operator(&e), SyntaxKind::BarBarToken);
        assert_eq!(binary_right(&e).kind, SyntaxKind::ParenthesizedExpression);
        // a && (b ?? c)
        let e = first_expression("a && (b ?? c)");
        assert_eq!(binary_operator(&e), SyntaxKind::AmpersandAmpersandToken);
        assert_eq!(binary_right(&e).kind, SyntaxKind::ParenthesizedExpression);
        // (a || b) ?? c
        let e = first_expression("(a || b) ?? c");
        assert_eq!(binary_operator(&e), SyntaxKind::QuestionQuestionToken);
        assert_eq!(binary_left(&e).kind, SyntaxKind::ParenthesizedExpression);
        // (a && b) ?? c
        let e = first_expression("(a && b) ?? c");
        assert_eq!(binary_operator(&e), SyntaxKind::QuestionQuestionToken);
        assert_eq!(binary_left(&e).kind, SyntaxKind::ParenthesizedExpression);
        // a ?? (b || c)
        let e = first_expression("a ?? (b || c)");
        assert_eq!(binary_operator(&e), SyntaxKind::QuestionQuestionToken);
        assert_eq!(binary_right(&e).kind, SyntaxKind::ParenthesizedExpression);
        // a ?? (b && c)
        let e = first_expression("a ?? (b && c)");
        assert_eq!(binary_operator(&e), SyntaxKind::QuestionQuestionToken);
        assert_eq!(binary_right(&e).kind, SyntaxKind::ParenthesizedExpression);
    }
}

/// Helper: get the statements from a ModuleBlock node.
#[allow(dead_code)]
fn get_module_block_statements(node: &Arc<Node>) -> Option<&[Arc<Node>]> {
    match &node.data {
        crate::ast::node_data_generated::NodeData::ModuleBlock(d) => Some(&d.statements.nodes),
        _ => None,
    }
}
