#![allow(unused_imports)]

use super::*;

impl Ternary {
    pub(crate) fn rank(self) -> u8 {
        match self {
            Ternary::False => 0,
            Ternary::Unknown => 1,
            Ternary::Maybe => 2,
            Ternary::True => 3,
        }
    }

    pub fn and(self, other: Ternary) -> Ternary {
        if self.rank() <= other.rank() {
            self
        } else {
            other
        }
    }

    pub fn or(self, other: Ternary) -> Ternary {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }

    pub fn not(self) -> Ternary {
        match self {
            Ternary::True => Ternary::False,
            Ternary::False => Ternary::True,
            Ternary::Unknown => Ternary::Unknown,
            Ternary::Maybe => Ternary::Maybe,
        }
    }

    pub fn is_true(self) -> bool {
        self == Ternary::True
    }
    pub fn is_false(self) -> bool {
        self == Ternary::False
    }
    pub fn is_maybe(self) -> bool {
        self == Ternary::Maybe
    }
    pub fn is_unknown(self) -> bool {
        self == Ternary::Unknown
    }
}

impl PartialOrd for Ternary {
    fn partial_cmp(&self, other: &Ternary) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ternary {
    fn cmp(&self, other: &Ternary) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl From<bool> for Ternary {
    fn from(b: bool) -> Self {
        if b { Ternary::True } else { Ternary::False }
    }
}

impl std::ops::BitAnd for Ternary {
    type Output = Ternary;
    fn bitand(self, other: Ternary) -> Ternary {
        self.and(other)
    }
}

impl std::ops::BitOr for Ternary {
    type Output = Ternary;
    fn bitor(self, other: Ternary) -> Ternary {
        self.or(other)
    }
}

impl std::ops::Not for Ternary {
    type Output = Ternary;
    fn not(self) -> Ternary {
        Ternary::not(self)
    }
}

pub type TypeComparer = fn(&Type, &Type, bool) -> Ternary;

#[derive(Debug)]
pub struct TypeAlias {
    pub symbol: Option<Arc<Symbol>>,
    pub type_arguments: Vec<Arc<Type>>,
}

impl TypeAlias {
    pub fn new(symbol: Option<Arc<Symbol>>, type_arguments: Vec<Arc<Type>>) -> Self {
        Self {
            symbol,
            type_arguments,
        }
    }
}

pub type MapFn = Arc<dyn Fn(&Arc<Type>) -> Arc<Type> + Send + Sync>;

pub struct TypeMapper {
    pub kind: TypeMapperKind,
    pub map_fn: MapFn,
    pub maps_this_only: bool,
}

impl std::fmt::Debug for TypeMapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeMapper")
            .field("kind", &self.kind)
            .field("maps_this_only", &self.maps_this_only)
            .finish()
    }
}

impl Clone for TypeMapper {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            map_fn: Arc::clone(&self.map_fn),
            maps_this_only: self.maps_this_only,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TypeMapperKind {
    #[default]
    Unknown,
    Simple,
    Array,
    Merged,
}

impl TypeMapper {
    pub fn new(map_fn: MapFn, kind: TypeMapperKind, maps_this_only: bool) -> Self {
        Self {
            kind,
            map_fn,
            maps_this_only,
        }
    }

    pub fn map(&self, t: &Arc<Type>) -> Arc<Type> {
        (self.map_fn)(t)
    }

    pub fn maps_this_only(&self) -> bool {
        self.maps_this_only
    }
}

#[derive(Debug)]
pub struct Type {
    pub flags: TypeFlags,
    pub object_flags: ObjectFlags,
    pub id: TypeId,
    pub symbol: Option<Arc<Symbol>>,
    pub alias: Option<Box<TypeAlias>>,
    pub data: TypeData,
}

#[derive(Debug)]
pub enum TypeData {
    Intrinsic(IntrinsicTypeData),
    Literal(LiteralTypeData),
    UniqueESSymbol(UniqueESSymbolTypeData),
    Object(ObjectTypeData),
    Interface(InterfaceTypeData),
    Tuple(TupleTypeData),
    Mapped(MappedTypeData),
    ReverseMapped(ReverseMappedTypeData),
    EvolvingArray(EvolvingArrayTypeData),
    InstantiationExpression(InstantiationExpressionTypeData),
    Union(UnionTypeData),
    Intersection(IntersectionTypeData),
    TypeParameter(TypeParameterData),
    Index(IndexTypeData),
    IndexedAccess(IndexedAccessTypeData),
    TemplateLiteral(TemplateLiteralTypeData),
    StringMapping(StringMappingTypeData),
    Substitution(SubstitutionTypeData),
    Conditional(ConditionalTypeData),
}

#[derive(Debug, Default)]
pub struct ConstrainedTypeData {
    pub resolved_base_constraint: OnceLock<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct StructuredTypeData {
    pub constrained: ConstrainedTypeData,
    pub members: SymbolTable,
    pub properties: Vec<Arc<Symbol>>,
    pub signatures: Vec<Arc<Signature>>,
    pub call_signature_count: usize,
    pub index_infos: Vec<Arc<IndexInfo>>,
    pub object_type_without_abstract_construct_signatures: OnceLock<Arc<Type>>,
}

impl StructuredTypeData {
    pub fn call_signatures(&self) -> &[Arc<Signature>] {
        &self.signatures[..self.call_signature_count]
    }

    pub fn construct_signatures(&self) -> &[Arc<Signature>] {
        &self.signatures[self.call_signature_count..]
    }
}

#[derive(Debug, Default)]
pub struct ObjectTypeData {
    pub structured: StructuredTypeData,
    pub target: Option<Arc<Type>>,
    pub mapper: Option<Arc<TypeMapper>>,

    pub type_arguments: Vec<Arc<Type>>,
}

#[derive(Debug)]
pub struct IntrinsicTypeData {
    pub intrinsic_name: String,
}

#[derive(Debug)]
pub struct LiteralTypeData {
    pub value: LiteralValue,
    pub fresh_type: OnceLock<Arc<Type>>,
    pub regular_type: OnceLock<Arc<Type>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(jsnum::Number),
    BigInt(jsnum::PseudoBigInt),
    Boolean(bool),

    None,
}

impl LiteralValue {
    pub fn to_string(&self) -> String {
        match self {
            LiteralValue::String(s) => format!("\"{}\"", s),
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::BigInt(b) => b.to_string(),
            LiteralValue::Boolean(b) => b.to_string(),
            LiteralValue::None => String::new(),
        }
    }
}

#[derive(Debug)]
pub struct UniqueESSymbolTypeData {
    pub name: String,
}

#[derive(Debug, Default)]
pub struct InterfaceTypeData {
    pub object: ObjectTypeData,
    pub all_type_parameters: Vec<Arc<Type>>,
    pub outer_type_parameter_count: usize,
    pub this_type: Option<Arc<Type>>,
    pub base_types_resolved: bool,
    pub declared_members_resolved: bool,
    pub resolved_base_constructor_type: OnceLock<Arc<Type>>,
    pub resolved_base_types: Vec<Arc<Type>>,
    pub declared_members: SymbolTable,
    pub declared_call_signatures: Vec<Arc<Signature>>,
    pub declared_construct_signatures: Vec<Arc<Signature>>,
    pub declared_index_infos: Vec<Arc<IndexInfo>>,
}
