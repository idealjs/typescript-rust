#![allow(unused_imports)]

use crate::checker::checker_impl_chunk::*;

impl Checker {
    pub fn get_fresh_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_fresh_type_of_literal_type"); 
        if !t.flags.intersects(TYPE_FLAGS_FRESHABLE) {
            return Arc::clone(t);
        }
        let lit = match &t.data {
            TypeData::Literal(lit) => lit,
            _ => {
                return Arc::clone(t);
            }
        };

        if let Some(fresh) = lit.fresh_type.get() {
            return Arc::clone(fresh);
        }

        let value = lit.value.clone();
        let flags = t.flags;
        let symbol = t.symbol.clone();
        let regular = Arc::clone(t);
        let mut fresh = Arc::new(Type::new(
            flags,
            TypeData::Literal(LiteralTypeData {
                value,

                fresh_type: OnceLock::new(),

                regular_type: OnceLock::from(regular),
            }),
        ));
        if let Some(f) = Arc::get_mut(&mut fresh) {
            f.symbol = symbol;
        }
        if let TypeData::Literal(fresh_lit) = &fresh.data {
            let _ = fresh_lit.fresh_type.set(Arc::clone(&fresh));
        }
        let _ = lit.fresh_type.set(Arc::clone(&fresh));
        fresh
    }

    pub fn is_literal_of_contextual_type(
        &mut self,
        candidate: &Arc<Type>,
        contextual: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("is_literal_of_contextual_type"); 
        if contextual
            .flags
            .intersects(TypeFlags::Union | TypeFlags::Intersection)
        {
            // Go isLiteralOfContextualType（checker.go:27055-27063）：联合与
            // 交集均按成分递归（core.Some(contextualType.Types(), ...)）
            if let Some(types) = contextual.types() {
                return types
                    .iter()
                    .any(|t| self.is_literal_of_contextual_type(candidate, t));
            }
            return false;
        }
        if contextual
            .flags
            .intersects(crate::checker::types_type_flags_instantiable_non_primitive::TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE)
        {
            // Go isLiteralOfContextualType 型参臂（checker.go:27064-27077）：
            // base 约束失联回落 unknown，掩码判定经 maybeTypeOfKind（联合按
            // 成分递归，联合型 flags 本体不聚合成分基元位——本仓联合构造与 Go
            // newUnionType 同不并入 includes，keyof T[K1] 的 base 约束
            // string|number|symbol 直接 flags 查恒 miss 误判非字面量语境）
            let constraint = self
                .get_base_constraint_of_type(contextual)
                .or_else(|| self.get_constraint_of_type_parameter(contextual))
                .unwrap_or_else(|| self.unknown_type());
            return (self.maybe_type_of_kind(&constraint, TypeFlags::String)
                && self.maybe_type_of_kind(candidate, TypeFlags::StringLiteral))
                || (self.maybe_type_of_kind(&constraint, TypeFlags::Number)
                    && self.maybe_type_of_kind(candidate, TypeFlags::NumberLiteral))
                || (self.maybe_type_of_kind(&constraint, TypeFlags::BigInt)
                    && self.maybe_type_of_kind(candidate, TypeFlags::BigIntLiteral))
                || (self.maybe_type_of_kind(&constraint, TypeFlags::ESSymbol)
                    && self.maybe_type_of_kind(candidate, TypeFlags::UniqueESSymbol))
                || self.is_literal_of_contextual_type(candidate, &constraint);
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
            || (contextual.flags.contains(TypeFlags::Boolean)
                && candidate.flags.intersects(TypeFlags::BooleanLiteral))
            || (contextual.flags.intersects(TypeFlags::UniqueESSymbol)
                && candidate.flags.intersects(TypeFlags::UniqueESSymbol))
    }

    pub fn get_widened_literal_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_widened_literal_type");
        if crate::checker::is_fresh_literal_type(t) {
            if t.flags.intersects(TYPE_FLAGS_ENUM_LIKE) {
                return self.get_base_type_of_enum_like_type(t);
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
            // Go getWidenedLiteralType 联合分支（checker.go:27017-27032）走
            // mapType→mapTypeEx→getUnionTypeEx（checker.go:27129-27131，
            // Literal 归约）重建
            return self.get_union_type_ex(
                widened,
                crate::checker::types::UnionReduction::Literal,
                None,
                None,
            );
        }
        Arc::clone(t)
    }

    pub fn get_regular_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_regular_type_of_literal_type"); 
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
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_widened_literal_type_for_initializer");
        if self
            .get_combined_node_flags(declaration)
            .intersects(NodeFlags::Constant)
            || crate::checker::is_declaration_readonly(declaration)
        {
            return Arc::clone(t);
        }
        self.get_widened_literal_type(t)
    }

    pub fn get_diagnostics(&self) -> &DiagnosticsCollection { ::tsox_core::fntrace::enter("get_diagnostics"); 
        &self.diagnostics
    }

    pub fn get_suggestion_diagnostics(&self) -> &DiagnosticsCollection { ::tsox_core::fntrace::enter("get_suggestion_diagnostics"); 
        &self.suggestion_diagnostics
    }

    pub fn get_combined_node_flags(&mut self, node: &Arc<Node>) -> NodeFlags { ::tsox_core::fntrace::enter("get_combined_node_flags");
        // Go GetCombinedNodeFlags（ast/utilities.go:1322-1336）：GetRootDeclaration
        // 之后仅并入 VariableDeclarationList 与 VariableStatement 两级 flags，
        // 不做全祖先遍历——否则外层 const 声明（如 const f = function() { var x = ''; }
        // 的 const 列表）的 NodeFlagsConst 会泄漏给函数体内的 var 声明，
        // widenTypeInferredFromInitializer（checker.go:17882）误判 const 上下文
        // 跳过 fresh 字面量拓宽（'' → string、1 → number 不发生）
        tsox_frontend::ast::utilities::get_combined_node_flags(node)
    }

    pub fn get_combined_modifier_flags(&mut self, node: &Arc<Node>) -> ModifierFlags { ::tsox_core::fntrace::enter("get_combined_modifier_flags"); 
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

    pub fn get_root_declaration(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_root_declaration"); 
        let mut current = Arc::clone(node);
        while current.kind == SyntaxKind::BindingElement {
            let parent = match current.parent() {
                Some(p) => Arc::clone(&p),
                None => break,
            };
            let grandparent = match parent.parent() {
                Some(gp) => gp,
                None => break,
            };
            current = grandparent;
        }
        current
    }

    pub fn get_declaration_container(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_declaration_container"); 
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
                current = n.parent();
                continue;
            }
            return n.parent();
        }
        None
    }

    pub fn is_global_source_file(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_global_source_file"); 
        if node.kind != SyntaxKind::SourceFile {
            return false;
        }
        !Self::is_external_or_common_js_module(node)
    }

    pub fn is_external_or_common_js_module(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_external_or_common_js_module"); 
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
                    if stmt.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Export) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_external_module_augmentation(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_external_module_augmentation"); 
        if !Self::is_ambient_module(node) {
            return false;
        }
        Self::is_module_augmentation_external(node)
    }
}
