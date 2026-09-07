#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_variable_declaration_list(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::VariableDeclarationList(data) = &node.data {
            for decl in data.declarations.iter() {
                if let crate::ast::NodeData::VariableDeclaration(vd) = &decl.data
                    && let Some(init) = &vd.initializer
                    && (node.has_syntactic_modifier(ModifierFlags::Ambient)
                        || node
                            .parent
                            .as_ref()
                            .is_some_and(|p| p.has_syntactic_modifier(ModifierFlags::Ambient))
                        || self.ambient_context_depth > 0
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file))
                    && self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                {
                    let is_const = node.flags.contains(NodeFlags::Const);
                    let is_simple_literal = match &init.data {
                        crate::ast::NodeData::StringLiteral(_)
                        | crate::ast::NodeData::NumericLiteral(_)
                        | crate::ast::NodeData::BigIntLiteral(_)
                        | crate::ast::NodeData::NoSubstitutionTemplateLiteral(_) => true,
                        _ if matches!(
                            init.kind,
                            SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword
                        ) =>
                        {
                            true
                        }

                        crate::ast::NodeData::PropertyAccessExpression(_)
                        | crate::ast::NodeData::ElementAccessExpression(_) => true,
                        _ => false,
                    };
                    let message = if is_const && vd.type_node.is_none() {
                        if is_simple_literal {
                            None
                        } else {
                            Some(
                                crate::diagnostics::messages_generated::
                                    A_CONST_INITIALIZER_IN_AN_AMBIENT_CONTEXT_MUST_BE_A_STRING_OR_NUMERIC_LITERAL_OR_LITERAL_ENUM_REFERENCE,
                            )
                        }
                    } else {
                        Some(
                            crate::diagnostics::messages_generated::
                                INITIALIZERS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS,
                        )
                    };
                    if let Some(message) = message {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            init.loc,
                            message,
                            vec![],
                        ));
                    }
                }

                if let crate::ast::NodeData::VariableDeclaration(vd) = &decl.data
                    && vd.name.kind == SyntaxKind::Identifier
                    && matches!(vd.name.text(), "eval" | "arguments")
                    && self.in_strict_context()
                {
                    let is_module = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.external_module_indicator.is_some());
                    let message = if is_module {
                        crate::diagnostics::messages_generated::
                            INVALID_USE_OF_0_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE
                    } else {
                        crate::diagnostics::messages_generated::INVALID_USE_OF_0_IN_STRICT_MODE
                    };
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        vd.name.loc,
                        message,
                        vec![vd.name.text().to_string()],
                    ));
                }
                self.check_variable_declaration(decl);
            }
        }
    }

    pub(crate) fn in_strict_context(&self) -> bool {
        if self.program.options().always_strict.is_true() {
            return true;
        }
        self.current_file.as_ref().is_some_and(|f| {
            f.external_module_indicator.is_some()
                || f.text.trim_start().starts_with("\"use strict\"")
                || f.text.trim_start().starts_with("'use strict'")
        })
    }

    pub(crate) fn report_abstract_property_access_in_ctor(
        &mut self,
        name_node: &Arc<Node>,
        prop_text: &str,
        this_type: &Arc<Type>,
    ) {
        let Some(structured) = this_type.as_structured() else {
            return;
        };
        let Some(member_symbol) = structured.members.get(prop_text) else {
            return;
        };
        let Some(abstract_decl) = member_symbol.declarations.iter().find(|d| {
            d.kind == SyntaxKind::PropertyDeclaration
                && d.has_syntactic_modifier(ModifierFlags::Abstract)
        }) else {
            return;
        };
        let Some(parent) = &abstract_decl.parent else {
            return;
        };
        let Some(class_name) = class_declaration_name(parent) else {
            return;
        };
        let file = self.current_file.clone();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            name_node.loc,
            crate::diagnostics::messages_generated::
                ABSTRACT_PROPERTY_0_IN_CLASS_1_CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR,
            vec![prop_text.to_string(), class_name],
        ));
    }

    pub(crate) fn access_in_property_initializer(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent.as_ref();
        while let Some(a) = cur {
            match a.kind {
                SyntaxKind::PropertyDeclaration => return true,
                SyntaxKind::Constructor
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::MethodSignature
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::ClassDeclaration
                | SyntaxKind::ClassExpression => return false,
                _ => {}
            }
            cur = a.parent.as_ref();
        }
        false
    }

    pub(crate) fn check_this_destructuring_abstract_properties(
        &mut self,
        pattern: &Arc<Node>,
        this_type: &Arc<Type>,
    ) {
        let Some(structured) = this_type.as_structured() else {
            return;
        };
        let crate::ast::NodeData::BindingPattern(data) = &pattern.data else {
            return;
        };
        for element in data.elements.iter() {
            let crate::ast::NodeData::BindingElement(el) = &element.data else {
                continue;
            };

            let Some(prop_name_node) = el
                .property_name
                .as_ref()
                .or(el.name.as_ref())
                .filter(|n| n.kind == SyntaxKind::Identifier)
            else {
                continue;
            };
            let prop_text = prop_name_node.text();
            let Some(member_symbol) = structured.members.get(prop_text) else {
                continue;
            };
            let Some(abstract_decl) = member_symbol.declarations.iter().find(|d| {
                d.kind == SyntaxKind::PropertyDeclaration
                    && d.has_syntactic_modifier(ModifierFlags::Abstract)
            }) else {
                continue;
            };
            let Some(parent) = &abstract_decl.parent else {
                continue;
            };
            let Some(class_name) = class_declaration_name(parent) else {
                continue;
            };
            let file = self.current_file.clone();
            let diagnostic = crate::ast::Diagnostic::new(
                file,
                prop_name_node.loc,
                crate::diagnostics::messages_generated::
                    ABSTRACT_PROPERTY_0_IN_CLASS_1_CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR,
                vec![prop_text.to_string(), class_name],
            );
            self.diagnostics.add(diagnostic);
        }
    }
}
