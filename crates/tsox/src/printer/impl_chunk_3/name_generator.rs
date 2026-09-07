#![allow(unused_imports)]

use super::*;

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

    pub(crate) fn get_scope_mut(
        &mut self,
        private_name: bool,
    ) -> &mut Option<Box<NameGenerationScope>> {
        if private_name {
            &mut self.private_name_generation_scope
        } else {
            &mut self.name_generation_scope
        }
    }

    pub(crate) fn get_temp_flags(&self, private_name: bool) -> i32 {
        let scope = if private_name {
            &self.private_name_generation_scope
        } else {
            &self.name_generation_scope
        };
        scope.as_ref().map_or(TEMP_FLAGS_AUTO, |s| s.temp_flags)
    }

    pub(crate) fn set_temp_flags(&mut self, private_name: bool, flags: i32) {
        let scope = self.get_scope_mut(private_name);
        if scope.is_none() {
            *scope = Some(Box::new(NameGenerationScope::new()));
        }
        scope.as_mut().unwrap().temp_flags = flags;
    }

    pub(crate) fn get_temp_flags_for_formatted_name(&self, private_name: bool, key: &str) -> i32 {
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

    pub(crate) fn set_temp_flags_for_formatted_name(
        &mut self,
        private_name: bool,
        key: String,
        flags: i32,
    ) {
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

    pub(crate) fn reserve_name(
        &mut self,
        name: &str,
        private_name: bool,
        scoped: bool,
        temp: bool,
    ) {
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

    pub(crate) fn is_reserved_name(&self, name: &str, private_name: bool) -> bool {
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

    pub(crate) fn is_unique_name(&self, name: &str, private_name: bool) -> bool {
        !self.is_reserved_name(name, private_name)
    }

    pub(crate) fn check_unique_name(&self, name: &str, private_name: bool) -> bool {
        self.is_unique_name(name, private_name)
    }

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

    pub(crate) fn generate_name_for_node_cached(
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

    pub(crate) fn generate_name_for_node(
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
}
