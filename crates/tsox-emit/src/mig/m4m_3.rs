#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_checker::checker::mig::m2c_5::TypeReferenceSerializationKind;
use tsox_core::core::compiler_options_kinds::ScriptTarget;
use tsox_core::core::text::TextRange;
use tsox_core::debug::fail;
use tsox_frontend::ast::deep_clone_node;
use tsox_frontend::ast::mig::m3g_3::skip_type_parentheses;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::{is_identifier, is_literal_type_node, NodeFlags, NodeList};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::format::mig::m4o::NodeFactory;

use crate::mig::m4m::r33k4_defs::arc_node_kind_string;
use crate::mig::m4m::MetadataSerializer;
use crate::mig::m4m::r36k5_defs::NodeDataExt;

impl MetadataSerializer {
    pub fn serialize_union_or_intersection_constituents(
        &mut self,
        types: &[Arc<Node>],
        is_intersection: bool,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_union_or_intersection_constituents"); 
        let mut serialized_type: Option<Arc<Node>> = None;
        for type_node in types {
            let type_node = skip_type_parentheses(type_node);
            if type_node.kind == SyntaxKind::NeverKeyword {
                if is_intersection {
                    return Some(self.factory().new_void_zero_expression());
                }
                continue;
            }

            if type_node.kind == SyntaxKind::UnknownKeyword {
                if !is_intersection {
                    return Some(self.factory().new_identifier("Object"));
                }
                continue;
            }

            if type_node.kind == SyntaxKind::AnyKeyword {
                return Some(self.factory().new_identifier("Object"));
            }

            if !self.strict_null_checks
                && ((is_literal_type_node(&type_node)
                    && type_node.as_literal_type_node().literal.kind == SyntaxKind::NullKeyword)
                    || type_node.kind == SyntaxKind::UndefinedKeyword)
            {
                continue;
            }

            let serialized_constituent = self.serialize_type_node(Some(&type_node)).unwrap();
            if is_identifier(&serialized_constituent)
                && serialized_constituent.text() == "Object"
            {
                return Some(serialized_constituent);
            }

            if let Some(ref existing) = serialized_type {
                if !self.equate_serialized_type_nodes(existing, &serialized_constituent) {
                    return Some(self.factory().new_identifier("Object"));
                }
            } else {
                serialized_type = Some(serialized_constituent);
            }
        }

        if let Some(result) = serialized_type {
            return Some(result);
        }
        Some(self.factory().new_void_zero_expression())
    }

