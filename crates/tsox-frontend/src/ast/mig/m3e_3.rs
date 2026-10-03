use std::sync::Arc;

use crate::ast::node_node::Node;
use crate::ast::node_node_list::NodeList;
use crate::ast::symbol_internal_names::INTERNAL_SYMBOL_NAME_PREFIX;
use crate::ast::syntax_kind_generated::SyntaxKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperatorPrecedence {
    Comma = 0,
    Spread = 1,
    Yield = 2,
    Assignment = 3,
    Conditional = 4,
    LogicalOr = 5,
    LogicalAnd = 6,
    BitwiseOr = 7,
    BitwiseXor = 8,
    BitwiseAnd = 9,
    Equality = 10,
    Relational = 11,
    Shift = 12,
    Additive = 13,
    Multiplicative = 14,
    Exponentiation = 15,
    Unary = 16,
    Update = 17,
    LeftHandSide = 18,
    OptionalChain = 19,
    Member = 20,
    Primary = 21,
    Parentheses = 22,
    Invalid = -1,
}

pub const OPERATOR_PRECEDENCE_LOWEST: OperatorPrecedence = OperatorPrecedence::Comma;
pub const OPERATOR_PRECEDENCE_HIGHEST: OperatorPrecedence = OperatorPrecedence::Parentheses;
pub const OPERATOR_PRECEDENCE_DISALLOW_COMMA: OperatorPrecedence = OperatorPrecedence::Yield;
pub const OPERATOR_PRECEDENCE_COALESCE: OperatorPrecedence = OperatorPrecedence::LogicalOr;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OperatorPrecedenceFlags(pub i32);

impl OperatorPrecedenceFlags {
    pub const NONE: OperatorPrecedenceFlags = OperatorPrecedenceFlags(0);
    pub const NEW_WITHOUT_ARGUMENTS: OperatorPrecedenceFlags = OperatorPrecedenceFlags(1 << 0);
    pub const OPTIONAL_CHAIN: OperatorPrecedenceFlags = OperatorPrecedenceFlags(1 << 1);

    pub fn contains(self, other: OperatorPrecedenceFlags) -> bool { ::tsox_core::fntrace::enter("contains"); 
        self.0 & other.0 == other.0
    }
}

pub fn get_operator(expression: &Arc<Node>) -> SyntaxKind { ::tsox_core::fntrace::enter("get_operator"); 
    match &expression.data {
        crate::ast::node_data_generated::NodeData::BinaryExpression(d) => d.operator_token.kind,
        crate::ast::node_data_generated::NodeData::PrefixUnaryExpression(d) => d.operator,
        crate::ast::node_data_generated::NodeData::PostfixUnaryExpression(d) => d.operator,
        _ => expression.kind,
    }
}

pub fn get_expression_precedence(expression: &Arc<Node>) -> OperatorPrecedence { ::tsox_core::fntrace::enter("get_expression_precedence"); 
    let operator = get_operator(expression);
    let mut flags = OperatorPrecedenceFlags::NONE;
    if expression.kind == SyntaxKind::NewExpression
        && crate::ast::mig::x1a::argument_list(expression).is_none()
    {
        flags = OperatorPrecedenceFlags::NEW_WITHOUT_ARGUMENTS;
    } else if crate::ast::utilities_expressions::is_optional_chain(expression) {
        flags = OperatorPrecedenceFlags::OPTIONAL_CHAIN;
    }
    get_operator_precedence(expression.kind, operator, flags)
}

