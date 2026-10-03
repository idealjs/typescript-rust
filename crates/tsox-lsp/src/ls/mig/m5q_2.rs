#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::ls::autoimport_import_adder::ImportAdderTrait;
use crate::ls::code_actions_missing_member::MissingMemberFixer;
use crate::ls::lsutil_utilities::get_quote_preference as lsutil_get_quote_preference;
use tsox_core::diagnostics;
use crate::ls::lsutil_user_preferences::QuotePreference;
use tsox_checker::checker::mig::m2f::{self as nb2, new_emit_context, new_node_builder_ex};
use tsox_checker::checker::mig::m2g::r21k9_defs::NodeFactoryExt21;
use tsox_checker::checker::nodecopy::NodeFactoryStub;
use tsox_checker::checker::symboltracker::{
    NodeBuilderFlags as Flags, NodeBuilderInternalFlags as InternalFlags,
};
use tsox_checker::checker::types::UnionReduction;
use tsox_checker::checker::{Checker, Signature, Type};
use tsox_frontend::ast::node_data_generated::{
    ArrayTypeNodeData, ArrowFunctionData, BlockData, ComputedPropertyNameData,
    FunctionDeclarationData, FunctionExpressionData, MethodDeclarationData, NewExpressionData,
    ParameterDeclarationData, StringLiteralData, ThrowStatementData, TypeParameterDeclarationData,
};
use tsox_frontend::ast::node_node_list::ModifierList;
use tsox_frontend::ast::{self, Node, NodeData, NodeList, SourceFile, Symbol, SyntaxKind};

impl<'a> MissingMemberFixer<'a> {
    pub fn create_node_builder(
        &mut self,
    ) -> (nb2::NodeBuilder<'_>, HashMap<u64, Arc<Symbol>>) { ::tsox_core::fntrace::enter("create_node_builder"); 
        let id_to_symbol: HashMap<u64, Arc<Symbol>> = HashMap::new();
        let node_builder = self.type_checker.get_node_builder_ex(HashMap::new());
        (node_builder, id_to_symbol)
    }

    pub fn get_call_signatures_for_type(
        &self,
        t: &Arc<Type>,
    ) -> Vec<Arc<Signature>> { ::tsox_core::fntrace::enter("get_call_signatures_for_type"); 
        if t.is_union() {
            let mut signatures: Vec<Arc<Signature>> = Vec::new();
            if let Some(types) = t.types() {
                for ty in types {
                    signatures.extend(self.type_checker.get_call_signatures(ty));
                }
            }
            return signatures;
        }
        self.type_checker.get_call_signatures(t)
    }

    pub fn create_type_node(
        &mut self,
        t: &Arc<Type>,
        enclosing_declaration: &Arc<Node>,
        flags: Flags,
        node_builder: &mut nb2::NodeBuilder<'_>,
        id_to_symbol: &mut HashMap<u64, Arc<Symbol>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_type_node"); 
        let type_node = node_builder.type_to_type_node_ex(
            t,
            Some(enclosing_declaration),
            flags,
            InternalFlags::None,
        );
        self.import_type_node(type_node, id_to_symbol)
    }

