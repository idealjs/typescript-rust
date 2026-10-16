#![allow(dead_code)]

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    ArrowFunctionData, ConstructorDeclarationData, FunctionExpressionData,
};

pub trait R37K14DataExt {
    fn as_constructor_declaration(&self) -> &ConstructorDeclarationData;
    fn as_arrow_function(&self) -> &ArrowFunctionData;
    fn as_function_expression(&self) -> &FunctionExpressionData;
}

macro_rules! as_data14 {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                tsox_frontend::ast::NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl R37K14DataExt for Node {
    as_data14!(
        as_constructor_declaration,
        ConstructorDeclaration,
        ConstructorDeclarationData
    );
    as_data14!(as_arrow_function, ArrowFunction, ArrowFunctionData);
    as_data14!(as_function_expression, FunctionExpression, FunctionExpressionData);
}
