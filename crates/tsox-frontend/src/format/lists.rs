//! Go format/util.go 列表定位逻辑的移植：getOpenTokenForList、
//! getCloseTokenForOpenToken、GetContainingList/getListByRange、
//! getVisualListRange、isListElement/isMemberListElement。

use std::sync::Arc;

use tsox_core::core::text::TextRange;

use crate::ast::node::{Node, NodeList};
use crate::ast::SyntaxKind;
use crate::ast::SourceFile;

use super::util::token_pos_of_node;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Field {
    TypeParameters,
    Parameters,
    TypeArguments,
    Arguments,
    Members,
    Elements,
    Properties,
    Declarations,
    Clauses,
    HeritageClauses,
    Statements,
}

fn list_fields(parent: &Arc<Node>) -> Vec<(Field, Arc<NodeList>)> {
    use Field::*;
    
    let data = &parent.data;
    let list = |l: &Arc<NodeList>| l.clone();
    let opt = |l: &Option<Arc<NodeList>>| l.as_ref().map(|l| l.clone());
    match &*data {
        crate::ast::node_data_generated::NodeData::FunctionDeclaration(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::FunctionExpression(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::ArrowFunction(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::MethodDeclaration(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::MethodSignatureDeclaration(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::CallSignatureDeclaration(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::ConstructSignatureDeclaration(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::ConstructorDeclaration(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::ConstructorTypeNode(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::FunctionTypeNode(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::GetAccessorDeclaration(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::SetAccessorDeclaration(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                Some((Parameters, list(&d.parameters))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::CallExpression(d) => {
            vec![
                opt(&d.type_arguments).map(|l| (TypeArguments, list(&l))),
                Some((Arguments, list(&d.arguments))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::NewExpression(d) => {
            vec![
                opt(&d.type_arguments).map(|l| (TypeArguments, list(&l))),
                d.arguments.as_ref().map(|l| (Arguments, list(l))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::ClassDeclaration(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                opt(&d.heritage_clauses).map(|l| (HeritageClauses, list(&l))),
                Some((Members, list(&d.members))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::ClassExpression(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                opt(&d.heritage_clauses).map(|l| (HeritageClauses, list(&l))),
                Some((Members, list(&d.members))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::InterfaceDeclaration(d) => {
            vec![
                opt(&d.type_parameters).map(|l| (TypeParameters, list(&l))),
                opt(&d.heritage_clauses).map(|l| (HeritageClauses, list(&l))),
                Some((Members, list(&d.members))),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
        crate::ast::node_data_generated::NodeData::TypeAliasDeclaration(d) => {
            opt(&d.type_parameters)
                .map(|l| (TypeParameters, list(&l)))
                .into_iter()
                .collect()
        }
        crate::ast::node_data_generated::NodeData::TypeReferenceNode(d) => opt(&d.type_arguments)
            .map(|l| (TypeArguments, list(&l)))
            .into_iter()
            .collect(),
        crate::ast::node_data_generated::NodeData::TaggedTemplateExpression(d) => {
            opt(&d.type_arguments)
                .map(|l| (TypeArguments, list(&l)))
                .into_iter()
                .collect()
        }
        crate::ast::node_data_generated::NodeData::TypeQueryNode(d) => opt(&d.type_arguments)
            .map(|l| (TypeArguments, list(&l)))
            .into_iter()
            .collect(),
        crate::ast::node_data_generated::NodeData::ExpressionWithTypeArguments(d) => {
            opt(&d.type_arguments)
                .map(|l| (TypeArguments, list(&l)))
                .into_iter()
                .collect()
        }
        crate::ast::node_data_generated::NodeData::ImportTypeNode(d) => opt(&d.type_arguments)
            .map(|l| (TypeArguments, list(&l)))
            .into_iter()
            .collect(),
        crate::ast::node_data_generated::NodeData::TypeLiteralNode(d) => {
            vec![(Members, list(&d.members))]
        }
        crate::ast::node_data_generated::NodeData::ObjectLiteralExpression(d) => {
            vec![(Properties, list(&d.properties))]
        }
        crate::ast::node_data_generated::NodeData::ArrayLiteralExpression(d) => {
            vec![(Elements, list(&d.elements))]
        }
        crate::ast::node_data_generated::NodeData::VariableDeclarationList(d) => {
            vec![(Declarations, list(&d.declarations))]
        }
        crate::ast::node_data_generated::NodeData::BindingPattern(d) => {
            vec![(Elements, list(&d.elements))]
        }
        crate::ast::node_data_generated::NodeData::NamedImports(d) => {
            vec![(Elements, list(&d.elements))]
        }
        crate::ast::node_data_generated::NodeData::NamedExports(d) => {
            vec![(Elements, list(&d.elements))]
        }
        crate::ast::node_data_generated::NodeData::CaseBlock(d) => {
            vec![(Clauses, list(&d.clauses))]
        }
        crate::ast::node_data_generated::NodeData::Block(d) => {
            vec![(Statements, list(&d.statements))]
        }
        crate::ast::node_data_generated::NodeData::ModuleBlock(d) => {
            vec![(Statements, list(&d.statements))]
        }
        crate::ast::node_data_generated::NodeData::SourceFile(d) => {
            vec![(Statements, list(&d.statements))]
        }
        crate::ast::node_data_generated::NodeData::EnumDeclaration(d) => {
            vec![(Members, list(&d.members))]
        }
        _ => Vec::new(),
    }
}

/// Go getOpenTokenForList
pub(crate) fn open_token_for_list(parent: &Arc<Node>, list: &Arc<NodeList>) -> SyntaxKind {
    use SyntaxKind::*;
    for (field, l) in list_fields(parent) {
        if !Arc::ptr_eq(&l, list) {
            continue;
        }
        return match parent.kind {
            Constructor
            | FunctionDeclaration
            | FunctionExpression
            | MethodDeclaration
            | MethodSignature
            | ArrowFunction
            | CallSignature
            | ConstructSignature
            | FunctionType
            | ConstructorType
            | GetAccessor
            | SetAccessor => match field {
                Field::TypeParameters => LessThanToken,
                Field::Parameters => OpenParenToken,
                _ => Unknown,
            },
            CallExpression | NewExpression => match field {
                Field::TypeArguments => LessThanToken,
                Field::Arguments => OpenParenToken,
                _ => Unknown,
            },
            ClassDeclaration | ClassExpression | InterfaceDeclaration | TypeAliasDeclaration => {
                match field {
                    Field::TypeParameters => LessThanToken,
                    _ => Unknown,
                }
            }
            TypeReference | TaggedTemplateExpression | TypeQuery | ExpressionWithTypeArguments
            | ImportType => match field {
                Field::TypeArguments => LessThanToken,
                _ => Unknown,
            },
            TypeLiteral => OpenBraceToken,
            _ => Unknown,
        };
    }
    Unknown
}

/// Go getCloseTokenForOpenToken
pub(crate) fn close_token_for_open_token(kind: SyntaxKind) -> SyntaxKind {
    match kind {
        SyntaxKind::OpenParenToken => SyntaxKind::CloseParenToken,
        SyntaxKind::LessThanToken => SyntaxKind::GreaterThanToken,
        SyntaxKind::OpenBraceToken => SyntaxKind::CloseBraceToken,
        _ => SyntaxKind::Unknown,
    }
}

/// Go getListByRange：返回 parent 中可视范围包含 [start,end) 的列表。
pub(crate) fn get_list_by_range(
    start: usize,
    end: usize,
    parent: &Arc<Node>,
    file: &SourceFile,
) -> Option<(Field, Arc<NodeList>)> {
    let r = TextRange::new(start, end);
    for (field, list) in list_fields(parent) {
        if r.contained_by(&get_visual_list_range(parent, list.loc, file)) {
            return Some((field, list));
        }
    }
    None
}

/// Go GetContainingList
pub(crate) fn get_containing_list(node: &Arc<Node>, file: &SourceFile) -> Option<(Field, Arc<NodeList>)> {
    let parent = node.parent()?;
    let start = token_pos_of_node(file, node);
    get_list_by_range(start, node.end(), &parent, file)
}

/// Go getVisualListRange：把列表范围扩展到前邻 token 末尾与后邻 token 起点之间。
pub(crate) fn get_visual_list_range(parent: &Arc<Node>, list_loc: TextRange, file: &SourceFile) -> TextRange {
    let _ = parent;
    let prior_end = crate::astnav::find_preceding_token(&file.node, list_loc.pos())
        .map(|t| t.end())
        .unwrap_or(list_loc.pos());
    let mut scan = crate::scanner::Scanner::new(file.text.clone());
    scan.set_range(list_loc.end(), file.text.len());
    scan.scan();
    let next_start = if scan.token() == SyntaxKind::EndOfFile {
        list_loc.end()
    } else {
        scan.token_pos()
    };
    TextRange::new(prior_end, next_start)
}

/// Go isListElement：node 是否为 parent 某个语句/成员列表的元素。
pub(crate) fn is_list_element(parent: &Arc<Node>, node: &Arc<Node>, file: &SourceFile) -> bool {
    use SyntaxKind::*;
    match parent.kind {
        ClassDeclaration | InterfaceDeclaration => node_in_field(parent, node, file, Field::Members),
        ModuleDeclaration => {
            let Some(body) = node_child(parent, SyntaxKind::ModuleBlock) else {
                return false;
            };
            node_in_field(&body, node, file, Field::Statements)
        }
        SourceFile | Block | ModuleBlock => node_in_field(parent, node, file, Field::Statements),
        CatchClause => {
            let Some(block) = node_child(parent, SyntaxKind::Block) else {
                return false;
            };
            node_in_field(&block, node, file, Field::Statements)
        }
        _ => false,
    }
}

/// Go isMemberListElement
pub(crate) fn is_member_list_element(parent: &Arc<Node>, node: &Arc<Node>, file: &SourceFile) -> bool {
    use SyntaxKind::*;
    match parent.kind {
        ClassDeclaration | ClassExpression | InterfaceDeclaration | EnumDeclaration
        | TypeLiteral | MappedType => node_in_field(parent, node, file, Field::Members),
        _ => false,
    }
}

fn node_in_field(parent: &Arc<Node>, node: &Arc<Node>, file: &SourceFile, want: Field) -> bool {
    for (field, list) in list_fields(parent) {
        if field != want {
            continue;
        }
        let vis = get_visual_list_range(parent, list.loc, file);
        if node.loc.contained_by(&vis) {
            return true;
        }
    }
    false
}

fn node_child(parent: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
    let mut hit = None;
    crate::ast::node_data_generated::for_each_child(parent, |c| {
        if c.kind == kind {
            hit = Some(c.clone());
            return true;
        }
        false
    });
    hit
}