    pub fn create_modifiers(
        &self,
        symbol: &Arc<Symbol>,
        declaration: Option<&Arc<Node>>,
    ) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("create_modifiers"); 
        let mut modifier_flags = ast::ModifierFlags::empty();
        if let Some(declaration) = declaration {
            let effective = tsox_checker::checker::get_declaration_modifier_flags_from_symbol(
                symbol,
            );
            modifier_flags = effective & ast::ModifierFlags::Static;
            if effective.contains(ast::ModifierFlags::Public) {
                modifier_flags |= ast::ModifierFlags::Public;
            } else if effective.contains(ast::ModifierFlags::Protected) {
                modifier_flags |= ast::ModifierFlags::Protected;
            }
            if tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration(declaration) {
                modifier_flags |= ast::ModifierFlags::Accessor;
            }
        }
        if self.should_add_override_keyword(declaration) {
            modifier_flags |= ast::ModifierFlags::Override;
        }
        if modifier_flags.is_empty() {
            return None;
        }
        let factory = &self.node_factory;
        Some(Arc::new(factory.new_modifier_list(
            tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags(
                modifier_flags,
                &|kind| factory.new_modifier(kind),
            ),
        )))
    }

    pub fn should_add_override_keyword(&self, declaration: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("should_add_override_keyword"); 
        match declaration {
            Some(declaration) => {
                self.program.options().no_implicit_override.is_true()
                    && ast::has_abstract_modifier(declaration)
            }
            None => false,
        }
    }

    pub fn create_signature_declaration_from_signature(
        &mut self,
        signature: &Arc<Signature>,
        kind: SyntaxKind,
        source_file: &Arc<SourceFile>,
        enclosing_declaration: &Arc<Node>,
        body: Option<Arc<Node>>,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<Arc<Node>>,
        optional: bool,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_signature_declaration_from_signature"); 
        let quote_preference = lsutil_get_quote_preference(source_file, self.preferences);
        let mut flags = Flags::NoTruncation
            | Flags::SuppressAnyReturnType
            | Flags::AllowEmptyTuple;
        if quote_preference == QuotePreference::Single {
            flags |= Flags::UseSingleQuotesForStringLiteralType;
        }

        let (mut node_builder, mut id_to_symbol) = self.create_node_builder();
        let signature_declaration = node_builder
            .signature_to_signature_declaration_ex(
                signature,
                kind,
                Some(enclosing_declaration),
                flags,
                InternalFlags::AllowUnresolvedNames,
            )?;

        let is_js = ast::is_in_js_file(enclosing_declaration);
        let mut parameters: Option<Arc<NodeList>> =
            tsox_frontend::ast::mig::m3b::parameter_list(&signature_declaration).cloned();
        let mut type_parameters: Option<Arc<NodeList>> = if is_js {
            None
        } else {
            tsox_frontend::ast::mig::m3c::type_parameter_list(&signature_declaration).cloned()
        };
        let mut type_node: Option<Arc<Node>> = if is_js {
            None
        } else {
            signature_declaration.type_node().cloned()
        };

        if let Some(type_parameters) = &type_parameters {
            if !type_parameters.nodes.is_empty() {
                let mut nodes: Vec<Arc<Node>> = Vec::with_capacity(type_parameters.nodes.len());
                for tp in &type_parameters.nodes {
                    let NodeData::TypeParameterDeclaration(type_parameter) = &tp.data else {
                        nodes.push(tp.clone());
                        continue;
                    };
                    let mut constraint = type_parameter.constraint.clone();
                    if let Some(constraint) = &mut constraint {
                        *constraint =
                            self.import_type_node(Some(constraint.clone()), &mut id_to_symbol)?;
                    }
                    let mut default_type = type_parameter.default_type.clone();
                    if let Some(default_type) = &mut default_type {
                        *default_type =
                            self.import_type_node(Some(default_type.clone()), &mut id_to_symbol)?;
                    }
                    nodes.push(update_type_parameter_declaration(
                        type_parameter.modifiers.clone(),
                        type_parameter.name.clone(),
                        constraint,
                        type_parameter.expression.clone(),
                        default_type,
                    ));
                }
                parameters = Some(Arc::new(self.node_factory.new_node_list(nodes)));
            }
        }

        if let Some(params) = &parameters {
            let mut nodes: Vec<Arc<Node>> = Vec::with_capacity(params.nodes.len());
            for p in &params.nodes {
                let NodeData::ParameterDeclaration(parameter) = &p.data else {
                    nodes.push(p.clone());
                    continue;
                };
                let mut parameter_type_node = parameter.type_node.clone();
                if let Some(parameter_type_node) = &mut parameter_type_node {
                    *parameter_type_node =
                        self.import_type_node(Some(parameter_type_node.clone()), &mut id_to_symbol)?;
                }
                nodes.push(update_parameter_declaration(
                    parameter.modifiers.clone(),
                    parameter.dot_dot_dot_token.clone(),
                    parameter.name.clone(),
                    if is_js {
                        None
                    } else {
                        parameter.question_token.clone()
                    },
                    parameter_type_node,
                    parameter.initializer.clone(),
                ));
            }
            parameters = Some(Arc::new(self.node_factory.new_node_list(nodes)));
        }

        if let Some(type_node) = &mut type_node {
            *type_node = self.import_type_node(Some(type_node.clone()), &mut id_to_symbol)?;
        }

        let question_token = if optional {
            Some(self.node_factory.new_token(SyntaxKind::QuestionToken))
        } else {
            None
        };

        match kind {
            SyntaxKind::FunctionExpression => {
                let NodeData::FunctionExpression(fn_node) = &signature_declaration.data else {
                    return None;
                };
                let name = match &name {
                    Some(n) if ast::is_identifier(n) => Some(n.clone()),
                    _ => None,
                };
                Some(update_function_expression(
                    modifiers,
                    fn_node.asterisk_token.clone(),
                    name,
                    type_parameters,
                    parameters,
                    type_node,
                    fn_node.full_signature.clone(),
                    Some(body.unwrap_or_else(|| fn_node.body.clone())),
                ))
            }
            SyntaxKind::ArrowFunction => {
                let NodeData::ArrowFunction(fn_node) = &signature_declaration.data else {
                    return None;
                };
                Some(update_arrow_function(
                    modifiers,
                    type_parameters,
                    parameters,
                    type_node,
                    fn_node.full_signature.clone(),
                    fn_node.equals_greater_than_token.clone(),
                    body.or_else(|| Some(fn_node.body.clone())),
                ))
            }
            SyntaxKind::MethodDeclaration => {
                let NodeData::MethodDeclaration(method) = &signature_declaration.data else {
                    return None;
                };
                let method_name = match &name {
                    Some(n) => create_property_name(n, quote_preference),
                    None => NodeFactoryExt21::new_identifier(&NodeFactoryStub, ""),
                };
                Some(update_method_declaration(
                    modifiers,
                    method.asterisk_token.clone(),
                    method_name,
                    question_token,
                    type_parameters,
                    parameters,
                    type_node,
                    method.full_signature.clone(),
                    body,
                    method.postfix_token.clone(),
                ))
            }
            SyntaxKind::FunctionDeclaration => {
                let NodeData::FunctionDeclaration(fn_node) = &signature_declaration.data else {
                    return None;
                };
                let name = match &name {
                    Some(n) if ast::is_identifier(n) => Some(n.clone()),
                    _ => None,
                };
                Some(update_function_declaration(
                    modifiers,
                    fn_node.asterisk_token.clone(),
                    name,
                    type_parameters,
                    parameters,
                    type_node,
                    fn_node.full_signature.clone(),
                    body.or_else(|| fn_node.body.clone()),
                ))
            }
            _ => None,
        }
    }

    pub fn create_signature_declaration_from_signatures(
        &mut self,
        signatures: &[Arc<Signature>],
        name: Option<Arc<Node>>,
        optional: bool,
        modifiers: Option<Arc<ModifierList>>,
        quote_preference: QuotePreference,
        body: Option<Arc<Node>>,
        enclosing_declaration: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_signature_declaration_from_signatures"); 
        if signatures.is_empty() {
            return None;
        }

        let mut max_args_signature = signatures[0].clone();
        let mut min_argument_count = signatures[0].min_argument_count();

        let mut has_rest_parameter = false;
        for signature in signatures {
            min_argument_count = min_argument_count.min(signature.min_argument_count());
            if signature.has_rest_parameter() {
                has_rest_parameter = true;
            }
            if signature.parameters().len() >= max_args_signature.parameters().len()
                && (!signature.has_rest_parameter() || max_args_signature.has_rest_parameter())
            {
                max_args_signature = signature.clone();
            }
        }

        let max_non_rest_args =
            max_args_signature.parameters().len() - if max_args_signature.has_rest_parameter() { 1 } else { 0 };
        let mut parameter_names: Vec<String> = Vec::with_capacity(max_args_signature.parameters().len());
        for symbol in max_args_signature.parameters() {
            parameter_names.push(symbol.name.clone());
        }
        let mut parameters = create_dummy_parameters(
            &NodeFactoryStub,
            max_non_rest_args,
            &parameter_names,
            &[],
            min_argument_count,
            ast::is_in_js_file(enclosing_declaration),
        );

        if has_rest_parameter {
            let mut rest_parameter_name = "rest".to_string();
            if max_non_rest_args < parameter_names.len() && !parameter_names[max_non_rest_args].is_empty()
            {
                rest_parameter_name = parameter_names[max_non_rest_args].clone();
            }

            let question_token = if max_non_rest_args >= min_argument_count {
                Some(self.node_factory.new_token(SyntaxKind::QuestionToken))
            } else {
                None
            };

            parameters.push(NodeFactoryExt21::new_parameter_declaration(
                &NodeFactoryStub,
                None,
                Some(self.node_factory.new_token(SyntaxKind::DotDotDotToken)),
                NodeFactoryExt21::new_identifier(&NodeFactoryStub, &rest_parameter_name),
                question_token,
                Some(new_array_type_node_m5q2(NodeFactoryExt21::new_keyword_type_node(
                    &NodeFactoryStub,
                    SyntaxKind::UnknownKeyword,
                ))),
                None,
            ));
        }

        let method_name = match &name {
            Some(n) => create_property_name(n, quote_preference),
            None => NodeFactoryExt21::new_identifier(&NodeFactoryStub, ""),
        };

        Some(new_method_declaration_m5q2(
            modifiers,
            None,
            method_name,
            if optional {
                Some(self.node_factory.new_token(SyntaxKind::QuestionToken))
            } else {
                None
            },
            None,
            Arc::new(self.node_factory.new_node_list(parameters)),
            self.get_return_type_from_signatures(signatures, enclosing_declaration),
            None,
            self.create_body(body, quote_preference, false),
        ))
    }

    pub fn get_return_type_from_signatures(
        &mut self,
        signatures: &[Arc<Signature>],
        enclosing_declaration: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_return_type_from_signatures"); 
        if signatures.is_empty() {
            return None;
        }

        let mut return_types: Vec<Arc<Type>> = Vec::with_capacity(signatures.len());
        for signature in signatures {
            if let Some(return_type) = self.type_checker.get_return_type_of_signature(signature) {
                return_types.push(return_type);
            }
        }

        let union_type = self
            .type_checker
            .get_union_type_ex(return_types, UnionReduction::Literal);
        let (mut node_builder, mut id_to_symbol) = self.create_node_builder();
        let type_node = node_builder.type_to_type_node_ex(
            &union_type,
            Some(enclosing_declaration),
            Flags::NoTruncation,
            InternalFlags::AllowUnresolvedNames,
        );
        self.import_type_node(type_node, &mut id_to_symbol)
    }

    pub fn import_type_node(
        &mut self,
        type_node: Option<Arc<Node>>,
        id_to_symbol: &HashMap<u64, Arc<Symbol>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_type_node"); 
        let type_node = type_node?;
        if self.import_adder.is_none() {
            return Some(type_node);
        }

        let (imported_type_node, symbols) =
            crate::mig::m5o3::try_get_auto_importable_reference_from_type_node(
                &type_node,
                id_to_symbol,
            );
        if let Some(imported_type_node) = imported_type_node {
            for symbol in symbols {
                let export_symbol = self.get_exported_symbol(&symbol);
                let export_symbol = match export_symbol {
                    Some(s) => s,
                    None => continue,
                };
                self.import_adder
                    .as_mut()
                    .unwrap()
                    .add_import_from_exported_symbol(&export_symbol, true);
            }
            return Some(imported_type_node);
        }

        let mut seen: HashMap<usize, bool> = HashMap::new();
        for symbol in id_to_symbol.values() {
            let key = Arc::as_ptr(symbol) as usize;
            if seen.contains_key(&key) {
                continue;
            }
            seen.insert(key, true);
            let export_symbol = self.get_exported_symbol(symbol);
            let export_symbol = match export_symbol {
                Some(s) => s,
                None => continue,
            };
            self.import_adder
                .as_mut()
                .unwrap()
                .add_import_from_exported_symbol(&export_symbol, true);
        }
        Some(type_node)
    }

    pub fn get_exported_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_exported_symbol"); 
        let symbol = self.type_checker.get_export_symbol_of_symbol(symbol);
        if symbol.parent().is_none() {
            return None;
        }
        Some(symbol)
    }

    pub fn create_index_signature_declaration_from_type(
        &mut self,
        class_declaration: &Arc<Node>,
        implemented_type: &Arc<Type>,
        key_type: &Arc<Type>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_index_signature_declaration_from_type"); 
        let index_info = self
            .type_checker
            .get_index_info_of_type(implemented_type, key_type)?;
        let mut builder =
            new_node_builder_ex(&*self.type_checker, new_emit_context(), HashMap::new());
        builder.impl_.index_info_to_index_signature_declaration(
            &index_info,
            Some(class_declaration),
            Flags::None,
            InternalFlags::None,
            None,
        )
    }

    pub fn create_body(
        &self,
        body: Option<Arc<Node>>,
        quote_preference: QuotePreference,
        signature_only: bool,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_body"); 
        if signature_only {
            return None;
        }
        let body = body.map(|b| ast::deep_clone_node(&b));
        match body {
            Some(body) => Some(body),
            None => self.create_stubbed_method_body(quote_preference),
        }
    }

    pub fn create_stubbed_method_body(&self, quote_preference: QuotePreference) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_stubbed_method_body"); 
        let token_flags = if quote_preference == QuotePreference::Single {
            tsox_frontend::scanner::TOKEN_FLAGS_SINGLE_QUOTE
        } else {
            0
        };

        let message = diagnostics::METHOD_NOT_IMPLEMENTED.localize(&self.locale, &[]);
        let throw_expr = new_new_expression_m5q2(
            NodeFactoryExt21::new_identifier(&NodeFactoryStub, "Error"),
            None,
            Some(Arc::new(
                self.node_factory
                    .new_node_list(vec![new_string_literal_m5q2(&message, token_flags)]),
            )),
        );
        Some(new_block_m5q2(
            Arc::new(self.node_factory.new_node_list(vec![new_throw_statement_m5q2(
                throw_expr,
            )])),
            true,
        ))
    }
}