pub fn get_operator_precedence(
    node_kind: SyntaxKind,
    operator_kind: SyntaxKind,
    flags: OperatorPrecedenceFlags,
) -> OperatorPrecedence { ::tsox_core::fntrace::enter("get_operator_precedence"); 
    match node_kind {
        SyntaxKind::SpreadElement => OperatorPrecedence::Spread,
        SyntaxKind::YieldExpression => OperatorPrecedence::Yield,
        SyntaxKind::ArrowFunction => OperatorPrecedence::Assignment,
        SyntaxKind::ConditionalExpression => OperatorPrecedence::Conditional,
        SyntaxKind::BinaryExpression => match operator_kind {
            SyntaxKind::CommaToken => OperatorPrecedence::Comma,
            SyntaxKind::EqualsToken
            | SyntaxKind::PlusEqualsToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::CaretEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken => OperatorPrecedence::Assignment,
            _ => get_binary_operator_precedence(operator_kind),
        },
        SyntaxKind::TypeAssertionExpression
        | SyntaxKind::NonNullExpression
        | SyntaxKind::PrefixUnaryExpression
        | SyntaxKind::TypeOfExpression
        | SyntaxKind::VoidExpression
        | SyntaxKind::DeleteExpression
        | SyntaxKind::AwaitExpression => OperatorPrecedence::Unary,
        SyntaxKind::PostfixUnaryExpression => OperatorPrecedence::Update,
        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
            if flags.contains(OperatorPrecedenceFlags::OPTIONAL_CHAIN) {
                OperatorPrecedence::OptionalChain
            } else {
                OperatorPrecedence::Member
            }
        }
        SyntaxKind::CallExpression => {
            if flags.contains(OperatorPrecedenceFlags::OPTIONAL_CHAIN) {
                OperatorPrecedence::OptionalChain
            } else {
                OperatorPrecedence::Member
            }
        }
        SyntaxKind::NewExpression => {
            if flags.contains(OperatorPrecedenceFlags::NEW_WITHOUT_ARGUMENTS) {
                OperatorPrecedence::LeftHandSide
            } else {
                OperatorPrecedence::Member
            }
        }
        SyntaxKind::TaggedTemplateExpression
        | SyntaxKind::MetaProperty
        | SyntaxKind::ExpressionWithTypeArguments => OperatorPrecedence::Member,
        SyntaxKind::AsExpression | SyntaxKind::SatisfiesExpression => OperatorPrecedence::Relational,
        SyntaxKind::ThisKeyword
        | SyntaxKind::SuperKeyword
        | SyntaxKind::ImportKeyword
        | SyntaxKind::Identifier
        | SyntaxKind::PrivateIdentifier
        | SyntaxKind::NullKeyword
        | SyntaxKind::TrueKeyword
        | SyntaxKind::FalseKeyword
        | SyntaxKind::NumericLiteral
        | SyntaxKind::BigIntLiteral
        | SyntaxKind::StringLiteral
        | SyntaxKind::ArrayLiteralExpression
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ClassExpression
        | SyntaxKind::RegularExpressionLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::TemplateExpression
        | SyntaxKind::OmittedExpression
        | SyntaxKind::JsxElement
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxFragment
        | SyntaxKind::MissingDeclaration => OperatorPrecedence::Primary,
        SyntaxKind::ParenthesizedExpression => OperatorPrecedence::Parentheses,
        _ => OperatorPrecedence::Invalid,
    }
}

pub fn get_binary_operator_precedence(operator_kind: SyntaxKind) -> OperatorPrecedence { ::tsox_core::fntrace::enter("get_binary_operator_precedence"); 
    match operator_kind {
        SyntaxKind::QuestionQuestionToken => OPERATOR_PRECEDENCE_COALESCE,
        SyntaxKind::BarBarToken => OperatorPrecedence::LogicalOr,
        SyntaxKind::AmpersandAmpersandToken => OperatorPrecedence::LogicalAnd,
        SyntaxKind::BarToken => OperatorPrecedence::BitwiseOr,
        SyntaxKind::CaretToken => OperatorPrecedence::BitwiseXor,
        SyntaxKind::AmpersandToken => OperatorPrecedence::BitwiseAnd,
        SyntaxKind::EqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken => OperatorPrecedence::Equality,
        SyntaxKind::LessThanToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::AsKeyword
        | SyntaxKind::SatisfiesKeyword => OperatorPrecedence::Relational,
        SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken => OperatorPrecedence::Shift,
        SyntaxKind::PlusToken | SyntaxKind::MinusToken => OperatorPrecedence::Additive,
        SyntaxKind::AsteriskToken | SyntaxKind::SlashToken | SyntaxKind::PercentToken => {
            OperatorPrecedence::Multiplicative
        }
        SyntaxKind::AsteriskAsteriskToken => OperatorPrecedence::Exponentiation,
        _ => OperatorPrecedence::Invalid,
    }
}

