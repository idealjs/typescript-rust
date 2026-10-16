#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::{self, Node, NodeList};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PseudoTypeKind {
    Direct,
    Inferred,
    NoResult,
    MaybeConstLocation,
    Union,
    Undefined,
    Null,
    Any,
    String,
    Number,
    BigInt,
    Boolean,
    False,
    True,
    SingleCallSignature,
    Tuple,
    ObjectLiteral,
    StringLiteral,
    NumericLiteral,
    BigIntLiteral,
}

#[derive(Clone)]
pub struct PseudoTypeBase;

#[derive(Clone)]
pub struct PseudoTypeDirect {
    pub type_node: Arc<Node>,
}

#[derive(Clone)]
pub struct PseudoTypeInferred {
    pub expression: Arc<Node>,
    pub error_nodes: Vec<Arc<Node>>,
    pub is_signature_return: bool,
}

#[derive(Clone)]
pub struct PseudoTypeNoResult {
    pub declaration: Arc<Node>,
}

#[derive(Clone)]
pub struct PseudoTypeMaybeConstLocation {
    pub node: Arc<Node>,
    pub const_type: Option<Box<PseudoType>>,
    pub regular_type: Option<Box<PseudoType>>,
}

#[derive(Clone)]
pub struct PseudoTypeUnion {
    pub types: Vec<PseudoType>,
}

#[derive(Clone)]
pub struct PseudoTypeSingleCallSignature {
    pub signature: Arc<Node>,
    pub parameters: Vec<PseudoParameter>,
    pub type_parameters: Vec<Arc<Node>>,
    pub return_type: Option<Box<PseudoType>>,
}

#[derive(Clone)]
pub struct PseudoTypeTuple {
    pub elements: Vec<PseudoType>,
}

#[derive(Clone)]
pub struct PseudoTypeObjectLiteral {
    pub elements: Vec<PseudoObjectElement>,
}

#[derive(Clone)]
pub struct PseudoTypeLiteral {
    pub node: Arc<Node>,
}

#[derive(Clone)]
pub enum PseudoTypeData {
    Base(PseudoTypeBase),
    Direct(PseudoTypeDirect),
    Inferred(PseudoTypeInferred),
    NoResult(PseudoTypeNoResult),
    MaybeConstLocation(PseudoTypeMaybeConstLocation),
    Union(PseudoTypeUnion),
    SingleCallSignature(PseudoTypeSingleCallSignature),
    Tuple(PseudoTypeTuple),
    ObjectLiteral(PseudoTypeObjectLiteral),
    Literal(PseudoTypeLiteral),
}

#[derive(Clone)]
pub struct PseudoType {
    pub kind: PseudoTypeKind,
    pub data: PseudoTypeData,
}

pub fn new_pseudo_type(kind: PseudoTypeKind, data: PseudoTypeData) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type"); 
    PseudoType { kind, data }
}

impl PseudoType {
    pub fn as_pseudo_type(&self) -> &PseudoType { ::tsox_core::fntrace::enter("as_pseudo_type"); 
        self
    }