pub fn create_dummy_parameters(
    factory: &NodeFactoryStub,
    arg_count: usize,
    names: &[String],
    types: &[Option<Arc<Node>>],
    min_argument_count: usize,
    in_js: bool,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("create_dummy_parameters"); 
    let mut parameters: Vec<Arc<Node>> = Vec::with_capacity(arg_count);
    let mut parameter_name_counts: HashMap<String, i32> = HashMap::new();

    for i in 0..arg_count {
        let mut parameter_name = if i < names.len() && !names[i].is_empty() {
            names[i].clone()
        } else {
            format!("arg{}", i)
        };

        let count = parameter_name_counts.entry(parameter_name.clone()).or_insert(0);
        *count += 1;
        let count_val = *count - 1;
        if count_val > 0 {
            parameter_name += &count_val.to_string();
        }

        let question_token = if i >= min_argument_count {
            Some(NodeFactoryExt21::new_token(
                factory,
                SyntaxKind::QuestionToken,
            ))
        } else {
            None
        };

        let type_node = if in_js {
            None
        } else if i < types.len() && types[i].is_some() {
            types[i].clone()
        } else {
            Some(NodeFactoryExt21::new_keyword_type_node(
                factory,
                SyntaxKind::UnknownKeyword,
            ))
        };
        parameters.push(NodeFactoryExt21::new_parameter_declaration(
            factory,
            None,
            None,
            NodeFactoryExt21::new_identifier(factory, &parameter_name),
            question_token,
            type_node,
            None,
        ));
    }
    parameters
}