pub fn get_leftmost_expression(
    node: &Arc<Node>,
    stop_at_call_expressions: bool,
) -> Arc<Node> { ::tsox_core::fntrace::enter("get_leftmost_expression"); 
    let mut node = node.clone();
    loop {
        let next = match &node.data {
            crate::ast::node_data_generated::NodeData::PostfixUnaryExpression(d) => {
                Some(d.operand.clone())
            }
            crate::ast::node_data_generated::NodeData::BinaryExpression(d) => Some(d.left.clone()),
            crate::ast::node_data_generated::NodeData::ConditionalExpression(d) => {
                Some(d.condition.clone())
            }
            crate::ast::node_data_generated::NodeData::TaggedTemplateExpression(d) => {
                Some(d.tag.clone())
            }
            crate::ast::node_data_generated::NodeData::CallExpression(_) if stop_at_call_expressions => {
                return node
            }
            crate::ast::node_data_generated::NodeData::CallExpression(_)
            | crate::ast::node_data_generated::NodeData::AsExpression(_)
            | crate::ast::node_data_generated::NodeData::ElementAccessExpression(_)
            | crate::ast::node_data_generated::NodeData::PropertyAccessExpression(_)
            | crate::ast::node_data_generated::NodeData::NonNullExpression(_)
            | crate::ast::node_data_generated::NodeData::PartiallyEmittedExpression(_)
            | crate::ast::node_data_generated::NodeData::SatisfiesExpression(_) => {
                node.expression().cloned()
            }
            _ => None,
        };
        match next {
            Some(next) => node = next,
            None => return node,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypePrecedence {
    Conditional = 0,
    Jsdoc = 1,
    Function = 2,
    Union = 3,
    Intersection = 4,
    TypeOperator = 5,
    Postfix = 6,
    NonArray = 7,
}

pub const TYPE_PRECEDENCE_LOWEST: TypePrecedence = TypePrecedence::Conditional;
pub const TYPE_PRECEDENCE_HIGHEST: TypePrecedence = TypePrecedence::NonArray;

pub fn get_type_node_precedence(n: &Arc<Node>) -> TypePrecedence { ::tsox_core::fntrace::enter("get_type_node_precedence"); 
    match n.kind {
        SyntaxKind::ConditionalType => TypePrecedence::Conditional,
        SyntaxKind::JSDocOptionalType | SyntaxKind::JSDocVariadicType => TypePrecedence::Jsdoc,
        SyntaxKind::FunctionType | SyntaxKind::ConstructorType => TypePrecedence::Function,
        SyntaxKind::UnionType => TypePrecedence::Union,
        SyntaxKind::IntersectionType => TypePrecedence::Intersection,
        SyntaxKind::TypeOperator => TypePrecedence::TypeOperator,
        SyntaxKind::InferType => {
            let has_constraint = match &n.data {
                crate::ast::node_data_generated::NodeData::InferTypeNode(d) => match &d.type_parameter.data {
                    crate::ast::node_data_generated::NodeData::TypeParameterDeclaration(tp) => {
                        tp.constraint.is_some()
                    }
                    _ => false,
                },
                _ => false,
            };
            if has_constraint {
                TypePrecedence::Function
            } else {
                TypePrecedence::TypeOperator
            }
        }
        SyntaxKind::IndexedAccessType | SyntaxKind::ArrayType | SyntaxKind::OptionalType => {
            TypePrecedence::Postfix
        }
        SyntaxKind::TypeQuery => TypePrecedence::TypeOperator,
        SyntaxKind::AnyKeyword
        | SyntaxKind::UnknownKeyword
        | SyntaxKind::StringKeyword
        | SyntaxKind::NumberKeyword
        | SyntaxKind::BigIntKeyword
        | SyntaxKind::SymbolKeyword
        | SyntaxKind::BooleanKeyword
        | SyntaxKind::UndefinedKeyword
        | SyntaxKind::NeverKeyword
        | SyntaxKind::ObjectKeyword
        | SyntaxKind::IntrinsicKeyword
        | SyntaxKind::VoidKeyword
        | SyntaxKind::JSDocAllType
        | SyntaxKind::JSDocNullableType
        | SyntaxKind::JSDocNonNullableType
        | SyntaxKind::LiteralType
        | SyntaxKind::TypePredicate
        | SyntaxKind::TypeReference
        | SyntaxKind::TypeLiteral
        | SyntaxKind::TupleType
        | SyntaxKind::RestType
        | SyntaxKind::ParenthesizedType
        | SyntaxKind::ThisType
        | SyntaxKind::MappedType
        | SyntaxKind::NamedTupleMember
        | SyntaxKind::TemplateLiteralType
        | SyntaxKind::ImportType
        | SyntaxKind::PropertyAccessExpression
        | SyntaxKind::ExpressionWithTypeArguments => TypePrecedence::NonArray,
        _ => panic!("unhandled TypeNode: {:?}", n.kind),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubtreeFacts(pub u32);

impl SubtreeFacts {
    pub const CONTAINS_TYPE_SCRIPT: SubtreeFacts = SubtreeFacts(1 << 0);
    pub const CONTAINS_JSX: SubtreeFacts = SubtreeFacts(1 << 1);
    pub const CONTAINS_ES_DECORATORS: SubtreeFacts = SubtreeFacts(1 << 2);
    pub const CONTAINS_USING: SubtreeFacts = SubtreeFacts(1 << 3);
    pub const CONTAINS_CLASS_STATIC_BLOCKS: SubtreeFacts = SubtreeFacts(1 << 4);
    pub const CONTAINS_ES_CLASS_FIELDS: SubtreeFacts = SubtreeFacts(1 << 5);
    pub const CONTAINS_LOGICAL_ASSIGNMENTS: SubtreeFacts = SubtreeFacts(1 << 6);
    pub const CONTAINS_NULLISH_COALESCING: SubtreeFacts = SubtreeFacts(1 << 7);
    pub const CONTAINS_OPTIONAL_CHAINING: SubtreeFacts = SubtreeFacts(1 << 8);
    pub const CONTAINS_MISSING_CATCH_CLAUSE_VARIABLE: SubtreeFacts = SubtreeFacts(1 << 9);
    pub const CONTAINS_ES_OBJECT_REST_OR_SPREAD: SubtreeFacts = SubtreeFacts(1 << 10);
    pub const CONTAINS_FOR_AWAIT_OR_ASYNC_GENERATOR: SubtreeFacts = SubtreeFacts(1 << 11);
    pub const CONTAINS_ANY_AWAIT: SubtreeFacts = SubtreeFacts(1 << 12);
    pub const CONTAINS_EXPONENTIATION_OPERATOR: SubtreeFacts = SubtreeFacts(1 << 13);
    pub const CONTAINS_LEXICAL_THIS: SubtreeFacts = SubtreeFacts(1 << 14);
    pub const CONTAINS_LEXICAL_SUPER: SubtreeFacts = SubtreeFacts(1 << 15);
    pub const CONTAINS_REST_OR_SPREAD: SubtreeFacts = SubtreeFacts(1 << 16);
    pub const CONTAINS_OBJECT_REST_OR_SPREAD: SubtreeFacts = SubtreeFacts(1 << 17);
    pub const CONTAINS_AWAIT: SubtreeFacts = SubtreeFacts(1 << 18);
    pub const CONTAINS_DYNAMIC_IMPORT: SubtreeFacts = SubtreeFacts(1 << 19);
    pub const CONTAINS_CLASS_FIELDS: SubtreeFacts = SubtreeFacts(1 << 20);
    pub const CONTAINS_DECORATORS: SubtreeFacts = SubtreeFacts(1 << 21);
    pub const CONTAINS_IDENTIFIER: SubtreeFacts = SubtreeFacts(1 << 22);
    pub const CONTAINS_PRIVATE_IDENTIFIER_IN_EXPRESSION: SubtreeFacts = SubtreeFacts(1 << 23);
    pub const CONTAINS_INVALID_TEMPLATE_ESCAPE: SubtreeFacts = SubtreeFacts(1 << 24);
    pub const COMPUTED: SubtreeFacts = SubtreeFacts(1 << 25);
    pub const NONE: SubtreeFacts = SubtreeFacts(0);

    pub fn contains(self, other: SubtreeFacts) -> bool { ::tsox_core::fntrace::enter("contains"); 
        self.0 & other.0 == other.0
    }

    pub fn union(self, other: SubtreeFacts) -> SubtreeFacts { ::tsox_core::fntrace::enter("union"); 
        SubtreeFacts(self.0 | other.0)
    }

    pub fn without(self, other: SubtreeFacts) -> SubtreeFacts { ::tsox_core::fntrace::enter("without"); 
        SubtreeFacts(self.0 & !other.0)
    }
}

pub fn propagate_eraseable_syntax_list_subtree_facts(
    children: Option<&NodeList>,
) -> SubtreeFacts { ::tsox_core::fntrace::enter("propagate_eraseable_syntax_list_subtree_facts"); 
    match children {
        Some(_) => SubtreeFacts::CONTAINS_TYPE_SCRIPT,
        None => SubtreeFacts::NONE,
    }
}

pub fn propagate_eraseable_syntax_subtree_facts(child: Option<&Arc<Node>>) -> SubtreeFacts { ::tsox_core::fntrace::enter("propagate_eraseable_syntax_subtree_facts"); 
    match child {
        Some(_) => SubtreeFacts::CONTAINS_TYPE_SCRIPT,
        None => SubtreeFacts::NONE,
    }
}

pub fn propagate_object_binding_element_subtree_facts(child: &Arc<Node>) -> SubtreeFacts { ::tsox_core::fntrace::enter("propagate_object_binding_element_subtree_facts"); 
    let mut facts = propagate_subtree_facts(Some(child));
    if facts.contains(SubtreeFacts::CONTAINS_REST_OR_SPREAD) {
        facts = facts.without(SubtreeFacts::CONTAINS_REST_OR_SPREAD);
        facts = facts
            .union(SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD)
            .union(SubtreeFacts::CONTAINS_ES_OBJECT_REST_OR_SPREAD);
    }
    facts
}

pub fn propagate_binding_element_subtree_facts(child: &Arc<Node>) -> SubtreeFacts { ::tsox_core::fntrace::enter("propagate_binding_element_subtree_facts"); 
    propagate_subtree_facts(Some(child)).without(SubtreeFacts::CONTAINS_REST_OR_SPREAD)
}

pub fn propagate_subtree_facts(child: Option<&Arc<Node>>) -> SubtreeFacts { ::tsox_core::fntrace::enter("propagate_subtree_facts"); 
    match child {
        None => SubtreeFacts::NONE,
        Some(child) => child.propagate_subtree_facts(),
    }
}

pub fn propagate_node_list_subtree_facts(
    children: Option<&NodeList>,
    propagate: impl Fn(&Arc<Node>) -> SubtreeFacts,
) -> SubtreeFacts { ::tsox_core::fntrace::enter("propagate_node_list_subtree_facts"); 
    let Some(children) = children else {
        return SubtreeFacts::NONE;
    };
    let mut facts = SubtreeFacts::NONE;
    for child in &children.nodes {
        facts = facts.union(propagate(child));
    }
    facts
}

pub fn propagate_modifier_list_subtree_facts(
    children: Option<&crate::ast::node_node_list::ModifierList>,
) -> SubtreeFacts { ::tsox_core::fntrace::enter("propagate_modifier_list_subtree_facts"); 
    let Some(children) = children else {
        return SubtreeFacts::NONE;
    };
    propagate_node_list_subtree_facts(Some(&children.list), |child| {
        propagate_subtree_facts(Some(child))
    })
}

pub fn symbol_name(symbol: &crate::ast::symbol::Symbol) -> String { ::tsox_core::fntrace::enter("symbol_name"); 
    if let Some(value_declaration) = &symbol.value_declaration {
        if crate::ast::mig::m3g_2::is_private_identifier_class_element_declaration(
            value_declaration,
        ) {
            return value_declaration
                .name()
                .map(|n| n.text().to_string())
                .unwrap_or_default();
        }
    }
    symbol.name.clone()
}

pub fn escape_all_internal_symbol_names(name: &str) -> String { ::tsox_core::fntrace::enter("escape_all_internal_symbol_names"); 
    name.replace(INTERNAL_SYMBOL_NAME_PREFIX, "__")
}

pub fn escape_symbol_name(name: &str) -> String { ::tsox_core::fntrace::enter("escape_symbol_name"); 
    if let Some(rest) = name.strip_prefix(INTERNAL_SYMBOL_NAME_PREFIX) {
        return format!("__{}", rest);
    }
    let bytes = name.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'_' && bytes[1] == b'_' {
        return format!("_{}", name);
    }
    name.to_string()
}
