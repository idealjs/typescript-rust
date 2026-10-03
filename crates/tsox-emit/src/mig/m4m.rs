#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_checker::checker::mig::m2c_5::TypeReferenceSerializationKind;
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_core::core::compiler_options_kinds::ScriptTarget;
use tsox_core::core::text::TextRange;
use tsox_core::debug::fail;
use tsox_frontend::ast::mig::m3b::{members, parameter_list};
use tsox_frontend::ast::mig::m3f::get_rest_parameter_element_type;
use tsox_frontend::ast::mig::m3f_3::is_async_function;
use tsox_frontend::ast::mig::m3g_2::is_this_parameter;
use tsox_frontend::ast::mig::m3g_3::skip_type_parentheses;
use tsox_frontend::ast::mig::w3::get_all_accessor_declarations;
use tsox_frontend::ast::mig::x4ast::get_first_constructor_with_body;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::{
    is_binary_expression, is_class_like, is_conditional_expression, is_function_like,
    is_identifier, is_literal_type_node, is_numeric_literal, is_parenthesized_expression,
    is_property_access_expression, is_string_literal, is_type_of_expression, is_void_expression,
    node_is_present, NodeFlags, NodeList,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::format::mig::m4o::NodeFactory;
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;

#[path = "r33k4_defs.rs"]
pub mod r33k4_defs;
#[path = "r36k5_defs.rs"]
pub mod r36k5_defs;

use self::r33k4_defs::arc_node_kind_string;
use self::r36k5_defs::NodeDataExt;
use crate::mig::m4m_2::is_generated_identifier;
use crate::printer::EmitContext;

pub struct MetadataSerializerContext {
    pub current_lexical_scope: Option<Arc<Node>>,
    pub current_name_scope: Option<Arc<Node>>,
    pub serializing_conditional_type_branch: bool,
}

impl Default for MetadataSerializerContext {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        MetadataSerializerContext {
            current_lexical_scope: None,
            current_name_scope: None,
            serializing_conditional_type_branch: false,
        }
    }
}

pub struct MetadataSerializer {
    pub resolver: EmitResolver,
    pub language_version: ScriptTarget,
    pub strict_null_checks: bool,
    pub f: NodeFactory,
    pub emit_context: EmitContext,
    pub c: MetadataSerializerContext,
}

pub fn new_metadata_serializer(
    resolver: EmitResolver,
    factory: NodeFactory,
    emit_context: EmitContext,
    language_version: ScriptTarget,
    strict_null_checks: bool,
) -> MetadataSerializer { ::tsox_core::fntrace::enter("new_metadata_serializer"); 
    MetadataSerializer {
        resolver,
        language_version,
        strict_null_checks,
        f: factory,
        emit_context,
        c: MetadataSerializerContext::default(),
    }
}

pub fn get_set_accessor_value_parameter(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_set_accessor_value_parameter"); 
    let parameters = node.parameters()?;
    let nodes = &parameters.nodes;
    if !nodes.is_empty() {
        if nodes.len() >= 2 && is_this_parameter(&nodes[0]) {
            return Some(nodes[1].clone());
        }
        return Some(nodes[0].clone());
    }
    None
}

pub fn get_set_accessor_type_annotation_node(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_set_accessor_type_annotation_node"); 
    let p = get_set_accessor_value_parameter(node)?;
    p.type_().cloned()
}

pub fn get_accessor_type_node(node: &Arc<Node>, container: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_accessor_type_node"); 
    let accessors = get_all_accessor_declarations(members(container), node);
    if let Some(set_accessor) = accessors.set_accessor {
        return get_set_accessor_type_annotation_node(&set_accessor);
    }
    if let Some(get_accessor) = accessors.get_accessor {
        return get_accessor.type_().cloned();
    }
    None
}

impl MetadataSerializer {
    pub(crate) fn factory(&self) -> &NodeFactory { ::tsox_core::fntrace::enter("factory"); 
        &self.f
    }