pub fn create_declaration_name(
    type_checker: &Checker,
    symbol: Option<&Arc<Symbol>>,
    declaration: Option<&Arc<Node>>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_declaration_name"); 
    if let Some(symbol) = symbol {
        if symbol.check_flags.intersects(ast::CheckFlags::Mapped) {
            let name_type = type_checker.get_name_type_of_symbol(symbol);
            if let Some(name_type) = name_type {
                if tsox_checker::checker::is_type_usable_as_property_name(&name_type) {
                    return Some(NodeFactoryExt21::new_identifier(
                        &NodeFactoryStub,
                        &tsox_checker::checker::get_property_name_from_type(&name_type),
                    ));
                }
            }
        }
    }
    if let Some(declaration) = declaration {
        if let Some(name) = declaration.name() {
            return Some(NodeFactoryExt21::clone_node(&NodeFactoryStub, name));
        }
    }
    symbol.map(|symbol| NodeFactoryExt21::new_identifier(&NodeFactoryStub, &symbol.name))
}

pub fn create_property_name(
    node: &Arc<Node>,
    quote_preference: QuotePreference,
) -> Arc<Node> { ::tsox_core::fntrace::enter("create_property_name"); 
    if ast::is_identifier(node) && node.text() == "constructor" {
        let token_flags = if quote_preference == QuotePreference::Single {
            tsox_frontend::scanner::TOKEN_FLAGS_SINGLE_QUOTE
        } else {
            0
        };
        return new_computed_property_name_m5q2(new_string_literal_m5q2(
            &node.text(),
            token_flags,
        ));
    }
    ast::deep_clone_node(node)
}

