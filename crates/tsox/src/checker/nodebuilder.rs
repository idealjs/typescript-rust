use std::sync::Arc;
use crate::ast::{
    ArrayTypeNodeData, BigIntLiteralData, FunctionTypeNodeData, IdentifierData,
    IntersectionTypeNodeData, LiteralTypeNodeData, MissingDeclarationData, Node, NodeData,
    NodeList, NumericLiteralData, ParameterDeclarationData, ParenthesizedTypeNodeData,
    PropertySignatureDeclarationData, RestTypeNodeData, StringLiteralData, Symbol, SymbolFlags,
    SyntaxKind, TupleTypeNodeData, TypeLiteralNodeData, TypeOperatorNodeData,
    TypeReferenceNodeData, UnionTypeNodeData,
};
use super::checker::Checker;
use super::types::*;
mod type_format_flags_2;
mod checker_7;
mod checker_8;
mod checker_9;
mod checker_10;
mod checker_11;
mod checker_12;
mod checker_13;
#[allow(unused_imports, ambiguous_glob_reexports)]
pub use type_format_flags_2::*;
#[allow(unused_imports)]
pub use checker_7::*;
#[allow(unused_imports)]
pub use checker_8::*;
#[allow(unused_imports)]
pub use checker_9::*;
#[allow(unused_imports)]
pub use checker_10::*;
#[allow(unused_imports)]
pub use checker_11::*;
#[allow(unused_imports)]
pub use checker_12::*;
#[allow(unused_imports)]
pub use checker_13::*;
#[cfg(test)]
mod tests;