    pub(crate) fn emit_context(&self) -> &EmitContext { ::tsox_core::fntrace::enter("emit_context"); 
        &self.emit_context
    }

    pub fn set_context(&mut self, ctx: MetadataSerializerContext) { ::tsox_core::fntrace::enter("set_context"); 
        self.c = ctx;
    }

    pub fn serialize_type_of_node(
        &mut self,
        ctx: MetadataSerializerContext,
        node: &Arc<Node>,
        container: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_type_of_node"); 
        let old_ctx = std::mem::replace(&mut self.c, ctx);
        let result = self.serialize_type_of_node_impl(node, container);
        self.c = old_ctx;
        result
    }

    pub fn serialize_parameter_types_of_node(
        &mut self,
        ctx: MetadataSerializerContext,
        node: &Arc<Node>,
        container: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_parameter_types_of_node"); 
        let old_ctx = std::mem::replace(&mut self.c, ctx);
        let result = self.serialize_parameter_types_of_node_impl(node, container);
        self.c = old_ctx;
        result
    }

    pub fn serialize_return_type_of_node(
        &mut self,
        ctx: MetadataSerializerContext,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_return_type_of_node"); 
        let old_ctx = std::mem::replace(&mut self.c, ctx);
        let result = self.serialize_return_type_of_node_impl(node);
        self.c = old_ctx;
        result
    }