fn update_type_parameter_declaration(
    modifiers: Option<Arc<ModifierList>>,
    name: Arc<Node>,
    constraint: Option<Arc<Node>>,
    expression: Option<Arc<Node>>,
    default_type: Option<Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("update_type_parameter_declaration"); 
    Arc::new(Node::new(
        SyntaxKind::TypeParameter,
        NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
            modifiers,
            name,
            constraint,
            expression,
            default_type,
        }),
    ))
}

fn update_parameter_declaration(
    modifiers: Option<Arc<ModifierList>>,
    dot_dot_dot_token: Option<Arc<Node>>,
    name: Arc<Node>,
    question_token: Option<Arc<Node>>,
    type_node: Option<Arc<Node>>,
    initializer: Option<Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("update_parameter_declaration"); 
    Arc::new(Node::new(
        SyntaxKind::Parameter,
        NodeData::ParameterDeclaration(ParameterDeclarationData {
            modifiers,
            dot_dot_dot_token,
            name,
            question_token,
            type_node,
            initializer,
        }),
    ))
}

fn update_function_expression(
    modifiers: Option<Arc<ModifierList>>,
    asterisk_token: Option<Arc<Node>>,
    name: Option<Arc<Node>>,
    type_parameters: Option<Arc<NodeList>>,
    parameters: Option<Arc<NodeList>>,
    type_node: Option<Arc<Node>>,
    full_signature: Option<Arc<Node>>,
    body: Option<Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("update_function_expression"); 
    Arc::new(Node::new(
        SyntaxKind::FunctionExpression,
        NodeData::FunctionExpression(FunctionExpressionData {
            modifiers,
            asterisk_token,
            name,
            type_parameters,
            parameters: parameters.unwrap_or_else(|| Arc::new(NodeList::new(Vec::new()))),
            type_node,
            full_signature,
            body: body.unwrap_or_else(|| Arc::new(Node::new(SyntaxKind::Block, NodeData::Block(BlockData {
                statements: Arc::new(NodeList::new(Vec::new())),
                multi_line: false,
            })))),
        }),
    ))
}

