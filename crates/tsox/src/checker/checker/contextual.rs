use std::sync::Arc;

use crate::ast::{
    ModifierFlags, Node, Symbol, SymbolFlags, SyntaxKind,
};
use crate::core::text::TextRange;
use crate::diagnostics::messages_generated::*;







use super::*;


impl Checker {
    pub(crate) fn check_contextual_elements(
        &mut self,
        expr: &Arc<Node>,
        target: &Arc<Type>,
        missing_anchor: TextRange,
    ) {
        if target.flags.contains(TypeFlags::Any) {
            return;
        }
        if expr.kind == SyntaxKind::ArrayLiteralExpression {
            let crate::ast::NodeData::ArrayLiteralExpression(data) = &expr.data else {
                return;
            };

            let elem_t = if self.is_array_type(target)
                || matches!(target.data, TypeData::EvolvingArray(_))
            {
                self.get_array_element_type(target)
            } else {
                self.get_any_type()
            };
            if elem_t.flags.contains(TypeFlags::Any) {

                let index_source = target.symbol.as_ref().and_then(|sym| {
                    let args = target.as_object()?.type_arguments.clone();
                    if sym.flags.contains(SymbolFlags::Interface) && !args.is_empty() {
                        Some(self.resolve_interface_type_ex(sym, Some(args)))
                    } else {
                        None
                    }
                });
                let index_source = index_source.unwrap_or_else(|| Arc::clone(target));
                let indexed = index_source.as_structured().and_then(|s| {
                    s.index_infos
                        .iter()
                        .find(|info| {
                            info.key_type
                                .as_ref()
                                .is_some_and(|k| k.flags.contains(TypeFlags::Number))
                        })
                        .and_then(|info| info.value_type.clone())
                });
                let Some(elem_t) = indexed else {
                    return;
                };
                if elem_t.flags.contains(TypeFlags::Any) {
                    return;
                }
                let mut inner = Vec::new();
                for el in data.elements.iter() {
                    if el.kind == SyntaxKind::SpreadElement {
                        continue;
                    }
                    inner.push(Arc::clone(el));
                }
                for el in inner {
                    let loc = el.loc;
                    self.check_contextual_elements(&el, &elem_t, loc);
                }
                return;
            }
            for el in data.elements.iter() {
                if el.kind == SyntaxKind::SpreadElement {
                    continue;
                }
                self.check_contextual_elements(el, &elem_t, el.loc);
            }
            return;
        }

        if matches!(
            expr.kind,
            SyntaxKind::TypeAssertionExpression | SyntaxKind::AsExpression
        ) {
            let target = Arc::clone(target);
            let anchor = expr.loc;
            let assertion_type = match &expr.data {
                crate::ast::NodeData::TypeAssertion(d) => {
                    self.get_type_from_type_node(&d.type_node)
                }
                crate::ast::NodeData::AsExpression(d) => {
                    self.get_type_from_type_node(&d.type_node)
                }
                _ => return,
            };
            let missing =
                self.get_missing_required_properties(&assertion_type, &target);
            let file = self.current_file.clone();
            let src_str = self.type_to_string(&assertion_type);
            let tgt_str = self.type_to_string(&target);
            if missing.len() == 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    anchor,
                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing[0].clone(), src_str, tgt_str],
                ));
            } else if missing.len() > 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    anchor,
                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![src_str, tgt_str, missing.join(", ")],
                ));
            }
            return;
        }
        let expr_type = self.get_type_of_node(expr);
        if expr.kind == SyntaxKind::ObjectLiteralExpression {
            if let Some(excess) = self.get_excess_property_name(&expr_type, target) {
                let loc = self
                    .find_object_literal_property_name_node(expr, &excess)
                    .unwrap_or(expr.loc);
                let tgt_str = self.type_to_string(target);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                    vec![excess, tgt_str],
                ));
                return;
            }
            let missing = self.get_missing_required_properties(&expr_type, target);
            let file = self.current_file.clone();
            let src_str = self.type_to_string(&expr_type);
            let tgt_str = self.type_to_string(target);
            if missing.len() == 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    missing_anchor,
                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing[0].clone(), src_str, tgt_str],
                ));
            } else if missing.len() > 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    missing_anchor,
                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![src_str, tgt_str, missing.join(", ")],
                ));
            }
            return;
        }

        if matches!(
            expr.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
        ) && !self.is_type_assignable_to(&expr_type, target)
        {
            let display_type = if crate::checker::is_literal_type(&expr_type) {
                self.get_base_type_of_literal_type(&expr_type)
            } else {
                expr_type.clone()
            };
            let src_str = self.type_to_string(&display_type);
            let tgt_str = self.type_to_string(target);

            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2322 && d.loc == expr.loc);
            if already {
                return;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                expr.loc,
                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                vec![src_str, tgt_str],
            ));
        }
    }

    pub(crate) fn unwrap_async_return_type(&self, declared: Arc<Type>, is_async: bool) -> Arc<Type> {
        if !is_async {
            return declared;
        }

        let is_promise = declared
            .symbol
            .as_ref()
            .is_some_and(|s| s.name == "Promise");
        if is_promise {
            if let crate::checker::TypeData::Object(obj) = &declared.data {
                if let Some(t) = obj.type_arguments.first() {
                    return Arc::clone(t);
                }
            }
            return self.get_any_type();
        }
        declared
    }

    pub fn get_awaited_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        self.get_awaited_type_with_depth(t, 0)
    }

    pub(crate) fn get_awaited_type_with_depth(
        &mut self,
        t: &Arc<Type>,
        depth: usize,
    ) -> Option<Arc<Type>> {

        if depth > 50 {
            return None;
        }

        if t.flags.contains(TypeFlags::Any) {
            return Some(Arc::clone(t));
        }

        if let crate::checker::TypeData::Union(u) = &t.data {
            let mut mapped: Vec<Arc<Type>> = Vec::with_capacity(u.union_or_intersection.types.len());
            for constituent in &u.union_or_intersection.types {
                let awaited = self
                    .get_awaited_type_with_depth(constituent, depth + 1)
                    .unwrap_or_else(|| Arc::clone(constituent));
                mapped.push(awaited);
            }
            return Some(self.get_union_type(mapped));
        }
        if let Some(promised) = self.get_promised_type_of_promise(t) {
            if Arc::ptr_eq(&promised, t) {

                return None;
            }
            return self.get_awaited_type_with_depth(&promised, depth + 1);
        }

        Some(Arc::clone(t))
    }

    pub(crate) fn get_promised_type_of_promise(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {

        if t.symbol.as_ref().is_some_and(|s| s.name == "Promise") {
            if let crate::checker::TypeData::Object(obj) = &t.data {
                if let Some(first) = obj.type_arguments.first() {
                    return Some(Arc::clone(first));
                }
            }
            return None;
        }

        if !t.flags.contains(TypeFlags::Object) {
            return None;
        }
        let then_fn = self.get_property_of_type(t, "then")?;
        let then_type = self.get_type_of_symbol(&then_fn);
        if then_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let then_signatures = self.get_signatures_of_type(&then_type, SignatureKind::Call);
        let then_sig = then_signatures.first()?;
        let onfulfilled = then_sig.parameters.first()?;
        let callback_type = self.get_type_of_symbol(onfulfilled);
        if callback_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let callback_signatures =
            self.get_signatures_of_type(&callback_type, SignatureKind::Call);
        let callback_sig = callback_signatures.first()?;
        let value_param = callback_sig.parameters.first()?;
        Some(self.get_type_of_symbol(value_param))
    }

    pub(crate) fn declared_annotation_type_of(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        if node.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(node)?;
        let decl = sym.value_declaration.clone()?;
        if decl.kind != SyntaxKind::VariableDeclaration {
            return None;
        }
        let crate::ast::NodeData::VariableDeclaration(vd) = &decl.data else {
            return None;
        };
        let tn = vd.type_node.as_ref()?;
        Some(self.get_type_from_type_node(tn))
    }

    pub(crate) fn is_valid_const_assertion_argument(&mut self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::TemplateExpression => true,
            SyntaxKind::ParenthesizedExpression => match &node.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => {
                    self.is_valid_const_assertion_argument(&p.expression)
                }
                _ => false,
            },
            SyntaxKind::PrefixUnaryExpression => match &node.data {
                crate::ast::NodeData::PrefixUnaryExpression(p) => {
                    let arg_kind = p.operand.kind;
                    (p.operator == SyntaxKind::MinusToken
                        && matches!(
                            arg_kind,
                            SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral
                        ))
                        || (p.operator == SyntaxKind::PlusToken
                            && arg_kind == SyntaxKind::NumericLiteral)
                }
                _ => false,
            },
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {

                let (obj, _name) = match &node.data {
                    crate::ast::NodeData::PropertyAccessExpression(d) => {
                        (Some(d.expression.clone()), d.name.text().to_string())
                    }
                    crate::ast::NodeData::ElementAccessExpression(d) => {
                        let arg = &d.argument_expression;
                        if arg.kind == SyntaxKind::StringLiteral {
                            (Some(d.expression.clone()), arg.text().to_string())
                        } else {
                            (None, String::new())
                        }
                    }
                    _ => (None, String::new()),
                };
                match obj {
                    Some(obj) if obj.kind == SyntaxKind::Identifier => {
                        self.resolve_qualified_symbol(node)
                            .or_else(|| self.resolve_identifier(&obj))
                            .map(|sym| {
                                sym.flags
                                    .intersects(SymbolFlags::ENUM | SymbolFlags::EnumMember)
                            })
                            .unwrap_or(false)
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub(crate) fn is_const_type_node(type_node: &Arc<Node>) -> bool {
        type_node.kind == SyntaxKind::ConstKeyword
    }

    pub(crate) fn check_delete_operand(&mut self, operand: &Arc<Node>) {
        let mut target = operand;
        while target.kind == SyntaxKind::ParenthesizedExpression {
            let inner = match &target.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => &p.expression,
                _ => break,
            };
            target = inner;
        }
        match target.kind {
            SyntaxKind::Identifier => {

                let strict =
                    self.program.options().get_strict_option_value(
                        self.program.options().always_strict,
                    ) || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.external_module_indicator.is_some());
                if strict {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        target.loc,
                        crate::diagnostics::messages_generated::
                            X_DELETE_CANNOT_BE_CALLED_ON_AN_IDENTIFIER_IN_STRICT_MODE,
                        vec![],
                    ));
                }
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    target.loc,
                    crate::diagnostics::messages_generated::
                        THE_OPERAND_OF_A_DELETE_OPERATOR_MUST_BE_A_PROPERTY_REFERENCE,
                    vec![],
                ));
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                let (obj_expr, name, _name_loc) = match &target.data {
                    crate::ast::NodeData::PropertyAccessExpression(d) => {
                        (&d.expression, d.name.text().to_string(), d.name.loc)
                    }
                    crate::ast::NodeData::ElementAccessExpression(d) => {
                        let arg = &d.argument_expression;
                        if arg.kind == SyntaxKind::StringLiteral {
                            (&d.expression, arg.text().to_string(), arg.loc)
                        } else {
                            return;
                        }
                    }
                    _ => return,
                };

                if matches!(&target.data, crate::ast::NodeData::PropertyAccessExpression(d) if d.name.kind == SyntaxKind::PrivateIdentifier)
                {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        target.loc,
                        crate::diagnostics::messages_generated::
                            THE_OPERAND_OF_A_DELETE_OPERATOR_CANNOT_BE_A_PRIVATE_IDENTIFIER,
                        vec![],
                    ));
                    return;
                }

                let obj_type = self.get_type_of_node(obj_expr);
                if obj_type.flags.contains(TypeFlags::Any) {
                    return;
                }
                if self.is_property_readonly(&obj_type, &name) {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        target.loc,
                        crate::diagnostics::messages_generated::
                            THE_OPERAND_OF_A_DELETE_OPERATOR_CANNOT_BE_A_READ_ONLY_PROPERTY,
                        vec![name],
                    ));
                    return;
                }

                if let Some(structured) = obj_type.as_structured() {
                    let readonly_index = structured.index_infos.iter().any(|info| {
                        info.is_readonly
                            && info
                                .key_type
                                .as_ref()
                                .is_some_and(|k| k.flags.contains(TypeFlags::String))
                    });
                    if readonly_index {
                        let type_name = self.type_to_string(&obj_type);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            target.loc,
                            crate::diagnostics::messages_generated::
                                INDEX_SIGNATURE_IN_TYPE_0_ONLY_PERMITS_READING,
                            vec![type_name],
                        ));
                        return;
                    }
                }

                if self.strict_null_checks && self.has_property_of_type(&obj_type, &name) {

                    let prop = obj_type.as_structured().and_then(|s| {
                        s.properties
                            .iter()
                            .find(|p| p.name == name)
                            .map(|p| Arc::clone(p))
                    });
                    let deletable = prop.as_ref().is_some_and(|p| {
                        if p.flags.contains(SymbolFlags::Optional) {
                            return true;
                        }
                        let t = self.get_type_of_symbol(p);
                        t.flags.intersects(
                            TypeFlags::Undefined
                                | TypeFlags::Any
                                | TypeFlags::Unknown
                                | TypeFlags::Never,
                        ) || match &t.data {
                            crate::checker::TypeData::Union(u) => u
                                .union_or_intersection
                                .types
                                .iter()
                                .any(|m| {
                                    m.flags.intersects(
                                        TypeFlags::Undefined
                                            | TypeFlags::Any
                                            | TypeFlags::Unknown
                                            | TypeFlags::Never,
                                    )
                                }),
                            _ => false,
                        }
                    }) || obj_type.as_structured().is_some_and(|s| {

                        s.index_infos.iter().any(|info| {
                            info.key_type
                                .as_ref()
                                .is_some_and(|k| k.flags.contains(TypeFlags::String))
                        })
                    });
                    if !deletable {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            target.loc,
                            crate::diagnostics::messages_generated::
                                THE_OPERAND_OF_A_DELETE_OPERATOR_MUST_BE_OPTIONAL,
                            vec![],
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn check_const_assignment_target(&mut self, operand: &Arc<Node>) {

        let mut target = operand;
        loop {
            target = match &target.data {

                crate::ast::NodeData::ParenthesizedExpression(p) => &p.expression,
                crate::ast::NodeData::NonNullExpression(n) => &n.expression,
                _ => break,
            };
        }
        let operand = target;
        if operand.kind == SyntaxKind::PropertyAccessExpression
            || operand.kind == SyntaxKind::ElementAccessExpression
        {
            self.check_const_property_assignment(operand);
            return;
        }
        if operand.kind != SyntaxKind::Identifier {
            return;
        }
        if let Some(symbol) = self.resolve_identifier(operand)
            && self.symbol_is_const_variable(&symbol)
        {
            let name_text = operand.text();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                operand.loc,
                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CONSTANT,
                vec![name_text.to_string()],
            ));
        }
    }

    pub(crate) fn check_const_property_assignment(&mut self, node: &Arc<Node>) {

        let (obj_expr, name, name_loc) = match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => {
                (&data.expression, &data.name, data.name.loc)
            }
            crate::ast::NodeData::ElementAccessExpression(data) => {
                let arg = &data.argument_expression;
                if arg.kind != SyntaxKind::StringLiteral {
                    return;
                }
                (&data.expression, arg, arg.loc)
            }
            _ => return,
        };
        if obj_expr.kind != SyntaxKind::Identifier {
            return;
        }
        let Some(sym) = self.resolve_identifier(obj_expr) else {
            return;
        };
        let base = self.resolve_alias_base(sym);
        if !base.flags.contains(SymbolFlags::ValueModule) {
            return;
        }
        let name_text = name.text();
        let member = base
            .exports
            .get(name_text)
            .or_else(|| base.members.get(name_text))
            .cloned()
            .or_else(|| {
                base.declarations
                    .iter()
                    .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                    .find_map(|d| {
                        self.program
                            .symbol_map()
                            .locals
                            .get(&d.id())
                            .and_then(|l| l.get(name_text).cloned())
                    })
            });
        if member.is_some_and(|m| self.symbol_is_const_variable(&m)) {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name_loc,
                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                vec![name_text.to_string()],
            ));
        }
    }

    pub(crate) fn check_block_scoped_variable_used_before_declaration(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
    ) {

        {
            let decl = symbol
                .value_declaration
                .as_ref()
                .or_else(|| symbol.declarations.first());
            if let Some(mut current) = decl {
                loop {
                    match current.kind {
                        SyntaxKind::VariableDeclaration => {
                            let is_var = current
                                .parent
                                .as_ref()
                                .is_some_and(|parent| {
                                    parent.kind == SyntaxKind::VariableDeclarationList
                                        && !parent.flags.intersects(
                                            crate::ast::NodeFlags::Let
                                                | crate::ast::NodeFlags::Const,
                                        )
                                });
                            if is_var {
                                return;
                            }
                            break;
                        }
                        SyntaxKind::BindingElement
                        | SyntaxKind::ObjectBindingPattern
                        | SyntaxKind::ArrayBindingPattern => match current.parent.as_ref() {
                            Some(parent) => current = parent,
                            None => break,
                        },
                        _ => break,
                    }
                }
            }
        }

        let mut enum_decl_count = 0;
        let is_const_enum = symbol
            .declarations
            .iter()
            .filter(|d| {
                if d.kind == SyntaxKind::EnumDeclaration {
                    enum_decl_count += 1;
                    true
                } else {
                    false
                }
            })
            .all(|d| {
                let Some(f) = self
                    .get_source_file_of_node(d)
                    .or_else(|| self.current_file.clone())
                else {
                    return false;
                };
                let text = &f.text;
                let start = d.loc.pos();

                let lo = start.saturating_sub(8);
                let window = &text[lo.min(text.len())..(start + 6).min(text.len())];
                window.contains("const")
            });
        if is_const_enum
            && enum_decl_count > 0
            && !self.compiler_options.isolated_modules.is_true()
        {
            return;
        }

        {

            let in_tp_default = {
                let mut cur = node.parent.as_ref();
                let mut hit = false;
                while let Some(a) = cur {
                    if a.kind == SyntaxKind::TypeParameter {
                        hit = true;
                        break;
                    }
                    if matches!(
                        a.kind,
                        SyntaxKind::FunctionDeclaration
                            | SyntaxKind::ClassDeclaration
                            | SyntaxKind::MethodDeclaration
                            | SyntaxKind::Block
                            | SyntaxKind::SourceFile
                    ) {
                        break;
                    }
                    cur = a.parent.as_ref();
                }
                hit
            };
            if in_tp_default {
                return;
            }
            let in_type_position = {
                let mut cur = node.parent.as_ref();
                let mut hit = false;
                while let Some(a) = cur {
                    if matches!(
                        a.kind,
                        SyntaxKind::TypeReference
                            | SyntaxKind::TypeParameter
                            | SyntaxKind::ArrayType
                            | SyntaxKind::UnionType
                            | SyntaxKind::IntersectionType
                            | SyntaxKind::ParenthesizedType
                            | SyntaxKind::TupleType
                            | SyntaxKind::TypeLiteral
                            | SyntaxKind::FunctionType
                            | SyntaxKind::ConstructorType
                            | SyntaxKind::QualifiedName
                            | SyntaxKind::HeritageClause
                    ) {
                        hit = true;
                        break;
                    }
                    if matches!(
                        a.kind,
                        SyntaxKind::FunctionDeclaration
                            | SyntaxKind::ClassDeclaration
                            | SyntaxKind::MethodDeclaration
                            | SyntaxKind::Block
                            | SyntaxKind::SourceFile
                    ) {
                        break;
                    }
                    cur = a.parent.as_ref();
                }
                hit
            };
            if in_type_position {
                return;
            }
        }
        if !symbol.flags.intersects(
            SymbolFlags::BlockScopedVariable
                | SymbolFlags::Class
                | SymbolFlags::ENUM,
        ) {
            return;
        }

        let declaration_for_scope = symbol.declarations.iter().find(|d| {
            matches!(
                d.kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::BindingElement
                    | SyntaxKind::EnumDeclaration
            )
        });
        if let Some(declaration_for_scope) = declaration_for_scope {

            let is_fn_like = |n: &Arc<Node>| {
                matches!(
                    n.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                )
            };
            let immediately_invoked = |n: &Arc<Node>| -> bool {
                let Some(p) = n.parent.as_ref() else {
                    return false;
                };
                match &p.data {
                    crate::ast::NodeData::CallExpression(_) => true,
                    crate::ast::NodeData::ParenthesizedExpression(_) => {

                        let mut cur = p.parent.as_ref();
                        while let Some(a) = cur {
                            if matches!(&a.data, crate::ast::NodeData::CallExpression(_)) {
                                return true;
                            }
                            if matches!(&a.data, crate::ast::NodeData::ParenthesizedExpression(_)) {
                                cur = a.parent.as_ref();
                                continue;
                            }
                            break;
                        }
                        false
                    }
                    _ => false,
                }
            };

            let mut dc = declaration_for_scope.parent.as_ref();
            let mut decl_container: Option<Arc<Node>> = None;
            while let Some(a) = dc {
                if is_fn_like(a) {
                    decl_container = Some(Arc::clone(a));
                    break;
                }
                dc = a.parent.as_ref();
            }
            let mut cur = node.parent.as_ref();
            let mut exempt = false;
            while let Some(a) = cur {
                if let Some(dcont) = &decl_container {
                    if Arc::ptr_eq(a, dcont) {
                        break;
                    }
                }
                if is_fn_like(a) {
                    if immediately_invoked(a) {
                        cur = a.parent.as_ref();
                        continue;
                    }
                    exempt = true;
                    break;
                }

                if a.kind == SyntaxKind::PropertyDeclaration {

                    let in_initializer = matches!(&a.data, crate::ast::NodeData::PropertyDeclaration(pd) if pd.initializer.as_ref().is_some_and(|init| init.loc.contains(node.loc.pos())));
                    let is_static_prop = a.has_syntactic_modifier(ModifierFlags::Static);
                    let is_decl_instance_prop = declaration_for_scope.kind
                        == SyntaxKind::PropertyDeclaration
                        && !declaration_for_scope.has_syntactic_modifier(ModifierFlags::Static);
                    if in_initializer && !is_static_prop && !is_decl_instance_prop {
                        exempt = true;
                        break;
                    }
                }
                cur = a.parent.as_ref();
            }
            if exempt {
                return;
            }
        }

        let declaration = symbol.declarations.iter().find(|d| {
            matches!(
                d.kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::BindingElement
                    | SyntaxKind::EnumDeclaration
            )
        });
        let Some(declaration) = declaration else {
            return;
        };

        if declaration.kind == SyntaxKind::VariableDeclaration
            && !is_let_or_const_declaration(declaration)
        {
            return;
        }

        if self
            .get_combined_modifier_flags(declaration)
            .contains(ModifierFlags::Ambient)
        {
            return;
        }

        let decl_name_pos = match &declaration.data {
            crate::ast::NodeData::VariableDeclaration(d) => d.name.pos(),
            crate::ast::NodeData::BindingElement(d) => d
                .name
                .as_ref()
                .map(|n| n.pos())
                .unwrap_or(declaration.pos()),
            _ => declaration.pos(),
        };
        if decl_name_pos <= node.pos() {

            let inside_own_initializer = {

                let mut cur = declaration.parent.as_ref();
                let mut found = false;
                while let Some(a) = cur {
                    if matches!(&a.data, crate::ast::NodeData::VariableDeclaration(vdd)
                        if vdd.initializer.as_ref().is_some_and(|init| init.loc.contains(node.loc.pos())))
                    {
                        found = true;
                        break;
                    }
                    if matches!(
                        a.kind,
                        SyntaxKind::BindingElement
                            | SyntaxKind::ArrayBindingPattern
                            | SyntaxKind::ObjectBindingPattern
                    ) {
                        cur = a.parent.as_ref();
                        continue;
                    }
                    break;
                }
                found
            };
            if !inside_own_initializer {
                return;
            }
        }

        let decl_file = self.get_source_file_of_node(declaration);
        let use_file = self.get_source_file_of_node(node);
        if let (Some(df), Some(uf)) = (&decl_file, &use_file) {
            if df.file_name != uf.file_name {
                return;
            }
        }
        let file = self.current_file.clone();

        let message = if symbol.flags.contains(SymbolFlags::Class) {
            crate::diagnostics::messages_generated::CLASS_0_USED_BEFORE_ITS_DECLARATION
        } else if symbol.flags.intersects(SymbolFlags::RegularEnum)
            || (symbol.flags.intersects(SymbolFlags::ConstEnum)
                && self.compiler_options.isolated_modules.is_true())
        {
            crate::diagnostics::messages_generated::ENUM_0_USED_BEFORE_ITS_DECLARATION
        } else {
            BLOCK_SCOPED_VARIABLE_0_USED_BEFORE_ITS_DECLARATION
        };
        let already = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == message.code && d.loc == node.loc);
        if !already {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                message,
                vec![name.to_string()],
            ));
        }
    }

    pub(crate) fn check_variable_used_before_assigned(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
    ) {

        if is_assignment_target(node) {
            return;
        }

        if !self.strict_null_checks {
            return;
        }

        let is_plain_var = symbol.flags.contains(SymbolFlags::FunctionScopedVariable)
            && symbol
                .value_declaration
                .as_ref()
                .is_some_and(|d| d.kind == SyntaxKind::VariableDeclaration);
        if !symbol.flags.contains(SymbolFlags::BlockScopedVariable) && !is_plain_var {
            return;
        }

        let declaration = symbol.value_declaration.as_ref().or_else(|| {
            symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::VariableDeclaration)
        });
        let Some(declaration) = declaration else {
            return;
        };

        let crate::ast::NodeData::VariableDeclaration(vd) = &declaration.data else {
            return;
        };

        if vd.type_node.is_none() && vd.initializer.is_none() {
            return;
        }

        if self
            .get_combined_modifier_flags(declaration)
            .contains(ModifierFlags::Ambient)
            || vd.exclamation_token.is_some()
        {
            return;
        }

        let declared_type = self.get_type_of_symbol(symbol);
        if declared_type.flags.contains(TypeFlags::Any)
            || type_contains_undefined(&declared_type)
        {
            return;
        }

        let flow_container_of = |n: &Arc<Node>| -> Option<Arc<Node>> {
            let mut current = Arc::clone(n);
            loop {
                if matches!(
                    current.kind,
                    SyntaxKind::SourceFile
                        | SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                        | SyntaxKind::ModuleDeclaration

                        | SyntaxKind::PropertyDeclaration
                        | SyntaxKind::PropertySignature
                ) {
                    return Some(current);
                }
                current = Arc::clone(current.parent.as_ref()?);
            }
        };
        let same_scope = match (flow_container_of(node), flow_container_of(declaration)) {
            (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
            _ => true,
        };
        if !same_scope {
            return;
        }

        if node
            .parent
            .as_ref()
            .is_some_and(|p| p.kind == SyntaxKind::NonNullExpression)
        {
            return;
        }

        if !self.strict_null_checks {
            return;
        }
        if let Some(flow_type) = self.get_definite_assignment_flow_type(symbol, node) {
            if type_contains_undefined(&flow_type) {
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    VARIABLE_0_IS_USED_BEFORE_BEING_ASSIGNED,
                    vec![name.to_string()],
                ));
            }
        }
    }

    pub(crate) fn push_ts2304_suppression(&mut self) {
        self.suppress_cannot_find_name_in_type_nodes += 1;
        if self.suppress_source_file.is_none() {
            self.suppress_source_file = self.current_file.as_ref().map(|f| f.node.id());
        }
    }

    pub(crate) fn pop_ts2304_suppression(&mut self) {
        self.suppress_cannot_find_name_in_type_nodes = self
            .suppress_cannot_find_name_in_type_nodes
            .saturating_sub(1);
        if self.suppress_cannot_find_name_in_type_nodes == 0 {
            self.suppress_source_file = None;
        }
    }

    pub(crate) fn ts2304_reporting_allowed_for(&self, node: &Arc<Node>) -> bool {
        if self.suppress_cannot_find_name_in_type_nodes == 0 {
            return true;
        }
        match (
            self.get_source_file_of_node(node),
            self.suppress_source_file,
        ) {
            (Some(f), Some(origin)) => {
                if f.node.id() == origin {

                    false
                } else {

                    !f.file_name.starts_with("bundled://")
                }
            }
            _ => false,
        }
    }

    pub(crate) fn push_scope(&mut self, node: &Arc<Node>) {
        self.scope_stack.push(node.id());
    }

    pub(crate) fn push_function_scope(&mut self, node: &Arc<Node>) {
        self.function_scope_count += 1;
        self.scope_stack.push(node.id());
    }

    pub(crate) fn pop_function_scope(&mut self) {
        self.function_scope_count -= 1;
        self.scope_stack.pop();
    }

    pub(crate) fn push_arrow_function_scope(&mut self, node: &Arc<Node>) {
        self.arrow_function_scope_count += 1;
        self.scope_stack.push(node.id());
    }

    pub(crate) fn pop_arrow_function_scope(&mut self) {
        self.arrow_function_scope_count -= 1;
        self.scope_stack.pop();
    }
}
