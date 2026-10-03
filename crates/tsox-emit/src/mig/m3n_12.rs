use super::DeclarationTransformer;
use crate::mig::m4e::{R38K1NodeVisitorExt, R42K01DataExt};
use crate::mig::m4e::r39k01_defs::R39K01DataExt;
use crate::mig::m4e::r37k1_defs::R37K1DataExt;
use std::sync::Arc;
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::{is_entity_name, is_entity_name_expression, NodeList, SyntaxKind};

pub fn transform_subtree_kind(tx: &mut DeclarationTransformer, input: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_subtree_kind"); 
    match input.kind {
        SyntaxKind::PropertyDeclaration => tx.transform_property_declaration_subtree(input),
        SyntaxKind::PropertySignature => tx.transform_property_signature_subtree(input),
        SyntaxKind::MethodDeclaration => tx.transform_method_declaration_subtree(input),
        SyntaxKind::MethodSignature => tx.transform_method_signature_subtree(input),
        SyntaxKind::GetAccessor => tx.transform_get_accessor_subtree(input),
        SyntaxKind::SetAccessor => tx.transform_set_accessor_subtree(input),
        SyntaxKind::VariableDeclaration => tx.transform_variable_declaration_subtree(input),
        SyntaxKind::ExpressionWithTypeArguments => {
            let expression = input.as_expression_with_type_arguments().expression.clone();
            if is_entity_name(&expression) || is_entity_name_expression(&expression) {
                let enclosing = tx.enclosing_declaration.clone();
                tx.check_entity_name_visibility(&expression, enclosing.as_ref());
            }
            tx.visitor().visit_each_child(input)
        }
        SyntaxKind::QualifiedName => {
            let right = input.as_qualified_name().right.clone();
            if right.kind == SyntaxKind::PrivateIdentifier {
                tx.tracker.state.add_diagnostic(super::super::m3n_5::create_diagnostic_for_node(
                    input,
                    Some(&super::super::m4e::Diagnostics::DeclarationEmitElidesPrivateMembersBut0RefersToAPrivateMemberWriteAnExplicitTypeHere),
                    &[right.text().to_string()],
                ));
            }
            tx.visitor().visit_each_child(input)
        }
        _ => tx.visitor().visit_each_child(input),
    }
}

fn is_effectively_private(tx: &DeclarationTransformer, input: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_effectively_private"); 
    let parse_node = tx.emit_context().parse_node(input);
    parse_node
        .as_ref()
        .map(|pn| {
            tx.host
                .get_effective_declaration_flags(pn, ModifierFlags::Private)
                != ModifierFlags::empty()
        })
        .unwrap_or(false)
}

fn is_this_parameter(p: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_this_parameter"); 
    p.name()
        .map(|n| n.kind == SyntaxKind::Identifier && n.text() == "this")
        .unwrap_or(false)
}

impl DeclarationTransformer {
    pub fn transform_property_declaration_subtree(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_property_declaration_subtree"); 
        if input.name().map(|n| n.kind == SyntaxKind::PrivateIdentifier).unwrap_or(false) {
            return None;
        }
        let mut postfix_token = input.as_property_declaration().postfix_token.clone();
        if let Some(tok) = &postfix_token {
            if tok.kind == SyntaxKind::ExclamationToken {
                postfix_token = None;
            }
        }
        let ensured_type = self.ensure_type(input, false);
        let no_initializer = self.ensure_no_initializer(input);
        Some(self.factory().update_property_declaration(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap(),
            postfix_token,
            ensured_type,
            no_initializer,
        ))
    }

    pub fn transform_property_signature_subtree(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_property_signature_subtree"); 
        if input.name().map(|n| n.kind == SyntaxKind::PrivateIdentifier).unwrap_or(false) {
            return None;
        }
        let ps = input.as_property_signature_declaration();
        let mods = self.ensure_modifiers(input);
        let ensured_type = self.ensure_type(input, false);
        let no_initializer = self.ensure_no_initializer(input);
        let result = self.factory().update_property_signature_declaration(
            input,
            mods,
            input.name().unwrap().clone(),
            ps.postfix_token.clone(),
            ensured_type,
            no_initializer,
        );
        self.preserve_partial_js_doc(&result, input);
        Some(result)
    }

