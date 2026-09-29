use crate::checker::types_type_flags_instantiable_non_primitive::{
    ObjectFlags, OBJECT_FLAGS_CLASS_OR_INTERFACE,
};
use crate::checker::types_type_id::{
    TypeFlags, TYPE_FLAGS_BIG_INT_LIKE, TYPE_FLAGS_BOOLEAN_LIKE, TYPE_FLAGS_ENUM_LIKE,
    TYPE_FLAGS_ES_SYMBOL_LIKE, TYPE_FLAGS_NUMBER_LIKE, TYPE_FLAGS_STRING_LIKE,
    TYPE_FLAGS_STRUCTURED_TYPE, TYPE_FLAGS_TYPE_VARIABLE,
};
use tsox_frontend::ast::{CheckFlags, ModifierFlags, NodeFlags};

pub(crate) use tsox_core::core::compiler_options::ModuleKind;

pub const MODULE_KIND_NONE: ModuleKind = ModuleKind::None;
pub const MODULE_KIND_COMMON_JS: ModuleKind = ModuleKind::CommonJS;
pub const MODULE_KIND_AMD: ModuleKind = ModuleKind::AMD;
pub const MODULE_KIND_UMD: ModuleKind = ModuleKind::UMD;
pub const MODULE_KIND_SYSTEM: ModuleKind = ModuleKind::System;
pub const MODULE_KIND_ES2015: ModuleKind = ModuleKind::ES2015;
pub const MODULE_KIND_ES2020: ModuleKind = ModuleKind::ES2020;
pub const MODULE_KIND_ES2022: ModuleKind = ModuleKind::ES2022;
pub const MODULE_KIND_ES_NEXT: ModuleKind = ModuleKind::ESNext;
pub const MODULE_KIND_NODE16: ModuleKind = ModuleKind::Node16;
pub const MODULE_KIND_NODE18: ModuleKind = ModuleKind::Node18;
pub const MODULE_KIND_NODE20: ModuleKind = ModuleKind::Node20;
pub const MODULE_KIND_NODE_NEXT: ModuleKind = ModuleKind::NodeNext;
pub const MODULE_KIND_PRESERVE: ModuleKind = ModuleKind::Preserve;

pub const NODE_FLAGS_LET: NodeFlags = NodeFlags::Let;
pub const NODE_FLAGS_CONST: NodeFlags = NodeFlags::Const;
pub const NODE_FLAGS_USING: NodeFlags = NodeFlags::Using;
pub const NODE_FLAGS_AWAIT_USING: NodeFlags = NodeFlags::AwaitUsing;
pub const NODE_FLAGS_CONTAINS_THIS: NodeFlags = NodeFlags::ContainsThis;
pub const NODE_FLAGS_BLOCK_SCOPED: NodeFlags = NodeFlags::BlockScoped;
pub const NODE_FLAGS_CONSTANT: NodeFlags = NodeFlags::Constant;
pub const NODE_FLAGS_AMBIENT: NodeFlags = NodeFlags::Ambient;
pub const NODE_FLAGS_SYNTHESIZED: NodeFlags = NodeFlags::Synthesized;
pub const NODE_FLAGS_JAVASCRIPT_FILE: NodeFlags = NodeFlags::JavaScriptFile;

pub const MODIFIER_FLAGS_PUBLIC: ModifierFlags = ModifierFlags::Public;
pub const MODIFIER_FLAGS_PRIVATE: ModifierFlags = ModifierFlags::Private;
pub const MODIFIER_FLAGS_PROTECTED: ModifierFlags = ModifierFlags::Protected;
pub const MODIFIER_FLAGS_READONLY: ModifierFlags = ModifierFlags::Readonly;
pub const MODIFIER_FLAGS_EXPORT: ModifierFlags = ModifierFlags::Export;
pub const MODIFIER_FLAGS_ABSTRACT: ModifierFlags = ModifierFlags::Abstract;
pub const MODIFIER_FLAGS_AMBIENT: ModifierFlags = ModifierFlags::Ambient;
pub const MODIFIER_FLAGS_STATIC: ModifierFlags = ModifierFlags::Static;
pub const MODIFIER_FLAGS_ACCESSOR: ModifierFlags = ModifierFlags::Accessor;
pub const MODIFIER_FLAGS_ASYNC: ModifierFlags = ModifierFlags::Async;
pub const MODIFIER_FLAGS_DEFAULT: ModifierFlags = ModifierFlags::Default;
pub const MODIFIER_FLAGS_CONST: ModifierFlags = ModifierFlags::Const;