fn update_arrow_function(
    modifiers: Option<Arc<ModifierList>>,
    type_parameters: Option<Arc<NodeList>>,
    parameters: Option<Arc<NodeList>>,
    type_node: Option<Arc<Node>>,
    full_signature: Option<Arc<Node>>,
    equals_greater_than_token: Arc<Node>,
    body: Option<Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("update_arrow_function"); 
    Arc::new(Node::new(
        SyntaxKind::ArrowFunction,
        NodeData::ArrowFunction(ArrowFunctionData {
            modifiers,
            type_parameters,
            parameters: parameters.unwrap_or_else(|| Arc::new(NodeList::new(Vec::new()))),
            type_node,
            full_signature,
            equals_greater_than_token,
            body: body.unwrap_or_else(|| Arc::new(Node::new(SyntaxKind::Block, NodeData::Block(BlockData {
                statements: Arc::new(NodeList::new(Vec::new())),
                multi_line: false,
            })))),
        }),
    ))
}

fn update_method_declaration(
    modifiers: Option<Arc<ModifierList>>,
    asterisk_token: Option<Arc<Node>>,
    name: Arc<Node>,
    question_token: Option<Arc<Node>>,
    type_parameters: Option<Arc<NodeList>>,
    parameters: Option<Arc<NodeList>>,
    type_node: Option<Arc<Node>>,
    full_signature: Option<Arc<Node>>,
    body: Option<Arc<Node>>,
    postfix_token: Option<Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("update_method_declaration"); 
    Arc::new(Node::new(
        SyntaxKind::MethodDeclaration,
        NodeData::MethodDeclaration(MethodDeclarationData {
            modifiers,
            asterisk_token,
            name,
            postfix_token,
            type_parameters,
            parameters: parameters.unwrap_or_else(|| Arc::new(NodeList::new(Vec::new()))),
            type_node,
            full_signature,
            body,
        }),
    ))
}