    pub fn as_pseudo_type_direct(&self) -> Option<&PseudoTypeDirect> { ::tsox_core::fntrace::enter("as_pseudo_type_direct"); 
        match &self.data {
            PseudoTypeData::Direct(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_type_inferred(&self) -> Option<&PseudoTypeInferred> { ::tsox_core::fntrace::enter("as_pseudo_type_inferred"); 
        match &self.data {
            PseudoTypeData::Inferred(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_type_no_result(&self) -> Option<&PseudoTypeNoResult> { ::tsox_core::fntrace::enter("as_pseudo_type_no_result"); 
        match &self.data {
            PseudoTypeData::NoResult(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_type_maybe_const_location(&self) -> Option<&PseudoTypeMaybeConstLocation> { ::tsox_core::fntrace::enter("as_pseudo_type_maybe_const_location"); 
        match &self.data {
            PseudoTypeData::MaybeConstLocation(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_type_union(&self) -> Option<&PseudoTypeUnion> { ::tsox_core::fntrace::enter("as_pseudo_type_union"); 
        match &self.data {
            PseudoTypeData::Union(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_type_single_call_signature(&self) -> Option<&PseudoTypeSingleCallSignature> { ::tsox_core::fntrace::enter("as_pseudo_type_single_call_signature"); 
        match &self.data {
            PseudoTypeData::SingleCallSignature(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_type_tuple(&self) -> Option<&PseudoTypeTuple> { ::tsox_core::fntrace::enter("as_pseudo_type_tuple"); 
        match &self.data {
            PseudoTypeData::Tuple(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_type_object_literal(&self) -> Option<&PseudoTypeObjectLiteral> { ::tsox_core::fntrace::enter("as_pseudo_type_object_literal"); 
        match &self.data {
            PseudoTypeData::ObjectLiteral(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_type_literal(&self) -> Option<&PseudoTypeLiteral> { ::tsox_core::fntrace::enter("as_pseudo_type_literal"); 
        match &self.data {
            PseudoTypeData::Literal(d) => Some(d),
            _ => None,
        }
    }
}

pub fn pseudo_type_undefined() -> PseudoType { ::tsox_core::fntrace::enter("pseudo_type_undefined"); 
    new_pseudo_type(PseudoTypeKind::Undefined, PseudoTypeData::Base(PseudoTypeBase))
}

pub fn pseudo_type_null() -> PseudoType { ::tsox_core::fntrace::enter("pseudo_type_null"); 
    new_pseudo_type(PseudoTypeKind::Null, PseudoTypeData::Base(PseudoTypeBase))
}

pub fn pseudo_type_any() -> PseudoType { ::tsox_core::fntrace::enter("pseudo_type_any"); 
    new_pseudo_type(PseudoTypeKind::Any, PseudoTypeData::Base(PseudoTypeBase))
}

pub fn pseudo_type_string() -> PseudoType { ::tsox_core::fntrace::enter("pseudo_type_string"); 
    new_pseudo_type(PseudoTypeKind::String, PseudoTypeData::Base(PseudoTypeBase))
}

pub fn pseudo_type_number() -> PseudoType { ::tsox_core::fntrace::enter("pseudo_type_number"); 
    new_pseudo_type(PseudoTypeKind::Number, PseudoTypeData::Base(PseudoTypeBase))
}

pub fn pseudo_type_big_int() -> PseudoType { ::tsox_core::fntrace::enter("pseudo_type_big_int"); 
    new_pseudo_type(PseudoTypeKind::BigInt, PseudoTypeData::Base(PseudoTypeBase))
}

pub fn pseudo_type_boolean() -> PseudoType { ::tsox_core::fntrace::enter("pseudo_type_boolean"); 
    new_pseudo_type(PseudoTypeKind::Boolean, PseudoTypeData::Base(PseudoTypeBase))
}

pub fn pseudo_type_false() -> PseudoType { ::tsox_core::fntrace::enter("pseudo_type_false"); 
    new_pseudo_type(PseudoTypeKind::False, PseudoTypeData::Base(PseudoTypeBase))
}

pub fn pseudo_type_true() -> PseudoType { ::tsox_core::fntrace::enter("pseudo_type_true"); 
    new_pseudo_type(PseudoTypeKind::True, PseudoTypeData::Base(PseudoTypeBase))
}

pub fn new_pseudo_type_direct(type_node: Arc<Node>) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_direct"); 
    new_pseudo_type(
        PseudoTypeKind::Direct,
        PseudoTypeData::Direct(PseudoTypeDirect { type_node }),
    )
}

pub fn new_pseudo_type_inferred(expr: Arc<Node>, is_signature_return: bool) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_inferred"); 
    new_pseudo_type(
        PseudoTypeKind::Inferred,
        PseudoTypeData::Inferred(PseudoTypeInferred {
            expression: expr,
            error_nodes: Vec::new(),
            is_signature_return,
        }),
    )
}

pub fn new_pseudo_type_inferred_with_errors(
    expr: Arc<Node>,
    is_signature_return: bool,
    error_nodes: Vec<Arc<Node>>,
) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_inferred_with_errors"); 
    new_pseudo_type(
        PseudoTypeKind::Inferred,
        PseudoTypeData::Inferred(PseudoTypeInferred {
            expression: expr,
            error_nodes,
            is_signature_return,
        }),
    )
}

pub fn new_pseudo_type_no_result(decl: Arc<Node>) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_no_result"); 
    new_pseudo_type(
        PseudoTypeKind::NoResult,
        PseudoTypeData::NoResult(PseudoTypeNoResult { declaration: decl }),
    )
}

pub fn new_pseudo_type_maybe_const_location(
    loc: Arc<Node>,
    const_type: Option<PseudoType>,
    regular_type: Option<PseudoType>,
) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_maybe_const_location"); 
    new_pseudo_type(
        PseudoTypeKind::MaybeConstLocation,
        PseudoTypeData::MaybeConstLocation(PseudoTypeMaybeConstLocation {
            node: loc,
            const_type: const_type.map(Box::new),
            regular_type: regular_type.map(Box::new),
        }),
    )
}

pub fn new_pseudo_type_union(types: Vec<PseudoType>) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_union"); 
    new_pseudo_type(PseudoTypeKind::Union, PseudoTypeData::Union(PseudoTypeUnion { types }))
}

pub fn new_pseudo_type_single_call_signature(
    signature: Arc<Node>,
    parameters: Vec<PseudoParameter>,
    type_parameters: Vec<Arc<Node>>,
    return_type: Option<PseudoType>,
) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_single_call_signature"); 
    new_pseudo_type(
        PseudoTypeKind::SingleCallSignature,
        PseudoTypeData::SingleCallSignature(PseudoTypeSingleCallSignature {
            signature,
            parameters,
            type_parameters,
            return_type: return_type.map(Box::new),
        }),
    )
}

pub fn new_pseudo_type_tuple(elements: Vec<PseudoType>) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_tuple"); 
    new_pseudo_type(PseudoTypeKind::Tuple, PseudoTypeData::Tuple(PseudoTypeTuple { elements }))
}

pub fn new_pseudo_type_object_literal(elements: Vec<PseudoObjectElement>) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_object_literal"); 
    new_pseudo_type(
        PseudoTypeKind::ObjectLiteral,
        PseudoTypeData::ObjectLiteral(PseudoTypeObjectLiteral { elements }),
    )
}

pub fn new_pseudo_type_string_literal(node: Arc<Node>) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_string_literal"); 
    new_pseudo_type(
        PseudoTypeKind::StringLiteral,
        PseudoTypeData::Literal(PseudoTypeLiteral { node }),
    )
}

pub fn new_pseudo_type_numeric_literal(node: Arc<Node>) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_numeric_literal"); 
    new_pseudo_type(
        PseudoTypeKind::NumericLiteral,
        PseudoTypeData::Literal(PseudoTypeLiteral { node }),
    )
}

pub fn new_pseudo_type_big_int_literal(node: Arc<Node>) -> PseudoType { ::tsox_core::fntrace::enter("new_pseudo_type_big_int_literal"); 
    new_pseudo_type(
        PseudoTypeKind::BigIntLiteral,
        PseudoTypeData::Literal(PseudoTypeLiteral { node }),
    )
}

#[derive(Clone)]
pub struct PseudoParameter {
    pub rest: bool,
    pub name: Arc<Node>,
    pub optional: bool,
    pub type_: PseudoType,
}

pub fn new_pseudo_parameter(
    is_rest: bool,
    name: Arc<Node>,
    is_optional: bool,
    type_: PseudoType,
) -> PseudoParameter { ::tsox_core::fntrace::enter("new_pseudo_parameter"); 
    PseudoParameter {
        rest: is_rest,
        name,
        optional: is_optional,
        type_,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PseudoObjectElementKind {
    Method,
    PropertyAssignment,
    SetAccessor,
    GetAccessor,
}

#[derive(Clone)]
pub struct PseudoObjectMethod {
    pub signature: Arc<Node>,
    pub type_parameters: Vec<Arc<Node>>,
    pub parameters: Vec<PseudoParameter>,
    pub return_type: Option<Box<PseudoType>>,
}

#[derive(Clone)]
pub struct PseudoPropertyAssignment {
    pub readonly: bool,
    pub type_: PseudoType,
}

#[derive(Clone)]
pub struct PseudoSetAccessor {
    pub signature: Arc<Node>,
    pub parameter: PseudoParameter,
}

#[derive(Clone)]
pub struct PseudoGetAccessor {
    pub signature: Arc<Node>,
    pub type_: PseudoType,
}

#[derive(Clone)]
pub enum PseudoObjectElementData {
    Method(PseudoObjectMethod),
    PropertyAssignment(PseudoPropertyAssignment),
    SetAccessor(PseudoSetAccessor),
    GetAccessor(PseudoGetAccessor),
}

#[derive(Clone)]
pub struct PseudoObjectElement {
    pub name: Arc<Node>,
    pub optional: bool,
    pub kind: PseudoObjectElementKind,
    pub data: PseudoObjectElementData,
}

pub fn new_pseudo_object_element(
    kind: PseudoObjectElementKind,
    name: Arc<Node>,
    optional: bool,
    data: PseudoObjectElementData,
) -> PseudoObjectElement { ::tsox_core::fntrace::enter("new_pseudo_object_element"); 
    PseudoObjectElement {
        name,
        optional,
        kind,
        data,
    }
}

impl PseudoObjectElement {
    pub fn as_pseudo_object_element(&self) -> &PseudoObjectElement { ::tsox_core::fntrace::enter("as_pseudo_object_element"); 
        self
    }

    pub fn signature(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("signature"); 
        match &self.data {
            PseudoObjectElementData::Method(d) => Some(&d.signature),
            PseudoObjectElementData::SetAccessor(d) => Some(&d.signature),
            PseudoObjectElementData::GetAccessor(d) => Some(&d.signature),
            _ => None,
        }
    }

    pub fn as_pseudo_object_method(&self) -> Option<&PseudoObjectMethod> { ::tsox_core::fntrace::enter("as_pseudo_object_method"); 
        match &self.data {
            PseudoObjectElementData::Method(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_property_assignment(&self) -> Option<&PseudoPropertyAssignment> { ::tsox_core::fntrace::enter("as_pseudo_property_assignment"); 
        match &self.data {
            PseudoObjectElementData::PropertyAssignment(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_set_accessor(&self) -> Option<&PseudoSetAccessor> { ::tsox_core::fntrace::enter("as_pseudo_set_accessor"); 
        match &self.data {
            PseudoObjectElementData::SetAccessor(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pseudo_get_accessor(&self) -> Option<&PseudoGetAccessor> { ::tsox_core::fntrace::enter("as_pseudo_get_accessor"); 
        match &self.data {
            PseudoObjectElementData::GetAccessor(d) => Some(d),
            _ => None,
        }
    }
}

pub fn new_pseudo_object_method(
    signature: Arc<Node>,
    name: Arc<Node>,
    optional: bool,
    type_parameters: Vec<Arc<Node>>,
    parameters: Vec<PseudoParameter>,
    return_type: Option<PseudoType>,
) -> PseudoObjectElement { ::tsox_core::fntrace::enter("new_pseudo_object_method"); 
    new_pseudo_object_element(
        PseudoObjectElementKind::Method,
        name,
        optional,
        PseudoObjectElementData::Method(PseudoObjectMethod {
            signature,
            type_parameters,
            parameters,
            return_type: return_type.map(Box::new),
        }),
    )
}

pub fn new_pseudo_property_assignment(
    readonly: bool,
    name: Arc<Node>,
    optional: bool,
    type_: PseudoType,
) -> PseudoObjectElement { ::tsox_core::fntrace::enter("new_pseudo_property_assignment"); 
    new_pseudo_object_element(
        PseudoObjectElementKind::PropertyAssignment,
        name,
        optional,
        PseudoObjectElementData::PropertyAssignment(PseudoPropertyAssignment { readonly, type_ }),
    )
}

pub fn new_pseudo_set_accessor(
    signature: Arc<Node>,
    name: Arc<Node>,
    optional: bool,
    parameter: PseudoParameter,
) -> PseudoObjectElement { ::tsox_core::fntrace::enter("new_pseudo_set_accessor"); 
    new_pseudo_object_element(
        PseudoObjectElementKind::SetAccessor,
        name,
        optional,
        PseudoObjectElementData::SetAccessor(PseudoSetAccessor { signature, parameter }),
    )
}

pub fn new_pseudo_get_accessor(
    signature: Arc<Node>,
    name: Arc<Node>,
    optional: bool,
    type_: PseudoType,
) -> PseudoObjectElement { ::tsox_core::fntrace::enter("new_pseudo_get_accessor"); 
    new_pseudo_object_element(
        PseudoObjectElementKind::GetAccessor,
        name,
        optional,
        PseudoObjectElementData::GetAccessor(PseudoGetAccessor { signature, type_ }),
    )
}
