#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::types_type_id::TYPE_FLAGS_ANY_OR_UNKNOWN;
use std::sync::Arc;
use tsox_core::core::compiler_options_kinds::ScriptTarget;

#[allow(non_snake_case)]
pub mod LanguageFeatureMinimumTarget {
    use tsox_core::core::compiler_options_kinds::ScriptTarget;

    pub const Exponentiation: ScriptTarget = ScriptTarget::ES2016;
    pub const AsyncFunctions: ScriptTarget = ScriptTarget::ES2017;
    pub const ForAwaitOf: ScriptTarget = ScriptTarget::ES2018;
    pub const AsyncGenerators: ScriptTarget = ScriptTarget::ES2018;
    pub const AsyncIteration: ScriptTarget = ScriptTarget::ES2018;
    pub const ObjectSpreadRest: ScriptTarget = ScriptTarget::ES2018;
    pub const RegularExpressionFlagsDotAll: ScriptTarget = ScriptTarget::ES2018;
    pub const BindinglessCatch: ScriptTarget = ScriptTarget::ES2019;
    pub const BigInt: ScriptTarget = ScriptTarget::ES2020;
    pub const NullishCoalesce: ScriptTarget = ScriptTarget::ES2020;
    pub const OptionalChaining: ScriptTarget = ScriptTarget::ES2020;
    pub const LogicalAssignment: ScriptTarget = ScriptTarget::ES2021;
    pub const TopLevelAwait: ScriptTarget = ScriptTarget::ES2022;
    pub const ClassFields: ScriptTarget = ScriptTarget::ES2022;
    pub const PrivateNamesAndClassStaticBlocks: ScriptTarget = ScriptTarget::ES2022;
    pub const RegularExpressionFlagsHasIndices: ScriptTarget = ScriptTarget::ES2022;
    pub const ShebangComments: ScriptTarget = ScriptTarget::ES2022;
    pub const UsingAndAwaitUsing: ScriptTarget = ScriptTarget::ES2022;
    pub const ClassAndClassElementDecorators: ScriptTarget = ScriptTarget::ES2022;
    pub const RegularExpressionFlagsUnicodeSets: ScriptTarget = ScriptTarget::ES2022;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappedTypeNameTypeKind {
    None,
    Remapping,
    Rename,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnusedKind {
    Parameter,
    Local,
}

pub enum TypeReferenceSerializationKind {
    Unknown,
    TypeWithConstructSignatureAndValue,
    VoidNullableOrNeverType,
    BigIntLikeType,
    BooleanType,
    NumberLikeType,
    StringLikeType,
    ArrayLikeType,
    ESSymbolType,
    TypeWithCallSignature,
    ObjectTypeWithNullPrototype,
    ObjectTypeWithNonNullOrUndefinedPrototype,
    ObjectTypeWithPrototypeOfType,
}

pub type Flags = TypeFlags;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InternalFlags(pub u32);

impl InternalFlags {
    pub const empty: InternalFlags = InternalFlags(0);
    pub fn contains(self, other: InternalFlags) -> bool {
        self.0 & other.0 == other.0
    }
    pub fn intersects(self, other: InternalFlags) -> bool {
        self.0 & other.0 != 0
    }
}

pub type SymbolTracker = dyn crate::checker::symboltracker::SymbolTracker;

pub fn get_string_literal_value(t: &Arc<Type>) -> String {
    match &t.as_literal_type().unwrap().value {
        LiteralValue::String(s) => s.clone(),
        _ => String::new(),
    }
}

pub fn append_type_mapping(
    mapper: Option<&Arc<TypeMapper>>,
    type_parameter: &Arc<Type>,
    key_type: &Arc<Type>,
) -> Option<Arc<TypeMapper>> {
    let mapper = mapper?;
    let key_mapper = crate::checker::mig::w9a::new_type_mapper(
        vec![Arc::clone(type_parameter)],
        vec![Arc::clone(key_type)],
    );
    crate::checker::mapper::merge_type_mappers(Some(mapper.as_ref()), Some(&key_mapper))
        .map(Arc::new)
}

impl Checker {
    pub fn get_awaited_type_no_alias(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        self.get_awaited_type_no_alias_ex(t, None, None, &[])
    }

    pub fn combine_type_mappers(
        &self,
        m1: Option<&Arc<TypeMapper>>,
        m2: Option<&Arc<TypeMapper>>,
    ) -> Option<Arc<TypeMapper>> {
        match m1 {
            Some(m1) => {
                crate::checker::mapper::merge_type_mappers(Some(m1.as_ref()), m2.map(|m| m.as_ref()))
                    .map(Arc::new)
            }
            None => m2.cloned(),
        }
    }

    pub fn get_property_of_type_ex(
        &mut self,
        t: &Arc<Type>,
        name: &str,
        skip_object_function_property_augment: bool,
        include_type_only_members: bool,
    ) -> Option<Arc<Symbol>> {
        let _ = (skip_object_function_property_augment, include_type_only_members);
        self.get_property_of_type(t, name)
    }

    pub fn get_symbol_of_declaration_opt(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.get_symbol_of_declaration(node)
    }

    pub fn get_function_flags(&self, node: &Arc<Node>) -> tsox_frontend::ast::mig::m3e::FunctionFlags {
        tsox_frontend::ast::mig::m3e::get_function_flags(Some(node))
    }

    pub fn get_target_type(&self, t: &Arc<Type>) -> Arc<Type> {
        t.target().cloned().unwrap_or_else(|| Arc::clone(t))
    }
}
