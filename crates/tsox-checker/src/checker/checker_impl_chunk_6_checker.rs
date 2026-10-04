#![allow(unused_imports)]

use crate::checker::checker_impl_chunk_6::*;
use crate::checker::mig::m1b::jsnum_from_string;
use crate::checker::mig::m1b::parse_pseudo_big_int;
use crate::checker::mig::m1b_4::jsnum_negate;
use tsox_core::diagnostics::messages_generated::OPERATOR_0_CANNOT_BE_APPLIED_TO_TYPE_1;

impl Checker {
    pub fn get_type_of_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_node");
        if node.kind == SyntaxKind::ThisKeyword {
            return self.compute_type_of_node(node);
        }

        // Go getTypeOfNode（checker.go:33787-33818）：class extends 的 EWT 既非
        // type node 也非 expression node，落 classType 分支取 getBaseTypes 首基型。
        // 该分支先于节点级缓存：Go 此查询无缓存，check 期经 check_heritage_clause
        // 系写入 type_node_links 的静态侧产物不得遮蔽
        if node.kind == SyntaxKind::ExpressionWithTypeArguments {
            if let Some((class_decl, false)) =
                tsox_frontend::ast::mig::m3g_3::try_get_class_implementing_or_extending_heritage_clause_element(node)
            {
                let result = self.type_of_class_extends_element(&class_decl);
                self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
                return result;
            }
        }