    fn serialize_type_of_node_impl(
        &mut self,
        node: &Arc<Node>,
        container: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_type_of_node_impl"); 
        match node.kind {
            SyntaxKind::PropertyDeclaration | SyntaxKind::Parameter => {
                self.serialize_type_node(node.type_())
            }
            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                let container = container.expect("accessor requires container");
                self.serialize_type_node(get_accessor_type_node(node, container).as_ref())
            }
            SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::MethodDeclaration => Some(
                self.factory()
                    .new_identifier("Function"),
            ),
            _ => Some(self.factory().new_void_zero_expression()),
        }
    }

    fn serialize_parameter_types_of_node_impl(
        &mut self,
        node: &Arc<Node>,
        container: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_parameter_types_of_node_impl"); 
        let value_declaration: Option<Arc<Node>> = if is_class_like(node) {
            get_first_constructor_with_body(node)
        } else if is_function_like(node) && node_is_present(node.body()) {
            Some(node.clone())
        } else {
            None
        };

        let value_declaration = match value_declaration {
            Some(d) => d,
            None => {
                return Some(
                    self.factory()
                        .new_array_literal_expression(self.factory().new_node_list(&[]), false),
                )
            }
        };

        let mut expressions: Vec<Arc<Node>> = vec![];
        let parameters = get_parameters_of_decorated_declaration(&value_declaration, container);
        for (i, parameter) in parameters.iter().enumerate() {
            if i == 0
                && is_identifier(parameter)
                && parameter.name().map(|n| n.text() == "this").unwrap_or(false)
            {
                continue;
            }
            if parameter
                .as_parameter_declaration()
                .dot_dot_dot_token
                .is_some()
            {
                expressions.push(
                    self.serialize_type_node(
                        get_rest_parameter_element_type(parameter.type_()).as_ref(),
                    )
                    .unwrap(),
                );
            } else {
                expressions.push(
                    self.serialize_type_of_node_impl(parameter, container).unwrap(),
                );
            }
        }
        Some(
            self.factory()
                .new_array_literal_expression(self.factory().new_node_list(&expressions), false),
        )
    }

    fn serialize_return_type_of_node_impl(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_return_type_of_node_impl"); 
        if is_function_like(node) && node.type_().is_some() {
            self.serialize_type_node(node.type_())
        } else if is_async_function(node) {
            Some(self.factory().new_identifier("Promise"))
        } else {
            Some(self.factory().new_void_zero_expression())
        }
    }

    pub(crate) fn serialize_type_node(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_type_node"); 
        let node = match node {
            Some(n) => n,
            None => return Some(self.factory().new_identifier("Object")),
        };

        let node = skip_type_parentheses(node);

        match node.kind {
            SyntaxKind::VoidKeyword | SyntaxKind::UndefinedKeyword | SyntaxKind::NeverKeyword => {
                Some(self.factory().new_void_zero_expression())
            }
            SyntaxKind::FunctionType | SyntaxKind::ConstructorType => {
                Some(self.factory().new_identifier("Function"))
            }
            SyntaxKind::ArrayType | SyntaxKind::TupleType => {
                Some(self.factory().new_identifier("Array"))
            }
            SyntaxKind::TypePredicate => {
                if node.as_type_predicate_node().asserts_modifier.is_some() {
                    Some(self.factory().new_void_zero_expression())
                } else {
                    Some(self.factory().new_identifier("Boolean"))
                }
            }
            SyntaxKind::BooleanKeyword => Some(self.factory().new_identifier("Boolean")),
            SyntaxKind::TemplateLiteralType | SyntaxKind::StringKeyword => {
                Some(self.factory().new_identifier("String"))
            }
            SyntaxKind::ObjectKeyword => Some(self.factory().new_identifier("Object")),
            SyntaxKind::LiteralType => self
                .serialize_literal_of_literal_type_node(&node.as_literal_type_node().literal),
            SyntaxKind::NumberKeyword => Some(self.factory().new_identifier("Number")),
            SyntaxKind::BigIntKeyword => self.serialize_big_int_constructor(),
            SyntaxKind::SymbolKeyword => Some(self.factory().new_identifier("Symbol")),
            SyntaxKind::TypeReference => self.serialize_type_reference_node(&node),
            SyntaxKind::IntersectionType => {
                let types = node.as_intersection_type_node().types.nodes.clone();
                self.serialize_union_or_intersection_constituents(&types, true)
            }
            SyntaxKind::UnionType => {
                let types = node.as_union_type_node().types.nodes.clone();
                self.serialize_union_or_intersection_constituents(&types, false)
            }
            SyntaxKind::ConditionalType => {
                let old_state = self.c.serializing_conditional_type_branch;
                self.c.serializing_conditional_type_branch = true;
                let conditional = node.as_conditional_type_node();
                let constituents = vec![
                    conditional.true_type.clone(),
                    conditional.false_type.clone(),
                ];
                let result =
                    self.serialize_union_or_intersection_constituents(&constituents, false);
                self.c.serializing_conditional_type_branch = old_state;
                result
            }
            SyntaxKind::TypeOperator => {
                if node.as_type_operator_node().operator == SyntaxKind::ReadonlyKeyword {
                    return self.serialize_type_node(node.type_());
                }
                Some(self.factory().new_identifier("Object"))
            }
            SyntaxKind::TypeQuery
            | SyntaxKind::IndexedAccessType
            | SyntaxKind::MappedType
            | SyntaxKind::TypeLiteral
            | SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::ThisType
            | SyntaxKind::ImportType
            | SyntaxKind::JSDocAllType
            | SyntaxKind::JSDocVariadicType => Some(self.factory().new_identifier("Object")),
            SyntaxKind::JSDocNullableType
            | SyntaxKind::JSDocNonNullableType
            | SyntaxKind::JSDocOptionalType => {
                self.serialize_type_node(node.type_())
            }
            _ => fail(&format!(
                "Unexpected node.\nNode {} was unexpected.",
                arc_node_kind_string(&node)
            )),
        }
    }
}

pub fn get_parameters_of_decorated_declaration(
    node: &Arc<Node>,
    container: Option<&Arc<Node>>,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_parameters_of_decorated_declaration"); 
    if let Some(container) = container {
        if node.kind == SyntaxKind::GetAccessor {
            let acc = get_all_accessor_declarations(members(container), node);
            if let Some(set_accessor) = acc.set_accessor {
                return set_accessor
                    .parameters()
                    .map(|list| list.nodes.clone())
                    .unwrap_or_default();
            }
        }
    }
    parameter_list(node)
        .map(|list| list.nodes.clone())
        .unwrap_or_default()
}
