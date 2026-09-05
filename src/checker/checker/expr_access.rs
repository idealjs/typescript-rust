use std::sync::Arc;

use crate::ast::{
    CheckFlags, ModifierFlags, Node, NodeData, Symbol, SymbolFlags, SyntaxKind,
};







use super::*;


impl Checker {
    pub(crate) fn get_type_of_binary_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        use crate::ast::SyntaxKind::*;
        if let crate::ast::NodeData::BinaryExpression(data) = &node.data {
            match data.operator_token.kind {

                PlusToken => {
                    let lt = self.get_type_of_node(&data.left);
                    let rt = self.get_type_of_node(&data.right);
                    let string_like = |t: &Arc<Type>| {
                        t.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral)
                    };
                    if string_like(&lt) || string_like(&rt) {
                        self.string_type()
                    } else if lt.flags.contains(TypeFlags::Any)
                        || rt.flags.contains(TypeFlags::Any)
                    {
                        self.get_any_type()
                    } else {
                        self.number_type()
                    }
                }

                MinusToken
                | AsteriskToken
                | SlashToken
                | PercentToken
                | AsteriskAsteriskToken
                | LessThanLessThanToken
                | GreaterThanGreaterThanToken
                | GreaterThanGreaterThanGreaterThanToken
                | AmpersandToken
                | BarToken
                | CaretToken => self.number_type(),

                LessThanToken
                | GreaterThanToken
                | LessThanEqualsToken
                | GreaterThanEqualsToken
                | EqualsEqualsToken
                | ExclamationEqualsToken
                | EqualsEqualsEqualsToken
                | ExclamationEqualsEqualsToken
                | InKeyword
                | InstanceOfKeyword => self.boolean_type(),

                AmpersandAmpersandToken | BarBarToken | QuestionQuestionToken => {
                    self.get_type_of_node(&data.left)
                }

                CommaToken => self.get_type_of_node(&data.right),

                EqualsToken
                | PlusEqualsToken
                | MinusEqualsToken
                | AsteriskEqualsToken
                | SlashEqualsToken
                | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | AmpersandEqualsToken
                | BarEqualsToken
                | CaretEqualsToken
                | BarBarEqualsToken
                | AmpersandAmpersandEqualsToken
                | QuestionQuestionEqualsToken => self.get_type_of_node(&data.right),
                _ => self.get_any_type(),
            }
        } else {
            self.get_any_type()
        }
    }

    pub(crate) fn get_type_of_property_access(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (obj_expr, name) = match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => (&data.expression, &data.name),
            _ => return self.get_any_type(),
        };

        if obj_expr.kind == SyntaxKind::Identifier
            && let Some(sym) = self.resolve_identifier(obj_expr)
        {
            let base = self.resolve_alias_base(sym);
            if base.flags.contains(SymbolFlags::ValueModule) {
                let name_text = name.text();
                let member = base
                    .exports
                    .get(name_text)
                    .or_else(|| base.members.get(name_text))
                    .cloned()
                    .or_else(|| self.ambient_namespace_local(&base, name_text));

                if member.is_none() && !self.ambient_namespace_locals_visible(&base) {
                    return self.error_type();
                }
                if let Some(member) = member {
                    if let Some(t) = self
                        .value_symbol_links
                        .get(&member)
                        .and_then(|l| l.resolved_type.clone())
                    {
                        return t;
                    }
                    for decl in &member.declarations {
                        match decl.kind {
                            SyntaxKind::FunctionDeclaration => {
                                return self.get_type_of_function_like(decl);
                            }
                            SyntaxKind::ClassDeclaration => {
                                return self.get_type_of_class_declaration(decl);
                            }

                            SyntaxKind::ImportEqualsDeclaration => {
                                let t = self.type_of_imported_symbol(&member);
                                let resolved = match t {
                                    Some(t)
                                        if !(t.flags.contains(TypeFlags::Any)
                                            && t.intrinsic_name() == Some("any")) =>
                                    {
                                        Some(t)
                                    }
                                    _ => {
                                        let base =
                                            self.resolve_alias_base(Arc::clone(&member));
                                        base.declarations
                                            .iter()
                                            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                                            .map(|cd| self.get_type_of_class_declaration(cd))
                                    }
                                };
                                if let Some(t) = resolved {
                                    return t;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        let obj_type = self.get_type_of_node(obj_expr);

        if obj_type.intrinsic_name() == Some("error") {
            return self.error_type();
        }
        let name_text = name.text();

        if obj_type.is_union() {
            let parts: Vec<Arc<Type>> = self
                .constituent_types(&obj_type)
                .into_iter()
                .filter_map(|c| {
                    let sym = self.get_property_of_type(&c, &name_text)?;
                    if let Some(sub) = self.instantiate_array_member_type(&c, &sym) {
                        return Some(sub);
                    }
                    if c.as_object().is_some_and(|o| !o.type_arguments.is_empty()) {
                        return Some(self.substituted_member_type_of(&c, &sym));
                    }
                    Some(self.get_type_of_symbol(&sym))
                })
                .collect();
            if !parts.is_empty() {
                let t = if parts.len() == 1 {
                    parts.into_iter().next().expect("exactly one")
                } else {
                    self.get_union_type(parts)
                };
                return self.flow_type_of_access_expression(node, None, t);
            }
        }

        if (self.is_auto_array_type(&obj_type)
            || obj_type.object_flags.contains(ObjectFlags::EvolvingArray))
            && self.is_array_mutation_method(&name_text)
        {
            return self.get_any_type();
        }
        if let Some(sym) = self.get_property_of_type(&obj_type, &name_text) {

            if let Some(substituted) = self.instantiate_array_member_type(&obj_type, &sym) {
                return self.flow_type_of_access_expression(node, Some(&sym), substituted);
            }

            if obj_type
                .as_object()
                .is_some_and(|o| !o.type_arguments.is_empty())
            {
                let substituted = self.substituted_member_type_of(&obj_type, &sym);
                return self.flow_type_of_access_expression(node, Some(&sym), substituted);
            }
            let prop_type = self.get_type_of_symbol(&sym);
            return self.flow_type_of_access_expression(node, Some(&sym), prop_type);
        }

        if name_text == "length" && self.is_array_type(&obj_type) {
            return self.number_type();
        }
        self.get_any_type()
    }

    pub(crate) fn flow_type_of_access_expression(
        &mut self,
        node: &Arc<Node>,
        prop: Option<&Arc<Symbol>>,
        prop_type: Arc<Type>,
    ) -> Arc<Type> {
        if Self::is_definite_assignment_target(node) {
            return prop_type;
        }
        if let Some(prop) = prop {
            let eligible = prop
                .flags
                .intersects(SymbolFlags::VARIABLE | SymbolFlags::Property | SymbolFlags::ACCESSOR)
                || (prop.flags.contains(SymbolFlags::Method) && prop_type.is_union());
            if !eligible {
                return prop_type;
            }
        }
        self.get_flow_type_of_reference(node, &prop_type)
    }

    pub(crate) fn is_definite_assignment_target(node: &Arc<Node>) -> bool {
        let Some(parent) = &node.parent else {
            return false;
        };
        match &parent.data {
            NodeData::BinaryExpression(bin) => {
                Self::is_assignment_operator(bin.operator_token.kind)
                    && Arc::ptr_eq(&bin.left, node)
            }
            NodeData::PostfixUnaryExpression(unary) => {
                matches!(unary.operator, SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken)
                    && Arc::ptr_eq(&unary.operand, node)
            }
            NodeData::PrefixUnaryExpression(unary) => {
                matches!(unary.operator, SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken)
                    && Arc::ptr_eq(&unary.operand, node)
            }
            _ => false,
        }
    }

    pub(crate) fn is_assignment_operator(kind: crate::ast::SyntaxKind) -> bool {
        use crate::ast::SyntaxKind::*;
        matches!(
            kind,
            EqualsToken
                | PlusEqualsToken
                | MinusEqualsToken
                | AsteriskEqualsToken
                | SlashEqualsToken
                | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | AmpersandEqualsToken
                | BarEqualsToken
                | CaretEqualsToken
                | BarBarEqualsToken
                | AmpersandAmpersandEqualsToken
                | QuestionQuestionEqualsToken
        )
    }

    pub(crate) fn is_block_terminating_statement(stmt: &Arc<Node>) -> bool {
        matches!(
            stmt.kind,
            SyntaxKind::ReturnStatement
                | SyntaxKind::ThrowStatement
                | SyntaxKind::BreakStatement
                | SyntaxKind::ContinueStatement
        )
    }

    #[allow(dead_code)]
    pub(crate) fn property_type_includes_undefined(
        &mut self,
        data: &crate::ast::node_data_generated::PropertyDeclarationData,
    ) -> bool {
        let Some(tn) = &data.type_node else {
            return false;
        };
        let t = self.get_type_from_type_node(tn);
        if t.flags.contains(TypeFlags::Undefined) {
            return true;
        }
        if let Some(u) = t.as_union_or_intersection() {
            return u.types.iter().any(|m| m.flags.contains(TypeFlags::Undefined));
        }
        false
    }

    #[allow(dead_code)]
    pub(crate) fn class_constructor_assigns_property(&self, name: &str) -> bool {
        let Some(class) = self.enclosing_class_stack.last() else {
            return false;
        };
        let crate::ast::NodeData::ClassDeclaration(cd) = &class.data else {
            return false;
        };
        cd.members.iter().any(|member| {
            if member.kind != SyntaxKind::Constructor {
                return false;
            }
            let crate::ast::NodeData::ConstructorDeclaration(ctor) = &member.data else {
                return false;
            };
            ctor.body
                .as_ref()
                .is_some_and(|body| body_assigns_this_property(body, name))
        })
    }

    pub(crate) fn check_call_arg_with_context(
        &mut self,
        callee_expr: &Arc<Node>,
        arg_index: usize,
        arg: &Arc<Node>,
    ) {
        let is_function_arg =
            matches!(arg.kind, SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression);
        if is_function_arg {
            let ctx = self.contextual_param_count_for_arg(callee_expr, arg_index);
            if std::env::var_os("TSOX_DEBUG_SYMBOL").is_some() {
                eprintln!("[ctx-arg] pushed ctx={ctx}");
            }
            self.call_arg_arrow_context.push(ctx);
        }
        self.check_expression(arg);
        if is_function_arg {
            self.call_arg_arrow_context.pop();
        }
    }

    pub(crate) fn contextual_signature_of_arrow(&mut self, node: &Arc<Node>) -> Option<Arc<Signature>> {
        if std::env::var_os("TSOX_DEBUG_SYMBOL").is_some() {
            eprintln!(
                "[arrow-ctx] entered parent={:?}",
                node.parent.as_ref().map(|p| p.kind)
            );
        }
        let t = self.get_contextual_type(node, ContextFlags::None)?;
        if let TypeData::IndexedAccess(ia) = &t.data
            && let (Some(o), Some(i)) = (&ia.object_type, &ia.index_type)
            && o.flags.contains(TypeFlags::TypeParameter)
        {
            let resolved = self.get_indexed_access_type(o, i);
            if !matches!(resolved.intrinsic_name(), Some("any") | Some("error")) {
                return self.first_call_signature(&resolved);
            }
        }
        self.first_call_signature(&t)
    }

    pub(crate) fn first_call_signature(&mut self, t: &Arc<Type>) -> Option<Arc<Signature>> {
        if let TypeData::Union(u) = &t.data {
            for constituent in &u.union_or_intersection.types {
                if constituent
                    .flags
                    .intersects(TypeFlags::Undefined | TypeFlags::Null)
                {
                    continue;
                }
                if let Some(sig) = self.first_call_signature(constituent) {
                    return Some(sig);
                }
            }
            return None;
        }
        let structured = t.as_structured()?;
        structured.call_signatures().first().cloned()
    }

    pub(crate) fn contextual_param_count_for_arg(
        &mut self,
        callee_expr: &Arc<Node>,
        arg_index: usize,
    ) -> usize {
        let t = self.get_type_of_node(callee_expr);
        if std::env::var_os("TSOX_DEBUG_SYMBOL").is_some() {
            eprintln!(
                "[ctx-arg] callee={:?} intr={:?} union={} structured={}",
                callee_expr.kind,
                t.intrinsic_name(),
                matches!(&t.data, TypeData::Union(_)),
                t.as_structured()
                    .map(|s| s.call_signatures().len())
                    .unwrap_or(usize::MAX),
            );
        }
        if t.flags.contains(TypeFlags::Any) {

            if let crate::ast::NodeData::PropertyAccessExpression(data) = &callee_expr.data {
                let method = data.name.text().to_string();
                const ARRAY_CALLBACK_SIGS: &[(&str, usize)] = &[
                    ("map", 3),
                    ("filter", 3),
                    ("forEach", 3),
                    ("every", 3),
                    ("some", 3),
                    ("find", 3),
                    ("findIndex", 3),
                    ("findLast", 3),
                    ("findLastIndex", 3),
                    ("flatMap", 3),
                    ("reduce", 4),
                    ("reduceRight", 4),
                    ("sort", 2),
                ];
                if let Some((_, count)) = ARRAY_CALLBACK_SIGS.iter().find(|(m, _)| *m == method) {
                    let recv_type = self.get_type_of_node(&data.expression);
                    if self.is_array_type(&recv_type) {
                        return *count;
                    }
                }
            }
            return 0;
        }

        let t = if let TypeData::Union(u) = &t.data {
            match u.union_or_intersection.types.iter().find(|c| {
                !c.flags.intersects(TypeFlags::Undefined | TypeFlags::Null)
                    && c.as_structured()
                        .is_some_and(|s| !s.call_signatures().is_empty())
            }) {
                Some(c) => Arc::clone(c),
                None => return 0,
            }
        } else {
            t
        };
        let Some(structured) = t.as_structured() else {
            return 0;
        };

        let Some(sig) = structured
            .call_signatures()
            .iter()
            .find(|s| s.parameters.len() > arg_index)
            .or_else(|| structured.call_signatures().first())
        else {
            return 0;
        };
        let Some(param) = sig.parameters.get(arg_index) else {
            return 0;
        };
        let param_type = self.get_type_of_symbol(param);
        if param_type.flags.contains(TypeFlags::Any) {
            return 0;
        }
        let Some(param_structured) = param_type.as_structured() else {
            return 0;
        };
        param_structured
            .call_signatures()
            .first()
            .map_or(0, |callback_sig| callback_sig.parameters.len())
    }

    pub(crate) fn symbol_is_abstract_class(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if decl.kind == SyntaxKind::ClassDeclaration
                && decl.has_syntactic_modifier(ModifierFlags::Abstract)
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn type_includes_abstract_constructor(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Any) {
            return false;
        }
        if let Some(u) = t.as_union_or_intersection() {
            return u.types.iter().any(|m| self.type_includes_abstract_constructor(m));
        }

        if t.flags.contains(TypeFlags::Object) {
            if let Some(s) = t.as_structured()
                && s.construct_signatures().iter().any(|sig| {
                    sig.flags.contains(crate::checker::types::SignatureFlags::Abstract)
                })
            {
                return true;
            }
        }
        if let Some(symbol) = &t.symbol {
            return self.symbol_is_abstract_class(symbol);
        }
        false
    }

    pub(crate) fn declaring_class_of_member(&self, member_symbol: &Arc<Symbol>) -> Option<Arc<Node>> {
        self.declaring_class_of_private_member(member_symbol)
            .or_else(|| {
                for decl in &member_symbol.declarations {
                    if matches!(
                        decl.kind,
                        SyntaxKind::PropertyDeclaration | SyntaxKind::MethodDeclaration
                    ) {
                        if let Some(parent) = &decl.parent {
                            if parent.kind == SyntaxKind::ClassDeclaration {
                                return Some(Arc::clone(parent));
                            }
                        }
                    }
                }
                None
            })
    }

    pub(crate) fn declaring_class_of_private_member(
        &self,
        member_symbol: &Arc<Symbol>,
    ) -> Option<Arc<Node>> {
        for decl in &member_symbol.declarations {
            if matches!(
                decl.kind,
                SyntaxKind::PropertyDeclaration
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) {
                if let Some(parent) = &decl.parent {
                    if matches!(
                        parent.kind,
                        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                    ) {
                        return Some(Arc::clone(parent));
                    }
                }
            }
        }
        None
    }

    pub(crate) fn lookup_private_identifier_declaration(
        &self,
        text: &str,
        location: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        let mut current = Some(Arc::clone(location));
        while let Some(n) = current {
            if matches!(n.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
                if let Some(sym) = symbol_map.symbol_of(&n) {
                    if let Some(prop) = sym.members.get(text) {
                        return Some(Arc::clone(prop));
                    }
                    if let Some(prop) = sym.exports.get(text) {
                        return Some(Arc::clone(prop));
                    }
                }
            }
            current = n.parent.clone();
        }
        None
    }

    pub(crate) fn is_ancestor_class_of(&self, node: &Arc<Node>, ancestor: &Arc<Node>) -> bool {
        let mut current = Some(Arc::clone(node));
        while let Some(n) = current {
            if Arc::ptr_eq(&n, ancestor) {
                return true;
            }
            current = n.parent.clone();
        }
        false
    }

    pub(crate) fn check_private_identifier_access(
        &mut self,
        node: &Arc<Node>,
        name: &Arc<Node>,
        name_text: &str,
        obj_type: &Arc<Type>,
    ) -> bool {
        let assignment_kind = crate::checker::utilities::get_assignment_target_kind(node);
        let lexical = self.lookup_private_identifier_declaration(name_text, name);

        if assignment_kind != crate::checker::utilities::AssignmentKind::None
            && let Some(lx) = &lexical
            && lx.declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::MethodDeclaration)
        {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    CANNOT_ASSIGN_TO_PRIVATE_METHOD_0_PRIVATE_METHODS_ARE_NOT_WRITABLE,
                vec![name_text.to_string()],
            ));
        }

        let type_member: Option<Arc<Symbol>> = obj_type
            .as_structured()
            .and_then(|s| s.members.get(name_text))
            .map(Arc::clone);
        let resolved = match (&lexical, &type_member) {
            (Some(lx), Some(m)) => {
                let same_decl = lx.declarations.iter().any(|ld| {
                    m.declarations.iter().any(|d| d.id() == ld.id())
                });
                let same_class = lx
                    .declarations
                    .first()
                    .and_then(|ld| ld.parent.clone())
                    .zip(m.declarations.first().and_then(|d| d.parent.clone()))
                    .is_some_and(|(a, b)| a.id() == b.id());
                let synthetic_same_class = m.declarations.is_empty()
                    && lx.declarations
                        .first()
                        .and_then(|d| d.parent.clone())
                        .and_then(|class| self.program.symbol_map().symbol_of(&class))
                        .zip(obj_type.symbol.clone())
                        .is_some_and(|(a, b)| Arc::ptr_eq(&a, &b));
                (same_decl || same_class || synthetic_same_class).then(|| Arc::clone(m))
            }
            _ => None,
        };

        if resolved.is_none() {

            let property_on_type = type_member.as_ref().filter(|m| {
                m.declarations.iter().any(|d| {
                    d.name()
                        .is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier)
                })
            });
            if let Some(property) = property_on_type {
                let type_class = self.declaring_class_of_private_member(property);
                if let (Some(lx), Some(type_class)) = (&lexical, &type_class) {

                    let lexical_class = self.declaring_class_of_private_member(lx);
                    if lexical_class.is_some_and(|lc| self.is_ancestor_class_of(&lc, type_class))
                    {
                        let type_str = self.type_to_string(obj_type);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            name.loc,
                            crate::diagnostics::messages_generated::THE_PROPERTY_0_CANNOT_BE_ACCESSED_ON_TYPE_1_WITHIN_THIS_CLASS_BECAUSE_IT_IS_SHADOWED_BY_ANOTHER_PRIVATE_IDENTIFIER_WITH_THE_SAME_SPELLING,
                            vec![name_text.to_string(), type_str],
                        ));
                        return true;
                    }
                }
                let class_name = type_class.map_or_else(
                    || "(anonymous)".to_string(),
                    |c| match &c.data {
                        crate::ast::NodeData::ClassDeclaration(d) => d
                            .name
                            .as_ref()
                            .map(|n| n.text().to_string())
                            .unwrap_or_else(|| "(anonymous)".to_string()),
                        _ => "(anonymous)".to_string(),
                    },
                );
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    crate::diagnostics::messages_generated::PROPERTY_0_IS_NOT_ACCESSIBLE_OUTSIDE_CLASS_1_BECAUSE_IT_HAS_A_PRIVATE_IDENTIFIER,
                    vec![name_text.to_string(), class_name],
                ));
                return true;
            }
            return false;
        }

        let setonly = resolved.as_ref().is_some_and(|m| {
            m.flags.contains(SymbolFlags::SetAccessor) && !m.flags.contains(SymbolFlags::GetAccessor)
        });
        if setonly && assignment_kind != crate::checker::utilities::AssignmentKind::Definite {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                node.loc,
                crate::diagnostics::messages_generated::PRIVATE_ACCESSOR_WAS_DEFINED_WITHOUT_A_GETTER,
                vec![],
            ));
        }
        false
    }

    pub(crate) fn is_within_declaring_class(&self, class_node: &Arc<Node>) -> bool {
        self.enclosing_class_stack
            .iter()
            .any(|c| Arc::ptr_eq(c, class_node))
    }

    pub(crate) fn super_in_computed_name_of_innermost_class(&self, node: &Arc<Node>) -> bool {
        let Some(innermost) = self.enclosing_class_stack.last() else {
            return false;
        };
        let mut in_computed_name = false;
        let mut cur = node.parent.as_ref();
        while let Some(c) = cur {
            if Arc::ptr_eq(c, innermost) {
                return in_computed_name;
            }
            if c.kind == SyntaxKind::ComputedPropertyName {
                in_computed_name = true;
            }
            if matches!(c.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {

                return false;
            }
            cur = c.parent.as_ref();
        }
        false
    }

    pub(crate) fn function_body_definitely_returns(&self, body: &Arc<Node>) -> bool {
        if body.kind != SyntaxKind::Block {
            return false;
        }
        if let crate::ast::NodeData::Block(data) = &body.data {
            if let Some(last) = data.statements.nodes.last() {
                return self.statement_always_returns(last);
            }
        }
        false
    }

    pub(crate) fn statement_always_returns(&self, stmt: &Arc<Node>) -> bool {
        match stmt.kind {
            SyntaxKind::ReturnStatement | SyntaxKind::ThrowStatement => true,
            SyntaxKind::Block => {
                if let crate::ast::NodeData::Block(data) = &stmt.data {
                    if let Some(last) = data.statements.nodes.last() {
                        return self.statement_always_returns(last);
                    }
                }
                false
            }
            SyntaxKind::IfStatement => {
                if let crate::ast::NodeData::IfStatement(data) = &stmt.data {
                    let then_returns = self.statement_always_returns(&data.then_statement);
                    let else_returns = data
                        .else_statement
                        .as_ref()
                        .map_or(false, |e| self.statement_always_returns(e));
                    then_returns && else_returns
                } else {
                    false
                }
            }

            SyntaxKind::WhileStatement | SyntaxKind::DoStatement => {
                let (condition, body) = match &stmt.data {
                    crate::ast::NodeData::WhileStatement(data) => {
                        (&data.expression, &data.statement)
                    }
                    crate::ast::NodeData::DoStatement(data) => {
                        (&data.expression, &data.statement)
                    }
                    _ => return false,
                };
                condition.kind == SyntaxKind::TrueKeyword
                    && !Self::loop_has_escaping_break(body, true)
            }

            SyntaxKind::ForStatement => {
                if let crate::ast::NodeData::ForStatement(data) = &stmt.data {
                    data.condition
                        .as_ref()
                        .map_or(true, |c| c.kind == SyntaxKind::TrueKeyword)
                        && !Self::loop_has_escaping_break(&data.statement, true)
                } else {
                    false
                }
            }

            SyntaxKind::SwitchStatement => {
                if let crate::ast::NodeData::SwitchStatement(data) = &stmt.data
                    && let crate::ast::NodeData::CaseBlock(block) = &data.case_block.data
                {
                    let has_default = block.clauses.iter().any(|c| {
                        c.kind == SyntaxKind::DefaultClause
                    });
                    if !has_default {
                        return false;
                    }
                    block.clauses.iter().all(|c| {
                        match &c.data {
                            crate::ast::NodeData::CaseOrDefaultClause(cd) => cd
                                .statements
                                .nodes
                                .last()
                                .map_or(true, |l| self.statement_always_returns(l)),
                            _ => false,
                        }
                    })
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub(crate) fn is_property_readonly(&self, t: &Arc<Type>, name: &str) -> bool {
        let Some(structured) = t.as_structured() else {
            return false;
        };
        let Some(symbol) = structured.members.get(name) else {
            return false;
        };

        for decl in &symbol.declarations {
            let modifiers = match &decl.data {
                crate::ast::NodeData::PropertyDeclaration(d) => &d.modifiers,
                crate::ast::NodeData::PropertySignatureDeclaration(d) => &d.modifiers,
                crate::ast::NodeData::ParameterDeclaration(d) => &d.modifiers,
                _ => continue,
            };
            if let Some(m) = modifiers {
                if m.modifier_flags.contains(ModifierFlags::Readonly) {
                    return true;
                }
            }
        }

        if symbol.check_flags.contains(CheckFlags::Readonly) {
            return true;
        }
        false
    }
}
