#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::mig::m4g::{ClassFieldsTransformer, PrivateEnvironment, PrivateIdentifierInfo};
use crate::mig::m4g_2::r37k13_defs::ClassFieldsTransformerR37k13;
use crate::mig::m4g::r39k15_defs::NodeFactoryR39k15;
use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags};
use super::m4q::r33k12_defs::PrivateIdentifierKind;
use tsox_frontend::ast::has_static_modifier;
use super::m4h_3::new_for_await_transformer;
use tsox_core::core::compiler_options_kinds::ScriptTarget;
use tsox_frontend::ast::is_class_expression;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::node_data_generated::{
    is_get_accessor_declaration, is_method_declaration, is_property_declaration,
    is_set_accessor_declaration,
};

use super::m3m::chain;
use super::m4f_3::ClassFieldsTransformer as M4F3ClassFieldsTransformer;
use super::m4h::new_exponentiation_transformer;
use super::m4h::new_logical_assignment_transformer;
use super::m4h_5::new_es_decorator_transformer;
use super::m4i::new_nullish_coalescing_transformer;
use super::m4i::new_optional_catch_transformer;
use super::m4i::new_tagged_template_lift_restriction_transformer;
use super::m4i_3::new_object_rest_spread_transformer;
use super::m4i_6::new_optional_chain_transformer;
use super::m4i_8::new_using_declaration_transformer;

#[path = "r39k24_defs.rs"]
pub mod r39k24_defs;

use crate::printer::EmitContext;

fn generated_name_key(emit_context: &EmitContext, name: &Arc<Node>) -> *const Node {
    let node_for_name = emit_context.get_node_for_generated_name(name);
    Arc::as_ptr(&node_for_name)
}

type TransformerFactory = fn(&super::m3m::TransformOptions) -> Option<Box<super::m4k_2::Transformer>>;

fn shell_transformer_visit(_tx: &mut super::m4k_2::Transformer, node: Arc<Node>) -> Option<Arc<Node>> {
    Some(node)
}

fn boxed_shell_transformer(opts: &super::m3m::TransformOptions) -> Box<super::m4k_2::Transformer> {
    Box::new(super::m4k_2::Transformer::new(
        shell_transformer_visit,
        Some(opts.context.clone()),
    ))
}

fn es_decorator_and_class_fields(opts: &super::m3m::TransformOptions) -> Option<Box<super::m4k_2::Transformer>> {
    chain(vec![
        (|opts: &super::m3m::TransformOptions| new_es_decorator_transformer(opts).map(Box::new)) as TransformerFactory,
        (|opts: &super::m3m::TransformOptions| {
            M4F3ClassFieldsTransformer::new_class_fields_transformer(opts).map(|tx| {
                Box::new(super::m4k_2::Transformer::new(
                    super::m4f::r39k19_defs::class_fields_transformer_visit_entry,
                    Some(tx.emit_context()),
                ))
            })
        }) as TransformerFactory,
    ])(opts)
}

pub const NEW_ES_NEXT_TRANSFORMER: TransformerFactory = |opts| {
    chain(vec![
        (|opts: &super::m3m::TransformOptions| {
            let _tx = new_using_declaration_transformer(opts);
            Some(boxed_shell_transformer(opts))
        }) as TransformerFactory,
        es_decorator_and_class_fields,
    ])(opts)
};

pub const NEW_ES2021_TRANSFORMER: TransformerFactory = |opts| {
    chain(vec![
        NEW_ES_NEXT_TRANSFORMER,
        (|opts: &super::m3m::TransformOptions| Some(Box::new(new_logical_assignment_transformer(opts)))) as TransformerFactory,
    ])(opts)
};

pub const NEW_ES2020_TRANSFORMER: TransformerFactory = |opts| {
    chain(vec![
        NEW_ES2021_TRANSFORMER,
        (|opts: &super::m3m::TransformOptions| {
            let _tx = new_nullish_coalescing_transformer(opts);
            Some(boxed_shell_transformer(opts))
        }) as TransformerFactory,
        (|opts: &super::m3m::TransformOptions| {
            let _tx = new_optional_chain_transformer(opts);
            Some(boxed_shell_transformer(opts))
        }) as TransformerFactory,
    ])(opts)
};

