#![allow(dead_code, unused_imports)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::format::mig::m4o_2::EmitContext;

use crate::printer::mig::m4m_2::SnippetElement;

macro_rules! as_statement_data {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

macro_rules! as_statement_data_sig {
    ($name:ident, $ty:ty) => {
        fn $name(&self) -> &$ty;
    };
}

pub trait NodeStatementExt37k4 {
    as_statement_data_sig!(as_block, BlockData);
    as_statement_data_sig!(as_variable_statement, VariableStatementData);
    as_statement_data_sig!(as_expression_statement, ExpressionStatementData);
    as_statement_data_sig!(as_if_statement, IfStatementData);
    as_statement_data_sig!(as_do_statement, DoStatementData);
    as_statement_data_sig!(as_while_statement, WhileStatementData);
    as_statement_data_sig!(as_for_statement, ForStatementData);
    as_statement_data_sig!(as_for_in_or_of_statement, ForInOrOfStatementData);
    as_statement_data_sig!(as_continue_statement, ContinueStatementData);
    as_statement_data_sig!(as_break_statement, BreakStatementData);
    as_statement_data_sig!(as_return_statement, ReturnStatementData);
    as_statement_data_sig!(as_with_statement, WithStatementData);
    as_statement_data_sig!(as_switch_statement, SwitchStatementData);
    as_statement_data_sig!(as_labeled_statement, LabeledStatementData);
    as_statement_data_sig!(as_throw_statement, ThrowStatementData);
    as_statement_data_sig!(as_try_statement, TryStatementData);
    as_statement_data_sig!(as_function_declaration, FunctionDeclarationData);
    as_statement_data_sig!(as_class_declaration, ClassDeclarationData);
    as_statement_data_sig!(as_interface_declaration, InterfaceDeclarationData);
    as_statement_data_sig!(as_type_alias_declaration, TypeAliasDeclarationData);
    as_statement_data_sig!(as_enum_declaration, EnumDeclarationData);
    as_statement_data_sig!(as_module_declaration, ModuleDeclarationData);
    as_statement_data_sig!(as_namespace_export_declaration, NamespaceExportDeclarationData);
    as_statement_data_sig!(as_import_equals_declaration, ImportEqualsDeclarationData);
    as_statement_data_sig!(as_import_declaration, ImportDeclarationData);
    as_statement_data_sig!(as_export_assignment, ExportAssignmentData);
    as_statement_data_sig!(as_export_declaration, ExportDeclarationData);
}

impl NodeStatementExt37k4 for Node {
    as_statement_data!(as_block, Block, BlockData);
    as_statement_data!(as_variable_statement, VariableStatement, VariableStatementData);
    as_statement_data!(as_expression_statement, ExpressionStatement, ExpressionStatementData);
    as_statement_data!(as_if_statement, IfStatement, IfStatementData);
    as_statement_data!(as_do_statement, DoStatement, DoStatementData);
    as_statement_data!(as_while_statement, WhileStatement, WhileStatementData);
    as_statement_data!(as_for_statement, ForStatement, ForStatementData);
    as_statement_data!(as_for_in_or_of_statement, ForInOrOfStatement, ForInOrOfStatementData);
    as_statement_data!(as_continue_statement, ContinueStatement, ContinueStatementData);
    as_statement_data!(as_break_statement, BreakStatement, BreakStatementData);
    as_statement_data!(as_return_statement, ReturnStatement, ReturnStatementData);
    as_statement_data!(as_with_statement, WithStatement, WithStatementData);
    as_statement_data!(as_switch_statement, SwitchStatement, SwitchStatementData);
    as_statement_data!(as_labeled_statement, LabeledStatement, LabeledStatementData);
    as_statement_data!(as_throw_statement, ThrowStatement, ThrowStatementData);
    as_statement_data!(as_try_statement, TryStatement, TryStatementData);
    as_statement_data!(as_function_declaration, FunctionDeclaration, FunctionDeclarationData);
    as_statement_data!(as_class_declaration, ClassDeclaration, ClassDeclarationData);
    as_statement_data!(as_interface_declaration, InterfaceDeclaration, InterfaceDeclarationData);
    as_statement_data!(as_type_alias_declaration, TypeAliasDeclaration, TypeAliasDeclarationData);
    as_statement_data!(as_enum_declaration, EnumDeclaration, EnumDeclarationData);
    as_statement_data!(as_module_declaration, ModuleDeclaration, ModuleDeclarationData);
    as_statement_data!(
        as_namespace_export_declaration,
        NamespaceExportDeclaration,
        NamespaceExportDeclarationData
    );
    as_statement_data!(
        as_import_equals_declaration,
        ImportEqualsDeclaration,
        ImportEqualsDeclarationData
    );
    as_statement_data!(as_import_declaration, ImportDeclaration, ImportDeclarationData);
    as_statement_data!(as_export_assignment, ExportAssignment, ExportAssignmentData);
    as_statement_data!(as_export_declaration, ExportDeclaration, ExportDeclarationData);
}

pub trait EmitContextExt37k4 {
    fn snippet_element(&self, node: &Node) -> Option<SnippetElement>;
    fn source_map_range(&self, node: &Node) -> TextRange;
    fn token_source_map_range(&self, node: &Node, kind: SyntaxKind) -> (TextRange, bool);
    fn emit_flags_of(&self, node: &Node) -> u32;
}

impl EmitContextExt37k4 for EmitContext {
    fn snippet_element(&self, _node: &Node) -> Option<SnippetElement> {
        None
    }

    fn source_map_range(&self, node: &Node) -> TextRange {
        node.loc
    }

    fn token_source_map_range(&self, _node: &Node, _kind: SyntaxKind) -> (TextRange, bool) {
        (TextRange::undefined(), false)
    }

    fn emit_flags_of(&self, _node: &Node) -> u32 {
        0
    }
}

pub struct SourceMapStateSnapshot {
    pub emit_flags: u32,
    pub source_map_range: TextRange,
    pub has_token_source_map_range: bool,
}

impl SourceMapStateSnapshot {
    pub fn new(emit_flags: u32, source_map_range: TextRange, has_token_source_map_range: bool) -> Self {
        Self {
            emit_flags,
            source_map_range,
            has_token_source_map_range,
        }
    }
}