    pub fn serialize_literal_of_literal_type_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_literal_of_literal_type_node"); 
        match node.kind {
            SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral => {
                Some(self.factory().new_identifier("String"))
            }
            SyntaxKind::PrefixUnaryExpression => {
                let operand = node.as_prefix_unary_expression().operand.clone();
                match operand.kind {
                    SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral => {
                        self.serialize_literal_of_literal_type_node(&operand)
                    }
                    _ => fail(&format!(
                        "Unexpected node.\nNode {} was unexpected.",
                        arc_node_kind_string(&operand)
                    )),
                }
            }
            SyntaxKind::NumericLiteral => Some(self.factory().new_identifier("Number")),
            SyntaxKind::BigIntLiteral => self.serialize_big_int_constructor(),
            SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword => {
                Some(self.factory().new_identifier("Boolean"))
            }
            SyntaxKind::NullKeyword => Some(self.factory().new_void_zero_expression()),
            _ => fail(&format!(
                "Unexpected node.\nNode {} was unexpected.",
                arc_node_kind_string(node)
            )),
        }
    }

    pub fn serialize_type_reference_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_type_reference_node"); 
        let mut serial_scope = self.c.current_name_scope.clone();
        if serial_scope.is_none() {
            serial_scope = self.c.current_lexical_scope.clone();
        }
        let serial_scope = serial_scope.unwrap();
        let type_name = node.as_type_reference_node().type_name.clone();
        let parsed_type_name = self.emit_context().parse_node(&type_name);
        let parsed_scope = self.emit_context().parse_node(&serial_scope);
        let kind = match (parsed_type_name, parsed_scope) {
            (Some(type_name), Some(location)) => self
                .resolver
                .get_type_reference_serialization_kind_unsafe(&type_name, &location),
            _ => TypeReferenceSerializationKind::Unknown,
        };
        match kind {
            TypeReferenceSerializationKind::Unknown => {
                if self.c.serializing_conditional_type_branch {
                    return Some(self.factory().new_identifier("Object"));
                }

                let serialized =
                    self.serialize_entity_name_as_expression_fallback(&type_name).unwrap();
                let name_factory = crate::printer::NodeFactory::new(&self.emit_context);
                let temp =
                    name_factory.generated_name_node(&name_factory.new_temp_variable());
                self.emit_context.add_variable_declaration(&temp);
                let assignment = self.factory().new_binary_expression(
                    None,
                    &temp,
                    None,
                    self.factory().new_token(SyntaxKind::EqualsToken),
                    &serialized,
                );
                Some(self.factory().new_conditional_expression(
                    &self.factory().new_type_check(&assignment, "function"),
                    self.factory().new_token(SyntaxKind::QuestionToken),
                    &temp,
                    self.factory().new_token(SyntaxKind::ColonToken),
                    &self.factory().new_identifier("Object"),
                ))
            }
            TypeReferenceSerializationKind::TypeWithConstructSignatureAndValue => {
                self.serialize_entity_name_as_expression(&type_name)
            }
            TypeReferenceSerializationKind::VoidNullableOrNeverType => {
                Some(self.factory().new_void_zero_expression())
            }
            TypeReferenceSerializationKind::BigIntLikeType => {
                self.serialize_big_int_constructor()
            }
            TypeReferenceSerializationKind::BooleanType => {
                Some(self.factory().new_identifier("Boolean"))
            }
            TypeReferenceSerializationKind::NumberLikeType => {
                Some(self.factory().new_identifier("Number"))
            }
            TypeReferenceSerializationKind::StringLikeType => {
                Some(self.factory().new_identifier("String"))
            }
            TypeReferenceSerializationKind::ArrayLikeType => {
                Some(self.factory().new_identifier("Array"))
            }
            TypeReferenceSerializationKind::ESSymbolType => {
                Some(self.factory().new_identifier("Symbol"))
            }
            TypeReferenceSerializationKind::TypeWithCallSignature => {
                Some(self.factory().new_identifier("Function"))
            }
            TypeReferenceSerializationKind::Promise => {
                Some(self.factory().new_identifier("Promise"))
            }
            TypeReferenceSerializationKind::ObjectType => {
                Some(self.factory().new_identifier("Object"))
            }
        }
    }

    pub fn serialize_big_int_constructor(&mut self) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_big_int_constructor"); 
        if (self.language_version as u32) >= (ScriptTarget::ES2020 as u32) {
            return Some(self.factory().new_identifier("BigInt"));
        }
        Some(self.factory().new_conditional_expression(
            &self
                .factory()
                .new_type_check(&self.factory().new_identifier("BigInt"), "function"),
            self.factory().new_token(SyntaxKind::QuestionToken),
            &self.factory().new_identifier("BigInt"),
            self.factory().new_token(SyntaxKind::ColonToken),
            &self.factory().new_identifier("Object"),
        ))
    }

    pub fn serialize_entity_name_as_expression(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_entity_name_as_expression"); 
        match node.kind {
            SyntaxKind::Identifier => {
                let name = deep_clone_node(node);
                self.emit_context.unset_original(&name);
                if let Some(parent) = self
                    .emit_context
                    .parse_node(self.c.current_lexical_scope.as_ref().unwrap())
                {
                    name.set_parent(&parent);
                }
                Some(name)
            }
            SyntaxKind::QualifiedName => {
                self.serialize_qualified_name_as_expression(node)
            }
            _ => None,
        }
    }

    pub fn serialize_qualified_name_as_expression(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_qualified_name_as_expression"); 
        let qualified = node.as_qualified_name();
        let serialized_left = self.serialize_entity_name_as_expression(&qualified.left).unwrap();
        Some(self.factory().new_property_access_expression(
            &serialized_left,
            None,
            &qualified.right,
            NodeFlags::empty(),
        ))
    }

    pub fn serialize_entity_name_as_expression_fallback(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_entity_name_as_expression_fallback"); 
        if node.kind == SyntaxKind::Identifier {
            let copied = self.serialize_entity_name_as_expression(node).unwrap();
            return self.create_checked_value(&copied, &copied);
        }
        let qualified = node.as_qualified_name();
        if qualified.left.kind == SyntaxKind::Identifier {
            let left = self
                .serialize_entity_name_as_expression(&qualified.left)
                .unwrap();
            let right = self.serialize_entity_name_as_expression(node).unwrap();
            return self.create_checked_value(&left, &right);
        }
        let left =
            self.serialize_entity_name_as_expression_fallback(&qualified.left)
                .unwrap();
        let name_factory = crate::printer::NodeFactory::new(&self.emit_context);
        let temp = name_factory.generated_name_node(&name_factory.new_temp_variable());
        self.emit_context.add_variable_declaration(&temp);
        let inner_left = left.as_binary_expression().left.clone();
        let inner_right = left.as_binary_expression().right.clone();
        Some(self.factory().new_logical_and_expression(
            &self.factory().new_logical_and_expression(
                &inner_left,
                &self.factory().new_strict_inequality_expression(
                    &self.factory().new_binary_expression(
                        None,
                        &temp,
                        None,
                        self.factory().new_token(SyntaxKind::EqualsToken),
                        &inner_right,
                    ),
                    &self.factory().new_void_zero_expression(),
                ),
            ),
            &self.factory().new_property_access_expression(
                &temp,
                None,
                &qualified.right,
                NodeFlags::empty(),
            ),
        ))
    }
}