pub const NEW_ES2019_TRANSFORMER: TransformerFactory = |opts| {
    chain(vec![
        NEW_ES2020_TRANSFORMER,
        (|opts: &super::m3m::TransformOptions| {
            let _tx = new_optional_catch_transformer(opts);
            Some(boxed_shell_transformer(opts))
        }) as TransformerFactory,
    ])(opts)
};

pub const NEW_ES2018_TRANSFORMER: TransformerFactory = |opts| {
    chain(vec![
        NEW_ES2019_TRANSFORMER,
        (|opts: &super::m3m::TransformOptions| {
            let _tx = new_object_rest_spread_transformer(opts);
            Some(boxed_shell_transformer(opts))
        }) as TransformerFactory,
        (|opts: &super::m3m::TransformOptions| Some(Box::new(new_for_await_transformer(opts)))) as TransformerFactory,
        (|opts: &super::m3m::TransformOptions| {
            let _tx = new_tagged_template_lift_restriction_transformer(opts);
            Some(boxed_shell_transformer(opts))
        }) as TransformerFactory,
    ])(opts)
};

pub const NEW_ES2017_TRANSFORMER: TransformerFactory = |opts| {
    chain(vec![
        NEW_ES2018_TRANSFORMER,
        (|opts: &super::m3m::TransformOptions| {
            let tx = super::m4f::AsyncTransformer::new_async_transformer(opts);
            let emit_context = tx.emit_context();
            Some(Box::new(super::m4k_2::Transformer::new(
                super::m4f::r39k19_defs::async_transformer_visit_entry,
                Some(emit_context),
            )))
        }) as TransformerFactory,
    ])(opts)
};

pub const NEW_ES2016_TRANSFORMER: TransformerFactory = |opts| {
    chain(vec![
        NEW_ES2017_TRANSFORMER,
        (|opts: &super::m3m::TransformOptions| Some(Box::new(new_exponentiation_transformer(opts)))) as TransformerFactory,
    ])(opts)
};

pub fn get_es_transformer(opts: &super::m3m::TransformOptions) -> Option<Box<super::m4k_2::Transformer>> {
    let options = &opts.compiler_options;
    match options.get_emit_script_target() {
        ScriptTarget::ESNext => es_decorator_and_class_fields(opts),
        ScriptTarget::ES2025 | ScriptTarget::ES2024 | ScriptTarget::ES2023 | ScriptTarget::ES2022 | ScriptTarget::ES2021 => {
            NEW_ES_NEXT_TRANSFORMER(opts)
        }
        ScriptTarget::ES2020 => NEW_ES2021_TRANSFORMER(opts),
        ScriptTarget::ES2019 => NEW_ES2020_TRANSFORMER(opts),
        ScriptTarget::ES2018 => NEW_ES2019_TRANSFORMER(opts),
        ScriptTarget::ES2017 => NEW_ES2018_TRANSFORMER(opts),
        ScriptTarget::ES2016 => NEW_ES2017_TRANSFORMER(opts),
        _ => NEW_ES2016_TRANSFORMER(opts),
    }
}

impl ClassFieldsTransformer {
    pub fn end_class_lexical_environment(&mut self) {
        self.lexical_environment = self
            .lexical_environment
            .take()
            .and_then(|env| env.previous.map(|previous| *previous));
    }

    pub fn get_class_lexical_environment(&mut self) -> &mut super::m4g::ClassLexicalEnvironment {
        let env = self
            .lexical_environment
            .as_mut()
            .expect("lexicalEnvironment should be set");
        if env.data.is_none() {
            env.data = Some(super::m4g::ClassLexicalEnvironment {
                facts: super::m4g::ClassFacts::default(),
                class_constructor: None,
                class_this: None,
                super_class_reference: None,
            });
        }
        env.data.as_mut().unwrap()
    }

