#![allow(dead_code)]

use tsox_frontend::ast::Node;
use tsox_frontend::ast::node_data_generated::{
    BindingPatternData, ExportAssignmentData, ForInOrOfStatementData, SyntaxListData,
    VariableDeclarationData, VariableStatementData,
};

pub trait R38K6DataExt {
    fn as_for_in_or_of_statement(&self) -> &ForInOrOfStatementData;
    fn as_variable_declaration(&self) -> &VariableDeclarationData;
    fn as_variable_statement(&self) -> &VariableStatementData;
    fn as_syntax_list(&self) -> &SyntaxListData;
    fn as_export_assignment(&self) -> &ExportAssignmentData;
    fn as_binding_pattern(&self) -> &BindingPatternData;
}

macro_rules! as_data {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                tsox_frontend::ast::NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl R38K6DataExt for Node {
    as_data!(as_for_in_or_of_statement, ForInOrOfStatement, ForInOrOfStatementData);
    as_data!(as_variable_declaration, VariableDeclaration, VariableDeclarationData);
    as_data!(as_variable_statement, VariableStatement, VariableStatementData);
    as_data!(as_syntax_list, SyntaxList, SyntaxListData);
    as_data!(as_export_assignment, ExportAssignment, ExportAssignmentData);
    as_data!(as_binding_pattern, BindingPattern, BindingPatternData);
}
