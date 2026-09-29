//! 由 tools/gen_format_visit.py 生成，勿手改。
//! 按字段序枚举标量子节点与子列表（保留空列表）。
#![allow(clippy::too_many_lines)]

use std::sync::Arc;

use crate::ast::node::{ModifierList, Node, NodeList};
use crate::ast::node_data_generated::NodeData;

#[derive(Debug, Clone, Copy)]
pub(crate) enum VisitItem<'a> {
    Node(&'a Arc<Node>),
    List(&'a Arc<NodeList>),
    Modifiers(&'a Arc<ModifierList>),
    Slice(&'a [Arc<Node>]),
}

pub(crate) fn visit_items<'a>(
    node: &'a Node,
    f: &mut dyn FnMut(VisitItem<'a>),
) {
    match &node.data {
        NodeData::Identifier(_) => {}
        NodeData::PrivateIdentifier(_) => {}
        NodeData::QualifiedName(data) => {
            f(VisitItem::Node(&data.left));
            f(VisitItem::Node(&data.right));
        }
        NodeData::ComputedPropertyName(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::Decorator(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::IfStatement(data) => {
            f(VisitItem::Node(&data.expression));
            f(VisitItem::Node(&data.then_statement));
            if let Some(x) = &data.else_statement { f(VisitItem::Node(x)); }
        }
        NodeData::DoStatement(data) => {
            f(VisitItem::Node(&data.statement));
            f(VisitItem::Node(&data.expression));
        }
        NodeData::WhileStatement(data) => {
            f(VisitItem::Node(&data.expression));
            f(VisitItem::Node(&data.statement));
        }
        NodeData::ForStatement(data) => {
            if let Some(x) = &data.initializer { f(VisitItem::Node(x)); }
            if let Some(x) = &data.condition { f(VisitItem::Node(x)); }
            if let Some(x) = &data.incrementor { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.statement));
        }
        NodeData::ForInOrOfStatement(data) => {
            if let Some(x) = &data.await_modifier { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.initializer));
            f(VisitItem::Node(&data.expression));
            f(VisitItem::Node(&data.statement));
        }
        NodeData::BreakStatement(data) => {
            if let Some(x) = &data.label { f(VisitItem::Node(x)); }
        }
        NodeData::ContinueStatement(data) => {
            if let Some(x) = &data.label { f(VisitItem::Node(x)); }
        }
        NodeData::ReturnStatement(data) => {
            if let Some(x) = &data.expression { f(VisitItem::Node(x)); }
        }
        NodeData::WithStatement(data) => {
            f(VisitItem::Node(&data.expression));
            f(VisitItem::Node(&data.statement));
        }
        NodeData::SwitchStatement(data) => {
            f(VisitItem::Node(&data.expression));
            f(VisitItem::Node(&data.case_block));
        }
        NodeData::CaseBlock(data) => {
            f(VisitItem::List(&data.clauses));
        }
        NodeData::CaseOrDefaultClause(data) => {
            f(VisitItem::Node(&data.expression));
            f(VisitItem::List(&data.statements));
        }
        NodeData::ThrowStatement(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::TryStatement(data) => {
            f(VisitItem::Node(&data.try_block));
            if let Some(x) = &data.catch_clause { f(VisitItem::Node(x)); }
            if let Some(x) = &data.finally_block { f(VisitItem::Node(x)); }
        }
        NodeData::CatchClause(data) => {
            if let Some(x) = &data.variable_declaration { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.block));
        }
        NodeData::LabeledStatement(data) => {
            f(VisitItem::Node(&data.label));
            f(VisitItem::Node(&data.statement));
        }
        NodeData::ExpressionStatement(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::Block(data) => {
            f(VisitItem::List(&data.statements));
        }
        NodeData::VariableStatement(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.declaration_list));
        }
        NodeData::VariableDeclaration(data) => {
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.exclamation_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.initializer { f(VisitItem::Node(x)); }
        }
        NodeData::VariableDeclarationList(data) => {
            f(VisitItem::List(&data.declarations));
        }
        NodeData::BindingPattern(data) => {
            f(VisitItem::List(&data.elements));
        }
        NodeData::ParameterDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.dot_dot_dot_token { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.question_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.initializer { f(VisitItem::Node(x)); }
        }
        NodeData::BindingElement(data) => {
            if let Some(x) = &data.dot_dot_dot_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.property_name { f(VisitItem::Node(x)); }
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
            if let Some(x) = &data.initializer { f(VisitItem::Node(x)); }
        }
        NodeData::MissingDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
        }
        NodeData::FunctionDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.asterisk_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.full_signature { f(VisitItem::Node(x)); }
            if let Some(x) = &data.body { f(VisitItem::Node(x)); }
        }
        NodeData::ClassDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            if let Some(x) = &data.heritage_clauses { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.members));
        }
        NodeData::ClassExpression(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            if let Some(x) = &data.heritage_clauses { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.members));
        }
        NodeData::HeritageClause(data) => {
            f(VisitItem::List(&data.types));
        }
        NodeData::InterfaceDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            if let Some(x) = &data.heritage_clauses { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.members));
        }
        NodeData::TypeAliasDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::EnumMember(data) => {
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.initializer { f(VisitItem::Node(x)); }
        }
        NodeData::EnumDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            f(VisitItem::List(&data.members));
        }
        NodeData::ModuleBlock(data) => {
            f(VisitItem::List(&data.statements));
        }
        NodeData::ImportDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.import_clause { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.module_specifier));
            if let Some(x) = &data.attributes { f(VisitItem::Node(x)); }
        }
        NodeData::ExternalModuleReference(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::NamespaceImport(data) => {
            f(VisitItem::Node(&data.name));
        }
        NodeData::NamedImports(data) => {
            f(VisitItem::List(&data.elements));
        }
        NodeData::ExportAssignment(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.type_node));
            f(VisitItem::Node(&data.expression));
        }
        NodeData::NamespaceExportDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
        }
        NodeData::NamespaceExport(data) => {
            f(VisitItem::Node(&data.name));
        }
        NodeData::NamedExports(data) => {
            f(VisitItem::List(&data.elements));
        }
        NodeData::ExportSpecifier(data) => {
            if let Some(x) = &data.property_name { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.name));
        }
        NodeData::CallSignatureDeclaration(data) => {
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
        }
        NodeData::ConstructSignatureDeclaration(data) => {
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
        }
        NodeData::ConstructorDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.full_signature { f(VisitItem::Node(x)); }
            if let Some(x) = &data.body { f(VisitItem::Node(x)); }
        }
        NodeData::GetAccessorDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.full_signature { f(VisitItem::Node(x)); }
            if let Some(x) = &data.body { f(VisitItem::Node(x)); }
        }
        NodeData::SetAccessorDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.full_signature { f(VisitItem::Node(x)); }
            if let Some(x) = &data.body { f(VisitItem::Node(x)); }
        }
        NodeData::IndexSignatureDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::List(&data.parameters));
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::MethodSignatureDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.postfix_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
        }
        NodeData::MethodDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.asterisk_token { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.postfix_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.full_signature { f(VisitItem::Node(x)); }
            if let Some(x) = &data.body { f(VisitItem::Node(x)); }
        }
        NodeData::PropertySignatureDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.postfix_token { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.type_node));
            f(VisitItem::Node(&data.initializer));
        }
        NodeData::PropertyDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.postfix_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.initializer { f(VisitItem::Node(x)); }
        }
        NodeData::ClassStaticBlockDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.body));
        }
        NodeData::StringLiteral(_) => {}
        NodeData::NumericLiteral(_) => {}
        NodeData::BigIntLiteral(_) => {}
        NodeData::RegularExpressionLiteral(_) => {}
        NodeData::NoSubstitutionTemplateLiteral(_) => {}
        NodeData::BinaryExpression(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.left));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.operator_token));
            f(VisitItem::Node(&data.right));
        }
        NodeData::PrefixUnaryExpression(data) => {
            f(VisitItem::Node(&data.operand));
        }
        NodeData::PostfixUnaryExpression(data) => {
            f(VisitItem::Node(&data.operand));
        }
        NodeData::YieldExpression(data) => {
            if let Some(x) = &data.asterisk_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.expression { f(VisitItem::Node(x)); }
        }
        NodeData::ArrowFunction(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.full_signature { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.equals_greater_than_token));
            f(VisitItem::Node(&data.body));
        }
        NodeData::FunctionExpression(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.asterisk_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.full_signature { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.body));
        }
        NodeData::AsExpression(data) => {
            f(VisitItem::Node(&data.expression));
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::SatisfiesExpression(data) => {
            f(VisitItem::Node(&data.expression));
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::ConditionalExpression(data) => {
            f(VisitItem::Node(&data.condition));
            f(VisitItem::Node(&data.question_token));
            f(VisitItem::Node(&data.when_true));
            f(VisitItem::Node(&data.colon_token));
            f(VisitItem::Node(&data.when_false));
        }
        NodeData::PropertyAccessExpression(data) => {
            f(VisitItem::Node(&data.expression));
            if let Some(x) = &data.question_dot_token { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.name));
        }
        NodeData::ElementAccessExpression(data) => {
            f(VisitItem::Node(&data.expression));
            if let Some(x) = &data.question_dot_token { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.argument_expression));
        }
        NodeData::CallExpression(data) => {
            f(VisitItem::Node(&data.expression));
            if let Some(x) = &data.question_dot_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_arguments { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.arguments));
        }
        NodeData::NewExpression(data) => {
            f(VisitItem::Node(&data.expression));
            if let Some(x) = &data.type_arguments { f(VisitItem::List(x)); }
            if let Some(x) = &data.arguments { f(VisitItem::List(x)); }
        }
        NodeData::MetaProperty(data) => {
            f(VisitItem::Node(&data.name));
        }
        NodeData::NonNullExpression(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::SpreadElement(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::TemplateExpression(data) => {
            f(VisitItem::Node(&data.head));
            f(VisitItem::List(&data.template_spans));
        }
        NodeData::TemplateSpan(data) => {
            f(VisitItem::Node(&data.expression));
            f(VisitItem::Node(&data.literal));
        }
        NodeData::TaggedTemplateExpression(data) => {
            f(VisitItem::Node(&data.tag));
            if let Some(x) = &data.question_dot_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_arguments { f(VisitItem::List(x)); }
            f(VisitItem::Node(&data.template));
        }
        NodeData::ParenthesizedExpression(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::ArrayLiteralExpression(data) => {
            f(VisitItem::List(&data.elements));
        }
        NodeData::ObjectLiteralExpression(data) => {
            f(VisitItem::List(&data.properties));
        }
        NodeData::SpreadAssignment(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::PropertyAssignment(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.postfix_token { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.type_node));
            f(VisitItem::Node(&data.initializer));
        }
        NodeData::ShorthandPropertyAssignment(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.postfix_token { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.type_node));
            if let Some(x) = &data.equals_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.object_assignment_initializer { f(VisitItem::Node(x)); }
        }
        NodeData::DeleteExpression(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::TypeOfExpression(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::VoidExpression(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::AwaitExpression(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::TypeAssertion(data) => {
            f(VisitItem::Node(&data.type_node));
            f(VisitItem::Node(&data.expression));
        }
        NodeData::UnionTypeNode(data) => {
            f(VisitItem::List(&data.types));
        }
        NodeData::IntersectionTypeNode(data) => {
            f(VisitItem::List(&data.types));
        }
        NodeData::ConditionalTypeNode(data) => {
            f(VisitItem::Node(&data.check_type));
            f(VisitItem::Node(&data.extends_type));
            f(VisitItem::Node(&data.true_type));
            f(VisitItem::Node(&data.false_type));
        }
        NodeData::TypeOperatorNode(data) => {
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::InferTypeNode(data) => {
            f(VisitItem::Node(&data.type_parameter));
        }
        NodeData::ArrayTypeNode(data) => {
            f(VisitItem::Node(&data.element_type));
        }
        NodeData::IndexedAccessTypeNode(data) => {
            f(VisitItem::Node(&data.object_type));
            f(VisitItem::Node(&data.index_type));
        }
        NodeData::TypeReferenceNode(data) => {
            f(VisitItem::Node(&data.type_name));
            if let Some(x) = &data.type_arguments { f(VisitItem::List(x)); }
        }
        NodeData::ExpressionWithTypeArguments(data) => {
            f(VisitItem::Node(&data.expression));
            if let Some(x) = &data.type_arguments { f(VisitItem::List(x)); }
        }
        NodeData::LiteralTypeNode(data) => {
            f(VisitItem::Node(&data.literal));
        }
        NodeData::TypePredicateNode(data) => {
            if let Some(x) = &data.asserts_modifier { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.parameter_name));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
        }
        NodeData::ImportAttribute(data) => {
            f(VisitItem::Node(&data.name));
            f(VisitItem::Node(&data.value));
        }
        NodeData::ImportAttributes(data) => {
            f(VisitItem::List(&data.attributes));
        }
        NodeData::TypeQueryNode(data) => {
            f(VisitItem::Node(&data.expr_name));
            if let Some(x) = &data.type_arguments { f(VisitItem::List(x)); }
        }
        NodeData::MappedTypeNode(data) => {
            if let Some(x) = &data.readonly_token { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.type_parameter));
            if let Some(x) = &data.name_type { f(VisitItem::Node(x)); }
            if let Some(x) = &data.question_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
            if let Some(x) = &data.members { f(VisitItem::List(x)); }
        }
        NodeData::TypeLiteralNode(data) => {
            f(VisitItem::List(&data.members));
        }
        NodeData::TupleTypeNode(data) => {
            f(VisitItem::List(&data.elements));
        }
        NodeData::NamedTupleMember(data) => {
            if let Some(x) = &data.dot_dot_dot_token { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.question_token { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::OptionalTypeNode(data) => {
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::RestTypeNode(data) => {
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::ParenthesizedTypeNode(data) => {
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::FunctionTypeNode(data) => {
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
        }
        NodeData::ConstructorTypeNode(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
        }
        NodeData::TemplateHead(_) => {}
        NodeData::TemplateMiddle(_) => {}
        NodeData::TemplateTail(_) => {}
        NodeData::TemplateLiteralTypeNode(data) => {
            f(VisitItem::Node(&data.head));
            f(VisitItem::List(&data.template_spans));
        }
        NodeData::TemplateLiteralTypeSpan(data) => {
            f(VisitItem::Node(&data.type_node));
            f(VisitItem::Node(&data.literal));
        }
        NodeData::SyntheticExpression(data) => {
            if let Some(x) = &data.tuple_name_source { f(VisitItem::Node(x)); }
        }
        NodeData::PartiallyEmittedExpression(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::JsxElement(data) => {
            f(VisitItem::Node(&data.opening_element));
            f(VisitItem::List(&data.children));
            f(VisitItem::Node(&data.closing_element));
        }
        NodeData::JsxAttributes(data) => {
            f(VisitItem::List(&data.properties));
        }
        NodeData::JsxNamespacedName(data) => {
            f(VisitItem::Node(&data.namespace));
            f(VisitItem::Node(&data.name));
        }
        NodeData::JsxOpeningElement(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.type_arguments { f(VisitItem::List(x)); }
            f(VisitItem::Node(&data.attributes));
        }
        NodeData::JsxSelfClosingElement(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.type_arguments { f(VisitItem::List(x)); }
            f(VisitItem::Node(&data.attributes));
        }
        NodeData::JsxFragment(data) => {
            f(VisitItem::Node(&data.opening_fragment));
            f(VisitItem::List(&data.children));
            f(VisitItem::Node(&data.closing_fragment));
        }
        NodeData::JsxAttribute(data) => {
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.initializer { f(VisitItem::Node(x)); }
        }
        NodeData::JsxSpreadAttribute(data) => {
            f(VisitItem::Node(&data.expression));
        }
        NodeData::JsxClosingElement(data) => {
            f(VisitItem::Node(&data.tag_name));
        }
        NodeData::JsxExpression(data) => {
            if let Some(x) = &data.dot_dot_dot_token { f(VisitItem::Node(x)); }
            if let Some(x) = &data.expression { f(VisitItem::Node(x)); }
        }
        NodeData::JsxText(_) => {}
        NodeData::SyntaxList(data) => {
            f(VisitItem::Slice(&data.children));
        }
        NodeData::JSDoc(data) => {
            f(VisitItem::List(&data.comment));
            if let Some(x) = &data.tags { f(VisitItem::List(x)); }
        }
        NodeData::JSDocTypeExpression(data) => {
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::JSDocNonNullableType(data) => {
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::JSDocNullableType(data) => {
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::JSDocVariadicType(data) => {
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::JSDocOptionalType(data) => {
            f(VisitItem::Node(&data.type_node));
        }
        NodeData::JSDocTypeTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            f(VisitItem::Node(&data.type_expression));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocUnknownTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocTemplateTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            f(VisitItem::Node(&data.constraint));
            f(VisitItem::List(&data.type_parameters));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocReturnTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.type_expression { f(VisitItem::Node(x)); }
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocPublicTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocPrivateTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocProtectedTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocReadonlyTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocOverrideTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocDeprecatedTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocSeeTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            f(VisitItem::Node(&data.name_expression));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocImplementsTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            f(VisitItem::Node(&data.class_name));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocAugmentsTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            f(VisitItem::Node(&data.class_name));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocSatisfiesTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            f(VisitItem::Node(&data.type_expression));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocThrowsTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.type_expression { f(VisitItem::Node(x)); }
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocThisTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            f(VisitItem::Node(&data.type_expression));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocImportTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.import_clause { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.module_specifier));
            if let Some(x) = &data.attributes { f(VisitItem::Node(x)); }
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocCallbackTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            f(VisitItem::Node(&data.type_expression));
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocOverloadTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            f(VisitItem::Node(&data.type_expression));
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocTypedefTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            if let Some(x) = &data.type_expression { f(VisitItem::Node(x)); }
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        NodeData::JSDocSignature(data) => {
            if let Some(x) = &data.type_parameters { f(VisitItem::List(x)); }
            f(VisitItem::List(&data.parameters));
            if let Some(x) = &data.type_node { f(VisitItem::Node(x)); }
        }
        NodeData::JSDocNameReference(data) => {
            f(VisitItem::Node(&data.name));
        }
        NodeData::SourceFile(data) => {
            f(VisitItem::List(&data.statements));
            f(VisitItem::Node(&data.end_of_file_token));
        }
        NodeData::ModuleDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.body { f(VisitItem::Node(x)); }
        }
        NodeData::ImportEqualsDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            f(VisitItem::Node(&data.module_reference));
        }
        NodeData::ExportDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            if let Some(x) = &data.export_clause { f(VisitItem::Node(x)); }
            if let Some(x) = &data.module_specifier { f(VisitItem::Node(x)); }
            if let Some(x) = &data.attributes { f(VisitItem::Node(x)); }
        }
        NodeData::ImportTypeNode(data) => {
            f(VisitItem::Node(&data.argument));
            if let Some(x) = &data.attributes { f(VisitItem::Node(x)); }
            if let Some(x) = &data.qualifier { f(VisitItem::Node(x)); }
            if let Some(x) = &data.type_arguments { f(VisitItem::List(x)); }
        }
        NodeData::ImportClause(data) => {
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
            if let Some(x) = &data.named_bindings { f(VisitItem::Node(x)); }
        }
        NodeData::ImportSpecifier(data) => {
            if let Some(x) = &data.property_name { f(VisitItem::Node(x)); }
            f(VisitItem::Node(&data.name));
        }
        NodeData::JSDocText(_) => {}
        NodeData::JSDocLink(data) => {
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
        }
        NodeData::JSDocLinkPlain(data) => {
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
        }
        NodeData::JSDocLinkCode(data) => {
            if let Some(x) = &data.name { f(VisitItem::Node(x)); }
        }
        NodeData::TypeParameterDeclaration(data) => {
            if let Some(x) = &data.modifiers { f(VisitItem::Modifiers(x)); }
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.constraint { f(VisitItem::Node(x)); }
            if let Some(x) = &data.expression { f(VisitItem::Node(x)); }
            if let Some(x) = &data.default_type { f(VisitItem::Node(x)); }
        }
        NodeData::SyntheticReferenceExpression(data) => {
            f(VisitItem::Node(&data.expression));
            f(VisitItem::Node(&data.this_arg));
        }
        NodeData::JSDocTypeLiteral(_) => {}
        NodeData::JSDocParameterOrPropertyTag(data) => {
            f(VisitItem::Node(&data.tag_name));
            f(VisitItem::Node(&data.name));
            if let Some(x) = &data.type_expression { f(VisitItem::Node(x)); }
            if let Some(x) = &data.comment { f(VisitItem::List(x)); }
        }
        _ => {}
    }
}