    pub fn transform_method_declaration_subtree(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_method_declaration_subtree"); 
        if is_effectively_private(self, input) {
            return None;
        }
        if input.name().map(|n| n.kind == SyntaxKind::PrivateIdentifier).unwrap_or(false) {
            return None;
        }
        let md = input.as_method_declaration();
        let params = md.parameters.clone();
        let type_params = md
            .type_parameters
            .as_ref()
            .and_then(|tp| self.ensure_type_params(input, Some(tp)));
        let new_params = self.update_param_list(input, &params);
        let ensured_type = self.ensure_type(input, false);
        Some(self.factory().update_method_declaration(
            input,
            self.ensure_modifiers(input),
            None,
            input.name().unwrap(),
            md.postfix_token.clone(),
            type_params,
            &new_params,
            ensured_type,
            None,
            None,
        ))
    }

    pub fn transform_method_signature_subtree(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_method_signature_subtree"); 
        if is_effectively_private(self, input) {
            return None;
        }
        if input.name().map(|n| n.kind == SyntaxKind::PrivateIdentifier).unwrap_or(false) {
            return None;
        }
        let ms = input.as_method_signature_declaration();
        let params = ms.parameters.clone();
        let type_params = ms
            .type_parameters
            .as_ref()
            .and_then(|tp| self.ensure_type_params(input, Some(tp)));
        let new_params = self.update_param_list(input, &params);
        let ensured_type = self.ensure_type(input, false);
        Some(self.factory().update_method_signature_declaration(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap().clone(),
            ms.postfix_token.clone(),
            type_params,
            new_params,
            ensured_type,
        ))
    }

    pub fn transform_get_accessor_subtree(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_get_accessor_subtree"); 
        if input.name().map(|n| n.kind == SyntaxKind::PrivateIdentifier).unwrap_or(false) {
            return None;
        }
        let is_private = is_effectively_private(self, input);
        let new_params = self.update_accessor_param_list_subtree(input, is_private);
        let ensured_type = self.ensure_type(input, false);
        Some(self.factory().update_get_accessor_declaration(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap(),
            None,
            &new_params,
            ensured_type,
            None,
            None,
        ))
    }

    pub fn transform_set_accessor_subtree(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_set_accessor_subtree"); 
        if input.name().map(|n| n.kind == SyntaxKind::PrivateIdentifier).unwrap_or(false) {
            return None;
        }
        let is_private = is_effectively_private(self, input);
        let new_params = self.update_accessor_param_list_subtree(input, is_private);
        Some(self.factory().update_set_accessor_declaration(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap(),
            None,
            &new_params,
            None,
            None,
            None,
        ))
    }

    fn update_accessor_param_list_subtree(&mut self, input: &Arc<Node>, is_private: bool) -> Arc<NodeList> { ::tsox_core::fntrace::enter("update_accessor_param_list_subtree"); 
        let accessor_params: Arc<NodeList> = if input.kind == SyntaxKind::SetAccessor {
            input.as_set_accessor_declaration().parameters.clone()
        } else {
            input.as_get_accessor_declaration().parameters.clone()
        };
        let mut new_params: Vec<Arc<Node>> = Vec::new();
        if !is_private {
            if let Some(this_param) = accessor_params.nodes.iter().find(|p| is_this_parameter(p)) {
                new_params.push(self.ensure_parameter(this_param));
            }
        }
        if input.kind == SyntaxKind::SetAccessor {
            let params: Vec<Arc<Node>> = accessor_params.nodes.iter().cloned().collect();
            let value_param = if !is_private {
                if new_params.len() == 1 && params.len() >= 2 {
                    Some(self.ensure_parameter(&params[1]))
                } else if new_params.is_empty() && !params.is_empty() {
                    Some(self.ensure_parameter(&params[0]))
                } else {
                    None
                }
            } else {
                None
            };
            let value_param = match value_param {
                Some(p) => p,
                None => {
                    let t = if !is_private {
                        Some(self.factory().new_keyword_type_node(SyntaxKind::AnyKeyword))
                    } else {
                        None
                    };
                    self.factory().new_parameter_declaration(
                        None,
                        None,
                        &self.factory().new_identifier("value"),
                        None,
                        t.as_ref(),
                        None,
                    )
                }
            };
            new_params.push(value_param);
        }
        self.factory().new_node_list(new_params)
    }

    pub fn transform_variable_declaration_subtree(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_variable_declaration_subtree"); 
        let Some(name) = input.name() else {
            return self.visitor().visit_each_child(input);
        };
        if matches!(name.kind, SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern) {
            return self.visitor().visit_each_child(input);
        }
        self.suppress_new_diagnostic_contexts = true;
        let visited_name = self.binding_name_visitor.visit_node(name);
        let ensured_type = self.ensure_type(input, false);
        let no_initializer = self.ensure_no_initializer(input);
        Some(self.factory().update_variable_declaration_node(
            input,
            &visited_name,
            None,
            ensured_type,
            no_initializer,
        ))
    }
}