pub const CHECK_FLAGS_INSTANTIATED: CheckFlags = CheckFlags::Instantiated;
pub const CHECK_FLAGS_SYNTHETIC_PROPERTY: CheckFlags = CheckFlags::SyntheticProperty;
pub const CHECK_FLAGS_SYNTHETIC_METHOD: CheckFlags = CheckFlags::SyntheticMethod;
pub const CHECK_FLAGS_READONLY: CheckFlags = CheckFlags::Readonly;
pub const CHECK_FLAGS_LATE: CheckFlags = CheckFlags::Late;
pub const CHECK_FLAGS_MAPPED: CheckFlags = CheckFlags::Mapped;
pub const CHECK_FLAGS_CONTAINS_PRIVATE: CheckFlags = CheckFlags::ContainsPrivate;
pub const CHECK_FLAGS_CONTAINS_STATIC: CheckFlags = CheckFlags::ContainsStatic;
pub const CHECK_FLAGS_REVERSE_MAPPED: CheckFlags = CheckFlags::ReverseMapped;

pub const TYPE_FLAGS_ANY: TypeFlags = TypeFlags::Any;
pub const TYPE_FLAGS_UNKNOWN: TypeFlags = TypeFlags::Unknown;
pub const TYPE_FLAGS_UNDEFINED: TypeFlags = TypeFlags::Undefined;
pub const TYPE_FLAGS_NULL: TypeFlags = TypeFlags::Null;
pub const TYPE_FLAGS_VOID: TypeFlags = TypeFlags::Void;
pub const TYPE_FLAGS_STRING: TypeFlags = TypeFlags::String;
pub const TYPE_FLAGS_NUMBER: TypeFlags = TypeFlags::Number;
pub const TYPE_FLAGS_BIGINT: TypeFlags = TypeFlags::BigInt;
pub const TYPE_FLAGS_BOOLEAN: TypeFlags = TypeFlags::Boolean;
pub const TYPE_FLAGS_ES_SYMBOL: TypeFlags = TypeFlags::ESSymbol;
pub const TYPE_FLAGS_STRING_LITERAL: TypeFlags = TypeFlags::StringLiteral;
pub const TYPE_FLAGS_NUMBER_LITERAL: TypeFlags = TypeFlags::NumberLiteral;
pub const TYPE_FLAGS_BIGINT_LITERAL: TypeFlags = TypeFlags::BigIntLiteral;
pub const TYPE_FLAGS_BOOLEAN_LITERAL: TypeFlags = TypeFlags::BooleanLiteral;
pub const TYPE_FLAGS_ENUM_LITERAL: TypeFlags = TypeFlags::EnumLiteral;
pub const TYPE_FLAGS_NON_PRIMITIVE: TypeFlags = TypeFlags::NonPrimitive;
pub const TYPE_FLAGS_NEVER: TypeFlags = TypeFlags::Never;
pub const TYPE_FLAGS_TYPE_PARAMETER: TypeFlags = TypeFlags::TypeParameter;
pub const TYPE_FLAGS_OBJECT: TypeFlags = TypeFlags::Object;
pub const TYPE_FLAGS_UNION: TypeFlags = TypeFlags::Union;
pub const TYPE_FLAGS_INTERSECTION: TypeFlags = TypeFlags::Intersection;
pub const TYPE_FLAGS_INDEX: TypeFlags = TypeFlags::Index;
pub const TYPE_FLAGS_TEMPLATE_LITERAL: TypeFlags = TypeFlags::TemplateLiteral;
pub const TYPE_FLAGS_STRING_MAPPING: TypeFlags = TypeFlags::StringMapping;
pub const TYPE_FLAGS_SUBSTITUTION: TypeFlags = TypeFlags::Substitution;
pub const TYPE_FLAGS_INDEXED_ACCESS: TypeFlags = TypeFlags::IndexedAccess;
pub const TYPE_FLAGS_CONDITIONAL: TypeFlags = TypeFlags::Conditional;
pub const TYPE_FLAGS_BIGINT_LIKE: TypeFlags = TYPE_FLAGS_BIG_INT_LIKE;

impl TypeFlags {
    pub const STRING_OR_NUMBER_LITERAL: Self =
        Self::from_bits_truncate(Self::StringLiteral.bits() | Self::NumberLiteral.bits());
    pub const ENUM_LIKE: Self = crate::checker::types_type_id::TYPE_FLAGS_ENUM_LIKE;
    pub const STRUCTURED_OR_INSTANTIABLE: Self = Self::from_bits_truncate(
        TYPE_FLAGS_STRUCTURED_TYPE.bits()
            | (TYPE_FLAGS_TYPE_VARIABLE.bits() | Self::Conditional.bits() | Self::Substitution.bits())
            | TYPE_FLAGS_STRING_LIKE.bits()
            | TYPE_FLAGS_NUMBER_LIKE.bits()
            | TYPE_FLAGS_ES_SYMBOL_LIKE.bits()
            | TYPE_FLAGS_BIG_INT_LIKE.bits()
            | TYPE_FLAGS_BOOLEAN_LIKE.bits(),
    );
}

impl ObjectFlags {
    pub const CLASS_OR_INTERFACE: Self = OBJECT_FLAGS_CLASS_OR_INTERFACE;
}

pub enum ConstantValue {
    String(String),
    Number(f64),
    BigInt(tsox_core::jsnum::PseudoBigInt),
}