    pub fn get_private_identifier_environment(&mut self) -> &mut PrivateEnvironment {
        let env = self
            .lexical_environment
            .as_mut()
            .expect("lexicalEnvironment should be set");
        if env.private_env.is_none() {
            env.private_env = Some(PrivateEnvironment {
                data: super::m4g::PrivateEnvironmentData {
                    class_name: None,
                    weak_set_name: None,
                },
                members: HashMap::new(),
                generated_identifiers: HashMap::new(),
            });
        }
        env.private_env.as_mut().unwrap()
    }

    pub fn add_pending_expressions(&mut self, exprs: Vec<Arc<Node>>) {
        self.pending_expressions.extend(exprs);
    }

    pub fn set_private_identifier(&mut self, name: &Arc<Node>, info: PrivateIdentifierInfo) {
        if self.emit_context().has_auto_generate_info(name) {
            let key = generated_name_key(&self.emit_context(), name);
            let env = self.get_private_identifier_environment();
            env.generated_identifiers.insert(key, info);
        } else {
            let env = self.get_private_identifier_environment();
            env.members.insert(name.text().to_string(), info);
        }
    }

    pub fn get_private_identifier<'a>(
        &self,
        env: &'a PrivateEnvironment,
        name: &Arc<Node>,
    ) -> Option<&'a PrivateIdentifierInfo> {
        if self.emit_context().has_auto_generate_info(name) {
            let key = generated_name_key(&self.emit_context(), name);
            return env.generated_identifiers.get(&key);
        }
        env.members.get(name.text())
    }

    pub fn add_private_identifier_property_declaration_to_environment(
        &mut self,
        node: &Arc<Node>,
        name: &Arc<Node>,
    ) {
        let lex = self.get_class_lexical_environment();
        let brand_check_identifier = if lex.class_this.is_some() {
            lex.class_this.clone()
        } else {
            lex.class_constructor.clone()
        };
        let is_static = has_static_modifier(node);
        let emit_context = self.emit_context();
        let previous_info = {
            let env = self
                .lexical_environment
                .as_mut()
                .expect("lexicalEnvironment should be set")
                .private_env
                .as_mut()
                .expect("privateEnvironment should be set");
            get_private_identifier_in(&emit_context, env, name).cloned()
        };
        let is_valid = !self.is_reserved_private_name(name) && previous_info.is_none();
        if is_static {
            let variable_name = self.create_hoisted_variable_for_private_name(name, "");
            self.set_private_identifier(
                name,
                PrivateIdentifierInfo {
                    kind: PrivateIdentifierKind::Field,
                    is_static: true,
                    brand_check_identifier,
                    variable_name: Some(variable_name),
                    is_valid,
                    method_name: None,
                    getter_name: None,
                    setter_name: None,
                },
            );
        } else {
            let weak_map_name = self.create_hoisted_variable_for_private_name(name, "");
            self.set_private_identifier(
                name,
                PrivateIdentifierInfo {
                    kind: PrivateIdentifierKind::Field,
                    is_static: false,
                    brand_check_identifier: Some(weak_map_name.clone()),
                    variable_name: None,
                    is_valid,
                    method_name: None,
                    getter_name: None,
                    setter_name: None,
                },
            );
            let emit_context = self.emit_context();
            let factory = crate::printer::NodeFactory::new(&emit_context);
            let new_expr = factory.new_new_expression(
                &factory.new_identifier("WeakMap"),
                None,
                Some(factory.new_node_list(vec![])),
            );
            let assignment = factory.new_assignment_expression(&weak_map_name, &new_expr);
            self.add_pending_expressions(vec![assignment]);
        }
    }

    pub fn add_private_identifier_method_to_environment(
        &mut self,
        name: &Arc<Node>,
        lex: &super::m4g::ClassLexicalEnvironment,
        is_static: bool,
        is_valid: bool,
    ) {
        let method_name = self.create_hoisted_variable_for_private_name(name, "");
        let weak_set_name = self.current_weak_set_name();
        let brand_check_identifier = if is_static {
            lex.class_this.clone().or_else(|| lex.class_constructor.clone())
        } else {
            weak_set_name
        };
        self.set_private_identifier(
            name,
            PrivateIdentifierInfo {
                kind: PrivateIdentifierKind::Method,
                method_name: Some(method_name),
                brand_check_identifier,
                is_static,
                is_valid,
                variable_name: None,
                getter_name: None,
                setter_name: None,
            },
        );
    }

    pub fn add_private_identifier_get_accessor_to_environment(
        &mut self,
        name: &Arc<Node>,
        lex: &super::m4g::ClassLexicalEnvironment,
        is_static: bool,
        is_valid: bool,
        previous_info: Option<&PrivateIdentifierInfo>,
    ) {
        let getter_name = self.create_hoisted_variable_for_private_name(name, "_get");
        let weak_set_name = self.current_weak_set_name();
        let brand_check_identifier = if is_static {
            lex.class_this.clone().or_else(|| lex.class_constructor.clone())
        } else {
            weak_set_name
        };

        if let Some(previous) = previous_info {
            if matches!(previous.kind, PrivateIdentifierKind::Accessor)
                && previous.is_static == is_static
                && previous.getter_name.is_none()
            {
                let slot = self.current_private_identifier_slot(name);
                if let Some(info) = slot {
                    info.getter_name = Some(getter_name);
                    return;
                }
            }
        }
        self.set_private_identifier(
            name,
            PrivateIdentifierInfo {
                kind: PrivateIdentifierKind::Accessor,
                getter_name: Some(getter_name),
                brand_check_identifier,
                is_static,
                is_valid,
                variable_name: None,
                method_name: None,
                setter_name: None,
            },
        );
    }

    pub fn add_private_identifier_set_accessor_to_environment(
        &mut self,
        name: &Arc<Node>,
        lex: &super::m4g::ClassLexicalEnvironment,
        is_static: bool,
        is_valid: bool,
        previous_info: Option<&PrivateIdentifierInfo>,
    ) {
        let setter_name = self.create_hoisted_variable_for_private_name(name, "_set");
        let weak_set_name = self.current_weak_set_name();
        let brand_check_identifier = if is_static {
            lex.class_this.clone().or_else(|| lex.class_constructor.clone())
        } else {
            weak_set_name
        };

        if let Some(previous) = previous_info {
            if matches!(previous.kind, PrivateIdentifierKind::Accessor)
                && previous.is_static == is_static
                && previous.setter_name.is_none()
            {
                let slot = self.current_private_identifier_slot(name);
                if let Some(info) = slot {
                    info.setter_name = Some(setter_name);
                    return;
                }
            }
        }
        self.set_private_identifier(
            name,
            PrivateIdentifierInfo {
                kind: PrivateIdentifierKind::Accessor,
                setter_name: Some(setter_name),
                brand_check_identifier,
                is_static,
                is_valid,
                variable_name: None,
                method_name: None,
                getter_name: None,
            },
        );
    }

    pub fn add_private_identifier_auto_accessor_to_environment(
        &mut self,
        name: &Arc<Node>,
        lex: &super::m4g::ClassLexicalEnvironment,
        is_static: bool,
        is_valid: bool,
    ) {
        let getter_name = self.create_hoisted_variable_for_private_name(name, "_get");
        let setter_name = self.create_hoisted_variable_for_private_name(name, "_set");
        let weak_set_name = self.current_weak_set_name();
        let brand_check_identifier = if is_static {
            lex.class_this.clone().or_else(|| lex.class_constructor.clone())
        } else {
            weak_set_name
        };

        self.set_private_identifier(
            name,
            PrivateIdentifierInfo {
                kind: PrivateIdentifierKind::Accessor,
                getter_name: Some(getter_name),
                setter_name: Some(setter_name),
                brand_check_identifier,
                is_static,
                is_valid,
                variable_name: None,
                method_name: None,
            },
        );
    }

    pub fn add_private_identifier_to_environment(&mut self, node: &Arc<Node>) {
        let name = node.name().expect("PrivateIdentifier should have a name");
        let is_static = has_static_modifier(node);
        self.get_private_identifier_environment();
        let emit_context = self.emit_context();
        let previous_info = {
            let env = self
                .lexical_environment
                .as_mut()
                .expect("lexicalEnvironment should be set")
                .private_env
                .as_mut()
                .expect("privateEnvironment should be set");
            get_private_identifier_in(&emit_context, env, &name).cloned()
        };
        let is_valid = !self.is_reserved_private_name(&name) && previous_info.is_none();

        let lex = clone_class_lexical_environment_shallow(self.get_class_lexical_environment());
        if super::m4q::r33k12_defs::is_auto_accessor_property_declaration(node) {
            self.add_private_identifier_auto_accessor_to_environment(&name, &lex, is_static, is_valid);
        } else if is_property_declaration(node) {
            self.add_private_identifier_property_declaration_to_environment(node, &name);
        } else if is_method_declaration(node) {
            self.add_private_identifier_method_to_environment(&name, &lex, is_static, is_valid);
        } else if is_get_accessor_declaration(node) {
            self.add_private_identifier_get_accessor_to_environment(
                &name,
                &lex,
                is_static,
                is_valid,
                previous_info.as_ref(),
            );
        } else if is_set_accessor_declaration(node) {
            self.add_private_identifier_set_accessor_to_environment(
                &name,
                &lex,
                is_static,
                is_valid,
                previous_info.as_ref(),
            );
        }
    }

    fn current_weak_set_name(&mut self) -> Option<Arc<Node>> {
        self.lexical_environment
            .as_mut()
            .and_then(|lex| lex.private_env.as_mut())
            .and_then(|env| env.data.weak_set_name.clone())
    }

    fn current_private_identifier_slot(&mut self, name: &Arc<Node>) -> Option<&mut PrivateIdentifierInfo> {
        let emit_context = self.emit_context();
        let has_auto_generate = emit_context.has_auto_generate_info(name);
        let env = self
            .lexical_environment
            .as_mut()
            .expect("lexicalEnvironment should be set")
            .private_env
            .as_mut()
            .expect("privateEnvironment should be set");
        if has_auto_generate {
            let key = generated_name_key(&emit_context, name);
            env.generated_identifiers.get_mut(&key)
        } else {
            env.members.get_mut(name.text())
        }
    }

    pub fn create_hoisted_variable_for_class(
        &mut self,
        name_text: &str,
        node: &Arc<Node>,
        suffix: &str,
    ) -> Arc<Node> {
        let class_name_text = self
            .get_private_identifier_environment()
            .data
            .class_name
            .as_ref()
            .map(|n| n.text().to_string());
        let emit_context = self.emit_context();
        let factory = crate::printer::NodeFactory::new(&emit_context);
        let identifier = match &class_name_text {
            Some(class_name) => {
                let prefix = format!("_{}_", class_name);
                factory.generated_name_node(&factory.new_unique_name_ex(
                    &format!("{}{}", prefix, name_text),
                    AutoGenerateOptions {
                        flags: GeneratedIdentifierFlags::OPTIMISTIC
                            | GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
                        prefix: String::new(),
                        suffix: suffix.to_string(),
                    },
                ))
            }
            None => factory.generated_name_node(&factory.new_unique_name_ex(
                &format!("_{}", name_text),
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::OPTIMISTIC
                        | GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
                    prefix: String::new(),
                    suffix: suffix.to_string(),
                },
            )),
        };
        let _ = node;
        if requires_block_scoped_var_r36k24(self) {
            self.emit_context().add_lexical_declaration(&identifier);
        } else {
            self.emit_context().add_variable_declaration(&identifier);
        }
        identifier
    }

    pub fn create_hoisted_variable_for_class_from_node(&mut self, name: &Arc<Node>, suffix: &str) -> Arc<Node> {
        let env = self.get_private_identifier_environment();
        let prefix = match &env.data.class_name {
            Some(class_name) => format!("_{}_", class_name.text()),
            None => "_".to_string(),
        };
        let identifier = {
            let emit_context = self.emit_context();
            let factory = crate::printer::NodeFactory::new(&emit_context);
            factory.generated_name_node(&factory.new_generated_name_for_node_ex(
                name,
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::OPTIMISTIC
                        | GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
                    prefix,
                    suffix: suffix.to_string(),
                },
            ))
        };
        if requires_block_scoped_var_r36k24(self) {
            self.emit_context().add_lexical_declaration(&identifier);
        } else {
            self.emit_context().add_variable_declaration(&identifier);
        }
        identifier
    }

    pub fn create_hoisted_variable_for_private_name(&mut self, name: &Arc<Node>, suffix: &str) -> Arc<Node> {
        if self.emit_context().has_auto_generate_info(name) {
            return self.create_hoisted_variable_for_class_from_node(name, suffix);
        }
        let mut text = name.text().to_string();
        if text.starts_with('#') {
            text = text[1..].to_string();
        }
        self.create_hoisted_variable_for_class(&text, name, suffix)
    }

    pub fn access_private_identifier(&self, name: &Arc<Node>) -> Option<PrivateIdentifierInfo> {
        let mut env = self.lexical_environment.as_ref();
        while let Some(current) = env {
            if let Some(private_env) = &current.private_env {
                if let Some(info) = self.get_private_identifier(private_env, name) {
                    if matches!(info.kind, PrivateIdentifierKind::Untransformed) {
                        return None;
                    }
                    return Some(info.clone());
                }
            }
            env = current.previous.as_deref();
        }
        None
    }

    pub fn add_instance_method_statements(
        &mut self,
        statements: Vec<Arc<Node>>,
        methods: &[Arc<Node>],
        receiver: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        if !self.should_transform_private_elements_or_class_static_blocks || methods.is_empty() {
            return statements;
        }
        let env = self.get_private_identifier_environment();
        let weak_set_name = env
            .data
            .weak_set_name
            .clone()
            .expect("weakSetName should be set in private identifier environment");
        let initializer = self.factory().new_method_call(
            &weak_set_name,
            &self.factory().new_identifier("add"),
            &[receiver.clone()],
        );
        let mut result = statements;
        result.push(self.factory().new_expression_statement(&initializer));
        result
    }
}