        if let Some(cached) = self
            .type_node_links
            .get(node)
            .and_then(|l| l.resolved_type.clone())
        {
            if node.kind == SyntaxKind::PropertyAccessExpression
                && cached.flags == crate::checker::types::TypeFlags::Any
            {
                let result = self.compute_type_of_node(node);
                self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
                return result;
            }
            return cached;
        }
        let result = self.compute_type_of_node(node);
        self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
        result
    }

    pub(crate) fn type_of_class_extends_element(&mut self, class_decl: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("type_of_class_extends_element");
        let Some(symbol) = self.symbol_of_node(class_decl) else {
            return self.error_type();
        };
        let class_type = self.get_declared_type_of_class_or_interface(&symbol);
        let base = self
            .get_base_types(&class_type)
            .into_iter()
            .next()
            .or_else(|| {
                self.extends_base_class_node(class_decl).map(
                    |(_, base_symbol, heritage_expr)| {
                        self.get_type_from_class_or_interface_reference(&heritage_expr, &base_symbol)
                    },
                )
            });
        match base {
            Some(base) => {
                let this_argument =
                    crate::checker::mig::wc2::r22k3_defs::interface_this_type(&class_type);
                self.get_type_with_this_argument(&base, this_argument.as_ref(), false)
            }
            None => self.error_type(),
        }
    }

    pub(crate) fn compute_type_of_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("compute_type_of_node");
        // Go getTypeOfNode：类型节点整体委托 getTypeFromTypeNode（含 NamedTupleMember 等）
        if tsox_frontend::ast::is_type_node(node) {
            return self.get_type_from_type_node(node);
        }
        match node.kind {
            SyntaxKind::NumericLiteral => {
                if let tsox_frontend::ast::NodeData::NumericLiteral(data) = &node.data {
                    let lit = self.infer_number_literal_type(&data.text);
                    return self.get_fresh_type_of_literal_type(&lit);
                }
                self.number_type()
            }
            SyntaxKind::StringLiteral => {
                if let tsox_frontend::ast::NodeData::StringLiteral(data) = &node.data {
                    let lit = self.infer_string_literal_type(&data.text);
                    return self.get_fresh_type_of_literal_type(&lit);
                }
                self.string_type()
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                if let tsox_frontend::ast::NodeData::NoSubstitutionTemplateLiteral(data) = &node.data
                {
                    let lit = self.infer_string_literal_type(&data.text);
                    return self.get_fresh_type_of_literal_type(&lit);
                }
                self.string_type()
            }
            SyntaxKind::TrueKeyword => self.get_fresh_type_of_literal_type(&self.true_type()),
            SyntaxKind::FalseKeyword => self.get_fresh_type_of_literal_type(&self.false_type()),
            SyntaxKind::NullKeyword => self.nullish_widening_type(self.null_type()),
            SyntaxKind::UndefinedKeyword => self.nullish_widening_type(self.undefined_type()),
            SyntaxKind::BigIntLiteral => {
                self.check_grammar_big_int_literal(node);
                let literal = self.get_big_int_literal_type(parse_pseudo_big_int(&node.text()));
                self.get_fresh_type_of_literal_type(&literal)
            }
            SyntaxKind::ArrayLiteralExpression => {
                return self.get_type_of_array_literal(node);
            }
            SyntaxKind::ObjectLiteralExpression => {
                return self.get_type_of_object_literal(node);
            }
            SyntaxKind::PropertyAssignment => {
                if let tsox_frontend::ast::NodeData::PropertyAssignment(d) = &node.data
                    && let Some(literal) = node
                        .parent()
                        .as_ref()
                        .filter(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
                {
                    let name = self.get_property_name_from_node(&d.name);
                    return self.property_assignment_type(node, &d.initializer, literal, &name);
                }
                self.get_any_type()
            }
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
                let base = self.get_type_of_function_like(node);
                self.attach_expando_to_function_expression_type(node, base)
            }
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
                self.get_type_of_class_declaration(node)
            }
            SyntaxKind::RegularExpressionLiteral => self.global_regexp_type(),
            SyntaxKind::FunctionDeclaration => self.get_type_of_function_like(node),
            SyntaxKind::Identifier => self.get_type_of_identifier(node),
            SyntaxKind::MetaProperty => self.get_type_of_meta_property(node),

            SyntaxKind::BinaryExpression => self.get_type_of_binary_expression(node),
            SyntaxKind::PrefixUnaryExpression => {
                if let tsox_frontend::ast::NodeData::PrefixUnaryExpression(data) = &node.data {
                    if data.operator == SyntaxKind::MinusToken
                        && data.operand.kind == SyntaxKind::BigIntLiteral
                    {
                        self.check_grammar_big_int_literal(&data.operand);
                        let positive = parse_pseudo_big_int(&data.operand.text());
                        let negative =
                            tsox_core::jsnum::PseudoBigInt::new(&positive.to_string(), true);
                        let literal = self.get_big_int_literal_type(negative);
                        return self.get_fresh_type_of_literal_type(&literal);
                    }
                    if data.operator == SyntaxKind::PlusToken
                        && data.operand.kind == SyntaxKind::BigIntLiteral
                    {
                        self.check_grammar_big_int_literal(&data.operand);
                        self.error_message(
                            &data.operand,
                            OPERATOR_0_CANNOT_BE_APPLIED_TO_TYPE_1,
                            &["+".to_string(), "bigint".to_string()],
                        );
                        return self.number_type();
                    }
                    match data.operator {
                        SyntaxKind::MinusToken
                            if data.operand.kind == SyntaxKind::NumericLiteral =>
                        {
                            let value =
                                jsnum_negate(jsnum_from_string(data.operand.text()));
                            let literal = self.get_number_literal_type(value);
                            return self.get_fresh_type_of_literal_type(&literal);
                        }
                        SyntaxKind::PlusToken
                            if data.operand.kind == SyntaxKind::NumericLiteral =>
                        {
                            let value = jsnum_from_string(data.operand.text());
                            let literal = self.get_number_literal_type(value);
                            return self.get_fresh_type_of_literal_type(&literal);
                        }
                        SyntaxKind::ExclamationToken => return self.boolean_type(),

                        SyntaxKind::DeleteKeyword => return self.boolean_type(),

                        SyntaxKind::VoidKeyword => return self.undefined_type(),

                        _ => return self.number_type(),
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::PostfixUnaryExpression => self.number_type(),
            SyntaxKind::CallExpression => self.get_return_type_of_call_expression(node),
            SyntaxKind::NewExpression => self.get_return_type_of_new_expression(node),
            SyntaxKind::PropertyAccessExpression => self.get_type_of_property_access(node),
            SyntaxKind::ElementAccessExpression => self.get_type_of_element_access(node),
            SyntaxKind::ParenthesizedExpression => {
                if let tsox_frontend::ast::NodeData::ParenthesizedExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
                }
                self.get_any_type()
            }
            SyntaxKind::AsExpression => {
                if let tsox_frontend::ast::NodeData::AsExpression(data) = &node.data {
                    if data.type_node.kind == SyntaxKind::ConstKeyword {
                        return self.get_const_assertion_type(&data.expression);
                    }
                    return self.get_type_from_type_node(&data.type_node);
                }
                self.get_any_type()
            }
            SyntaxKind::SatisfiesExpression => {
                if let tsox_frontend::ast::NodeData::SatisfiesExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
                }
                self.get_any_type()
            }
            SyntaxKind::TypeAssertionExpression => {
                if let tsox_frontend::ast::NodeData::TypeAssertion(data) = &node.data {
                    // Go getTypeOfExpressionOfTypeAssertion：isConstTypeReference
                    // 不解析类型名，取表达式类型并保留字面量（const 断言）
                    if crate::checker::utilities_has_only_expression_initialization::is_const_type_reference(&data.type_node) {
                        return self.get_type_of_node(&data.expression);
                    }
                    return self.get_type_from_type_node(&data.type_node);
                }
                self.get_any_type()
            }
            SyntaxKind::NonNullExpression => {
                if let tsox_frontend::ast::NodeData::NonNullExpression(data) = &node.data {
                    let operand_type = self.get_type_of_node(&data.expression);
                    return self.remove_flags_from_union(
                        &operand_type,
                        TypeFlags::Undefined | TypeFlags::Null,
                    );
                }
                self.get_any_type()
            }
            SyntaxKind::ConditionalExpression => {
                if let tsox_frontend::ast::NodeData::ConditionalExpression(data) = &node.data {
                    let true_type = self.get_type_of_node(&data.when_true);
                    let false_type = self.get_type_of_node(&data.when_false);
                    let types = vec![true_type, false_type];
                    // Go checkConditionalExpression：UnionReductionSubtype（移除可赋给
                    // 其他成员的 structured 成员，如 any[] ⊑ number[] → number[]）
                    let reduced = self.remove_subtype_redundant_members(types);
                    return self.get_union_type(reduced);
                }
                self.get_any_type()
            }
            SyntaxKind::TemplateExpression => self.get_type_of_expression(node),
            SyntaxKind::TaggedTemplateExpression => {
                if let tsox_frontend::ast::NodeData::TaggedTemplateExpression(data) = &node.data {
                    let tag_type = self.get_type_of_node(&data.tag);
                    if let Some(structured) = tag_type.as_structured() {
                        for sig in structured.call_signatures() {
                            if let Some(rt) = self.get_return_type_of_signature(sig) {
                                return rt;
                            }
                            return self.get_any_type();
                        }
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::DeleteExpression => self.boolean_type(),
            SyntaxKind::VoidExpression => self.undefined_type(),
            SyntaxKind::TypeOfExpression => self.typeof_type(),
            SyntaxKind::YieldExpression => {
                // Go checkYieldExpression：yield* 的类型 = 操作数迭代器的 TReturn
                if let tsox_frontend::ast::NodeData::YieldExpression(data) = &node.data
                    && data.asterisk_token.is_some()
                    && let Some(expr) = &data.expression
                {
                    let operand_type = self.get_type_of_node(expr);
                    if let Some(t) = self.get_yield_star_return_type(&operand_type) {
                        return t;
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::AwaitExpression => {
                if let tsox_frontend::ast::NodeData::AwaitExpression(data) = &node.data {
                    let operand = Arc::clone(&data.expression);

                    if let Some(ns) = self.type_of_dynamic_import(&operand) {
                        return ns;
                    }
                    let operand_type = self.get_type_of_node(&operand);
                    return match self.get_awaited_type(&operand_type) {
                        Some(awaited) => awaited,
                        None => operand_type,
                    };
                }
                self.get_any_type()
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => {
                if node.kind == SyntaxKind::ThisKeyword {
                    return self.this_expression_type(node);
                }
                if node.kind == SyntaxKind::SuperKeyword
                    && self.super_in_computed_name_of_innermost_class(node)
                    && self.enclosing_class_stack.len() >= 2
                {
                    return self
                        .this_type_stack
                        .get(self.this_type_stack.len() - 2)
                        .cloned()
                        .unwrap_or_else(|| self.get_any_type());
                }

                let syntactic = crate::checker::checker_super_expressions::super_container_class_and_staticity(node);
                if let Some(class) = syntactic
                    .as_ref()
                    .map(|(c, _)| Arc::clone(c))
                    .or_else(|| self.enclosing_class_stack.last().cloned())
                {
                    let is_static_member = syntactic
                        .as_ref()
                        .map(|(_, s)| *s)
                        .unwrap_or(self.this_container_stack.last() == Some(&ThisContainerKind::StaticMember));
                    // Go checkSuperExpression（checker.go:8417-8422）：
                    // IsStatic(container) || isCallExpression → getBaseConstructorTypeOfClass
                    let is_call_expression = node.parent().is_some_and(|p| {
                        matches!(&p.data,
                            tsox_frontend::ast::NodeData::CallExpression(c) if Arc::ptr_eq(&c.expression, node))
                    });
                    if (is_static_member || is_call_expression)
                        && let Some(sym) = self.symbol_of_node(&class)
                    {
                        let class_type = self.get_declared_type_of_symbol(&sym);
                        if let Some(t) = self.get_base_constructor_type_of_class(&class_type) {
                            return t;
                        }
                    }
                    if let Some(heritage) =
                        crate::checker::checker_classes_ctor_super_calls::class_extends_heritage_element(&class)
                    {
                        if crate::checker::checker_classes_ctor_super_calls::class_decl_extends_null(&class)
                            && !is_static_member
                        {
                            return self.null_type();
                        }
                        let base_expr = crate::checker::checker_classes_ctor_super_calls::expression_with_type_arguments_expression(&heritage);
                        let base_symbol = (base_expr.kind == SyntaxKind::Identifier)
                            .then(|| self.resolve_identifier(&base_expr))
                            .flatten();
                        if let Some(sym) = base_symbol
                            && sym.flags.intersects(SymbolFlags::Class | SymbolFlags::Interface)
                        {
                            let t = if is_static_member {
                                sym.declarations
                                    .iter()
                                    .find(|d| {
                                        matches!(
                                            d.kind,
                                            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                                        )
                                    })
                                    .map(|d| self.get_type_of_class_declaration(d))
                                    .unwrap_or_else(|| self.get_declared_type_of_symbol(&sym))
                            } else {
                                self.get_declared_type_of_symbol(&sym)
                            };
                            if !t.flags.contains(TypeFlags::Any) {
                                return t;
                            }
                        }
                        if is_static_member {
                            let t = self.get_type_of_node(&base_expr);
                            if !t.flags.contains(TypeFlags::Any) {
                                return t;
                            }
                        }
                    }
                }
                let r = self
                    .this_type_stack
                    .last()
                    .cloned()
                    .unwrap_or_else(|| self.get_any_type());
                r
            }
            _ => self.get_any_type(),
        }
    }

    pub(crate) fn get_type_of_identifier(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_identifier");
        // Go getTypeOfNode（checker.go:33826-33832）：类型声明名走
        // IsTypeDeclarationName → getDeclaredTypeOfSymbol，先于
        // IsDeclarationNameOrImportPropertyName 的 getTypeOfSymbol（后者对
        // class 给出构造器型 typeof A）。getDeclaredTypeOfSymbol 目前只接入
        // Class/Interface/TypeAlias，其余声明名维持既有求值路径
        if tsox_frontend::ast::mig::m3g_2::is_type_declaration_name(node)
            && let Some(symbol) = self.resolve_identifier(node)
            && symbol
                .flags
                .intersects(SymbolFlags::Class | SymbolFlags::Interface)
        {
            return self.get_declared_type_of_symbol(&symbol);
        }
        if let Some(parent) = node.parent()
            && parent.kind == SyntaxKind::MetaProperty
            && tsox_frontend::ast::mig::m3g_2::is_right_side_of_qualified_name_or_property_access(node)
        {
            let t = self.get_type_of_expression(&parent);
            return self.get_regular_type_of_literal_type(&t);
        }
        if let Some(symbol) = self.resolve_identifier(node) {
            let module_without_value = if symbol.flags.intersects(SymbolFlags::Alias) {
                let effective = self.resolve_alias_base(Arc::clone(&symbol));
                effective.flags.contains(SymbolFlags::NamespaceModule)
                    && !effective.flags.contains(SymbolFlags::ValueModule)
            } else {
                symbol.flags.contains(SymbolFlags::NamespaceModule)
                    && !symbol.flags.contains(SymbolFlags::ValueModule)
            };
            if module_without_value {
                return self.error_type();
            }
            let declared = if symbol.flags == SymbolFlags::Alias {
                match self.type_of_imported_symbol(&symbol) {
                    Some(t) => t,
                    None => self.get_type_of_symbol(&symbol),
                }
            } else {
                self.get_type_of_symbol(&symbol)
            };
            let narrowed =
                if self.type_resolution_stack.is_empty()
                    && symbol.flags.intersects(
                        SymbolFlags::FunctionScopedVariable
                            | SymbolFlags::BlockScopedVariable
                            | SymbolFlags::Alias,
                    )
                {
                    let flow = self.program.symbol_map().flow_node_of(node).map(Arc::clone);
                    let narrowable = self.get_narrowable_type_for_reference(&declared, node);
                    self.get_narrowed_type_of_symbol_with_declared(
                        &symbol,
                        flow.as_ref(),
                        narrowable,
                        Some(node),
                    )
                } else {
                    declared
                };

            if narrowed.object_flags.contains(ObjectFlags::EvolvingArray)
                && self.is_evolving_array_operation_target(node)
            {
                return self.auto_array_type();
            }

            let final_type = self.finalize_evolving_array_type(&narrowed);

            let target_kind = get_assignment_target_kind(node);
            if target_kind == AssignmentKind::None
                && let Some(declared) = self.definite_assignment_violation_type(node, &symbol)
            {
                return declared;
            }
            let compound_like =
                target_kind == AssignmentKind::Definite && is_in_compound_like_assignment(node);
            if compound_like || target_kind == AssignmentKind::Compound {
                return self.get_base_type_of_literal_type(&final_type);
            }
            final_type
        } else {
            self.error_type()
        }
    }

    pub(crate) fn is_evolving_array_operation_target(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_evolving_array_operation_target"); 
        let root = self.get_reference_root(node);
        let Some(parent) = root.parent() else {
            return false;
        };

        if let NodeData::PropertyAccessExpression(pa) = &parent.data {
            if Arc::ptr_eq(&pa.expression, &root) {
                let name = pa.name.text();
                if name == "length" {
                    return true;
                }
                if name == "push" || name == "unshift" {
                    if let Some(grandparent) = parent.parent() {
                        if matches!(grandparent.kind, SyntaxKind::CallExpression) {
                            return true;
                        }
                    }
                }
            }
        }

        if let NodeData::ElementAccessExpression(ea) = &parent.data {
            if Arc::ptr_eq(&ea.expression, &root) {
                if let Some(grandparent) = parent.parent() {
                    if let NodeData::BinaryExpression(bin) = &grandparent.data {
                        if bin.operator_token.kind == SyntaxKind::EqualsToken
                            && Arc::ptr_eq(&bin.left, &parent)
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub(crate) fn get_reference_root(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_reference_root"); 
        let Some(parent) = node.parent() else {
            return Arc::clone(node);
        };
        let recurse = match &parent.data {
            NodeData::ParenthesizedExpression(_) => true,
            NodeData::BinaryExpression(bin) => {
                (bin.operator_token.kind == SyntaxKind::EqualsToken && Arc::ptr_eq(&bin.left, node))
                    || (bin.operator_token.kind == SyntaxKind::CommaToken
                        && Arc::ptr_eq(&bin.right, node))
            }
            _ => false,
        };
        if recurse {
            self.get_reference_root(&parent)
        } else {
            Arc::clone(node)
        }
    }
}

impl Checker {
    // Go removeSubtypes 的受限版：structured/instantiable 成员若可赋给另一成员则移除
    pub(crate) fn remove_subtype_redundant_members(&mut self, types: Vec<Arc<Type>>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("remove_subtype_redundant_members"); 
        if types.len() < 2 {
            return types;
        }
        // Go removeSubtypes：从后往前遍历，位置靠前的成员在互相可赋时优先保留
        let mut keep: Vec<bool> = vec![true; types.len()];
        for i in (0..types.len()).rev() {
            if !keep[i] {
                continue;
            }
            let source = &types[i];
            // 非严格模式下 widening null/undefined 可赋给任何类型，参与缩减
            let nullable_source = !self.strict_null_checks
                && source.flags.intersects(TypeFlags::Null | TypeFlags::Undefined);
            if !nullable_source
                && !source.flags.intersects(
                    TypeFlags::Object
                        | TypeFlags::Union
                        | TypeFlags::Intersection
                        | TypeFlags::TypeParameter,
                )
            {
                continue;
            }
            for (j, target) in types.iter().enumerate() {
                if i == j || !keep[j] {
                    continue;
                }
                if Arc::ptr_eq(source, target) {
                    continue;
                }
                // Go requireOptionalProperties 排除元组/数组源：其必需成员
                // （数字索引名）不因空对象成员参与子型缩减，保住联合有序性
                if (self.is_array_type(source) || self.is_tuple_type(source))
                    && !self.is_array_type(target)
                    && !self.is_tuple_type(target)
                    && self.is_empty_object_type(target)
                {
                    continue;
                }
                // Go removeSubtypes：strictSubtypeRelation 判缩减（assignable 会
                // 把 {a} 对 {} 误判为可缩减，fresh {} 目标须挡住）
                if self.is_type_strict_subtype_of(source, target) {
                    keep[i] = false;
                    break;
                }
            }
        }
        let result: Vec<Arc<Type>> = types
            .into_iter()
            .zip(keep)
            .filter(|(_, k)| *k)
            .map(|(t, _)| t)
            .collect();
        result
    }
}