fn update_function_declaration(
    modifiers: Option<Arc<ModifierList>>,
    asterisk_token: Option<Arc<Node>>,
    name: Option<Arc<Node>>,
    type_parameters: Option<Arc<NodeList>>,
    parameters: Option<Arc<NodeList>>,
    type_node: Option<Arc<Node>>,
    full_signature: Option<Arc<Node>>,
    body: Option<Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("update_function_declaration"); 
    Arc::new(Node::new(
        SyntaxKind::FunctionDeclaration,
        NodeData::FunctionDeclaration(FunctionDeclarationData {
            modifiers,
            asterisk_token,
            name,
            type_parameters,
            parameters: parameters.unwrap_or_else(|| Arc::new(NodeList::new(Vec::new()))),
            type_node,
            full_signature,
            body,
        }),
    ))
}

fn new_method_declaration_m5q2(
    modifiers: Option<Arc<ModifierList>>,
    asterisk_token: Option<Arc<Node>>,
    name: Arc<Node>,
    question_token: Option<Arc<Node>>,
    type_parameters: Option<Arc<NodeList>>,
    parameters: Arc<NodeList>,
    type_node: Option<Arc<Node>>,
    full_signature: Option<Arc<Node>>,
    body: Option<Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_method_declaration_m5q2"); 
    Arc::new(Node::new(
        SyntaxKind::MethodDeclaration,
        NodeData::MethodDeclaration(MethodDeclarationData {
            modifiers,
            asterisk_token,
            name,
            postfix_token: None,
            type_parameters,
            parameters,
            type_node,
            full_signature,
            body,
        }),
    ))
}

fn new_array_type_node_m5q2(element_type: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_array_type_node_m5q2"); 
    Arc::new(Node::new(
        SyntaxKind::ArrayType,
        NodeData::ArrayTypeNode(ArrayTypeNodeData { element_type }),
    ))
}

fn new_computed_property_name_m5q2(expression: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_computed_property_name_m5q2"); 
    Arc::new(Node::new(
        SyntaxKind::ComputedPropertyName,
        NodeData::ComputedPropertyName(ComputedPropertyNameData { expression }),
    ))
}

fn new_string_literal_m5q2(
    text: &str,
    token_flags: tsox_frontend::ast::node_data_generated::TokenFlags,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_string_literal_m5q2"); 
    Arc::new(Node::new(
        SyntaxKind::StringLiteral,
        NodeData::StringLiteral(StringLiteralData {
            text: text.to_string(),
            token_flags,
        }),
    ))
}

fn new_new_expression_m5q2(
    expression: Arc<Node>,
    type_arguments: Option<Arc<NodeList>>,
    arguments: Option<Arc<NodeList>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_new_expression_m5q2"); 
    Arc::new(Node::new(
        SyntaxKind::NewExpression,
        NodeData::NewExpression(NewExpressionData {
            expression,
            type_arguments,
            arguments,
        }),
    ))
}

fn new_throw_statement_m5q2(expression: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_throw_statement_m5q2"); 
    Arc::new(Node::new(
        SyntaxKind::ThrowStatement,
        NodeData::ThrowStatement(ThrowStatementData { expression }),
    ))
}

fn new_block_m5q2(statements: Arc<NodeList>, multi_line: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("new_block_m5q2"); 
    Arc::new(Node::new(
        SyntaxKind::Block,
        NodeData::Block(BlockData {
            statements,
            multi_line,
        }),
    ))
}
