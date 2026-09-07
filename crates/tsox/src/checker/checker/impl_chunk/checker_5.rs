#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_fresh_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
        if !t.flags.intersects(TYPE_FLAGS_FRESHABLE) {
            return Arc::clone(t);
        }
        let lit = match &t.data {
            TypeData::Literal(lit) => lit,
            _ => {
                return Arc::clone(t);
            }
        };

        if lit.regular_type.get().is_some() {
            return Arc::clone(t);
        }

        let value = lit.value.clone();
        let flags = t.flags;
        let regular = Arc::clone(t);
        let fresh = lit.fresh_type.get_or_init(move || {
            Arc::new(Type::new(
                flags,
                TypeData::Literal(LiteralTypeData {
                    value,

                    fresh_type: OnceLock::new(),

                    regular_type: OnceLock::from(regular),
                }),
            ))
        });
        Arc::clone(fresh)
    }

    pub fn is_literal_of_contextual_type(
        &self,
        candidate: &Arc<Type>,
        contextual: &Arc<Type>,
    ) -> bool {
        if contextual
            .flags
            .intersects(TypeFlags::Union | TypeFlags::Intersection)
        {
            if let TypeData::Union(u) = &contextual.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|t| self.is_literal_of_contextual_type(candidate, t));
            }
            return false;
        }
        if contextual.flags.intersects(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_base_constraint_of_type(contextual) {
                return (constraint.flags.intersects(TypeFlags::String)
                    && candidate.flags.intersects(TypeFlags::StringLiteral))
                    || (constraint.flags.intersects(TypeFlags::Number)
                        && candidate.flags.intersects(TypeFlags::NumberLiteral))
                    || self.is_literal_of_contextual_type(candidate, &constraint);
            }
            return false;
        }

        (contextual.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::Index
                | TypeFlags::TemplateLiteral
                | TypeFlags::StringMapping,
        ) && candidate.flags.intersects(TypeFlags::StringLiteral))
            || (contextual.flags.intersects(TypeFlags::NumberLiteral)
                && candidate.flags.intersects(TypeFlags::NumberLiteral))
            || (contextual.flags.intersects(TypeFlags::BigIntLiteral)
                && candidate.flags.intersects(TypeFlags::BigIntLiteral))
            || (contextual.flags.intersects(TypeFlags::BooleanLiteral)
                && candidate.flags.intersects(TypeFlags::BooleanLiteral))
            || (contextual.flags.intersects(TypeFlags::UniqueESSymbol)
                && candidate.flags.intersects(TypeFlags::UniqueESSymbol))
    }

    pub fn get_widened_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
        if crate::checker::is_fresh_literal_type(t) {
            if t.flags.intersects(TYPE_FLAGS_ENUM_LIKE) {
                if let Some(sym) = &t.symbol
                    && sym.flags.contains(SymbolFlags::EnumMember)
                    && let Some(parent) = &sym.parent
                    && let Some(cached) = self
                        .type_alias_links
                        .get(parent)
                        .and_then(|l| l.declared_type.clone())
                {
                    return cached;
                }
            }
            if t.flags.contains(TypeFlags::StringLiteral) {
                return self.string_type();
            }
            if t.flags.contains(TypeFlags::NumberLiteral) {
                return self.number_type();
            }
            if t.flags.contains(TypeFlags::BigIntLiteral) {
                return self.bigint_type();
            }
            if t.flags.contains(TypeFlags::BooleanLiteral) {
                return self.boolean_type();
            }
        }

        if let TypeData::Union(union_data) = &t.data {
            let widened: Vec<Arc<Type>> = union_data
                .union_or_intersection
                .types
                .iter()
                .map(|member| self.get_widened_literal_type(member))
                .collect();

            if widened
                .iter()
                .zip(union_data.union_or_intersection.types.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o))
            {
                return Arc::clone(t);
            }
            return self.build_union_from_types(widened);
        }
        Arc::clone(t)
    }

    pub fn get_regular_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(TYPE_FLAGS_FRESHABLE) {
            if let TypeData::Literal(lit) = &t.data {
                if let Some(regular) = lit.regular_type.get() {
                    return Arc::clone(regular);
                }
            }
        }

        if let TypeData::Union(union_data) = &t.data {
            let regularized: Vec<Arc<Type>> = union_data
                .union_or_intersection
                .types
                .iter()
                .map(|member| self.get_regular_type_of_literal_type(member))
                .collect();
            if regularized
                .iter()
                .zip(union_data.union_or_intersection.types.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o))
            {
                return Arc::clone(t);
            }
            return self.build_union_from_types(regularized);
        }
        Arc::clone(t)
    }

    pub fn get_widened_literal_type_for_initializer(
        &mut self,
        declaration: &Arc<Node>,
        t: &Arc<Type>,
    ) -> Arc<Type> {
        if self
            .get_combined_node_flags(declaration)
            .intersects(NodeFlags::Constant)
        {
            return Arc::clone(t);
        }
        self.get_widened_literal_type(t)
    }

    pub fn get_diagnostics(&self) -> &DiagnosticsCollection {
        &self.diagnostics
    }

    pub fn get_suggestion_diagnostics(&self) -> &DiagnosticsCollection {
        &self.suggestion_diagnostics
    }

    pub fn get_combined_node_flags(&mut self, node: &Arc<Node>) -> NodeFlags {
        let mut flags = node.flags;
        let mut parent = node.parent.clone();
        while let Some(p) = parent {
            if p.kind == SyntaxKind::SourceFile {
                break;
            }
            flags |= p.flags;
            parent = p.parent.clone();
        }
        flags
    }

    pub fn get_combined_modifier_flags(&mut self, node: &Arc<Node>) -> ModifierFlags {
        if let Some(cached) = &self.last_combined_modifier_flags_node {
            if Arc::ptr_eq(cached, node) {
                return self.last_combined_modifier_flags_result;
            }
        }
        let flags = ast_get_combined_modifier_flags(node);
        self.last_combined_modifier_flags_node = Some(Arc::clone(node));
        self.last_combined_modifier_flags_result = flags;
        flags
    }

    pub fn get_root_declaration(node: &Arc<Node>) -> Arc<Node> {
        let mut current = Arc::clone(node);
        while current.kind == SyntaxKind::BindingElement {
            let parent = match &current.parent {
                Some(p) => Arc::clone(p),
                None => break,
            };
            let grandparent = match &parent.parent {
                Some(gp) => Arc::clone(gp),
                None => break,
            };
            current = grandparent;
        }
        current
    }

    pub fn get_declaration_container(node: &Arc<Node>) -> Option<Arc<Node>> {
        let root = Self::get_root_declaration(node);

        let skip = |kind: SyntaxKind| {
            matches!(
                kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::VariableDeclarationList
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::NamedImports
                    | SyntaxKind::NamespaceImport
                    | SyntaxKind::ImportClause
            )
        };
        let mut current = Some(root);
        while let Some(n) = current {
            if skip(n.kind) {
                current = n.parent.clone();
                continue;
            }
            return n.parent.clone();
        }
        None
    }

    pub fn is_global_source_file(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::SourceFile {
            return false;
        }
        !Self::is_external_or_common_js_module(node)
    }

    pub fn is_external_or_common_js_module(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::SourceFile {
            return false;
        }
        let NodeData::SourceFile(data) = &node.data else {
            return false;
        };
        for stmt in data.statements.nodes.iter() {
            match stmt.kind {
                SyntaxKind::ImportDeclaration
                | SyntaxKind::ExportDeclaration
                | SyntaxKind::ExportAssignment
                | SyntaxKind::NamespaceExportDeclaration
                | SyntaxKind::ImportEqualsDeclaration => return true,
                _ => {
                    if stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_external_module_augmentation(node: &Arc<Node>) -> bool {
        if !Self::is_ambient_module(node) {
            return false;
        }
        Self::is_module_augmentation_external(node)
    }
}
