use std::sync::Arc;

use crate::ast::{
    ModifierFlags, Node, SymbolFlags, SyntaxKind,
};

use crate::checker::utilities::is_in_compound_like_assignment;
use crate::checker::utilities::{get_assignment_target_kind, AssignmentKind};






use super::*;


impl Checker {
    pub(crate) fn check_assignment_compat(
        &mut self,
        node: &Arc<Node>,
        data: &crate::ast::node_data_generated::BinaryExpressionData,
    ) {
        use crate::ast::SyntaxKind::*;

        if data.operator_token.kind == EqualsToken
            && matches!(
                data.left.kind,
                ObjectLiteralExpression | ArrayLiteralExpression
            )
        {
            return;
        }

        let mut target: &Arc<Node> = &data.left;
        loop {
            match &target.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => {
                    target = &p.expression;
                }
                crate::ast::NodeData::NonNullExpression(n) => {
                    target = &n.expression;
                }
                _ => break,
            }
        }

        let optional_chain = match &target.data {
            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                pa.question_dot_token.is_some()
            }
            crate::ast::NodeData::ElementAccessExpression(ea) => {
                ea.question_dot_token.is_some()
            }
            _ => false,
        };
        let is_reference = matches!(
            target.kind,
            Identifier | PropertyAccessExpression | ElementAccessExpression
        );
        if !is_reference || optional_chain {
            let message = if optional_chain {
                crate::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MAY_NOT_BE_AN_OPTIONAL_PROPERTY_ACCESS
            } else {
                crate::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS
            };

            let loc = if data.left.kind == SyntaxKind::ParenthesizedExpression {
                node.loc
            } else {
                target.loc
            };
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                loc,
                message,
                Vec::new(),
            ));

            self.check_expression(&data.left);
            return;
        }
        let Some(left_type) = self.assignment_target_type(target) else {
            return;
        };

        if target.kind == Identifier {
            if let Some(sym) = self.resolve_identifier(target) {
                let base = self.resolve_alias_base(sym);
                if base.flags.intersects(
                    SymbolFlags::Class | SymbolFlags::ENUM | SymbolFlags::ValueModule,
                ) && !base.flags.intersects(
                    SymbolFlags::VARIABLE
                        | SymbolFlags::PROPERTY_OR_ACCESSOR
                        | SymbolFlags::Function,
                ) {
                    return;
                }

                if self.symbol_is_const_variable(&base) {
                    return;
                }
            }
        }

        if left_type.flags.contains(TypeFlags::Any)
            && left_type.intrinsic_name() == Some("error")
        {
            return;
        }

        if self.assignment_target_is_readonly(target) {
            return;
        }
        let right_type = match data.operator_token.kind {
            EqualsToken => self.get_type_of_node(&data.right),

            AmpersandAmpersandEqualsToken | BarBarEqualsToken
            | QuestionQuestionEqualsToken => {
                match self.logical_rhs_frame(data.operator_token.kind, target) {
                    Some((sym, t)) => {
                        self.logical_rhs_narrowing_frames.push((sym, t));
                        let rt = self.get_type_of_node(&data.right);
                        self.logical_rhs_narrowing_frames.pop();
                        rt
                    }
                    None => self.get_type_of_node(&data.right),
                }
            }

            _ => {
                if self
                    .arith_operand_error_nodes
                    .contains(&(Arc::as_ptr(node) as *const crate::ast::Node))
                {
                    return;
                }
                self.get_type_of_node(node)
            }
        };

        let _ = self.check_type_assignable_to_and_optionally_elaborate(
            &right_type,
            &left_type,
            Some(target),
            Some(&data.right),
            None,
            None,
        );
    }

    pub(crate) fn write_type_of_property_symbol(
        &mut self,
        prop: &Arc<crate::ast::Symbol>,
    ) -> Arc<Type> {
        if prop.flags.contains(SymbolFlags::SetAccessor)
            && let Some(setter) = prop
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SetAccessor)
            && let crate::ast::NodeData::SetAccessorDeclaration(sd) = &setter.data
            && let Some(param) = sd.parameters.iter().next()
            && let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(tn) = &pd.type_node
        {
            return self.get_type_from_type_node(tn);
        }
        self.get_type_of_symbol(prop)
    }

    pub(crate) fn assignment_target_type(&mut self, target: &Arc<Node>) -> Option<Arc<Type>> {
        match &target.data {
            crate::ast::NodeData::Identifier(_) => {
                let sym = self.resolve_identifier(target)?;
                let declared = self.get_type_of_symbol(&sym);

                let target_kind = get_assignment_target_kind(target);
                let compound_like = target_kind == AssignmentKind::Definite
                    && is_in_compound_like_assignment(target);
                if compound_like || target_kind == AssignmentKind::Compound {
                    Some(self.get_base_type_of_literal_type(&declared))
                } else {
                    Some(declared)
                }
            }
            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                let obj_type = self.get_type_of_node(&pa.expression);

                self.get_property_of_type(&obj_type, &pa.name.text())
                    .map(|sym| self.write_type_of_property_symbol(&sym))
            }
            crate::ast::NodeData::ElementAccessExpression(ea) => {

                if ea.argument_expression.kind == SyntaxKind::StringLiteral {
                    let obj_type = self.get_type_of_node(&ea.expression);
                    let name = ea.argument_expression.text();
                    if let Some(prop) = self.get_property_of_type(&obj_type, name) {
                        return Some(self.write_type_of_property_symbol(&prop));
                    }
                }
                let obj_type = self.get_type_of_node(&ea.expression);
                let index_type = self.get_type_of_node(&ea.argument_expression);
                Some(self.get_indexed_access_type(&obj_type, &index_type))
            }
            _ => None,
        }
    }

    pub(crate) fn assignment_target_is_readonly(&mut self, target: &Arc<Node>) -> bool {
        match &target.data {
            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                let obj_type = self.get_type_of_node(&pa.expression);
                if let Some(sym) = self.get_property_of_type(&obj_type, &pa.name.text())
                    && (sym.check_flags.contains(crate::ast::CheckFlags::Readonly)
                        || sym
                            .declarations
                            .iter()
                            .any(|d| d.has_syntactic_modifier(ModifierFlags::Readonly)))
                {
                    return true;
                }
                self.namespace_const_member(&pa.expression, &pa.name.text())
                    .is_some()
            }

            crate::ast::NodeData::ElementAccessExpression(ea)
                if ea.argument_expression.kind == SyntaxKind::StringLiteral =>
            {
                self.namespace_const_member(
                    &ea.expression,
                    ea.argument_expression.text(),
                )
                .is_some()
            }
            _ => false,
        }
    }

    pub(crate) fn namespace_const_member(
        &mut self,
        obj_expr: &Arc<Node>,
        name: &str,
    ) -> Option<Arc<crate::ast::Symbol>> {
        if obj_expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(obj_expr)?;
        let base = self.resolve_alias_base(sym);
        if !base.flags.contains(SymbolFlags::ValueModule) {
            return None;
        }
        let member = base
            .exports
            .get(name)
            .or_else(|| base.members.get(name))
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
                            .and_then(|l| l.get(name).cloned())
                    })
            });
        member.filter(|m| self.symbol_is_const_variable(m))
    }

    pub(crate) fn get_type_of_meta_property(&mut self, node: &Arc<Node>) -> Arc<Type> {
        use crate::core::compiler_options::ModuleKind;
        let (keyword_token, name) = match &node.data {
            crate::ast::NodeData::MetaProperty(d) => (d.keyword_token, &d.name),
            _ => return self.error_type(),
        };
        match keyword_token {
            SyntaxKind::NewKeyword => {

                self.any_type()
            }
            SyntaxKind::ImportKeyword => {
                if name.text() == "defer" {
                    return self.error_type();
                }
                if name.text() == "meta" {
                    match self.compiler_options.module {
                        ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20
                        | ModuleKind::NodeNext => {
                            let esm = self
                                .current_file
                                .as_ref()
                                .map(|f| {
                                    self.program_implied_format(&f.file_name)
                                        == ModuleKind::ESNext
                                })
                                .unwrap_or(false);
                            if !esm {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    crate::diagnostics::messages_generated::
                                        THE_IMPORT_META_META_PROPERTY_IS_NOT_ALLOWED_IN_FILES_WHICH_WILL_BUILD_INTO_COMMONJS_OUTPUT,
                                    Vec::new(),
                                ));
                            }
                        }
                        m => {

                            let es2020_or_later = matches!(
                                m,
                                ModuleKind::ES2020
                                    | ModuleKind::ES2022
                                    | ModuleKind::ESNext
                                    | ModuleKind::Preserve
                                    | ModuleKind::Node16
                                    | ModuleKind::Node18
                                    | ModuleKind::Node20
                                    | ModuleKind::NodeNext
                            );
                            if !es2020_or_later && m != ModuleKind::System {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    crate::diagnostics::messages_generated::
                                        THE_IMPORT_META_META_PROPERTY_IS_ONLY_ALLOWED_WHEN_THE_MODULE_OPTION_IS_ES2020_ES2022_ESNEXT_SYSTEM_NODE16_NODE18_NODE20_OR_NODENEXT,
                                    Vec::new(),
                                ));
                            }
                        }
                    }

                    if let Some(sym) = self.globals.get("ImportMeta").cloned() {
                        return self.resolve_interface_type(&sym, None);
                    }
                    self.any_type()
                } else {

                    self.error_type()
                }
            }
            _ => self.error_type(),
        }
    }

    pub(crate) fn program_implied_format(&self, file_name: &str) -> crate::core::compiler_options::ModuleKind {
        use crate::core::compiler_options::ModuleKind;
        match self.program.get_emit_module_format_of_file(file_name) {
            ModuleKind::None => {

                crate::compiler::implied_node_format_of_file(file_name, &|p| {
                    self.program.read_file(p)
                })
            }
            ModuleKind::ES2020 | ModuleKind::ESNext => ModuleKind::ESNext,
            _ => ModuleKind::CommonJS,
        }
    }

    pub(crate) fn syntactic_truthy_semantics(&mut self, node: &Arc<Node>) -> (bool, bool) {
        let mut n: Arc<Node> = Arc::clone(node);
        loop {
            match &n.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => {
                    n = Arc::clone(&p.expression)
                }
                crate::ast::NodeData::NonNullExpression(p) => n = Arc::clone(&p.expression),
                crate::ast::NodeData::AsExpression(p) => n = Arc::clone(&p.expression),
                crate::ast::NodeData::TypeAssertion(p) => n = Arc::clone(&p.expression),
                _ => break,
            }
        }
        use SyntaxKind::*;
        match n.kind {
            NumericLiteral => {
                let t = n.text();
                if t == "0" || t == "1" {
                    (true, true)
                } else {
                    (true, false)
                }
            }
            ArrayLiteralExpression
            | ArrowFunction
            | BigIntLiteral
            | ClassExpression
            | FunctionExpression
            | JsxElement
            | JsxSelfClosingElement
            | ObjectLiteralExpression
            | RegularExpressionLiteral => (true, false),
            VoidExpression | NullKeyword => (false, true),
            NoSubstitutionTemplateLiteral | StringLiteral => {
                if !n.text().is_empty() {
                    (true, false)
                } else {
                    (false, true)
                }
            }
            ConditionalExpression => {
                if let crate::ast::NodeData::ConditionalExpression(d) = &n.data {
                    let (a1, n1) = self.syntactic_truthy_semantics(&d.when_true);
                    let (a2, n2) = self.syntactic_truthy_semantics(&d.when_false);
                    (a1 || a2, n1 || n2)
                } else {
                    (true, true)
                }
            }
            Identifier => {

                if let Some(sym) = self.resolve_identifier(&n)
                    && self.is_undefined_symbol(&sym)
                {
                    return (false, true);
                }
                (true, true)
            }
            _ => (true, true),
        }
    }

    pub(crate) fn check_truthiness_of_type(&mut self, node: &Arc<Node>) {
        let (always, never) = self.syntactic_truthy_semantics(node);
        let message = if always && !never {
            crate::diagnostics::messages_generated::THIS_KIND_OF_EXPRESSION_IS_ALWAYS_TRUTHY
        } else if never && !always {
            crate::diagnostics::messages_generated::THIS_KIND_OF_EXPRESSION_IS_ALWAYS_FALSY
        } else {
            return;
        };
        let file = self.current_file.clone();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            node.loc,
            message,
            Vec::new(),
        ));
    }
}
