use crate::ast::*;

fn is_declaration_statement_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::MissingDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::NamespaceExportDeclaration
    )
}

pub fn is_declaration_statement(node: &Node) -> bool {
    is_declaration_statement_kind(node.kind)
}

fn is_statement_kind_but_not_declaration_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::BreakStatement
            | SyntaxKind::ContinueStatement
            | SyntaxKind::DebuggerStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::ExpressionStatement
            | SyntaxKind::EmptyStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::IfStatement
            | SyntaxKind::LabeledStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::ThrowStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::VariableStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::NotEmittedStatement
    )
}

pub fn is_statement_but_not_declaration(node: &Node) -> bool {
    is_statement_kind_but_not_declaration_kind(node.kind)
}

pub fn is_function_block(node: &Node) -> bool {
    if node.kind != SyntaxKind::Block {
        return false;
    }
    match node.parent() {
        Some(parent) => is_function_like(&parent),
        None => false,
    }
}

pub fn is_block_statement(node: &Node) -> bool {
    if node.kind != SyntaxKind::Block {
        return false;
    }
    if let Some(parent) = node.parent() {
        if parent.kind == SyntaxKind::TryStatement || parent.kind == SyntaxKind::CatchClause {
            return false;
        }
    }
    !is_function_block(node)
}

pub fn is_statement(node: &Node) -> bool {
    let kind = node.kind;
    is_statement_kind_but_not_declaration_kind(kind)
        || is_declaration_statement_kind(kind)
        || is_block_statement(node)
}

pub fn is_iteration_statement(node: &Node, look_in_labeled_statements: bool) -> bool {
    match node.kind {
        SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement
        | SyntaxKind::DoStatement
        | SyntaxKind::WhileStatement => true,
        SyntaxKind::LabeledStatement => {
            if look_in_labeled_statements {
                if let NodeData::LabeledStatement(d) = &node.data {
                    return is_iteration_statement(&d.statement, look_in_labeled_statements);
                }
            }
            false
        }
        _ => false,
    }
}

pub fn is_prologue_directive(node: &Node) -> bool {
    if node.kind != SyntaxKind::ExpressionStatement {
        return false;
    }
    match node.expression() {
        Some(expr) => expr.kind == SyntaxKind::StringLiteral,
        None => false,
    }
}

/// Go IsPotentiallyExecutableNode：KindFirstStatement..KindLastStatement
/// 语句（var 须有初始化或块级绑定）或 class/enum/module 声明；
/// 函数/接口/类型别名/import/export 不算可执行
pub fn is_potentially_executable_node(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::BreakStatement
        | SyntaxKind::ContinueStatement
        | SyntaxKind::DebuggerStatement
        | SyntaxKind::DoStatement
        | SyntaxKind::ExpressionStatement
        | SyntaxKind::EmptyStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement
        | SyntaxKind::ForStatement
        | SyntaxKind::IfStatement
        | SyntaxKind::LabeledStatement
        | SyntaxKind::ReturnStatement
        | SyntaxKind::SwitchStatement
        | SyntaxKind::ThrowStatement
        | SyntaxKind::TryStatement
        | SyntaxKind::WhileStatement
        | SyntaxKind::WithStatement
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::ModuleDeclaration => true,
        SyntaxKind::VariableStatement => {
            let Some(crate::ast::node_data_generated::NodeData::VariableStatement(data)) =
                Some(&node.data)
            else {
                return false;
            };
            let list_node = &data.declaration_list;
            let Some(crate::ast::node_data_generated::NodeData::VariableDeclarationList(list)) = Some(&list_node.data)
            else {
                return false;
            };
            if list_node.flags.intersects(NodeFlags::BlockScoped) {
                return true;
            }
            list.declarations.iter().any(|d| {
                matches!(&d.data, NodeData::VariableDeclaration(vd) if vd.initializer.is_some())
            })
        }
        _ => false,
    }
}
