use std::sync::Arc;

use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

use super::checker::Checker;
use super::types::{
    SymbolAccessibility, SymbolAccessibilityResult,
};

impl Checker {

    pub fn is_declaration_visible(&mut self, node: &Arc<Node>) -> bool {

        let cached = self
            .declaration_links
            .get(node)
            .map(|l| l.is_visible)
            .unwrap_or_default();
        if !cached.is_unknown() {
            return cached.is_true();
        }
        let result = self.determine_if_declaration_is_visible(node);
        self.declaration_links.get_or_default(node).is_visible = result.into();
        result
    }

    fn determine_if_declaration_is_visible(&mut self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::BindingElement => node
                .parent
                .clone()
                .and_then(|p| p.parent.clone())
                .map(|gp| self.is_declaration_visible(&gp))
                .unwrap_or(false),

            SyntaxKind::VariableDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ImportEqualsDeclaration => {

                if node.kind == SyntaxKind::VariableDeclaration {
                    if let NodeData::VariableDeclaration(d) = &node.data {
                        let name = &d.name;
                        if name.kind == SyntaxKind::ObjectBindingPattern
                            || name.kind == SyntaxKind::ArrayBindingPattern
                        {
                            if let NodeData::BindingPattern(p) = &name.data {
                                if p.elements.nodes.is_empty() {
                                    return false;
                                }
                            }
                        }
                    }
                }

                if Self::is_external_module_augmentation(node) {
                    return true;
                }
                let parent = match Checker::get_declaration_container(node) {
                    Some(p) => p,
                    None => return false,
                };
                let is_exported = self
                    .get_combined_modifier_flags(node)
                    .contains(crate::ast::ModifierFlags::Export);

                let is_ambient_element = node.kind != SyntaxKind::ImportEqualsDeclaration
                    && parent.kind != SyntaxKind::SourceFile
                    && parent.flags.contains(crate::ast::NodeFlags::Ambient);
                if !is_exported && !is_ambient_element {

                    return Self::is_global_source_file(&parent);
                }

                self.is_declaration_visible(&parent)
            }

            SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature => {

                let flags = self.get_effective_declaration_flags(node);
                let private_protected = crate::ast::ModifierFlags::Private
                    .union(crate::ast::ModifierFlags::Protected)
                    .bits();
                if flags & private_protected != 0 {
                    return false;
                }
                node.parent
                    .clone()
                    .map(|p| self.is_declaration_visible(&p))
                    .unwrap_or(false)
            }

            SyntaxKind::Constructor
            | SyntaxKind::ConstructSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::Parameter
            | SyntaxKind::ModuleBlock
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::TypeLiteral
            | SyntaxKind::TypeReference
            | SyntaxKind::ArrayType
            | SyntaxKind::TupleType
            | SyntaxKind::UnionType
            | SyntaxKind::IntersectionType
            | SyntaxKind::ParenthesizedType
            | SyntaxKind::NamedTupleMember => node
                .parent
                .clone()
                .map(|p| self.is_declaration_visible(&p))
                .unwrap_or(false),

            SyntaxKind::ImportClause
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportSpecifier => false,

            SyntaxKind::TypeParameter => true,

            SyntaxKind::SourceFile | SyntaxKind::NamespaceExportDeclaration => true,

            SyntaxKind::ExportAssignment => false,

            SyntaxKind::ExportSpecifier => {
                let export_decl = match node.parent.clone().and_then(|p| p.parent.clone()) {
                    Some(ed) if ed.kind == SyntaxKind::ExportDeclaration => ed,
                    _ => return false,
                };
                let has_module_specifier = match &export_decl.data {
                    NodeData::ExportDeclaration(d) => d.module_specifier.is_some(),
                    _ => false,
                };
                if has_module_specifier {
                    return false;
                }
                export_decl
                    .parent
                    .clone()
                    .map(|p| self.is_declaration_visible(&p))
                    .unwrap_or(false)
            }

            _ => false,
        }
    }

    pub fn precalculate_declaration_emit_visibility(&mut self, file: &Arc<crate::ast::SourceFile>) {
        if self
            .declaration_file_links
            .get(file)
            .map(|l| l.aliases_marked)
            .unwrap_or(false)
        {
            return;
        }
        self.declaration_file_links
            .get_or_default(file)
            .aliases_marked = true;

        let saved_file = self.current_file.take();
        let saved_file_id = self.current_file_id;
        let saved_file_symbol = self.current_file_symbol.take();
        let saved_scope_stack = self.scope_stack.clone();

        self.current_file = Some(Arc::clone(file));
        self.current_file_id = file.node.id();
        self.current_file_symbol = self.program.symbol_map().symbol_of(&file.node).cloned();

        self.scope_stack.clear();
        self.scope_stack.push(file.node.id());

        let children = collect_children(&file.node);
        for child in children {
            self.alias_marking_visitor(&child);
        }

        self.current_file = saved_file;
        self.current_file_id = saved_file_id;
        self.current_file_symbol = saved_file_symbol;
        self.scope_stack = saved_scope_stack;
    }

    fn alias_marking_visitor(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::BinaryExpression => {

                if Self::is_common_js_module_exports(node) {
                    if let NodeData::BinaryExpression(bin) = &node.data {
                        if bin.right.kind == SyntaxKind::Identifier {
                            self.mark_linked_aliases(&bin.right);
                        }
                    }
                }
            }
            SyntaxKind::ExportAssignment => {
                if let Some(expr) = node.expression() {
                    if expr.kind == SyntaxKind::Identifier {
                        self.mark_linked_aliases(expr);
                    }
                }
            }
            SyntaxKind::ExportSpecifier => {
                if let Some(name) = Self::export_specifier_name(node) {
                    self.mark_linked_aliases(&name);
                }
            }
            _ => {}
        }

        let children = collect_children(node);
        for child in children {
            self.alias_marking_visitor(&child);
        }
    }

    fn mark_linked_aliases(&mut self, node: &Arc<Node>) {
        let export_symbol = self.resolve_export_symbol_for_alias(node);
        let mut export_symbol = export_symbol;

        let mut visited: Vec<u64> = Vec::new();
        while let Some(sym) = export_symbol {
            if visited.contains(&sym.id()) {
                break;
            }
            visited.push(sym.id());

            let mut next_symbol: Option<Arc<Symbol>> = None;
            let declarations = sym.declarations.clone();
            for declaration in declarations.iter() {
                self.declaration_links
                    .get_or_default(declaration)
                    .is_visible = true.into();

                if declaration.kind == SyntaxKind::ImportEqualsDeclaration {
                    if let NodeData::ImportEqualsDeclaration(d) = &declaration.data {
                        let first_id = Self::first_identifier_of(&d.module_reference);
                        if let Some(first_id) = first_id {

                            let saved = self.scope_stack.clone();

                            if let Some(parent) = declaration.parent.clone() {
                                self.scope_stack.push(parent.id());
                            }
                            let resolved = self.resolve_identifier_with_meaning(
                                &first_id,
                                SymbolFlags::VALUE
                                    | SymbolFlags::TYPE
                                    | SymbolFlags::NAMESPACE
                                    | SymbolFlags::Alias,
                            );
                            self.scope_stack = saved;
                            next_symbol = resolved;
                        }
                    }
                }
            }
            export_symbol = next_symbol;
        }
    }

    fn resolve_export_symbol_for_alias(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let parent = node.parent.clone()?;
        match parent.kind {
            SyntaxKind::ExportAssignment | SyntaxKind::BinaryExpression => {
                let name = node.text();
                self.resolve_name_in_file_scope(name)
            }
            SyntaxKind::ExportSpecifier => {

                let spec_name = Self::export_specifier_name(&parent)?;
                let name = spec_name.text();
                self.resolve_name_in_file_scope(name)
            }
            _ => None,
        }
    }

    fn resolve_name_in_file_scope(&self, name: &str) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        let file_id = self.current_file_id;
        if let Some(file_sym) = symbol_map.symbols.get(&file_id) {
            if let Some(sym) = file_sym.members.get(name) {
                return self.follow_alias(sym);
            }
        }

        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    return self.follow_alias(sym);
                }
            }
            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                if let Some(sym) = container_sym.members.get(name) {
                    return self.follow_alias(sym);
                }
            }
        }
        None
    }

    fn is_common_js_module_exports(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::BinaryExpression {
            return false;
        }

        let NodeData::BinaryExpression(bin) = &node.data else {
            return false;
        };
        let left_is_module_exports = matches!(&bin.left.data,
            NodeData::PropertyAccessExpression(pa)
                if pa.expression.kind == SyntaxKind::Identifier
                && pa.expression.text() == "module"
                && pa.name.text() == "exports");
        let left_is_exports_dot = matches!(&bin.left.data,
            NodeData::PropertyAccessExpression(pa)
                if pa.expression.kind == SyntaxKind::Identifier
                && pa.expression.text() == "exports");
        left_is_module_exports || left_is_exports_dot
    }

    fn export_specifier_name(node: &Arc<Node>) -> Option<Arc<Node>> {
        let NodeData::ExportSpecifier(d) = &node.data else {
            return None;
        };
        Some(if let Some(pn) = &d.property_name {
            Arc::clone(pn)
        } else {
            Arc::clone(&d.name)
        })
    }

    fn first_identifier_of(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(node);
        loop {
            match &current.data {
                NodeData::Identifier(_) => return Some(current),
                NodeData::QualifiedName(q) => current = Arc::clone(&q.left),
                NodeData::PropertyAccessExpression(p) => current = Arc::clone(&p.expression),
                _ => return None,
            }
        }
    }

    pub fn is_entity_name_visible(
        &mut self,
        entity_name: &Arc<Node>,
        enclosing_declaration: &Arc<Node>,
    ) -> SymbolAccessibilityResult {
        let meaning = Self::meaning_of_entity_name_reference(entity_name);
        let first_identifier =
            Self::first_identifier_of(entity_name).unwrap_or_else(|| Arc::clone(entity_name));

        let symbol =
            self.resolve_name_in_enclosure(enclosing_declaration, first_identifier.text(), meaning);

        if let Some(sym) = &symbol {
            if sym.flags.contains(SymbolFlags::TypeParameter) && meaning.contains(SymbolFlags::TYPE)
            {
                return SymbolAccessibilityResult {
                    accessibility: SymbolAccessibility::Accessible,
                    ..Default::default()
                };
            }
        }

        let symbol = match symbol {
            Some(s) => s,
            None => {
                return SymbolAccessibilityResult {
                    accessibility: SymbolAccessibility::NotResolved,
                    error_symbol_name: first_identifier.text().to_string(),
                    error_node: Some(first_identifier),
                    ..Default::default()
                };
            }
        };

        match self.has_visible_declarations(&symbol) {
            Some(result) => result,
            None => SymbolAccessibilityResult {
                accessibility: SymbolAccessibility::NotAccessible,
                error_symbol_name: first_identifier.text().to_string(),
                error_node: Some(first_identifier),
                ..Default::default()
            },
        }
    }

    fn resolve_name_in_enclosure(
        &self,
        enclosing_declaration: &Arc<Node>,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();

        let mut current: Option<Arc<Node>> = Some(Arc::clone(enclosing_declaration));
        while let Some(n) = current {
            if let Some(sym) = symbol_map.symbol_of(&n) {
                if let Some(found) = sym.members.get(name) {
                    if found.flags.intersects(meaning) {
                        return self.follow_alias(found);
                    }
                }
            }
            if let Some(locals) = symbol_map.locals.get(&n.id()) {
                if let Some(found) = locals.get(name) {
                    if found.flags.intersects(meaning) {
                        return self.follow_alias(found);
                    }
                }
            }
            current = n.parent.clone();
        }

        if let Some(file_sym) = symbol_map.symbols.get(&self.current_file_id) {
            if let Some(found) = file_sym.members.get(name) {
                if found.flags.intersects(meaning) {
                    return self.follow_alias(found);
                }
            }
        }
        None
    }

    fn meaning_of_entity_name_reference(entity_name: &Arc<Node>) -> SymbolFlags {
        let parent = match &entity_name.parent {
            Some(p) => p,
            None => return SymbolFlags::TYPE,
        };

        let is_value_position = matches!(
            parent.kind,
            SyntaxKind::TypeQuery
                | SyntaxKind::ComputedPropertyName
                | SyntaxKind::BinaryExpression
                | SyntaxKind::ExpressionWithTypeArguments
        ) || (parent.kind == SyntaxKind::TypePredicate
            && matches!(&parent.data, NodeData::TypePredicateNode(tp) if tp.parameter_name.id() == entity_name.id()));
        if is_value_position {
            return SymbolFlags::VALUE | SymbolFlags::ExportValue;
        }

        let is_namespace_position = entity_name.kind == SyntaxKind::QualifiedName
            || entity_name.kind == SyntaxKind::PropertyAccessExpression
            || parent.kind == SyntaxKind::ImportEqualsDeclaration
            || (parent.kind == SyntaxKind::QualifiedName
                && matches!(&parent.data, NodeData::QualifiedName(q) if q.left.id() == entity_name.id()))
            || (parent.kind == SyntaxKind::PropertyAccessExpression
                && matches!(&parent.data, NodeData::PropertyAccessExpression(pa) if pa.expression.id() == entity_name.id()))
            || (parent.kind == SyntaxKind::ElementAccessExpression
                && matches!(&parent.data, NodeData::ElementAccessExpression(ea) if ea.expression.id() == entity_name.id()));
        if is_namespace_position {
            return SymbolFlags::NAMESPACE;
        }
        SymbolFlags::TYPE
    }

    pub fn has_visible_declarations(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<SymbolAccessibilityResult> {
        let declarations = symbol.declarations.clone();
        for declaration in declarations.iter() {
            if declaration.kind == SyntaxKind::Identifier {
                continue;
            }
            if self.is_declaration_visible(declaration) {
                continue;
            }

            if let Some(any_import) = Checker::get_any_import_syntax(declaration) {
                let is_exported =
                    any_import.has_syntactic_modifier(crate::ast::ModifierFlags::Export);
                if !is_exported {
                    if let Some(parent) = any_import.parent.clone() {
                        if self.is_declaration_visible(&parent) {
                            self.declaration_links
                                .get_or_default(declaration)
                                .is_visible = true.into();
                            continue;
                        }
                    }
                }
            }

            if declaration.kind == SyntaxKind::VariableDeclaration {
                let var_list = declaration.parent.clone();
                let var_stmt = var_list.as_ref().and_then(|p| p.parent.clone());
                if let Some(vs) = &var_stmt {
                    if vs.kind == SyntaxKind::VariableStatement
                        && !vs.has_syntactic_modifier(crate::ast::ModifierFlags::Export)
                    {
                        if let Some(container) = vs.parent.clone() {
                            if self.is_declaration_visible(&container) {
                                self.declaration_links
                                    .get_or_default(declaration)
                                    .is_visible = true.into();
                                continue;
                            }
                        }
                    }
                }
            }

            if Checker::is_late_visibility_painted_statement(declaration)
                && !declaration.has_syntactic_modifier(crate::ast::ModifierFlags::Export)
            {
                if let Some(parent) = declaration.parent.clone() {
                    if self.is_declaration_visible(&parent) {
                        self.declaration_links
                            .get_or_default(declaration)
                            .is_visible = true.into();
                        continue;
                    }
                }
            }

            return None;
        }
        Some(SymbolAccessibilityResult {
            accessibility: SymbolAccessibility::Accessible,
            ..Default::default()
        })
    }

    pub fn get_enum_member_value_string(&mut self, node: &Arc<Node>) -> Option<String> {

        let NodeData::EnumMember(data) = &node.data else {
            return None;
        };
        let initializer = data.initializer.as_ref()?;
        match initializer.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(s) = &initializer.data {
                    Some(format!("\"{}\"", s.text))
                } else {
                    None
                }
            }
            SyntaxKind::NumericLiteral => {
                if let NodeData::NumericLiteral(n) = &initializer.data {
                    Some(n.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::PrefixUnaryExpression => {

                if let NodeData::PrefixUnaryExpression(unary) = &initializer.data {
                    let operand_text = match &unary.operand.data {
                        NodeData::NumericLiteral(n) => n.text.clone(),
                        _ => return None,
                    };
                    let op = match unary.operator {
                        SyntaxKind::MinusToken => "-",
                        SyntaxKind::PlusToken => "+",
                        _ => return None,
                    };
                    Some(format!("{}{}", op, operand_text))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn is_optional_parameter(&self, node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ParameterDeclaration(data) => {

                data.question_token.is_some() || node.kind == SyntaxKind::RestType
            }
            _ => false,
        }
    }

    pub fn is_literal_const_declaration(&self, node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::VariableDeclaration {
            return false;
        }
        let NodeData::VariableDeclaration(data) = &node.data else {
            return false;
        };

        if data.initializer.is_none() {
            return false;
        }
        let initializer = data.initializer.as_ref().unwrap();
        matches!(
            initializer.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::PrefixUnaryExpression
        )
    }

    pub fn get_constant_value(&mut self, node: &Arc<Node>) -> Option<String> {
        if node.kind == SyntaxKind::EnumMember {
            return self.get_enum_member_value_string(node);
        }
        match node.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(s) = &node.data {
                    Some(format!("\"{}\"", s.text))
                } else {
                    None
                }
            }
            SyntaxKind::NumericLiteral => {
                if let NodeData::NumericLiteral(n) = &node.data {
                    Some(n.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::TrueKeyword => Some("true".to_string()),
            SyntaxKind::FalseKeyword => Some("false".to_string()),
            SyntaxKind::NullKeyword => Some("null".to_string()),
            _ => None,
        }
    }

    pub fn is_referenced_alias_declaration(&self, node: &Arc<Node>) -> bool {
        if let Some(links) = self.declaration_links.get(node) {
            if links.is_visible.is_true() {
                return true;
            }
        }

        true
    }

    pub fn is_value_alias_declaration(&self, node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ImportSpecifier(data) => !data.is_type_only,
            _ => true,
        }
    }

    pub fn get_effective_declaration_flags(&self, node: &Arc<Node>) -> u32 {
        node.syntactic_modifier_flags().bits()
    }

    pub fn get_symbol_of_declaration(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.program.symbol_map().symbol_of(node).cloned()
    }

    pub fn is_const_enum_member(&self, symbol: &Symbol) -> bool {
        symbol.flags.contains(SymbolFlags::ConstEnum)
    }
}

fn collect_children(node: &Arc<Node>) -> Vec<Arc<Node>> {
    let mut children: Vec<Arc<Node>> = Vec::new();
    for_each_child(node, |child| {
        children.push(Arc::clone(child));
        false
    });
    children
}
