#![allow(unused_imports)]

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InferenceKey {
    pub source: TypeId,
    pub target: TypeId,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct InferencePriority: i32 {
        const None                         = 0;
        const NakedTypeVariable            = 1 << 0;
        const SpeculativeTuple             = 1 << 1;
        const SubstituteSource             = 1 << 2;
        const HomomorphicMappedType        = 1 << 3;
        const PartialHomomorphicMappedType = 1 << 4;
        const MappedTypeConstraint         = 1 << 5;
        const ContravariantConditional     = 1 << 6;
        const ReturnType                   = 1 << 7;
        const LiteralKeyof                 = 1 << 8;
        const NoConstraints                = 1 << 9;
        const AlwaysStrict                 = 1 << 10;
        const MaxValue                     = 1 << 11;
        const Circularity                  = -1;

        const PriorityImpliesCombination = Self::ReturnType.bits()
            | Self::MappedTypeConstraint.bits()
            | Self::LiteralKeyof.bits();
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct InferenceFlags: u32 {
        const None                   = 0;
        const NoDefault              = 1 << 0;
        const AnyDefault             = 1 << 1;
        const SkippedGenericFunction = 1 << 2;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ExpandingFlags: u8 {
        const None   = 0;
        const Source = 1 << 0;
        const Target = 1 << 1;
        const Both   = Self::Source.bits() | Self::Target.bits();
    }
}

#[derive(Debug, Clone)]
pub struct InferenceInfo {
    pub type_parameter: Arc<Type>,
    pub candidates: Vec<Arc<Type>>,
    pub candidate_depths: Vec<i32>,
    pub contra_candidates: Vec<Arc<Type>>,
    pub inferred_type: Option<Arc<Type>>,
    pub priority: InferencePriority,
    pub top_level: bool,
    pub is_fixed: bool,
    pub implied_arity: i32,
}

impl InferenceInfo {
    pub fn new(type_parameter: Arc<Type>) -> Self {
        Self {
            type_parameter,
            candidates: Vec::new(),
            candidate_depths: Vec::new(),
            contra_candidates: Vec::new(),
            inferred_type: None,
            priority: InferencePriority::MaxValue,
            top_level: true,
            is_fixed: false,
            implied_arity: -1,
        }
    }
}

pub struct InferenceContext {
    pub inferences: Vec<InferenceInfo>,
    pub signature: Option<Arc<Signature>>,
    pub flags: InferenceFlags,
    pub mapper: Option<Arc<TypeMapper>>,
    pub return_mapper: Option<Arc<TypeMapper>>,
    pub outer_return_mapper: Option<Arc<TypeMapper>>,
}

impl InferenceContext {
    pub fn new(inferences: Vec<InferenceInfo>) -> Self {
        Self {
            inferences,
            signature: None,
            flags: InferenceFlags::None,
            mapper: None,
            return_mapper: None,
            outer_return_mapper: None,
        }
    }
}

pub(crate) struct InferenceState<'a> {
    pub(crate) inferences: &'a mut [InferenceInfo],

    #[allow(dead_code)]
    pub(crate) original_source: Option<Arc<Type>>,
    #[allow(dead_code)]
    pub(crate) original_target: Option<Arc<Type>>,
    pub(crate) priority: InferencePriority,
    pub(crate) inference_priority: InferencePriority,
    pub(crate) contravariant: bool,
    pub(crate) bivariant: bool,
    #[allow(dead_code)]
    pub(crate) expanding_flags: ExpandingFlags,
    pub(crate) propagation_type: Option<Arc<Type>>,

    pub(crate) visited: HashMap<(u32, u32), InferencePriority>,
    pub(crate) depth: i32,
}
