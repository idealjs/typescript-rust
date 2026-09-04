#![allow(unused_imports)]
use super::SyntaxKind;
use super::node::{ModifierList, Node, NodeList};
use super::node_flags::{ModifierFlags, NodeFlags};
use crate::core::text::TextRange;
use std::sync::Arc;

pub type TokenFlags = u32;

#[derive(Debug)]
pub struct IdentifierData {
    pub text: String,
}

#[derive(Debug)]
pub struct PrivateIdentifierData {
    pub text: String,
}

#[derive(Debug)]
pub struct QualifiedNameData {
    pub left: Arc<Node>,
    pub right: Arc<Node>,
}

#[derive(Debug)]
pub struct ComputedPropertyNameData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct DecoratorData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct IfStatementData {
    pub expression: Arc<Node>,
    pub then_statement: Arc<Node>,
    pub else_statement: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct DoStatementData {
    pub statement: Arc<Node>,
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct WhileStatementData {
    pub expression: Arc<Node>,
    pub statement: Arc<Node>,
}

#[derive(Debug)]
pub struct ForStatementData {
    pub initializer: Option<Arc<Node>>,
    pub condition: Option<Arc<Node>>,
    pub incrementor: Option<Arc<Node>>,
    pub statement: Arc<Node>,
}

#[derive(Debug)]
pub struct ForInOrOfStatementData {
    pub await_modifier: Option<Arc<Node>>,
    pub initializer: Arc<Node>,
    pub expression: Arc<Node>,
    pub statement: Arc<Node>,
}

#[derive(Debug)]
pub struct BreakStatementData {
    pub label: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ContinueStatementData {
    pub label: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ReturnStatementData {
    pub expression: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct WithStatementData {
    pub expression: Arc<Node>,
    pub statement: Arc<Node>,
}

#[derive(Debug)]
pub struct SwitchStatementData {
    pub expression: Arc<Node>,
    pub case_block: Arc<Node>,
}

#[derive(Debug)]
pub struct CaseBlockData {
    pub clauses: Arc<NodeList>,
}

#[derive(Debug)]
pub struct CaseOrDefaultClauseData {
    pub expression: Arc<Node>,
    pub statements: Arc<NodeList>,
}

#[derive(Debug)]
pub struct ThrowStatementData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct TryStatementData {
    pub try_block: Arc<Node>,
    pub catch_clause: Option<Arc<Node>>,
    pub finally_block: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct CatchClauseData {
    pub variable_declaration: Option<Arc<Node>>,
    pub block: Arc<Node>,
}

#[derive(Debug)]
pub struct LabeledStatementData {
    pub label: Arc<Node>,
    pub statement: Arc<Node>,
}

#[derive(Debug)]
pub struct ExpressionStatementData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct BlockData {
    pub statements: Arc<NodeList>,
    pub multi_line: bool,
}

#[derive(Debug)]
pub struct VariableStatementData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub declaration_list: Arc<Node>,
}

#[derive(Debug)]
pub struct VariableDeclarationData {
    pub name: Arc<Node>,
    pub exclamation_token: Option<Arc<Node>>,
    pub type_node: Option<Arc<Node>>,
    pub initializer: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct VariableDeclarationListData {
    pub declarations: Arc<NodeList>,
}

#[derive(Debug)]
pub struct BindingPatternData {
    pub elements: Arc<NodeList>,
}

#[derive(Debug)]
pub struct ParameterDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub dot_dot_dot_token: Option<Arc<Node>>,
    pub name: Arc<Node>,
    pub question_token: Option<Arc<Node>>,
    pub type_node: Option<Arc<Node>>,
    pub initializer: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct BindingElementData {
    pub dot_dot_dot_token: Option<Arc<Node>>,
    pub property_name: Option<Arc<Node>>,
    pub name: Option<Arc<Node>>,
    pub initializer: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct MissingDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
}

#[derive(Debug)]
pub struct FunctionDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub asterisk_token: Option<Arc<Node>>,
    pub name: Option<Arc<Node>>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
    pub full_signature: Option<Arc<Node>>,
    pub body: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ClassDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Option<Arc<Node>>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub heritage_clauses: Option<Arc<NodeList>>,
    pub members: Arc<NodeList>,
}

#[derive(Debug)]
pub struct ClassExpressionData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Option<Arc<Node>>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub heritage_clauses: Option<Arc<NodeList>>,
    pub members: Arc<NodeList>,
}

#[derive(Debug)]
pub struct HeritageClauseData {
    pub token: SyntaxKind,
    pub types: Arc<NodeList>,
}

#[derive(Debug)]
pub struct InterfaceDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub heritage_clauses: Option<Arc<NodeList>>,
    pub members: Arc<NodeList>,
}

#[derive(Debug)]
pub struct TypeAliasDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct EnumMemberData {
    pub name: Arc<Node>,
    pub initializer: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct EnumDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub members: Arc<NodeList>,
}

#[derive(Debug)]
pub struct ModuleBlockData {
    pub statements: Arc<NodeList>,
}

#[derive(Debug)]
pub struct ImportDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub import_clause: Option<Arc<Node>>,
    pub module_specifier: Arc<Node>,
    pub attributes: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ExternalModuleReferenceData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct NamespaceImportData {
    pub name: Arc<Node>,
}

#[derive(Debug)]
pub struct NamedImportsData {
    pub elements: Arc<NodeList>,
}

#[derive(Debug)]
pub struct ExportAssignmentData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub is_export_equals: bool,
    pub type_node: Arc<Node>,
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct NamespaceExportDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
}

#[derive(Debug)]
pub struct NamespaceExportData {
    pub name: Arc<Node>,
}

#[derive(Debug)]
pub struct NamedExportsData {
    pub elements: Arc<NodeList>,
}

#[derive(Debug)]
pub struct ExportSpecifierData {
    pub is_type_only: bool,
    pub property_name: Option<Arc<Node>>,
    pub name: Arc<Node>,
}

#[derive(Debug)]
pub struct CallSignatureDeclarationData {
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ConstructSignatureDeclarationData {
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ConstructorDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
    pub full_signature: Option<Arc<Node>>,
    pub body: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct GetAccessorDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
    pub full_signature: Option<Arc<Node>>,
    pub body: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct SetAccessorDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
    pub full_signature: Option<Arc<Node>>,
    pub body: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct IndexSignatureDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct MethodSignatureDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub postfix_token: Option<Arc<Node>>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct MethodDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub asterisk_token: Option<Arc<Node>>,
    pub name: Arc<Node>,
    pub postfix_token: Option<Arc<Node>>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
    pub full_signature: Option<Arc<Node>>,
    pub body: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct PropertySignatureDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub postfix_token: Option<Arc<Node>>,
    pub type_node: Arc<Node>,
    pub initializer: Arc<Node>,
}

#[derive(Debug)]
pub struct PropertyDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub postfix_token: Option<Arc<Node>>,
    pub type_node: Option<Arc<Node>>,
    pub initializer: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ClassStaticBlockDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub body: Arc<Node>,
}

#[derive(Debug)]
pub struct StringLiteralData {
    pub text: String,
    pub token_flags: TokenFlags,
}

#[derive(Debug)]
pub struct NumericLiteralData {
    pub text: String,
    pub token_flags: TokenFlags,
}

#[derive(Debug)]
pub struct BigIntLiteralData {
    pub text: String,
    pub token_flags: TokenFlags,
}

#[derive(Debug)]
pub struct RegularExpressionLiteralData {
    pub text: String,
    pub token_flags: TokenFlags,
}

#[derive(Debug)]
pub struct NoSubstitutionTemplateLiteralData {
    pub text: String,
    pub template_flags: TokenFlags,
}

#[derive(Debug)]
pub struct BinaryExpressionData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub left: Arc<Node>,
    pub type_node: Option<Arc<Node>>,
    pub operator_token: Arc<Node>,
    pub right: Arc<Node>,
}

#[derive(Debug)]
pub struct PrefixUnaryExpressionData {
    pub operator: SyntaxKind,
    pub operand: Arc<Node>,
}

#[derive(Debug)]
pub struct PostfixUnaryExpressionData {
    pub operand: Arc<Node>,
    pub operator: SyntaxKind,
}

#[derive(Debug)]
pub struct YieldExpressionData {
    pub asterisk_token: Option<Arc<Node>>,
    pub expression: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ArrowFunctionData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
    pub full_signature: Option<Arc<Node>>,
    pub equals_greater_than_token: Arc<Node>,
    pub body: Arc<Node>,
}

#[derive(Debug)]
pub struct FunctionExpressionData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub asterisk_token: Option<Arc<Node>>,
    pub name: Option<Arc<Node>>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
    pub full_signature: Option<Arc<Node>>,
    pub body: Arc<Node>,
}

