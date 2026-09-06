use std::sync::Arc;

use crate::ast::{
    ModifierFlags, Node, NodeData, NodeList, Symbol, SyntaxKind,
};
use crate::core::text::TextRange;
use crate::jsnum;







use super::*;


impl Checker {
    pub(crate) fn check_assertion_overlap(&mut self, node: &Arc<Node>, expr: &Arc<Node>, type_node: &Arc<Node>) {

        if type_node.kind == SyntaxKind::TypeReference && type_node.text() == "const" {
            return;
        }
        let expr_type = self.get_type_of_node(expr);
        let target_type = self.get_type_from_type_node(type_node);
        let error_type = self.error_type();
        let exempt = |t: &Arc<Type>| {
            Arc::ptr_eq(t, &error_type)
                || t.flags.contains(TypeFlags::Any)
                || t.flags.contains(TypeFlags::Unknown)
                || t.flags.contains(TypeFlags::Never)
        };
        if exempt(&expr_type) || exempt(&target_type) {
            return;
        }
        let expr_base = if crate::checker::is_literal_type(&expr_type) {
            self.get_base_type_of_literal_type(&expr_type)
        } else {
            expr_type
        };

        let comparable = self.is_type_comparable_to(&expr_base, &target_type)
            || self.is_type_comparable_to(&target_type, &expr_base);
        if !comparable {
            let source_str = self.type_to_string(&expr_base);
            let target_str = self.type_to_string(&target_type);
            let file = self.current_file.clone();
            let mut diag = crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    CONVERSION_OF_TYPE_0_TO_TYPE_1_MAY_BE_A_MISTAKE_BECAUSE_NEITHER_TYPE_SUFFICIENTLY_OVERLAPS_WITH_THE_OTHER_IF_THIS_WAS_INTENTIONAL_CONVERT_THE_EXPRESSION_TO_UNKNOWN_FIRST,
                vec![source_str, target_str],
            );

            if let Some((prop_loc, prop_name, elem_target_str)) =
                self.assertion_excess_detail(&expr, &expr_base, &target_type)
            {
                diag.loc = prop_loc;
                diag.message_chain.push(crate::ast::Diagnostic::new(
                    None,
                    prop_loc,
                    crate::diagnostics::messages_generated::
                        OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                    vec![prop_name, elem_target_str],
                ));
            }
            self.diagnostics.add(diag);
        }
    }

    pub(crate) fn assertion_excess_detail(
        &mut self,
        expr: &Arc<Node>,
        expr_type: &Arc<Type>,
        target_type: &Arc<Type>,
    ) -> Option<(TextRange, String, String)> {

        let (elem_source, elem_target, literal_node) = match &expr.data {
            NodeData::ObjectLiteralExpression(_) => {
                (Arc::clone(expr_type), Arc::clone(target_type), Arc::clone(expr))
            }
            NodeData::ArrayLiteralExpression(d) => {
                let first_obj = d.elements.iter().find(|e| {
                    matches!(&e.data, NodeData::ObjectLiteralExpression(_))
                })?;
                let st = self.element_type_of(expr_type)?;
                let tt = self.element_type_of(target_type)?;
                (st, tt, Arc::clone(first_obj))
            }
            _ => return None,
        };
        let prop_name = self.get_excess_property_name(&elem_source, &elem_target)?;
        let prop_loc =
            self.find_object_literal_property_name_node(&literal_node, &prop_name)?;
        let elem_target_str = self.type_to_string(&elem_target);
        Some((prop_loc, prop_name, elem_target_str))
    }

    pub(crate) fn element_type_of(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.flags.contains(TypeFlags::Object) {
            if let TypeData::Object(obj) = &t.data
                && !obj.type_arguments.is_empty()
            {
                return Some(Arc::clone(&obj.type_arguments[0]));
            }
        }
        None
    }

    pub(crate) fn check_accessor_in_type_context(&mut self, member: &Arc<Node>) {
        let body = match &member.data {
            crate::ast::NodeData::GetAccessorDeclaration(d) => d.body.clone(),
            crate::ast::NodeData::SetAccessorDeclaration(d) => d.body.clone(),
            _ => return,
        };
        if let Some(body) = body {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                body.loc,
                crate::diagnostics::messages_generated::
                    AN_IMPLEMENTATION_CANNOT_BE_DECLARED_IN_AMBIENT_CONTEXTS,
                vec![],
            ));
        }
    }

    pub(crate) fn check_interface_members(&mut self, members: &NodeList) {

        {
            let mut seen: std::collections::HashMap<String, Vec<&Arc<Node>>> =
                std::collections::HashMap::new();
            for member in members.iter() {
                if let Some(name_node) = member.name() {
                    let name = match name_node.kind {
                        SyntaxKind::StringLiteral
                        | SyntaxKind::NumericLiteral
                        | SyntaxKind::Identifier
                        | SyntaxKind::PrivateIdentifier => name_node.text().to_string(),
                        _ => continue,
                    };
                    seen.entry(name).or_default().push(member);
                }
            }
            for (_, group) in seen.iter() {

                let all_methods = group
                    .iter()
                    .all(|m| m.kind == SyntaxKind::MethodSignature);
                let accessor_pair = group.iter().all(|m| {
                    matches!(
                        m.kind,
                        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                    )
                }) && group.iter().any(|m| m.kind == SyntaxKind::GetAccessor)
                    && group.iter().any(|m| m.kind == SyntaxKind::SetAccessor);
                if group.len() > 1 && !all_methods && !accessor_pair {
                    for m in group {
                        if let Some(name_node) = m.name() {
                            let name = name_node.text().to_string();
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_node.loc,
                                crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                                vec![name],
                            ));
                        }
                    }
                }
            }
        }
        for member in members.iter() {
            match member.kind {
                SyntaxKind::MethodSignature => {
                    let crate::ast::NodeData::MethodSignatureDeclaration(d) = &member.data
                    else {
                        continue;
                    };
                    self.check_parameter_property_modifiers(&d.parameters, false);
                    self.check_parameter_implicit_any(member, &d.parameters, 0);
                    for p in d.parameters.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                        }
                    }
                    if let Some(tn) = &d.type_node {
                        self.check_type_annotation(tn);
                    }

                    if self.no_implicit_any
                        && d.type_node.is_none()
                        && d.name.kind == SyntaxKind::Identifier
                    {
                        let file = self.current_file.clone();
                        let diagnostic = crate::ast::Diagnostic::new(
                            file,
                            d.name.loc,
                            crate::diagnostics::messages_generated::
                                X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                            vec![d.name.text().to_string(), "any".to_string()],
                        );
                        self.diagnostics.add(diagnostic);
                    }
                }
                SyntaxKind::ConstructSignature | SyntaxKind::CallSignature => {
                    let (params, type_node) = match &member.data {
                        crate::ast::NodeData::ConstructSignatureDeclaration(d) => {
                            (&d.parameters, d.type_node.as_ref())
                        }
                        crate::ast::NodeData::CallSignatureDeclaration(d) => {
                            (&d.parameters, d.type_node.as_ref())
                        }
                        _ => continue,
                    };
                    self.check_parameter_property_modifiers(params, false);
                    self.check_parameter_implicit_any(member, params, 0);
                    for p in params.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                        }
                    }
                    if let Some(tn) = type_node {
                        self.check_type_annotation(tn);
                    }

                    if self.no_implicit_any && type_node.is_none() {
                        let message = if member.kind == SyntaxKind::ConstructSignature {
                            crate::diagnostics::messages_generated::
                                CONSTRUCT_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE
                        } else {
                            crate::diagnostics::messages_generated::
                                CALL_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE
                        };
                        let file = self.current_file.clone();
                        let diagnostic =
                            crate::ast::Diagnostic::new(file, member.loc, message, vec![]);
                        self.diagnostics.add(diagnostic);
                    }
                }
                SyntaxKind::PropertySignature => {
                    if let crate::ast::NodeData::PropertySignatureDeclaration(d) = &member.data {
                        self.check_type_annotation(&d.type_node);
                    }
                }

                SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                    self.check_accessor_in_type_context(member);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn check_class_function_merge(&mut self, statements: &[Arc<Node>]) {
        let mut groups: std::collections::BTreeMap<String, Vec<Arc<Node>>> =
            std::collections::BTreeMap::new();
        for s in statements {
            match &s.data {
                crate::ast::NodeData::ClassDeclaration(d) => {
                    if let Some(n) = &d.name
                        && n.kind == SyntaxKind::Identifier
                    {
                        groups.entry(n.text().to_string()).or_default().push(Arc::clone(s));
                    }
                }
                crate::ast::NodeData::FunctionDeclaration(d) => {
                    if let Some(n) = &d.name
                        && n.kind == SyntaxKind::Identifier
                    {
                        groups.entry(n.text().to_string()).or_default().push(Arc::clone(s));
                    }
                }
                _ => {}
            }
        }
        for (name, decls) in groups {
            let has_non_ambient_class = decls.iter().any(|d| {
                d.kind == SyntaxKind::ClassDeclaration
                    && self.ambient_context_depth == 0
                    && !d.has_syntactic_modifier(ModifierFlags::Ambient)
            });
            let has_function = decls
                .iter()
                .any(|d| d.kind == SyntaxKind::FunctionDeclaration);
            if !(has_non_ambient_class && has_function) {
                continue;
            }
            for d in decls {
                let (name_node, message): (Option<&Arc<Node>>, _) = match &d.data {
                    crate::ast::NodeData::ClassDeclaration(cd) => (
                        cd.name.as_ref(),
                        crate::diagnostics::messages_generated::
                            CLASS_DECLARATION_CANNOT_IMPLEMENT_OVERLOAD_LIST_FOR_0,
                    ),
                    crate::ast::NodeData::FunctionDeclaration(fd) => (
                        fd.name.as_ref(),
                        crate::diagnostics::messages_generated::
                            FUNCTION_WITH_BODIES_CAN_ONLY_MERGE_WITH_CLASSES_THAT_ARE_AMBIENT,
                    ),
                    _ => continue,
                };
                let Some(name_node) = name_node else { continue };
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    name_node.loc,
                    message,
                    vec![name.clone()],
                ));
            }
        }
    }

    pub(crate) fn check_function_overloads_recursive(&mut self, statements: &[Arc<Node>]) {
        if self
            .current_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file)
        {
            return;
        }
        self.check_statement_function_overloads(statements);
        self.check_class_function_merge(statements);
        for s in statements {
            match &s.data {
                crate::ast::NodeData::Block(d) => {
                    self.check_function_overloads_recursive(&d.statements.nodes);
                }
                crate::ast::NodeData::ModuleDeclaration(d) => {

                    if d
                        .modifiers
                        .as_ref()
                        .is_some_and(|m| m.modifier_flags.intersects(ModifierFlags::Ambient))
                    {
                        continue;
                    }
                    if let Some(body) = &d.body
                        && matches!(body.kind, SyntaxKind::Block | SyntaxKind::ModuleBlock)
                        && let crate::ast::NodeData::Block(bd) = &body.data
                    {
                        self.check_function_overloads_recursive(&bd.statements.nodes);
                    }
                    if let Some(body) = &d.body
                        && body.kind == SyntaxKind::ModuleBlock
                        && let crate::ast::NodeData::ModuleBlock(bd) = &body.data
                    {
                        self.check_function_overloads_recursive(&bd.statements.nodes);
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn check_statement_function_overloads(&mut self, statements: &[Arc<Node>]) {

        let ambient_context = self.ambient_context_depth > 0
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        let statements: Vec<Arc<Node>> = statements
            .iter()
            .filter(|s| {
                !matches!(s.kind, SyntaxKind::FunctionDeclaration)
                    || !(ambient_context || s.has_syntactic_modifier(ModifierFlags::Ambient))
            })
            .cloned()
            .collect();
        let mut groups: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (idx, s) in statements.iter().enumerate() {
            if s.kind != SyntaxKind::FunctionDeclaration {
                continue;
            }
            if let crate::ast::NodeData::FunctionDeclaration(d) = &s.data
                && let Some(n) = &d.name
                && n.kind == SyntaxKind::Identifier
            {
                groups.entry(n.text().to_string()).or_default().push(idx);
            }
        }
        for (_, idxs) in groups {
            let mut prev: Option<usize> = None;
            let mut has_body = false;
            for &idx in &idxs {
                let body = matches!(
                    &statements[idx].data,
                    crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some()
                );
                if !body {
                    if let Some(p) = prev {
                        if p + 1 != idx {
                            self.report_function_impl_expected(&statements, p);
                        }
                    }
                } else {
                    has_body = true;
                }
                prev = Some(idx);
            }
            if !has_body {
                let last = idxs[idxs.len() - 1];
                if !statements[last].has_syntactic_modifier(ModifierFlags::Ambient) {
                    self.report_function_impl_expected(&statements, last);
                }
            } else {

                let fn_params = |f: &Arc<Node>| -> (usize, bool) {
                    if let crate::ast::NodeData::FunctionDeclaration(d) = &f.data {
                        let mut rest = false;
                        for p in d.parameters.iter() {
                            if p.kind == SyntaxKind::Parameter {
                                if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                                    if pd.dot_dot_dot_token.is_some() {
                                        rest = true;
                                        break;
                                    }

                                    let _ = pd.question_token.is_none();
                                }
                            }
                        }
                        (d.parameters.nodes.len(), rest)
                    } else {
                        (0, false)
                    }
                };
                let impl_idx = idxs
                    .iter()
                    .copied()
                    .find(|&i| {
                        matches!(
                            &statements[i].data,
                            crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some()
                        )
                    })
                    .unwrap_or_else(|| idxs[idxs.len() - 1]);
                let (_impl_total, impl_rest) = fn_params(&statements[impl_idx]);
                let impl_required = {

                    let mut n = 0;
                    if let crate::ast::NodeData::FunctionDeclaration(d) = &statements[impl_idx].data
                    {
                        for p in d.parameters.iter() {
                            if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                                && pd.dot_dot_dot_token.is_none()
                                && pd.question_token.is_none()
                            {
                                n += 1;
                            }
                        }
                    }
                    n
                };
                if !impl_rest {

                    let mut seen_shapes: Vec<String> = Vec::new();
                    for &i in &idxs {
                        if i == impl_idx {
                            continue;
                        }
                        let (overload_count, _) = fn_params(&statements[i]);
                        let shape = if let crate::ast::NodeData::FunctionDeclaration(d) =
                            &statements[i].data
                        {
                            let mut parts = Vec::new();
                            for p in d.parameters.iter() {
                                if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                                    let t = pd
                                        .type_node
                                        .as_ref()
                                        .map(|tn| tn.text())
                                        .unwrap_or_default();
                                    parts.push(format!(
                                        "{t}{}",
                                        if pd.question_token.is_some() { "?" } else { "" }
                                    ));
                                }
                            }
                            let ret = d
                                .type_node
                                .as_ref()
                                .map(|tn| tn.text())
                                .unwrap_or_default();
                            format!("({})=>{}", parts.join(","), ret)
                        } else {
                            String::new()
                        };
                        if seen_shapes.contains(&shape) {
                            continue;
                        }
                        seen_shapes.push(shape);
                        let arity_bad = !impl_rest && overload_count < impl_required;
                        let overload_node = Arc::clone(&statements[i]);
                        let impl_node = Arc::clone(&statements[impl_idx]);
                        let compat = self
                            .overload_signature_compatible_with_implementation(
                                &overload_node, &impl_node,
                            );
                        if (arity_bad || !compat)
                            && let crate::ast::NodeData::FunctionDeclaration(d) = &statements[i].data
                            && let Some(n) = &d.name
                        {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                n.loc,
                                crate::diagnostics::messages_generated::
                                    THIS_OVERLOAD_SIGNATURE_IS_NOT_COMPATIBLE_WITH_ITS_IMPLEMENTATION_SIGNATURE,
                                Vec::new(),
                            ));
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn report_function_impl_expected(&mut self, statements: &[Arc<Node>], idx: usize) {
        let node = Arc::clone(&statements[idx]);
        let (name_text, name_loc) = match &node.data {
            crate::ast::NodeData::FunctionDeclaration(d) => match &d.name {
                Some(n) => (n.text().to_string(), n.loc),
                None => return,
            },
            _ => return,
        };
        if let Some(sib) = statements.get(idx + 1) {
            if sib.kind == SyntaxKind::FunctionDeclaration {
                let sib_name = match &sib.data {
                    crate::ast::NodeData::FunctionDeclaration(d) => match &d.name {
                        Some(n) => (n.text().to_string(), n.loc, d.body.is_some()),
                        None => (String::new(), sib.loc, false),
                    },
                    _ => (String::new(), sib.loc, false),
                };
                if sib_name.0 == name_text {
                    return;
                }
                if sib_name.2 {
                    let file = self.current_file.clone();
                    let diagnostic = crate::ast::Diagnostic::new(
                        file,
                        sib_name.1,
                        crate::diagnostics::messages_generated::
                            FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![name_text],
                    );
                    self.diagnostics.add(diagnostic);
                    return;
                }
            }
        }
        let file = self.current_file.clone();
        let diagnostic = crate::ast::Diagnostic::new(
            file,
            name_loc,
            crate::diagnostics::messages_generated::
                FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
            Vec::new(),
        );
        self.diagnostics.add(diagnostic);
    }

    pub(crate) fn check_export_assignment_conflicts(&mut self, statements: &[Arc<Node>]) {
        let export_equals = statements.iter().find(|s| {
            matches!(
                &s.data,
                crate::ast::NodeData::ExportAssignment(d) if d.is_export_equals
            )
        });
        let Some(eq_decl) = export_equals else { return };
        let has_other_value_export = statements.iter().any(|s| {
            if Arc::ptr_eq(s, eq_decl) {
                return false;
            }
            let value_declaring = matches!(
                s.kind,
                SyntaxKind::ClassDeclaration
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::VariableStatement
                    | SyntaxKind::ModuleDeclaration
            );
            value_declaring && s.has_syntactic_modifier(ModifierFlags::Export)
        });
        if has_other_value_export {
            let file = self.current_file.clone();
            let diagnostic = crate::ast::Diagnostic::new(
                file,
                eq_decl.loc,
                crate::diagnostics::messages_generated::
                    AN_EXPORT_ASSIGNMENT_CANNOT_BE_USED_IN_A_MODULE_WITH_OTHER_EXPORTED_ELEMENTS,
                Vec::new(),
            );
            self.diagnostics.add(diagnostic);
        }
    }

    pub(crate) fn check_reserved_type_name(&mut self, name: &Arc<Node>, message: &'static crate::diagnostics::Message) {
        const RESERVED: &[&str] = &[
            "any", "unknown", "never", "number", "bigint", "boolean", "string", "symbol",
            "void", "object", "undefined",
        ];
        let text = name.text();
        if RESERVED.contains(&text) {
            let file = self.current_file.clone();
            let diagnostic = crate::ast::Diagnostic::new(
                file,
                name.loc,
                *message,
                vec![text.to_string()],
            );
            self.diagnostics.add(diagnostic);
        }
    }

    pub(crate) fn is_type_assignable_to_kind_snf(&mut self, source: &Arc<Type>, kind: TypeFlags) -> bool {
        if source.flags.intersects(kind) {
            return true;
        }
        let number = self.number_type();
        if kind.intersects(crate::checker::types::TYPE_FLAGS_NUMBER_LIKE)
            && self.is_type_assignable_to(source, &number)
        {
            return true;
        }
        let string = self.string_type();
        if kind.intersects(crate::checker::types::TYPE_FLAGS_STRING_LIKE)
            && self.is_type_assignable_to(source, &string)
        {
            return true;
        }
        let symbol = self.es_symbol_type();
        if kind.intersects(TypeFlags::ESSymbol) && self.is_type_assignable_to(source, &symbol) {
            return true;
        }
        false
    }

    pub(crate) fn check_computed_property_name(&mut self, name: &Arc<Node>) {
        if name.kind != SyntaxKind::ComputedPropertyName {
            return;
        }
        if !self.computed_property_name_checked.insert(Arc::as_ptr(name)) {
            return;
        }
        let expr = match &name.data {
            crate::ast::NodeData::ComputedPropertyName(data) => Arc::clone(&data.expression),
            _ => return,
        };

        let invalid_in_form = matches!(&expr.data, crate::ast::NodeData::BinaryExpression(b)
            if b.operator_token.kind == SyntaxKind::InKeyword)
            && name.parent.as_ref().is_some_and(|member| {
                !matches!(
                    member.kind,
                    SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                ) && member
                    .parent
                    .as_ref()
                    .is_some_and(|container| {
                        matches!(
                            container.kind,
                            SyntaxKind::TypeLiteral
                                | SyntaxKind::ClassDeclaration
                                | SyntaxKind::ClassExpression
                                | SyntaxKind::InterfaceDeclaration
                        )
                    })
            });
        if invalid_in_form {
            return;
        }

        self.check_expression(&expr);
        let t = self.get_type_of_node(&expr);

        let kind = crate::checker::types::TYPE_FLAGS_STRING_LIKE
            | crate::checker::types::TYPE_FLAGS_NUMBER_LIKE
            | crate::checker::types::TYPE_FLAGS_ES_SYMBOL_LIKE;
        let bad = t.flags.intersects(crate::checker::types::TYPE_FLAGS_NULLABLE)
            || (!self.is_type_assignable_to_kind_snf(&t, kind) && {
                let target = self.get_union_type(vec![
                    self.string_type(),
                    self.number_type(),
                    self.es_symbol_type(),
                ]);
                !self.is_type_assignable_to(&t, &target)
            });
        if bad {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name.loc,
                crate::diagnostics::messages_generated::
                    A_COMPUTED_PROPERTY_NAME_MUST_BE_OF_TYPE_STRING_NUMBER_SYMBOL_OR_ANY,
                vec![],
            ));
        }
    }

    pub(crate) fn member_name_node(node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            crate::ast::NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::MethodSignatureDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::PropertyDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::PropertySignatureDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::PropertyAssignment(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::ShorthandPropertyAssignment(d) => Some(Arc::clone(&d.name)),
            _ => None,
        }
    }

    pub(crate) fn property_name_key_type(&mut self, name: &Arc<Node>) -> Option<Arc<Type>> {
        match &name.data {
            crate::ast::NodeData::ComputedPropertyName(data) => {
                let expr = &data.expression;
                match &expr.data {
                    crate::ast::NodeData::StringLiteral(s) => {
                        Some(self.get_string_literal_type(&s.text))
                    }
                    crate::ast::NodeData::NumericLiteral(n) => {
                        Some(self.get_number_literal_type(jsnum::Number::from_string(&n.text)))
                    }
                    _ => {

                        Some(self.get_type_of_node(expr))
                    }
                }
            }
            crate::ast::NodeData::Identifier(data) => {
                if let Ok(_) = data.text.parse::<f64>() {
                    Some(self.get_number_literal_type(jsnum::Number::from_string(&data.text)))
                } else {
                    Some(self.get_string_literal_type(&data.text))
                }
            }
            crate::ast::NodeData::StringLiteral(data) => {
                Some(self.get_string_literal_type(&data.text))
            }
            crate::ast::NodeData::NumericLiteral(data) => Some(
                self.get_number_literal_type(jsnum::Number::from_string(&data.text)),
            ),
            _ => None,
        }
    }

    pub(crate) fn property_name_display(&self, name: &Arc<Node>) -> String {
        if name.kind == SyntaxKind::ComputedPropertyName {
            if let Some(text) = self.node_source_text(name) {
                let inner = text
                    .strip_prefix('[')
                    .and_then(|t| t.strip_suffix(']'))
                    .unwrap_or(&text);
                return format!("[{inner}]");
            }
        }
        name.text().to_string()
    }

    pub(crate) fn node_source_text(&self, node: &Arc<Node>) -> Option<String> {
        let mut root: &Arc<Node> = node;
        while let Some(p) = root.parent.as_ref() {
            root = p;
        }
        for f in &self.files {
            if Arc::ptr_eq(&f.node, root) {
                return f
                    .text
                    .get(node.loc.pos()..node.loc.end())
                    .map(|s| s.to_string());
            }
        }
        None
    }

    pub(crate) fn member_declared_type_for_index_check(&mut self, member: &Arc<Node>) -> Option<Arc<Type>> {
        match &member.data {
            crate::ast::NodeData::GetAccessorDeclaration(d) => Some(
                self.infer_function_return_type(d.body.as_ref(), d.type_node.as_ref()),
            ),
            crate::ast::NodeData::SetAccessorDeclaration(d) => {
                let tn = d.parameters.iter().next().and_then(|p| {
                    match &p.data {
                        crate::ast::NodeData::ParameterDeclaration(pd) => pd.type_node.clone(),
                        _ => None,
                    }
                });
                match tn {
                    Some(t) => Some(self.get_type_from_type_node(&t)),
                    None => Some(self.any_type()),
                }
            }
            crate::ast::NodeData::PropertyDeclaration(d) => {
                if let Some(t) = &d.type_node {
                    Some(self.get_type_from_type_node(t))
                } else if let Some(init) = &d.initializer {
                    let init_t = self.get_type_of_node(init);
                    Some(self.widen_initializer_type(&init_t))
                } else {
                    None
                }
            }
            crate::ast::NodeData::PropertySignatureDeclaration(d) => {
                Some(self.get_type_from_type_node(&d.type_node))
            }
            _ => None,
        }
    }

    pub(crate) fn check_index_constraints(&mut self, t: &Arc<Type>, declaration: &Arc<Node>) {
        let index_infos = self.get_index_infos_of_type(t);
        if index_infos.is_empty() {
            return;
        }

        let local_index: Option<Arc<crate::checker::IndexInfo>> = index_infos
            .iter()
            .find(|info| {
                info.declaration
                    .as_ref()
                    .and_then(|d| d.parent.as_ref())
                    .is_some_and(|p| Arc::ptr_eq(p, declaration))
            })
            .cloned();
        let is_interface = declaration.kind == SyntaxKind::InterfaceDeclaration;

        for prop in self.get_properties_of_type(t) {
            let Some(first_decl) = prop.declarations.first().cloned() else {
                continue;
            };
            if first_decl
                .parent
                .as_ref()
                .is_some_and(|p| Arc::ptr_eq(p, declaration))
            {
                continue;
            }
            let Some(name) = Self::member_name_node(&first_decl) else {
                continue;
            };
            if name.kind == SyntaxKind::ComputedPropertyName {
                continue;
            }
            let Some(key_type) = self.property_name_key_type(&name) else {
                continue;
            };
            let prop_type = self.get_type_of_symbol(&prop);
            let display = self.property_name_display(&name);
            self.check_index_constraint_for_property(
                t,
                &key_type,
                &prop_type,
                &name,
                &display,
                None,
                local_index.clone(),
                is_interface.then(|| Arc::clone(declaration)),
                &index_infos,
            );
        }

        let props_by_name: std::collections::HashMap<String, Arc<Symbol>> = self
            .get_properties_of_type(t)
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();
        let members: Vec<Arc<Node>> = match &declaration.data {
            crate::ast::NodeData::ClassDeclaration(d) => {
                d.members.iter().cloned().collect()
            }
            crate::ast::NodeData::InterfaceDeclaration(d) => {
                d.members.iter().cloned().collect()
            }
            _ => Vec::new(),
        };
        for member in &members {
            if member.kind == SyntaxKind::IndexSignature {
                continue;
            }
            let Some(name) = Self::member_name_node(member) else {
                continue;
            };
            let member_symbol = self.program.symbol_map().symbol_of(member).cloned();
            let Some(key_type) = self.property_name_key_type(&name) else {
                continue;
            };
            let prop_type = if name.kind != SyntaxKind::ComputedPropertyName {

                match props_by_name.get(name.text()) {
                    Some(sym) => self.get_type_of_symbol(sym),
                    None => match self.member_declared_type_for_index_check(member) {
                        Some(t) => t,
                        None => continue,
                    },
                }
            } else {

                match self.member_declared_type_for_index_check(member) {
                    Some(t) => t,
                    None => match &member_symbol {
                        Some(sym) => self.get_type_of_symbol(sym),
                        None => continue,
                    },
                }
            };
            let display = self.property_name_display(&name);
            let local_name_node = Some(Arc::clone(&name));
            self.check_index_constraint_for_property(
                t,
                &key_type,
                &prop_type,
                &name,
                &display,
                local_name_node,
                local_index.clone(),
                is_interface.then(|| Arc::clone(declaration)),
                &index_infos,
            );
        }

        let mut bases: Vec<Arc<Node>> = Vec::new();
        let mut worklist: Vec<Arc<Node>> = vec![Arc::clone(declaration)];
        let mut guard = 0;
        while let Some(d) = worklist.pop() {
            guard += 1;
            if guard > 32 {
                break;
            }
            let heritage = match &d.data {
                crate::ast::NodeData::ClassDeclaration(cd) => {
                    cd.heritage_clauses.clone()
                }
                crate::ast::NodeData::InterfaceDeclaration(id) => id.heritage_clauses.clone(),
                _ => continue,
            };
            let Some(clauses) = heritage else { continue };
            for clause in clauses.iter() {
                let crate::ast::NodeData::HeritageClause(hc) = &clause.data else {
                    continue;
                };
                for type_ref in hc.types.iter() {
                    let base_expr = match &type_ref.data {
                        crate::ast::NodeData::ExpressionWithTypeArguments(e) => {
                            Arc::clone(&e.expression)
                        }
                        _ => continue,
                    };
                    let base_symbol = if base_expr.kind == SyntaxKind::Identifier {
                        self.resolve_identifier(&base_expr)
                    } else {
                        None
                    };
                    let Some(base_symbol) = base_symbol else {
                        continue;
                    };
                    for bd in &base_symbol.declarations {
                        if matches!(
                            bd.kind,
                            SyntaxKind::ClassDeclaration | SyntaxKind::InterfaceDeclaration
                        ) && !bases.iter().any(|b| Arc::ptr_eq(b, bd))
                            && !Arc::ptr_eq(bd, &d)
                        {
                            bases.push(Arc::clone(bd));
                            worklist.push(Arc::clone(bd));
                        }
                    }
                }
            }
        }
        for base in &bases {
            let base_members: Vec<Arc<Node>> = match &base.data {
                crate::ast::NodeData::ClassDeclaration(d) => {
                    d.members.iter().cloned().collect()
                }
                crate::ast::NodeData::InterfaceDeclaration(d) => {
                    d.members.iter().cloned().collect()
                }
                _ => continue,
            };
            for member in base_members {
                let Some(name) = Self::member_name_node(&member) else {
                    continue;
                };
                if name.kind != SyntaxKind::ComputedPropertyName {
                    continue;
                }
                let Some(key_type) = self.property_name_key_type(&name) else {
                    continue;
                };
                let Some(symbol) = self.program.symbol_map().symbol_of(&member).cloned()
                else {
                    continue;
                };
                let prop_type = self
                    .member_declared_type_for_index_check(&member)
                    .unwrap_or_else(|| self.get_type_of_symbol(&symbol));
                let display = self.property_name_display(&name);
                let index_for_error = local_index.clone();
                let iface_decl = is_interface.then(|| Arc::clone(declaration));
                self.check_index_constraint_for_property(
                    t,
                    &key_type,
                    &prop_type,
                    &name,
                    &display,
                    None,
                    index_for_error,
                    iface_decl,
                    &index_infos,
                );
            }
        }
    }

    pub(crate) fn check_index_constraint_for_property(
        &mut self,
        _t: &Arc<Type>,
        key_type: &Arc<Type>,
        prop_type: &Arc<Type>,
        name: &Arc<Node>,
        display: &str,
        local_name: Option<Arc<Node>>,
        local_index: Option<Arc<crate::checker::IndexInfo>>,
        interface_decl: Option<Arc<Node>>,
        index_infos: &[Arc<crate::checker::IndexInfo>],
    ) {
        for info in index_infos {
            let Some(info_key) = info.key_type.clone() else {
                continue;
            };
            if !self.is_applicable_index_type(key_type, &info_key) {
                continue;
            }
            let info_value = match info.value_type.clone() {
                Some(v) => v,
                None => continue,
            };
            if self.is_type_assignable_to(prop_type, &info_value) {
                continue;
            }

            let (error_loc, related_index_decl) = if let Some(n) = &local_name {
                (n.loc, None)
            } else if let Some(idx) = &local_index {
                (
                    idx.declaration
                        .as_ref()
                        .map(|d| d.loc)
                        .unwrap_or(name.loc),
                    idx.declaration.clone(),
                )
            } else if let Some(idecl) = &interface_decl {
                (idecl.loc, None)
            } else {
                continue;
            };
            let file = self.current_file.clone();
            let mut diagnostic = crate::ast::Diagnostic::new(
                file,
                error_loc,
                crate::diagnostics::messages_generated::
                    PROPERTY_0_OF_TYPE_1_IS_NOT_ASSIGNABLE_TO_2_INDEX_TYPE_3,
                vec![
                    display.to_string(),
                    self.type_to_string(prop_type),
                    self.type_to_string(&info_key),
                    self.type_to_string(&info_value),
                ],
            );
            if let Some(idx_decl) = related_index_decl {
                diagnostic.related_information = vec![crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    idx_decl.loc,
                    crate::diagnostics::messages_generated::X_0_IS_DECLARED_HERE,
                    vec![display.to_string()],
                )];
            }
            self.diagnostics.add(diagnostic);
        }
    }



    pub(crate) fn explicit_type_argument_count(node: &Arc<Node>) -> usize {
        match &node.data {
            crate::ast::NodeData::CallExpression(d) => d
                .type_arguments
                .as_ref()
                .map(|t| t.len())
                .unwrap_or(0),
            crate::ast::NodeData::NewExpression(d) => d
                .type_arguments
                .as_ref()
                .map(|t| t.len())
                .unwrap_or(0),
            _ => 0,
        }
    }

    pub(crate) fn has_explicit_type_arguments(node: &Arc<Node>) -> bool {
        Self::explicit_type_argument_count(node) > 0
    }
}