fn get_private_identifier_in<'a>(
    emit_context: &EmitContext,
    env: &'a PrivateEnvironment,
    name: &Arc<Node>,
) -> Option<&'a PrivateIdentifierInfo> {
    if emit_context.has_auto_generate_info(name) {
        let key = generated_name_key(emit_context, name);
        return env.generated_identifiers.get(&key);
    }
    env.members.get(name.text())
}

fn requires_block_scoped_var_r36k24(t: &ClassFieldsTransformer) -> bool {
    t.in_iteration_statement
        && t.current_class_container.is_some()
        && is_class_expression(t.current_class_container.as_ref().unwrap())
}

fn clone_class_lexical_environment_shallow(
    env: &super::m4g::ClassLexicalEnvironment,
) -> super::m4g::ClassLexicalEnvironment {
    super::m4g::ClassLexicalEnvironment {
        facts: env.facts,
        class_constructor: env.class_constructor.clone(),
        class_this: env.class_this.clone(),
        super_class_reference: env.super_class_reference.clone(),
    }
}

impl Clone for PrivateIdentifierKind {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for PrivateIdentifierKind {}

impl Clone for PrivateIdentifierInfo {
    fn clone(&self) -> Self {
        PrivateIdentifierInfo {
            kind: self.kind,
            brand_check_identifier: self.brand_check_identifier.clone(),
            is_static: self.is_static,
            is_valid: self.is_valid,
            variable_name: self.variable_name.clone(),
            method_name: self.method_name.clone(),
            getter_name: self.getter_name.clone(),
            setter_name: self.setter_name.clone(),
        }
    }
}