#[derive(Debug)]
pub struct AsExpressionData {
    pub expression: Arc<Node>,
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct SatisfiesExpressionData {
    pub expression: Arc<Node>,
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct ConditionalExpressionData {
    pub condition: Arc<Node>,
    pub question_token: Arc<Node>,
    pub when_true: Arc<Node>,
    pub colon_token: Arc<Node>,
    pub when_false: Arc<Node>,
}

#[derive(Debug)]
pub struct PropertyAccessExpressionData {
    pub expression: Arc<Node>,
    pub question_dot_token: Option<Arc<Node>>,
    pub name: Arc<Node>,
}

#[derive(Debug)]
pub struct ElementAccessExpressionData {
    pub expression: Arc<Node>,
    pub question_dot_token: Option<Arc<Node>>,
    pub argument_expression: Arc<Node>,
}

#[derive(Debug)]
pub struct CallExpressionData {
    pub expression: Arc<Node>,
    pub question_dot_token: Option<Arc<Node>>,
    pub type_arguments: Option<Arc<NodeList>>,
    pub arguments: Arc<NodeList>,
}

#[derive(Debug)]
pub struct NewExpressionData {
    pub expression: Arc<Node>,
    pub type_arguments: Option<Arc<NodeList>>,
    pub arguments: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct MetaPropertyData {
    pub keyword_token: SyntaxKind,
    pub name: Arc<Node>,
}

#[derive(Debug)]
pub struct NonNullExpressionData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct SpreadElementData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct TemplateExpressionData {
    pub head: Arc<Node>,
    pub template_spans: Arc<NodeList>,
}

#[derive(Debug)]
pub struct TemplateSpanData {
    pub expression: Arc<Node>,
    pub literal: Arc<Node>,
}

#[derive(Debug)]
pub struct TaggedTemplateExpressionData {
    pub tag: Arc<Node>,
    pub question_dot_token: Arc<Node>,
    pub type_arguments: Option<Arc<NodeList>>,
    pub template: Arc<Node>,
}

#[derive(Debug)]
pub struct ParenthesizedExpressionData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct ArrayLiteralExpressionData {
    pub elements: Arc<NodeList>,
    pub multi_line: bool,
}

#[derive(Debug)]
pub struct ObjectLiteralExpressionData {
    pub properties: Arc<NodeList>,
    pub multi_line: bool,
}

#[derive(Debug)]
pub struct SpreadAssignmentData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct PropertyAssignmentData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub postfix_token: Option<Arc<Node>>,
    pub type_node: Arc<Node>,
    pub initializer: Arc<Node>,
}

#[derive(Debug)]
pub struct ShorthandPropertyAssignmentData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub postfix_token: Option<Arc<Node>>,
    pub type_node: Arc<Node>,
    pub equals_token: Option<Arc<Node>>,
    pub object_assignment_initializer: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct DeleteExpressionData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct TypeOfExpressionData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct VoidExpressionData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct AwaitExpressionData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct TypeAssertionData {
    pub type_node: Arc<Node>,
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct UnionTypeNodeData {
    pub types: Arc<NodeList>,
}

#[derive(Debug)]
pub struct IntersectionTypeNodeData {
    pub types: Arc<NodeList>,
}

#[derive(Debug)]
pub struct ConditionalTypeNodeData {
    pub check_type: Arc<Node>,
    pub extends_type: Arc<Node>,
    pub true_type: Arc<Node>,
    pub false_type: Arc<Node>,
}

#[derive(Debug)]
pub struct TypeOperatorNodeData {
    pub operator: SyntaxKind,
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct InferTypeNodeData {
    pub type_parameter: Arc<Node>,
}

#[derive(Debug)]
pub struct ArrayTypeNodeData {
    pub element_type: Arc<Node>,
}

#[derive(Debug)]
pub struct IndexedAccessTypeNodeData {
    pub object_type: Arc<Node>,
    pub index_type: Arc<Node>,
}

#[derive(Debug)]
pub struct TypeReferenceNodeData {
    pub type_name: Arc<Node>,
    pub type_arguments: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct ExpressionWithTypeArgumentsData {
    pub expression: Arc<Node>,
    pub type_arguments: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct LiteralTypeNodeData {
    pub literal: Arc<Node>,
}

#[derive(Debug)]
pub struct TypePredicateNodeData {
    pub asserts_modifier: Option<Arc<Node>>,
    pub parameter_name: Arc<Node>,
    pub type_node: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ImportAttributeData {
    pub name: Arc<Node>,
    pub value: Arc<Node>,
}

#[derive(Debug)]
pub struct ImportAttributesData {
    pub token: SyntaxKind,
    pub attributes: Arc<NodeList>,
    pub multi_line: bool,
}

#[derive(Debug)]
pub struct TypeQueryNodeData {
    pub expr_name: Arc<Node>,
    pub type_arguments: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct MappedTypeNodeData {
    pub readonly_token: Option<Arc<Node>>,
    pub type_parameter: Arc<Node>,
    pub name_type: Option<Arc<Node>>,
    pub question_token: Option<Arc<Node>>,
    pub type_node: Option<Arc<Node>>,
    pub members: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct TypeLiteralNodeData {
    pub members: Arc<NodeList>,
}

#[derive(Debug)]
pub struct TupleTypeNodeData {
    pub elements: Arc<NodeList>,
}

#[derive(Debug)]
pub struct NamedTupleMemberData {
    pub dot_dot_dot_token: Option<Arc<Node>>,
    pub name: Arc<Node>,
    pub question_token: Option<Arc<Node>>,
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct OptionalTypeNodeData {
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct RestTypeNodeData {
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct ParenthesizedTypeNodeData {
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct FunctionTypeNodeData {
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ConstructorTypeNodeData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct TemplateHeadData {
    pub text: String,
    pub raw_text: String,
    pub template_flags: TokenFlags,
}

#[derive(Debug)]
pub struct TemplateMiddleData {
    pub text: String,
    pub raw_text: String,
    pub template_flags: TokenFlags,
}

#[derive(Debug)]
pub struct TemplateTailData {
    pub text: String,
    pub raw_text: String,
    pub template_flags: TokenFlags,
}

#[derive(Debug)]
pub struct TemplateLiteralTypeNodeData {
    pub head: Arc<Node>,
    pub template_spans: Arc<NodeList>,
}

#[derive(Debug)]
pub struct TemplateLiteralTypeSpanData {
    pub type_node: Arc<Node>,
    pub literal: Arc<Node>,
}

#[derive(Debug)]
pub struct SyntheticExpressionData {
    pub type_node: Option<String>,
    pub is_spread: bool,
    pub tuple_name_source: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct PartiallyEmittedExpressionData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct JsxElementData {
    pub opening_element: Arc<Node>,
    pub children: Arc<NodeList>,
    pub closing_element: Arc<Node>,
}

#[derive(Debug)]
pub struct JsxAttributesData {
    pub properties: Arc<NodeList>,
}

#[derive(Debug)]
pub struct JsxNamespacedNameData {
    pub namespace: Arc<Node>,
    pub name: Arc<Node>,
}

#[derive(Debug)]
pub struct JsxOpeningElementData {
    pub tag_name: Arc<Node>,
    pub type_arguments: Option<Arc<NodeList>>,
    pub attributes: Arc<Node>,
}

#[derive(Debug)]
pub struct JsxSelfClosingElementData {
    pub tag_name: Arc<Node>,
    pub type_arguments: Option<Arc<NodeList>>,
    pub attributes: Arc<Node>,
}

#[derive(Debug)]
pub struct JsxFragmentData {
    pub opening_fragment: Arc<Node>,
    pub children: Arc<NodeList>,
    pub closing_fragment: Arc<Node>,
}

#[derive(Debug)]
pub struct JsxAttributeData {
    pub name: Arc<Node>,
    pub initializer: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct JsxSpreadAttributeData {
    pub expression: Arc<Node>,
}

#[derive(Debug)]
pub struct JsxClosingElementData {
    pub tag_name: Arc<Node>,
}

#[derive(Debug)]
pub struct JsxExpressionData {
    pub dot_dot_dot_token: Option<Arc<Node>>,
    pub expression: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct JsxTextData {
    pub text: String,
    pub contains_only_trivia_white_spaces: bool,
}

#[derive(Debug)]
pub struct SyntaxListData {
    pub children: Vec<Arc<Node>>,
}

#[derive(Debug)]
pub struct JSDocData {
    pub comment: Arc<NodeList>,
    pub tags: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocTypeExpressionData {
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct JSDocNonNullableTypeData {
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct JSDocNullableTypeData {
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct JSDocVariadicTypeData {
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct JSDocOptionalTypeData {
    pub type_node: Arc<Node>,
}

#[derive(Debug)]
pub struct JSDocTypeTagData {
    pub tag_name: Arc<Node>,
    pub type_expression: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocUnknownTagData {
    pub tag_name: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocTemplateTagData {
    pub tag_name: Arc<Node>,
    pub constraint: Arc<Node>,
    pub type_parameters: Arc<NodeList>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocReturnTagData {
    pub tag_name: Arc<Node>,
    pub type_expression: Option<Arc<Node>>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocPublicTagData {
    pub tag_name: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocPrivateTagData {
    pub tag_name: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocProtectedTagData {
    pub tag_name: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocReadonlyTagData {
    pub tag_name: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocOverrideTagData {
    pub tag_name: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocDeprecatedTagData {
    pub tag_name: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocSeeTagData {
    pub tag_name: Arc<Node>,
    pub name_expression: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocImplementsTagData {
    pub tag_name: Arc<Node>,
    pub class_name: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocAugmentsTagData {
    pub tag_name: Arc<Node>,
    pub class_name: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocSatisfiesTagData {
    pub tag_name: Arc<Node>,
    pub type_expression: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocThrowsTagData {
    pub tag_name: Arc<Node>,
    pub type_expression: Option<Arc<Node>>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocThisTagData {
    pub tag_name: Arc<Node>,
    pub type_expression: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocImportTagData {
    pub tag_name: Arc<Node>,
    pub import_clause: Option<Arc<Node>>,
    pub module_specifier: Arc<Node>,
    pub attributes: Option<Arc<Node>>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocCallbackTagData {
    pub tag_name: Arc<Node>,
    pub type_expression: Arc<Node>,
    pub name: Option<Arc<Node>>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocOverloadTagData {
    pub tag_name: Arc<Node>,
    pub type_expression: Arc<Node>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocTypedefTagData {
    pub tag_name: Arc<Node>,
    pub type_expression: Option<Arc<Node>>,
    pub name: Option<Arc<Node>>,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct JSDocSignatureData {
    pub type_parameters: Option<Arc<NodeList>>,
    pub parameters: Arc<NodeList>,
    pub type_node: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct JSDocNameReferenceData {
    pub name: Arc<Node>,
}

#[derive(Debug)]
pub struct SourceFileData {
    pub statements: Arc<NodeList>,
    pub end_of_file_token: Arc<Node>,
}

#[derive(Debug)]
pub struct ModuleDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub keyword: SyntaxKind,
    pub name: Arc<Node>,
    pub body: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ImportEqualsDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub is_type_only: bool,
    pub name: Arc<Node>,
    pub module_reference: Arc<Node>,
}

#[derive(Debug)]
pub struct ExportDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub is_type_only: bool,
    pub export_clause: Option<Arc<Node>>,
    pub module_specifier: Option<Arc<Node>>,
    pub attributes: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ImportTypeNodeData {
    pub is_type_of: bool,
    pub argument: Arc<Node>,
    pub attributes: Option<Arc<Node>>,
    pub qualifier: Option<Arc<Node>>,
    pub type_arguments: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub struct ImportClauseData {
    pub phase_modifier: Option<SyntaxKind>,
    pub name: Option<Arc<Node>>,
    pub named_bindings: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct ImportSpecifierData {
    pub is_type_only: bool,
    pub property_name: Option<Arc<Node>>,
    pub name: Arc<Node>,
}

#[derive(Debug)]
pub struct JSDocTextData {
    pub text: Vec<String>,
}

#[derive(Debug)]
pub struct JSDocLinkData {
    pub name: Option<Arc<Node>>,
    pub text: Vec<String>,
}

#[derive(Debug)]
pub struct JSDocLinkPlainData {
    pub name: Option<Arc<Node>>,
    pub text: Vec<String>,
}

#[derive(Debug)]
pub struct JSDocLinkCodeData {
    pub name: Option<Arc<Node>>,
    pub text: Vec<String>,
}

#[derive(Debug)]
pub struct TypeParameterDeclarationData {
    pub modifiers: Option<Arc<ModifierList>>,
    pub name: Arc<Node>,
    pub constraint: Option<Arc<Node>>,
    pub expression: Option<Arc<Node>>,
    pub default_type: Option<Arc<Node>>,
}

#[derive(Debug)]
pub struct SyntheticReferenceExpressionData {
    pub expression: Arc<Node>,
    pub this_arg: Arc<Node>,
}

#[derive(Debug)]
pub struct JSDocTypeLiteralData {
    pub jsdoc_property_tags: Option<Vec<Arc<Node>>>,
    pub is_array_type: bool,
}

#[derive(Debug)]
pub struct JSDocParameterOrPropertyTagData {
    pub tag_name: Arc<Node>,
    pub name: Arc<Node>,
    pub is_bracketed: bool,
    pub type_expression: Option<Arc<Node>>,
    pub is_name_first: bool,
    pub comment: Option<Arc<NodeList>>,
}

#[derive(Debug)]
pub enum NodeData {
    Token,
    Identifier(IdentifierData),
    PrivateIdentifier(PrivateIdentifierData),
    QualifiedName(QualifiedNameData),
    ComputedPropertyName(ComputedPropertyNameData),
    Decorator(DecoratorData),
    EmptyStatement,
    IfStatement(IfStatementData),
    DoStatement(DoStatementData),
    WhileStatement(WhileStatementData),
    ForStatement(ForStatementData),
    ForInOrOfStatement(ForInOrOfStatementData),
    BreakStatement(BreakStatementData),
    ContinueStatement(ContinueStatementData),
    ReturnStatement(ReturnStatementData),
    WithStatement(WithStatementData),
    SwitchStatement(SwitchStatementData),
    CaseBlock(CaseBlockData),
    CaseOrDefaultClause(CaseOrDefaultClauseData),
    ThrowStatement(ThrowStatementData),
    TryStatement(TryStatementData),
    CatchClause(CatchClauseData),
    DebuggerStatement,
    LabeledStatement(LabeledStatementData),
    ExpressionStatement(ExpressionStatementData),
    Block(BlockData),
    VariableStatement(VariableStatementData),
    VariableDeclaration(VariableDeclarationData),
    VariableDeclarationList(VariableDeclarationListData),
    BindingPattern(BindingPatternData),
    ParameterDeclaration(ParameterDeclarationData),
    BindingElement(BindingElementData),
    MissingDeclaration(MissingDeclarationData),
    FunctionDeclaration(FunctionDeclarationData),
    ClassDeclaration(ClassDeclarationData),
    ClassExpression(ClassExpressionData),
    HeritageClause(HeritageClauseData),
    InterfaceDeclaration(InterfaceDeclarationData),
    TypeAliasDeclaration(TypeAliasDeclarationData),
    EnumMember(EnumMemberData),
    EnumDeclaration(EnumDeclarationData),
    ModuleBlock(ModuleBlockData),
    NotEmittedStatement,
    NotEmittedTypeElement,
    ImportDeclaration(ImportDeclarationData),
    ExternalModuleReference(ExternalModuleReferenceData),
    NamespaceImport(NamespaceImportData),
    NamedImports(NamedImportsData),
    ExportAssignment(ExportAssignmentData),
    NamespaceExportDeclaration(NamespaceExportDeclarationData),
    NamespaceExport(NamespaceExportData),
    NamedExports(NamedExportsData),
    ExportSpecifier(ExportSpecifierData),
    CallSignatureDeclaration(CallSignatureDeclarationData),
    ConstructSignatureDeclaration(ConstructSignatureDeclarationData),
    ConstructorDeclaration(ConstructorDeclarationData),
    GetAccessorDeclaration(GetAccessorDeclarationData),
    SetAccessorDeclaration(SetAccessorDeclarationData),
    IndexSignatureDeclaration(IndexSignatureDeclarationData),
    MethodSignatureDeclaration(MethodSignatureDeclarationData),
    MethodDeclaration(MethodDeclarationData),
    PropertySignatureDeclaration(PropertySignatureDeclarationData),
    PropertyDeclaration(PropertyDeclarationData),
    SemicolonClassElement,
    ClassStaticBlockDeclaration(ClassStaticBlockDeclarationData),
    OmittedExpression,
    KeywordExpression,
    StringLiteral(StringLiteralData),
    NumericLiteral(NumericLiteralData),
    BigIntLiteral(BigIntLiteralData),
    RegularExpressionLiteral(RegularExpressionLiteralData),
    NoSubstitutionTemplateLiteral(NoSubstitutionTemplateLiteralData),
    BinaryExpression(BinaryExpressionData),
    PrefixUnaryExpression(PrefixUnaryExpressionData),
    PostfixUnaryExpression(PostfixUnaryExpressionData),
    YieldExpression(YieldExpressionData),
    ArrowFunction(ArrowFunctionData),
    FunctionExpression(FunctionExpressionData),
    AsExpression(AsExpressionData),
    SatisfiesExpression(SatisfiesExpressionData),
    ConditionalExpression(ConditionalExpressionData),
    PropertyAccessExpression(PropertyAccessExpressionData),
    ElementAccessExpression(ElementAccessExpressionData),
    CallExpression(CallExpressionData),
    NewExpression(NewExpressionData),
    MetaProperty(MetaPropertyData),
    NonNullExpression(NonNullExpressionData),
    SpreadElement(SpreadElementData),
    TemplateExpression(TemplateExpressionData),
    TemplateSpan(TemplateSpanData),
    TaggedTemplateExpression(TaggedTemplateExpressionData),
    ParenthesizedExpression(ParenthesizedExpressionData),
    ArrayLiteralExpression(ArrayLiteralExpressionData),
    ObjectLiteralExpression(ObjectLiteralExpressionData),
    SpreadAssignment(SpreadAssignmentData),
    PropertyAssignment(PropertyAssignmentData),
    ShorthandPropertyAssignment(ShorthandPropertyAssignmentData),
    DeleteExpression(DeleteExpressionData),
    TypeOfExpression(TypeOfExpressionData),
    VoidExpression(VoidExpressionData),
    AwaitExpression(AwaitExpressionData),
    TypeAssertion(TypeAssertionData),
    KeywordTypeNode,
    UnionTypeNode(UnionTypeNodeData),
    IntersectionTypeNode(IntersectionTypeNodeData),
    ConditionalTypeNode(ConditionalTypeNodeData),
    TypeOperatorNode(TypeOperatorNodeData),
    InferTypeNode(InferTypeNodeData),
    ArrayTypeNode(ArrayTypeNodeData),
    IndexedAccessTypeNode(IndexedAccessTypeNodeData),
    TypeReferenceNode(TypeReferenceNodeData),
    ExpressionWithTypeArguments(ExpressionWithTypeArgumentsData),
    LiteralTypeNode(LiteralTypeNodeData),
    ThisTypeNode,
    TypePredicateNode(TypePredicateNodeData),
    ImportAttribute(ImportAttributeData),
    ImportAttributes(ImportAttributesData),
    TypeQueryNode(TypeQueryNodeData),
    MappedTypeNode(MappedTypeNodeData),
    TypeLiteralNode(TypeLiteralNodeData),
    TupleTypeNode(TupleTypeNodeData),
    NamedTupleMember(NamedTupleMemberData),
    OptionalTypeNode(OptionalTypeNodeData),
    RestTypeNode(RestTypeNodeData),
    ParenthesizedTypeNode(ParenthesizedTypeNodeData),
    FunctionTypeNode(FunctionTypeNodeData),
    ConstructorTypeNode(ConstructorTypeNodeData),
    TemplateHead(TemplateHeadData),
    TemplateMiddle(TemplateMiddleData),
    TemplateTail(TemplateTailData),
    TemplateLiteralTypeNode(TemplateLiteralTypeNodeData),
    TemplateLiteralTypeSpan(TemplateLiteralTypeSpanData),
    SyntheticExpression(SyntheticExpressionData),
    PartiallyEmittedExpression(PartiallyEmittedExpressionData),
    JsxElement(JsxElementData),
    JsxAttributes(JsxAttributesData),
    JsxNamespacedName(JsxNamespacedNameData),
    JsxOpeningElement(JsxOpeningElementData),
    JsxSelfClosingElement(JsxSelfClosingElementData),
    JsxFragment(JsxFragmentData),
    JsxOpeningFragment,
    JsxClosingFragment,
    JsxAttribute(JsxAttributeData),
    JsxSpreadAttribute(JsxSpreadAttributeData),
    JsxClosingElement(JsxClosingElementData),
    JsxExpression(JsxExpressionData),
    JsxText(JsxTextData),
    SyntaxList(SyntaxListData),
    JSDoc(JSDocData),
    JSDocTypeExpression(JSDocTypeExpressionData),
    JSDocNonNullableType(JSDocNonNullableTypeData),
    JSDocNullableType(JSDocNullableTypeData),
    JSDocAllType,
    JSDocVariadicType(JSDocVariadicTypeData),
    JSDocOptionalType(JSDocOptionalTypeData),
    JSDocTypeTag(JSDocTypeTagData),
    JSDocUnknownTag(JSDocUnknownTagData),
    JSDocTemplateTag(JSDocTemplateTagData),
    JSDocReturnTag(JSDocReturnTagData),
    JSDocPublicTag(JSDocPublicTagData),
    JSDocPrivateTag(JSDocPrivateTagData),
    JSDocProtectedTag(JSDocProtectedTagData),
    JSDocReadonlyTag(JSDocReadonlyTagData),
    JSDocOverrideTag(JSDocOverrideTagData),
    JSDocDeprecatedTag(JSDocDeprecatedTagData),
    JSDocSeeTag(JSDocSeeTagData),
    JSDocImplementsTag(JSDocImplementsTagData),
    JSDocAugmentsTag(JSDocAugmentsTagData),
    JSDocSatisfiesTag(JSDocSatisfiesTagData),
    JSDocThrowsTag(JSDocThrowsTagData),
    JSDocThisTag(JSDocThisTagData),
    JSDocImportTag(JSDocImportTagData),
    JSDocCallbackTag(JSDocCallbackTagData),
    JSDocOverloadTag(JSDocOverloadTagData),
    JSDocTypedefTag(JSDocTypedefTagData),
    JSDocSignature(JSDocSignatureData),
    JSDocNameReference(JSDocNameReferenceData),
    SourceFile(SourceFileData),
    ModuleDeclaration(ModuleDeclarationData),
    ImportEqualsDeclaration(ImportEqualsDeclarationData),
    ExportDeclaration(ExportDeclarationData),
    ImportTypeNode(ImportTypeNodeData),
    ImportClause(ImportClauseData),
    ImportSpecifier(ImportSpecifierData),
    JSDocText(JSDocTextData),
    JSDocLink(JSDocLinkData),
    JSDocLinkPlain(JSDocLinkPlainData),
    JSDocLinkCode(JSDocLinkCodeData),
    TypeParameterDeclaration(TypeParameterDeclarationData),
    SyntheticReferenceExpression(SyntheticReferenceExpressionData),
    JSDocTypeLiteral(JSDocTypeLiteralData),
    JSDocParameterOrPropertyTag(JSDocParameterOrPropertyTagData),
}

pub fn for_each_child<F>(node: &Node, mut visitor: F) -> bool
where
    F: FnMut(&Arc<Node>) -> bool,
{
    match &node.data {
        NodeData::QualifiedName(data) => {
            if visitor(&data.left) {
                return true;
            }
            if visitor(&data.right) {
                return true;
            }
        }
        NodeData::ComputedPropertyName(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::Decorator(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::IfStatement(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if visitor(&data.then_statement) {
                return true;
            }
            if let Some(child) = &data.else_statement {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::DoStatement(data) => {
            if visitor(&data.statement) {
                return true;
            }
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::WhileStatement(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if visitor(&data.statement) {
                return true;
            }
        }
        NodeData::ForStatement(data) => {
            if let Some(child) = &data.initializer {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.condition {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.incrementor {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.statement) {
                return true;
            }
        }
        NodeData::ForInOrOfStatement(data) => {
            if let Some(child) = &data.await_modifier {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.initializer) {
                return true;
            }
            if visitor(&data.expression) {
                return true;
            }
            if visitor(&data.statement) {
                return true;
            }
        }
        NodeData::BreakStatement(data) => {
            if let Some(child) = &data.label {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ContinueStatement(data) => {
            if let Some(child) = &data.label {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ReturnStatement(data) => {
            if let Some(child) = &data.expression {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::WithStatement(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if visitor(&data.statement) {
                return true;
            }
        }
        NodeData::SwitchStatement(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if visitor(&data.case_block) {
                return true;
            }
        }
        NodeData::CaseBlock(data) => {
            for child in data.clauses.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::CaseOrDefaultClause(data) => {
            if visitor(&data.expression) {
                return true;
            }
            for child in data.statements.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ThrowStatement(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::TryStatement(data) => {
            if visitor(&data.try_block) {
                return true;
            }
            if let Some(child) = &data.catch_clause {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.finally_block {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::CatchClause(data) => {
            if let Some(child) = &data.variable_declaration {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.block) {
                return true;
            }
        }
        NodeData::LabeledStatement(data) => {
            if visitor(&data.label) {
                return true;
            }
            if visitor(&data.statement) {
                return true;
            }
        }
        NodeData::ExpressionStatement(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::Block(data) => {
            for child in data.statements.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::VariableStatement(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.declaration_list) {
                return true;
            }
        }
        NodeData::VariableDeclaration(data) => {
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.exclamation_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.initializer {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::VariableDeclarationList(data) => {
            for child in data.declarations.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::BindingPattern(data) => {
            for child in data.elements.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ParameterDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(child) = &data.dot_dot_dot_token {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.question_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.initializer {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::BindingElement(data) => {
            if let Some(child) = &data.dot_dot_dot_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.property_name {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.initializer {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::MissingDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::FunctionDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(child) = &data.asterisk_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.full_signature {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.body {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ClassDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(list) = &data.heritage_clauses {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.members.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ClassExpression(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(list) = &data.heritage_clauses {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.members.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::HeritageClause(data) => {
            for child in data.types.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::InterfaceDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(list) = &data.heritage_clauses {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.members.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::TypeAliasDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::EnumMember(data) => {
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.initializer {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::EnumDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            for child in data.members.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ModuleBlock(data) => {
            for child in data.statements.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ImportDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(child) = &data.import_clause {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.module_specifier) {
                return true;
            }
            if let Some(child) = &data.attributes {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ExternalModuleReference(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::NamespaceImport(data) => {
            if visitor(&data.name) {
                return true;
            }
        }
        NodeData::NamedImports(data) => {
            for child in data.elements.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ExportAssignment(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.type_node) {
                return true;
            }
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::NamespaceExportDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
        }
        NodeData::NamespaceExport(data) => {
            if visitor(&data.name) {
                return true;
            }
        }
        NodeData::NamedExports(data) => {
            for child in data.elements.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ExportSpecifier(data) => {
            if let Some(child) = &data.property_name {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.name) {
                return true;
            }
        }
        NodeData::CallSignatureDeclaration(data) => {
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ConstructSignatureDeclaration(data) => {
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ConstructorDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.full_signature {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.body {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::GetAccessorDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.full_signature {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.body {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::SetAccessorDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.full_signature {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.body {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::IndexSignatureDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::MethodSignatureDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.postfix_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::MethodDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(child) = &data.asterisk_token {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.postfix_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.full_signature {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.body {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::PropertySignatureDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.postfix_token {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.type_node) {
                return true;
            }
            if visitor(&data.initializer) {
                return true;
            }
        }
        NodeData::PropertyDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.postfix_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.initializer {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ClassStaticBlockDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.body) {
                return true;
            }
        }
        NodeData::BinaryExpression(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.left) {
                return true;
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.operator_token) {
                return true;
            }
            if visitor(&data.right) {
                return true;
            }
        }
        NodeData::PrefixUnaryExpression(data) => {
            if visitor(&data.operand) {
                return true;
            }
        }
        NodeData::PostfixUnaryExpression(data) => {
            if visitor(&data.operand) {
                return true;
            }
        }
        NodeData::YieldExpression(data) => {
            if let Some(child) = &data.asterisk_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.expression {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ArrowFunction(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.full_signature {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.equals_greater_than_token) {
                return true;
            }
            if visitor(&data.body) {
                return true;
            }
        }
        NodeData::FunctionExpression(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(child) = &data.asterisk_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.full_signature {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.body) {
                return true;
            }
        }
        NodeData::AsExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::SatisfiesExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::ConditionalExpression(data) => {
            if visitor(&data.condition) {
                return true;
            }
            if visitor(&data.question_token) {
                return true;
            }
            if visitor(&data.when_true) {
                return true;
            }
            if visitor(&data.colon_token) {
                return true;
            }
            if visitor(&data.when_false) {
                return true;
            }
        }
        NodeData::PropertyAccessExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if let Some(child) = &data.question_dot_token {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.name) {
                return true;
            }
        }
        NodeData::ElementAccessExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if let Some(child) = &data.question_dot_token {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.argument_expression) {
                return true;
            }
        }
        NodeData::CallExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if let Some(child) = &data.question_dot_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.type_arguments {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.arguments.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::NewExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if let Some(list) = &data.type_arguments {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(list) = &data.arguments {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::MetaProperty(data) => {
            if visitor(&data.name) {
                return true;
            }
        }
        NodeData::NonNullExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::SpreadElement(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::TemplateExpression(data) => {
            if visitor(&data.head) {
                return true;
            }
            for child in data.template_spans.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::TemplateSpan(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if visitor(&data.literal) {
                return true;
            }
        }
        NodeData::TaggedTemplateExpression(data) => {
            if visitor(&data.tag) {
                return true;
            }
            if visitor(&data.question_dot_token) {
                return true;
            }
            if let Some(list) = &data.type_arguments {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.template) {
                return true;
            }
        }
        NodeData::ParenthesizedExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::ArrayLiteralExpression(data) => {
            for child in data.elements.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ObjectLiteralExpression(data) => {
            for child in data.properties.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::SpreadAssignment(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::PropertyAssignment(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.postfix_token {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.type_node) {
                return true;
            }
            if visitor(&data.initializer) {
                return true;
            }
        }
        NodeData::ShorthandPropertyAssignment(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.postfix_token {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.type_node) {
                return true;
            }
            if let Some(child) = &data.equals_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.object_assignment_initializer {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::DeleteExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::TypeOfExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::VoidExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::AwaitExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::TypeAssertion(data) => {
            if visitor(&data.type_node) {
                return true;
            }
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::UnionTypeNode(data) => {
            for child in data.types.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::IntersectionTypeNode(data) => {
            for child in data.types.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ConditionalTypeNode(data) => {
            if visitor(&data.check_type) {
                return true;
            }
            if visitor(&data.extends_type) {
                return true;
            }
            if visitor(&data.true_type) {
                return true;
            }
            if visitor(&data.false_type) {
                return true;
            }
        }
        NodeData::TypeOperatorNode(data) => {
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::InferTypeNode(data) => {
            if visitor(&data.type_parameter) {
                return true;
            }
        }
        NodeData::ArrayTypeNode(data) => {
            if visitor(&data.element_type) {
                return true;
            }
        }
        NodeData::IndexedAccessTypeNode(data) => {
            if visitor(&data.object_type) {
                return true;
            }
            if visitor(&data.index_type) {
                return true;
            }
        }
        NodeData::TypeReferenceNode(data) => {
            if visitor(&data.type_name) {
                return true;
            }
            if let Some(list) = &data.type_arguments {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::ExpressionWithTypeArguments(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if let Some(list) = &data.type_arguments {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::LiteralTypeNode(data) => {
            if visitor(&data.literal) {
                return true;
            }
        }
        NodeData::TypePredicateNode(data) => {
            if let Some(child) = &data.asserts_modifier {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.parameter_name) {
                return true;
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ImportAttribute(data) => {
            if visitor(&data.name) {
                return true;
            }
            if visitor(&data.value) {
                return true;
            }
        }
        NodeData::ImportAttributes(data) => {
            for child in data.attributes.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::TypeQueryNode(data) => {
            if visitor(&data.expr_name) {
                return true;
            }
            if let Some(list) = &data.type_arguments {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::MappedTypeNode(data) => {
            if let Some(child) = &data.readonly_token {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.type_parameter) {
                return true;
            }
            if let Some(child) = &data.name_type {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.question_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.members {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::TypeLiteralNode(data) => {
            for child in data.members.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::TupleTypeNode(data) => {
            for child in data.elements.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::NamedTupleMember(data) => {
            if let Some(child) = &data.dot_dot_dot_token {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.question_token {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::OptionalTypeNode(data) => {
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::RestTypeNode(data) => {
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::ParenthesizedTypeNode(data) => {
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::FunctionTypeNode(data) => {
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ConstructorTypeNode(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::TemplateLiteralTypeNode(data) => {
            if visitor(&data.head) {
                return true;
            }
            for child in data.template_spans.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::TemplateLiteralTypeSpan(data) => {
            if visitor(&data.type_node) {
                return true;
            }
            if visitor(&data.literal) {
                return true;
            }
        }
        NodeData::SyntheticExpression(data) => {
            if let Some(child) = &data.tuple_name_source {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::PartiallyEmittedExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::JsxElement(data) => {
            if visitor(&data.opening_element) {
                return true;
            }
            for child in data.children.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.closing_element) {
                return true;
            }
        }
        NodeData::JsxAttributes(data) => {
            for child in data.properties.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::JsxNamespacedName(data) => {
            if visitor(&data.namespace) {
                return true;
            }
            if visitor(&data.name) {
                return true;
            }
        }
        NodeData::JsxOpeningElement(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(list) = &data.type_arguments {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.attributes) {
                return true;
            }
        }
        NodeData::JsxSelfClosingElement(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(list) = &data.type_arguments {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.attributes) {
                return true;
            }
        }
        NodeData::JsxFragment(data) => {
            if visitor(&data.opening_fragment) {
                return true;
            }
            for child in data.children.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.closing_fragment) {
                return true;
            }
        }
        NodeData::JsxAttribute(data) => {
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.initializer {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::JsxSpreadAttribute(data) => {
            if visitor(&data.expression) {
                return true;
            }
        }
        NodeData::JsxClosingElement(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
        }
        NodeData::JsxExpression(data) => {
            if let Some(child) = &data.dot_dot_dot_token {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.expression {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::SyntaxList(data) => {
            for child in data.children.iter() {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::JSDoc(data) => {
            for child in data.comment.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.tags {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocTypeExpression(data) => {
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::JSDocNonNullableType(data) => {
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::JSDocNullableType(data) => {
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::JSDocVariadicType(data) => {
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::JSDocOptionalType(data) => {
            if visitor(&data.type_node) {
                return true;
            }
        }
        NodeData::JSDocTypeTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if visitor(&data.type_expression) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocUnknownTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocTemplateTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if visitor(&data.constraint) {
                return true;
            }
            for child in data.type_parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocReturnTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(child) = &data.type_expression {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocPublicTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocPrivateTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocProtectedTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocReadonlyTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocOverrideTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocDeprecatedTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocSeeTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if visitor(&data.name_expression) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocImplementsTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if visitor(&data.class_name) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocAugmentsTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if visitor(&data.class_name) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocSatisfiesTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if visitor(&data.type_expression) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocThrowsTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(child) = &data.type_expression {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocThisTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if visitor(&data.type_expression) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocImportTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(child) = &data.import_clause {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.module_specifier) {
                return true;
            }
            if let Some(child) = &data.attributes {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocCallbackTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if visitor(&data.type_expression) {
                return true;
            }
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocOverloadTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if visitor(&data.type_expression) {
                return true;
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocTypedefTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if let Some(child) = &data.type_expression {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocSignature(data) => {
            if let Some(list) = &data.type_parameters {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            for child in data.parameters.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.type_node {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::JSDocNameReference(data) => {
            if visitor(&data.name) {
                return true;
            }
        }
        NodeData::SourceFile(data) => {
            for child in data.statements.iter() {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.end_of_file_token) {
                return true;
            }
        }
        NodeData::ModuleDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.body {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ImportEqualsDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if visitor(&data.module_reference) {
                return true;
            }
        }
        NodeData::ExportDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if let Some(child) = &data.export_clause {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.module_specifier {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.attributes {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ImportTypeNode(data) => {
            if visitor(&data.argument) {
                return true;
            }
            if let Some(child) = &data.attributes {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.qualifier {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.type_arguments {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::ImportClause(data) => {
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.named_bindings {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::ImportSpecifier(data) => {
            if let Some(child) = &data.property_name {
                if visitor(child) {
                    return true;
                }
            }
            if visitor(&data.name) {
                return true;
            }
        }
        NodeData::JSDocLink(data) => {
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::JSDocLinkPlain(data) => {
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::JSDocLinkCode(data) => {
            if let Some(child) = &data.name {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::TypeParameterDeclaration(data) => {
            if let Some(list) = &data.modifiers {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.constraint {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.expression {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(child) = &data.default_type {
                if visitor(child) {
                    return true;
                }
            }
        }
        NodeData::SyntheticReferenceExpression(data) => {
            if visitor(&data.expression) {
                return true;
            }
            if visitor(&data.this_arg) {
                return true;
            }
        }
        NodeData::JSDocTypeLiteral(data) => {
            if let Some(list) = &data.jsdoc_property_tags {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        NodeData::JSDocParameterOrPropertyTag(data) => {
            if visitor(&data.tag_name) {
                return true;
            }
            if visitor(&data.name) {
                return true;
            }
            if let Some(child) = &data.type_expression {
                if visitor(child) {
                    return true;
                }
            }
            if let Some(list) = &data.comment {
                for child in list.iter() {
                    if visitor(child) {
                        return true;
                    }
                }
            }
        }
        _ => {}
    }
    false
}

pub fn node_text(node: &Node) -> &str {
    match &node.data {
        NodeData::Identifier(d) => &d.text,
        NodeData::PrivateIdentifier(d) => &d.text,
        NodeData::StringLiteral(d) => &d.text,
        NodeData::NumericLiteral(d) => &d.text,
        NodeData::BigIntLiteral(d) => &d.text,
        NodeData::RegularExpressionLiteral(d) => &d.text,
        NodeData::NoSubstitutionTemplateLiteral(d) => &d.text,
        NodeData::TemplateHead(d) => &d.text,
        NodeData::TemplateMiddle(d) => &d.text,
        NodeData::TemplateTail(d) => &d.text,
        NodeData::JsxText(d) => &d.text,
        _ => "",
    }
}

pub fn node_expression(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ComputedPropertyName(d) => Some(&d.expression),
        NodeData::Decorator(d) => Some(&d.expression),
        NodeData::IfStatement(d) => Some(&d.expression),
        NodeData::DoStatement(d) => Some(&d.expression),
        NodeData::WhileStatement(d) => Some(&d.expression),
        NodeData::ForInOrOfStatement(d) => Some(&d.expression),
        NodeData::WithStatement(d) => Some(&d.expression),
        NodeData::SwitchStatement(d) => Some(&d.expression),
        NodeData::CaseOrDefaultClause(d) => Some(&d.expression),
        NodeData::ThrowStatement(d) => Some(&d.expression),
        NodeData::ExpressionStatement(d) => Some(&d.expression),
        NodeData::ExternalModuleReference(d) => Some(&d.expression),
        NodeData::ExportAssignment(d) => Some(&d.expression),
        NodeData::AsExpression(d) => Some(&d.expression),
        NodeData::SatisfiesExpression(d) => Some(&d.expression),
        NodeData::PropertyAccessExpression(d) => Some(&d.expression),
        NodeData::ElementAccessExpression(d) => Some(&d.expression),
        NodeData::CallExpression(d) => Some(&d.expression),
        NodeData::NewExpression(d) => Some(&d.expression),
        NodeData::NonNullExpression(d) => Some(&d.expression),
        NodeData::SpreadElement(d) => Some(&d.expression),
        NodeData::TemplateSpan(d) => Some(&d.expression),
        NodeData::ParenthesizedExpression(d) => Some(&d.expression),
        NodeData::SpreadAssignment(d) => Some(&d.expression),
        NodeData::DeleteExpression(d) => Some(&d.expression),
        NodeData::TypeOfExpression(d) => Some(&d.expression),
        NodeData::VoidExpression(d) => Some(&d.expression),
        NodeData::AwaitExpression(d) => Some(&d.expression),
        NodeData::TypeAssertion(d) => Some(&d.expression),
        NodeData::ExpressionWithTypeArguments(d) => Some(&d.expression),
        NodeData::PartiallyEmittedExpression(d) => Some(&d.expression),
        NodeData::JsxSpreadAttribute(d) => Some(&d.expression),
        NodeData::SyntheticReferenceExpression(d) => Some(&d.expression),
        _ => None,
    }
}

pub fn node_name(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::VariableDeclaration(d) => Some(&d.name),
        NodeData::ParameterDeclaration(d) => Some(&d.name),
        NodeData::BindingElement(d) => d.name.as_ref(),
        NodeData::FunctionDeclaration(d) => d.name.as_ref(),
        NodeData::ClassDeclaration(d) => d.name.as_ref(),
        NodeData::ClassExpression(d) => d.name.as_ref(),
        NodeData::InterfaceDeclaration(d) => Some(&d.name),
        NodeData::TypeAliasDeclaration(d) => Some(&d.name),
        NodeData::EnumMember(d) => Some(&d.name),
        NodeData::EnumDeclaration(d) => Some(&d.name),
        NodeData::NamespaceImport(d) => Some(&d.name),
        NodeData::NamespaceExportDeclaration(d) => Some(&d.name),
        NodeData::NamespaceExport(d) => Some(&d.name),
        NodeData::ExportSpecifier(d) => Some(&d.name),
        NodeData::GetAccessorDeclaration(d) => Some(&d.name),
        NodeData::SetAccessorDeclaration(d) => Some(&d.name),
        NodeData::MethodSignatureDeclaration(d) => Some(&d.name),
        NodeData::MethodDeclaration(d) => Some(&d.name),
        NodeData::PropertySignatureDeclaration(d) => Some(&d.name),
        NodeData::PropertyDeclaration(d) => Some(&d.name),
        NodeData::FunctionExpression(d) => d.name.as_ref(),
        NodeData::PropertyAccessExpression(d) => Some(&d.name),
        NodeData::MetaProperty(d) => Some(&d.name),
        NodeData::PropertyAssignment(d) => Some(&d.name),
        NodeData::ShorthandPropertyAssignment(d) => Some(&d.name),
        NodeData::ImportAttribute(d) => Some(&d.name),
        NodeData::NamedTupleMember(d) => Some(&d.name),
        NodeData::JsxNamespacedName(d) => Some(&d.name),
        NodeData::JsxAttribute(d) => Some(&d.name),
        NodeData::JSDocCallbackTag(d) => d.name.as_ref(),
        NodeData::JSDocTypedefTag(d) => d.name.as_ref(),
        NodeData::JSDocNameReference(d) => Some(&d.name),
        NodeData::ModuleDeclaration(d) => Some(&d.name),
        NodeData::ImportEqualsDeclaration(d) => Some(&d.name),
        NodeData::ImportClause(d) => d.name.as_ref(),
        NodeData::ImportSpecifier(d) => Some(&d.name),
        NodeData::JSDocLink(d) => d.name.as_ref(),
        NodeData::JSDocLinkPlain(d) => d.name.as_ref(),
        NodeData::JSDocLinkCode(d) => d.name.as_ref(),
        NodeData::TypeParameterDeclaration(d) => Some(&d.name),
        NodeData::JSDocParameterOrPropertyTag(d) => Some(&d.name),
        _ => None,
    }
}

pub fn node_type(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ArrayTypeNode(d) => Some(&d.element_type),
        NodeData::ParenthesizedTypeNode(d) => Some(&d.type_node),
        NodeData::TypeOperatorNode(d) => Some(&d.type_node),
        NodeData::OptionalTypeNode(d) => Some(&d.type_node),
        NodeData::RestTypeNode(d) => Some(&d.type_node),
        NodeData::NamedTupleMember(d) => Some(&d.type_node),
        NodeData::FunctionTypeNode(d) => d.type_node.as_ref(),
        NodeData::ConstructorTypeNode(d) => d.type_node.as_ref(),
        NodeData::TypePredicateNode(d) => d.type_node.as_ref(),
        NodeData::MappedTypeNode(d) => d.type_node.as_ref(),
        NodeData::JSDocNonNullableType(d) => Some(&d.type_node),
        NodeData::JSDocNullableType(d) => Some(&d.type_node),
        NodeData::JSDocVariadicType(d) => Some(&d.type_node),
        NodeData::JSDocOptionalType(d) => Some(&d.type_node),

        NodeData::FunctionDeclaration(d) => d.type_node.as_ref(),
        NodeData::FunctionExpression(d) => d.type_node.as_ref(),
        NodeData::ArrowFunction(d) => d.type_node.as_ref(),
        NodeData::MethodDeclaration(d) => d.type_node.as_ref(),
        NodeData::MethodSignatureDeclaration(d) => d.type_node.as_ref(),
        NodeData::ConstructorDeclaration(d) => d.type_node.as_ref(),
        NodeData::ConstructSignatureDeclaration(d) => d.type_node.as_ref(),
        NodeData::CallSignatureDeclaration(d) => d.type_node.as_ref(),
        NodeData::GetAccessorDeclaration(d) => d.type_node.as_ref(),
        NodeData::SetAccessorDeclaration(d) => d.type_node.as_ref(),
        _ => None,
    }
}

pub fn is_token(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::Unknown
        | SyntaxKind::EndOfFile
        | SyntaxKind::SingleLineCommentTrivia
        | SyntaxKind::MultiLineCommentTrivia
        | SyntaxKind::NewLineTrivia
        | SyntaxKind::WhitespaceTrivia
        | SyntaxKind::ConflictMarkerTrivia
        | SyntaxKind::NonTextFileMarkerTrivia
        | SyntaxKind::NumericLiteral
        | SyntaxKind::BigIntLiteral
        | SyntaxKind::StringLiteral
        | SyntaxKind::JsxText
        | SyntaxKind::JsxTextAllWhiteSpaces
        | SyntaxKind::RegularExpressionLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::TemplateHead
        | SyntaxKind::TemplateMiddle
        | SyntaxKind::TemplateTail
        | SyntaxKind::OpenBraceToken
        | SyntaxKind::CloseBraceToken
        | SyntaxKind::OpenParenToken
        | SyntaxKind::CloseParenToken
        | SyntaxKind::OpenBracketToken
        | SyntaxKind::CloseBracketToken
        | SyntaxKind::DotToken
        | SyntaxKind::DotDotDotToken
        | SyntaxKind::SemicolonToken
        | SyntaxKind::CommaToken
        | SyntaxKind::QuestionDotToken
        | SyntaxKind::LessThanToken
        | SyntaxKind::LessThanSlashToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::EqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken
        | SyntaxKind::EqualsGreaterThanToken
        | SyntaxKind::PlusToken
        | SyntaxKind::MinusToken
        | SyntaxKind::AsteriskToken
        | SyntaxKind::AsteriskAsteriskToken
        | SyntaxKind::SlashToken
        | SyntaxKind::PercentToken
        | SyntaxKind::PlusPlusToken
        | SyntaxKind::MinusMinusToken
        | SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
        | SyntaxKind::AmpersandToken
        | SyntaxKind::BarToken
        | SyntaxKind::CaretToken
        | SyntaxKind::ExclamationToken
        | SyntaxKind::TildeToken
        | SyntaxKind::AmpersandAmpersandToken
        | SyntaxKind::BarBarToken
        | SyntaxKind::QuestionToken
        | SyntaxKind::ColonToken
        | SyntaxKind::AtToken
        | SyntaxKind::QuestionQuestionToken
        | SyntaxKind::BacktickToken
        | SyntaxKind::HashToken
        | SyntaxKind::EqualsToken
        | SyntaxKind::PlusEqualsToken
        | SyntaxKind::MinusEqualsToken
        | SyntaxKind::AsteriskEqualsToken
        | SyntaxKind::AsteriskAsteriskEqualsToken
        | SyntaxKind::SlashEqualsToken
        | SyntaxKind::PercentEqualsToken
        | SyntaxKind::LessThanLessThanEqualsToken
        | SyntaxKind::GreaterThanGreaterThanEqualsToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
        | SyntaxKind::AmpersandEqualsToken
        | SyntaxKind::BarEqualsToken
        | SyntaxKind::BarBarEqualsToken
        | SyntaxKind::AmpersandAmpersandEqualsToken
        | SyntaxKind::QuestionQuestionEqualsToken
        | SyntaxKind::CaretEqualsToken
        | SyntaxKind::Identifier
        | SyntaxKind::PrivateIdentifier
        | SyntaxKind::JSDocCommentTextToken
        | SyntaxKind::BreakKeyword
        | SyntaxKind::CaseKeyword
        | SyntaxKind::CatchKeyword
        | SyntaxKind::ClassKeyword
        | SyntaxKind::ConstKeyword
        | SyntaxKind::ContinueKeyword
        | SyntaxKind::DebuggerKeyword
        | SyntaxKind::DefaultKeyword
        | SyntaxKind::DeleteKeyword
        | SyntaxKind::DoKeyword
        | SyntaxKind::ElseKeyword
        | SyntaxKind::EnumKeyword
        | SyntaxKind::ExportKeyword
        | SyntaxKind::ExtendsKeyword
        | SyntaxKind::FalseKeyword
        | SyntaxKind::FinallyKeyword
        | SyntaxKind::ForKeyword
        | SyntaxKind::FunctionKeyword
        | SyntaxKind::IfKeyword
        | SyntaxKind::ImportKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::NewKeyword
        | SyntaxKind::NullKeyword
        | SyntaxKind::ReturnKeyword
        | SyntaxKind::SuperKeyword
        | SyntaxKind::SwitchKeyword
        | SyntaxKind::ThisKeyword
        | SyntaxKind::ThrowKeyword
        | SyntaxKind::TrueKeyword
        | SyntaxKind::TryKeyword
        | SyntaxKind::TypeOfKeyword
        | SyntaxKind::VarKeyword
        | SyntaxKind::VoidKeyword
        | SyntaxKind::WhileKeyword
        | SyntaxKind::WithKeyword
        | SyntaxKind::ImplementsKeyword
        | SyntaxKind::InterfaceKeyword
        | SyntaxKind::LetKeyword
        | SyntaxKind::PackageKeyword
        | SyntaxKind::PrivateKeyword
        | SyntaxKind::ProtectedKeyword
        | SyntaxKind::PublicKeyword
        | SyntaxKind::StaticKeyword
        | SyntaxKind::YieldKeyword
        | SyntaxKind::AbstractKeyword
        | SyntaxKind::AccessorKeyword
        | SyntaxKind::AsKeyword
        | SyntaxKind::AssertsKeyword
        | SyntaxKind::AssertKeyword
        | SyntaxKind::AnyKeyword
        | SyntaxKind::AsyncKeyword
        | SyntaxKind::AwaitKeyword
        | SyntaxKind::BooleanKeyword
        | SyntaxKind::ConstructorKeyword
        | SyntaxKind::DeclareKeyword
        | SyntaxKind::GetKeyword
        | SyntaxKind::ImmediateKeyword
        | SyntaxKind::InferKeyword
        | SyntaxKind::IntrinsicKeyword
        | SyntaxKind::IsKeyword
        | SyntaxKind::KeyOfKeyword
        | SyntaxKind::ModuleKeyword
        | SyntaxKind::NamespaceKeyword
        | SyntaxKind::NeverKeyword
        | SyntaxKind::OutKeyword
        | SyntaxKind::ReadonlyKeyword
        | SyntaxKind::RequireKeyword
        | SyntaxKind::NumberKeyword
        | SyntaxKind::ObjectKeyword
        | SyntaxKind::SatisfiesKeyword
        | SyntaxKind::SetKeyword
        | SyntaxKind::StringKeyword
        | SyntaxKind::SymbolKeyword
        | SyntaxKind::TypeKeyword
        | SyntaxKind::UndefinedKeyword
        | SyntaxKind::UniqueKeyword
        | SyntaxKind::UnknownKeyword
        | SyntaxKind::UsingKeyword
        | SyntaxKind::FromKeyword
        | SyntaxKind::GlobalKeyword
        | SyntaxKind::BigIntKeyword
        | SyntaxKind::OverrideKeyword
        | SyntaxKind::OfKeyword
        | SyntaxKind::DeferKeyword => true,
        _ => false,
    }
}

pub fn is_identifier(node: &Node) -> bool {
    node.kind == SyntaxKind::Identifier
}

pub fn is_private_identifier(node: &Node) -> bool {
    node.kind == SyntaxKind::PrivateIdentifier
}

pub fn is_qualified_name(node: &Node) -> bool {
    node.kind == SyntaxKind::QualifiedName
}

pub fn is_computed_property_name(node: &Node) -> bool {
    node.kind == SyntaxKind::ComputedPropertyName
}

pub fn is_decorator(node: &Node) -> bool {
    node.kind == SyntaxKind::Decorator
}

pub fn is_empty_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::EmptyStatement
}

pub fn is_if_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::IfStatement
}

pub fn is_do_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::DoStatement
}

pub fn is_while_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::WhileStatement
}

pub fn is_for_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::ForStatement
}

pub fn is_for_in_or_of_statement(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => true,
        _ => false,
    }
}

pub fn is_break_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::BreakStatement
}

pub fn is_continue_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::ContinueStatement
}

pub fn is_return_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::ReturnStatement
}

pub fn is_with_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::WithStatement
}

pub fn is_switch_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::SwitchStatement
}

pub fn is_case_block(node: &Node) -> bool {
    node.kind == SyntaxKind::CaseBlock
}

pub fn is_case_or_default_clause(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::CaseClause | SyntaxKind::DefaultClause => true,
        _ => false,
    }
}

pub fn is_throw_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::ThrowStatement
}

pub fn is_try_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::TryStatement
}

pub fn is_catch_clause(node: &Node) -> bool {
    node.kind == SyntaxKind::CatchClause
}

pub fn is_debugger_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::DebuggerStatement
}

pub fn is_labeled_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::LabeledStatement
}

pub fn is_expression_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::ExpressionStatement
}

pub fn is_block(node: &Node) -> bool {
    node.kind == SyntaxKind::Block
}

pub fn is_variable_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::VariableStatement
}

pub fn is_variable_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::VariableDeclaration
}

pub fn is_variable_declaration_list(node: &Node) -> bool {
    node.kind == SyntaxKind::VariableDeclarationList
}

pub fn is_binding_pattern(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern => true,
        _ => false,
    }
}

pub fn is_parameter_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::Parameter
}

pub fn is_binding_element(node: &Node) -> bool {
    node.kind == SyntaxKind::BindingElement
}

pub fn is_missing_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::MissingDeclaration
}

pub fn is_function_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::FunctionDeclaration
}

pub fn is_class_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::ClassDeclaration
}

pub fn is_class_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::ClassExpression
}

pub fn is_heritage_clause(node: &Node) -> bool {
    node.kind == SyntaxKind::HeritageClause
}

pub fn is_interface_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::InterfaceDeclaration
}

pub fn is_type_alias_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::TypeAliasDeclaration
}

pub fn is_enum_member(node: &Node) -> bool {
    node.kind == SyntaxKind::EnumMember
}

pub fn is_enum_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::EnumDeclaration
}

pub fn is_module_block(node: &Node) -> bool {
    node.kind == SyntaxKind::ModuleBlock
}

pub fn is_not_emitted_statement(node: &Node) -> bool {
    node.kind == SyntaxKind::NotEmittedStatement
}

pub fn is_not_emitted_type_element(node: &Node) -> bool {
    node.kind == SyntaxKind::NotEmittedTypeElement
}

pub fn is_import_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::ImportDeclaration
}

pub fn is_external_module_reference(node: &Node) -> bool {
    node.kind == SyntaxKind::ExternalModuleReference
}

pub fn is_namespace_import(node: &Node) -> bool {
    node.kind == SyntaxKind::NamespaceImport
}

pub fn is_named_imports(node: &Node) -> bool {
    node.kind == SyntaxKind::NamedImports
}

pub fn is_export_assignment(node: &Node) -> bool {
    node.kind == SyntaxKind::ExportAssignment
}

pub fn is_namespace_export_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::NamespaceExportDeclaration
}

pub fn is_namespace_export(node: &Node) -> bool {
    node.kind == SyntaxKind::NamespaceExport
}

pub fn is_named_exports(node: &Node) -> bool {
    node.kind == SyntaxKind::NamedExports
}

pub fn is_export_specifier(node: &Node) -> bool {
    node.kind == SyntaxKind::ExportSpecifier
}

pub fn is_call_signature_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::CallSignature
}

pub fn is_construct_signature_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::ConstructSignature
}

pub fn is_constructor_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::Constructor
}

pub fn is_get_accessor_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::GetAccessor
}

pub fn is_set_accessor_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::SetAccessor
}

pub fn is_index_signature_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::IndexSignature
}

pub fn is_method_signature_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::MethodSignature
}

pub fn is_method_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::MethodDeclaration
}

pub fn is_property_signature_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::PropertySignature
}

pub fn is_property_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::PropertyDeclaration
}

pub fn is_semicolon_class_element(node: &Node) -> bool {
    node.kind == SyntaxKind::SemicolonClassElement
}

pub fn is_class_static_block_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::ClassStaticBlockDeclaration
}

pub fn is_omitted_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::OmittedExpression
}

pub fn is_keyword_expression(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::NullKeyword
        | SyntaxKind::TrueKeyword
        | SyntaxKind::FalseKeyword
        | SyntaxKind::ThisKeyword
        | SyntaxKind::SuperKeyword
        | SyntaxKind::ImportKeyword => true,
        _ => false,
    }
}

pub fn is_string_literal(node: &Node) -> bool {
    node.kind == SyntaxKind::StringLiteral
}

pub fn is_numeric_literal(node: &Node) -> bool {
    node.kind == SyntaxKind::NumericLiteral
}

pub fn is_big_int_literal(node: &Node) -> bool {
    node.kind == SyntaxKind::BigIntLiteral
}

pub fn is_regular_expression_literal(node: &Node) -> bool {
    node.kind == SyntaxKind::RegularExpressionLiteral
}

pub fn is_no_substitution_template_literal(node: &Node) -> bool {
    node.kind == SyntaxKind::NoSubstitutionTemplateLiteral
}

pub fn is_binary_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::BinaryExpression
}

pub fn is_prefix_unary_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::PrefixUnaryExpression
}

pub fn is_postfix_unary_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::PostfixUnaryExpression
}

pub fn is_yield_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::YieldExpression
}

pub fn is_arrow_function(node: &Node) -> bool {
    node.kind == SyntaxKind::ArrowFunction
}

pub fn is_function_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::FunctionExpression
}

pub fn is_as_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::AsExpression
}

pub fn is_satisfies_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::SatisfiesExpression
}

pub fn is_conditional_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::ConditionalExpression
}

pub fn is_property_access_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::PropertyAccessExpression
}

pub fn is_element_access_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::ElementAccessExpression
}

pub fn is_call_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::CallExpression
}

pub fn is_new_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::NewExpression
}

pub fn is_meta_property(node: &Node) -> bool {
    node.kind == SyntaxKind::MetaProperty
}

pub fn is_non_null_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::NonNullExpression
}

pub fn is_spread_element(node: &Node) -> bool {
    node.kind == SyntaxKind::SpreadElement
}

pub fn is_template_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::TemplateExpression
}

pub fn is_template_span(node: &Node) -> bool {
    node.kind == SyntaxKind::TemplateSpan
}

pub fn is_tagged_template_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::TaggedTemplateExpression
}

pub fn is_parenthesized_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::ParenthesizedExpression
}

pub fn is_array_literal_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::ArrayLiteralExpression
}

pub fn is_object_literal_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::ObjectLiteralExpression
}

pub fn is_spread_assignment(node: &Node) -> bool {
    node.kind == SyntaxKind::SpreadAssignment
}

pub fn is_property_assignment(node: &Node) -> bool {
    node.kind == SyntaxKind::PropertyAssignment
}

pub fn is_shorthand_property_assignment(node: &Node) -> bool {
    node.kind == SyntaxKind::ShorthandPropertyAssignment
}

pub fn is_delete_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::DeleteExpression
}

pub fn is_type_of_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::TypeOfExpression
}

pub fn is_void_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::VoidExpression
}

pub fn is_await_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::AwaitExpression
}

pub fn is_type_assertion(node: &Node) -> bool {
    node.kind == SyntaxKind::TypeAssertionExpression
}

pub fn is_keyword_type_node(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::AnyKeyword
        | SyntaxKind::BigIntKeyword
        | SyntaxKind::BooleanKeyword
        | SyntaxKind::IntrinsicKeyword
        | SyntaxKind::NeverKeyword
        | SyntaxKind::NumberKeyword
        | SyntaxKind::ObjectKeyword
        | SyntaxKind::StringKeyword
        | SyntaxKind::SymbolKeyword
        | SyntaxKind::UndefinedKeyword
        | SyntaxKind::UnknownKeyword
        | SyntaxKind::VoidKeyword => true,
        _ => false,
    }
}

pub fn is_union_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::UnionType
}

pub fn is_intersection_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::IntersectionType
}

pub fn is_conditional_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::ConditionalType
}

pub fn is_type_operator_node(node: &Node) -> bool {
    node.kind == SyntaxKind::TypeOperator
}

pub fn is_infer_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::InferType
}

pub fn is_array_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::ArrayType
}

pub fn is_indexed_access_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::IndexedAccessType
}

pub fn is_type_reference_node(node: &Node) -> bool {
    node.kind == SyntaxKind::TypeReference
}

pub fn is_expression_with_type_arguments(node: &Node) -> bool {
    node.kind == SyntaxKind::ExpressionWithTypeArguments
}

pub fn is_literal_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::LiteralType
}

pub fn is_this_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::ThisType
}

pub fn is_type_predicate_node(node: &Node) -> bool {
    node.kind == SyntaxKind::TypePredicate
}

pub fn is_import_attribute(node: &Node) -> bool {
    node.kind == SyntaxKind::ImportAttribute
}

pub fn is_import_attributes(node: &Node) -> bool {
    node.kind == SyntaxKind::ImportAttributes
}

pub fn is_type_query_node(node: &Node) -> bool {
    node.kind == SyntaxKind::TypeQuery
}

pub fn is_mapped_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::MappedType
}

pub fn is_type_literal_node(node: &Node) -> bool {
    node.kind == SyntaxKind::TypeLiteral
}

pub fn is_tuple_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::TupleType
}

pub fn is_named_tuple_member(node: &Node) -> bool {
    node.kind == SyntaxKind::NamedTupleMember
}

pub fn is_optional_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::OptionalType
}

pub fn is_rest_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::RestType
}

pub fn is_parenthesized_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::ParenthesizedType
}

pub fn is_function_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::FunctionType
}

pub fn is_constructor_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::ConstructorType
}

pub fn is_template_head(node: &Node) -> bool {
    node.kind == SyntaxKind::TemplateHead
}

pub fn is_template_middle(node: &Node) -> bool {
    node.kind == SyntaxKind::TemplateMiddle
}

pub fn is_template_tail(node: &Node) -> bool {
    node.kind == SyntaxKind::TemplateTail
}

pub fn is_template_literal_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::TemplateLiteralType
}

pub fn is_template_literal_type_span(node: &Node) -> bool {
    node.kind == SyntaxKind::TemplateLiteralTypeSpan
}

pub fn is_synthetic_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::SyntheticExpression
}

pub fn is_partially_emitted_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::PartiallyEmittedExpression
}

pub fn is_jsx_element(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxElement
}

pub fn is_jsx_attributes(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxAttributes
}

pub fn is_jsx_namespaced_name(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxNamespacedName
}

pub fn is_jsx_opening_element(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxOpeningElement
}

pub fn is_jsx_self_closing_element(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxSelfClosingElement
}

pub fn is_jsx_fragment(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxFragment
}

pub fn is_jsx_opening_fragment(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxOpeningFragment
}

pub fn is_jsx_closing_fragment(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxClosingFragment
}

pub fn is_jsx_attribute(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxAttribute
}

pub fn is_jsx_spread_attribute(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxSpreadAttribute
}

pub fn is_jsx_closing_element(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxClosingElement
}

pub fn is_jsx_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxExpression
}

pub fn is_jsx_text(node: &Node) -> bool {
    node.kind == SyntaxKind::JsxText
}

pub fn is_syntax_list(node: &Node) -> bool {
    node.kind == SyntaxKind::SyntaxList
}

pub fn is_jsdoc(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDoc
}

pub fn is_jsdoc_type_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocTypeExpression
}

pub fn is_jsdoc_non_nullable_type(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocNonNullableType
}

pub fn is_jsdoc_nullable_type(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocNullableType
}

pub fn is_jsdoc_all_type(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocAllType
}

pub fn is_jsdoc_variadic_type(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocVariadicType
}

pub fn is_jsdoc_optional_type(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocOptionalType
}

pub fn is_jsdoc_type_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocTypeTag
}

pub fn is_jsdoc_unknown_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocUnknownTag
}

pub fn is_jsdoc_template_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocTemplateTag
}

pub fn is_jsdoc_return_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocReturnTag
}

pub fn is_jsdoc_public_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocPublicTag
}

pub fn is_jsdoc_private_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocPrivateTag
}

pub fn is_jsdoc_protected_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocProtectedTag
}

pub fn is_jsdoc_readonly_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocReadonlyTag
}

pub fn is_jsdoc_override_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocOverrideTag
}

pub fn is_jsdoc_deprecated_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocDeprecatedTag
}

pub fn is_jsdoc_see_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocSeeTag
}

pub fn is_jsdoc_implements_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocImplementsTag
}

pub fn is_jsdoc_augments_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocAugmentsTag
}

pub fn is_jsdoc_satisfies_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocSatisfiesTag
}

pub fn is_jsdoc_throws_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocThrowsTag
}

pub fn is_jsdoc_this_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocThisTag
}

pub fn is_jsdoc_import_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocImportTag
}

pub fn is_jsdoc_callback_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocCallbackTag
}

pub fn is_jsdoc_overload_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocOverloadTag
}

pub fn is_jsdoc_typedef_tag(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocTypedefTag
}

pub fn is_jsdoc_signature(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocSignature
}

pub fn is_jsdoc_name_reference(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocNameReference
}

pub fn is_source_file(node: &Node) -> bool {
    node.kind == SyntaxKind::SourceFile
}

pub fn is_module_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::ModuleDeclaration
}

pub fn is_import_equals_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::ImportEqualsDeclaration
}

pub fn is_export_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::ExportDeclaration
}

pub fn is_import_type_node(node: &Node) -> bool {
    node.kind == SyntaxKind::ImportType
}

pub fn is_import_clause(node: &Node) -> bool {
    node.kind == SyntaxKind::ImportClause
}

pub fn is_import_specifier(node: &Node) -> bool {
    node.kind == SyntaxKind::ImportSpecifier
}

pub fn is_jsdoc_text(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocText
}

pub fn is_jsdoc_link(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocLink
}

pub fn is_jsdoc_link_plain(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocLinkPlain
}

pub fn is_jsdoc_link_code(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocLinkCode
}

pub fn is_type_parameter_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::TypeParameter
}

pub fn is_synthetic_reference_expression(node: &Node) -> bool {
    node.kind == SyntaxKind::SyntheticReferenceExpression
}

pub fn is_jsdoc_type_literal(node: &Node) -> bool {
    node.kind == SyntaxKind::JSDocTypeLiteral
}

pub fn is_jsdoc_parameter_or_property_tag(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::JSDocParameterTag | SyntaxKind::JSDocPropertyTag => true,
        _ => false,
    }
}

pub fn is_trivia_kind(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::SingleLineCommentTrivia
        | SyntaxKind::MultiLineCommentTrivia
        | SyntaxKind::NewLineTrivia
        | SyntaxKind::WhitespaceTrivia
        | SyntaxKind::ConflictMarkerTrivia => true,
        _ => false,
    }
}

pub fn is_literal_kind(kind: SyntaxKind) -> bool {
    (kind as i16) >= (SyntaxKind::NumericLiteral as i16)
        && (kind as i16) <= (SyntaxKind::NoSubstitutionTemplateLiteral as i16)
}

pub fn is_pseudo_literal_kind(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::TemplateHead | SyntaxKind::TemplateMiddle | SyntaxKind::TemplateTail => true,
        _ => false,
    }
}

pub fn is_punctuation_kind(kind: SyntaxKind) -> bool {
    (kind as i16) >= (SyntaxKind::OpenBraceToken as i16)
        && (kind as i16) <= (SyntaxKind::CaretEqualsToken as i16)
}

pub fn is_keyword_kind(kind: SyntaxKind) -> bool {
    (kind as i16) >= (SyntaxKind::BreakKeyword as i16)
        && (kind as i16) <= (SyntaxKind::DeferKeyword as i16)
}

pub fn is_modifier_kind(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AbstractKeyword
        | SyntaxKind::AccessorKeyword
        | SyntaxKind::AsyncKeyword
        | SyntaxKind::ConstKeyword
        | SyntaxKind::DeclareKeyword
        | SyntaxKind::DefaultKeyword
        | SyntaxKind::ExportKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::PrivateKeyword
        | SyntaxKind::ProtectedKeyword
        | SyntaxKind::PublicKeyword
        | SyntaxKind::ReadonlyKeyword
        | SyntaxKind::OutKeyword
        | SyntaxKind::OverrideKeyword
        | SyntaxKind::StaticKeyword => true,
        _ => false,
    }
}

pub fn is_keyword_type_kind(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AnyKeyword
        | SyntaxKind::BigIntKeyword
        | SyntaxKind::BooleanKeyword
        | SyntaxKind::IntrinsicKeyword
        | SyntaxKind::NeverKeyword
        | SyntaxKind::NumberKeyword
        | SyntaxKind::ObjectKeyword
        | SyntaxKind::StringKeyword
        | SyntaxKind::SymbolKeyword
        | SyntaxKind::UndefinedKeyword
        | SyntaxKind::UnknownKeyword
        | SyntaxKind::VoidKeyword => true,
        _ => false,
    }
}

pub fn is_keyword_expression_kind(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::NullKeyword
        | SyntaxKind::TrueKeyword
        | SyntaxKind::FalseKeyword
        | SyntaxKind::ThisKeyword
        | SyntaxKind::SuperKeyword
        | SyntaxKind::ImportKeyword => true,
        _ => false,
    }
}

pub fn is_token_kind(kind: SyntaxKind) -> bool {
    (kind as i16) >= (SyntaxKind::Unknown as i16)
        && (kind as i16) <= (SyntaxKind::DeferKeyword as i16)
}

pub fn is_jsx_token_kind(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::LessThanSlashToken
        | SyntaxKind::EndOfFile
        | SyntaxKind::ConflictMarkerTrivia
        | SyntaxKind::JsxText
        | SyntaxKind::JsxTextAllWhiteSpaces
        | SyntaxKind::OpenBraceToken
        | SyntaxKind::LessThanToken => true,
        _ => false,
    }
}

pub fn is_jsdoc_node_kind(kind: SyntaxKind) -> bool {
    (kind as i16) >= (SyntaxKind::JSDocTypeExpression as i16)
        && (kind as i16) <= (SyntaxKind::JSDocImportTag as i16)
}

pub fn is_import_phase_modifier_kind(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::TypeKeyword | SyntaxKind::DeferKeyword => true,
        _ => false,
    }
}

pub fn is_postfix_unary_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken => true,
        _ => false,
    }
}

pub fn is_prefix_unary_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::PlusToken
        | SyntaxKind::MinusToken
        | SyntaxKind::TildeToken
        | SyntaxKind::ExclamationToken
        | SyntaxKind::PlusPlusToken
        | SyntaxKind::MinusMinusToken => true,
        _ => false,
    }
}

pub fn is_assignment_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::EqualsToken
        | SyntaxKind::PlusEqualsToken
        | SyntaxKind::MinusEqualsToken
        | SyntaxKind::AsteriskAsteriskEqualsToken
        | SyntaxKind::AsteriskEqualsToken
        | SyntaxKind::SlashEqualsToken
        | SyntaxKind::PercentEqualsToken
        | SyntaxKind::AmpersandEqualsToken
        | SyntaxKind::BarEqualsToken
        | SyntaxKind::CaretEqualsToken
        | SyntaxKind::LessThanLessThanEqualsToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
        | SyntaxKind::GreaterThanGreaterThanEqualsToken
        | SyntaxKind::BarBarEqualsToken
        | SyntaxKind::AmpersandAmpersandEqualsToken
        | SyntaxKind::QuestionQuestionEqualsToken => true,
        _ => false,
    }
}

pub fn is_binary_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::QuestionQuestionToken
        | SyntaxKind::AsteriskAsteriskToken
        | SyntaxKind::AsteriskToken
        | SyntaxKind::SlashToken
        | SyntaxKind::PercentToken
        | SyntaxKind::PlusToken
        | SyntaxKind::MinusToken
        | SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
        | SyntaxKind::LessThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::EqualsEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::AmpersandToken
        | SyntaxKind::BarToken
        | SyntaxKind::CaretToken
        | SyntaxKind::AmpersandAmpersandToken
        | SyntaxKind::BarBarToken
        | SyntaxKind::EqualsToken
        | SyntaxKind::PlusEqualsToken
        | SyntaxKind::MinusEqualsToken
        | SyntaxKind::AsteriskAsteriskEqualsToken
        | SyntaxKind::AsteriskEqualsToken
        | SyntaxKind::SlashEqualsToken
        | SyntaxKind::PercentEqualsToken
        | SyntaxKind::AmpersandEqualsToken
        | SyntaxKind::BarEqualsToken
        | SyntaxKind::CaretEqualsToken
        | SyntaxKind::LessThanLessThanEqualsToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
        | SyntaxKind::GreaterThanGreaterThanEqualsToken
        | SyntaxKind::BarBarEqualsToken
        | SyntaxKind::AmpersandAmpersandEqualsToken
        | SyntaxKind::QuestionQuestionEqualsToken
        | SyntaxKind::CommaToken => true,
        _ => false,
    }
}

pub fn is_exponentiation_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AsteriskAsteriskToken => true,
        _ => false,
    }
}

pub fn is_multiplicative_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AsteriskToken | SyntaxKind::SlashToken | SyntaxKind::PercentToken => true,
        _ => false,
    }
}

pub fn is_multiplicative_operator_or_higher(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AsteriskAsteriskToken
        | SyntaxKind::AsteriskToken
        | SyntaxKind::SlashToken
        | SyntaxKind::PercentToken => true,
        _ => false,
    }
}

pub fn is_additive_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::PlusToken | SyntaxKind::MinusToken => true,
        _ => false,
    }
}

pub fn is_additive_operator_or_higher(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AsteriskAsteriskToken
        | SyntaxKind::AsteriskToken
        | SyntaxKind::SlashToken
        | SyntaxKind::PercentToken
        | SyntaxKind::PlusToken
        | SyntaxKind::MinusToken => true,
        _ => false,
    }
}

pub fn is_shift_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken => true,
        _ => false,
    }
}

pub fn is_shift_operator_or_higher(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AsteriskAsteriskToken
        | SyntaxKind::AsteriskToken
        | SyntaxKind::SlashToken
        | SyntaxKind::PercentToken
        | SyntaxKind::PlusToken
        | SyntaxKind::MinusToken
        | SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken => true,
        _ => false,
    }
}

pub fn is_relational_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::LessThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword => true,
        _ => false,
    }
}

pub fn is_relational_operator_or_higher(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AsteriskAsteriskToken
        | SyntaxKind::AsteriskToken
        | SyntaxKind::SlashToken
        | SyntaxKind::PercentToken
        | SyntaxKind::PlusToken
        | SyntaxKind::MinusToken
        | SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
        | SyntaxKind::LessThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword => true,
        _ => false,
    }
}

pub fn is_equality_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::EqualsEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken => true,
        _ => false,
    }
}

pub fn is_equality_operator_or_higher(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AsteriskAsteriskToken
        | SyntaxKind::AsteriskToken
        | SyntaxKind::SlashToken
        | SyntaxKind::PercentToken
        | SyntaxKind::PlusToken
        | SyntaxKind::MinusToken
        | SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
        | SyntaxKind::LessThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::EqualsEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken => true,
        _ => false,
    }
}

pub fn is_bitwise_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AmpersandToken | SyntaxKind::BarToken | SyntaxKind::CaretToken => true,
        _ => false,
    }
}

pub fn is_bitwise_operator_or_higher(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AsteriskAsteriskToken
        | SyntaxKind::AsteriskToken
        | SyntaxKind::SlashToken
        | SyntaxKind::PercentToken
        | SyntaxKind::PlusToken
        | SyntaxKind::MinusToken
        | SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
        | SyntaxKind::LessThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::EqualsEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::AmpersandToken
        | SyntaxKind::BarToken
        | SyntaxKind::CaretToken => true,
        _ => false,
    }
}

pub fn is_logical_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AmpersandAmpersandToken | SyntaxKind::BarBarToken => true,
        _ => false,
    }
}

pub fn is_logical_operator_or_higher(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AsteriskAsteriskToken
        | SyntaxKind::AsteriskToken
        | SyntaxKind::SlashToken
        | SyntaxKind::PercentToken
        | SyntaxKind::PlusToken
        | SyntaxKind::MinusToken
        | SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
        | SyntaxKind::LessThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::EqualsEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::AmpersandToken
        | SyntaxKind::BarToken
        | SyntaxKind::CaretToken
        | SyntaxKind::AmpersandAmpersandToken
        | SyntaxKind::BarBarToken => true,
        _ => false,
    }
}

pub fn is_compound_assignment_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::PlusEqualsToken
        | SyntaxKind::MinusEqualsToken
        | SyntaxKind::AsteriskAsteriskEqualsToken
        | SyntaxKind::AsteriskEqualsToken
        | SyntaxKind::SlashEqualsToken
        | SyntaxKind::PercentEqualsToken
        | SyntaxKind::AmpersandEqualsToken
        | SyntaxKind::BarEqualsToken
        | SyntaxKind::CaretEqualsToken
        | SyntaxKind::LessThanLessThanEqualsToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
        | SyntaxKind::GreaterThanGreaterThanEqualsToken
        | SyntaxKind::BarBarEqualsToken
        | SyntaxKind::AmpersandAmpersandEqualsToken
        | SyntaxKind::QuestionQuestionEqualsToken => true,
        _ => false,
    }
}

pub fn is_assignment_operator_or_higher(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::QuestionQuestionToken
        | SyntaxKind::AsteriskAsteriskToken
        | SyntaxKind::AsteriskToken
        | SyntaxKind::SlashToken
        | SyntaxKind::PercentToken
        | SyntaxKind::PlusToken
        | SyntaxKind::MinusToken
        | SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
        | SyntaxKind::LessThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::EqualsEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::AmpersandToken
        | SyntaxKind::BarToken
        | SyntaxKind::CaretToken
        | SyntaxKind::AmpersandAmpersandToken
        | SyntaxKind::BarBarToken
        | SyntaxKind::EqualsToken
        | SyntaxKind::PlusEqualsToken
        | SyntaxKind::MinusEqualsToken
        | SyntaxKind::AsteriskAsteriskEqualsToken
        | SyntaxKind::AsteriskEqualsToken
        | SyntaxKind::SlashEqualsToken
        | SyntaxKind::PercentEqualsToken
        | SyntaxKind::AmpersandEqualsToken
        | SyntaxKind::BarEqualsToken
        | SyntaxKind::CaretEqualsToken
        | SyntaxKind::LessThanLessThanEqualsToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
        | SyntaxKind::GreaterThanGreaterThanEqualsToken
        | SyntaxKind::BarBarEqualsToken
        | SyntaxKind::AmpersandAmpersandEqualsToken
        | SyntaxKind::QuestionQuestionEqualsToken => true,
        _ => false,
    }
}

pub fn is_logical_or_coalescing_assignment_operator(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AmpersandAmpersandEqualsToken
        | SyntaxKind::BarBarEqualsToken
        | SyntaxKind::QuestionQuestionEqualsToken => true,
        _ => false,
    }
}
