use super::super::types::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ProbeMode {
    Permissive,

    Restrictive,
}

bitflags::bitflags! {

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct SignatureCheckMode: u32 {
        const None              = 0;
        const BivariantCallback = 1 << 0;
        const StrictCallback    = 1 << 1;
        const IgnoreReturnTypes = 1 << 2;
        const StrictArity       = 1 << 3;
        const StrictTopSignature= 1 << 4;
    }
}

pub const SIGNATURE_CHECK_MODE_CALLBACK: SignatureCheckMode =
    SignatureCheckMode::from_bits_truncate(
        SignatureCheckMode::BivariantCallback.bits() | SignatureCheckMode::StrictCallback.bits(),
    );

impl SignatureCheckMode {
    #[allow(non_upper_case_globals)]
    pub const Callback: Self = SIGNATURE_CHECK_MODE_CALLBACK;
}

bitflags::bitflags! {

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct IntersectionState: u32 {
        const None   = 0;
        const Source = 1 << 0;
        const Target = 1 << 1;
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct RecursionFlags: u32 {
        const None   = 0;
        const Source = 1 << 0;
        const Target = 1 << 1;
    }
}

pub const RECURSION_FLAGS_BOTH: RecursionFlags = RecursionFlags::from_bits_truncate(
    RecursionFlags::Source.bits() | RecursionFlags::Target.bits(),
);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ExpandingFlags(pub u8);

impl ExpandingFlags {
    pub const NONE: Self = Self(0);
    pub const SOURCE: Self = Self(1 << 0);
    pub const TARGET: Self = Self(1 << 1);
    pub const BOTH: Self = Self(Self::SOURCE.0 | Self::TARGET.0);
}

bitflags::bitflags! {

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct RelationComparisonResult: u32 {
        const None                = 0;
        const Succeeded           = 1 << 0;
        const Failed              = 1 << 1;
        const ReportsUnmeasurable = 1 << 3;
        const ReportsUnreliable   = 1 << 4;
        const ComplexityOverflow  = 1 << 5;
        const StackDepthOverflow  = 1 << 6;
    }
}

pub const RELATION_COMPARISON_RESULT_REPORTS_MASK: RelationComparisonResult =
    RelationComparisonResult::from_bits_truncate(
        RelationComparisonResult::ReportsUnmeasurable.bits()
            | RelationComparisonResult::ReportsUnreliable.bits(),
    );

pub const RELATION_COMPARISON_RESULT_OVERFLOW: RelationComparisonResult =
    RelationComparisonResult::from_bits_truncate(
        RelationComparisonResult::ComplexityOverflow.bits()
            | RelationComparisonResult::StackDepthOverflow.bits(),
    );

bitflags::bitflags! {

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct MinArgumentCountFlags: u32 {
        const None                    = 0;
        const StrongArityForUntypedJS = 1 << 0;
        const VoidIsNonOptional       = 1 << 1;
    }
}

pub const RELATER_MAX_DEPTH: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RelationKind {
    #[default]
    Identity,
    Subtype,
    StrictSubtype,
    Assignable,
    Comparable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelationCacheKey {
    pub source_id: u32,
    pub target_id: u32,
    pub relation: RelationKind,
}

#[derive(Debug, Clone)]
pub struct RelaterChainEntry {
    pub message: crate::diagnostics::Message,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct Relation {
    pub kind: RelationKind,
    results: HashMap<CacheHashKey, RelationComparisonResult>,
}

impl Relation {
    pub fn new(kind: RelationKind) -> Self {
        Self {
            kind,
            results: HashMap::new(),
        }
    }

    pub fn get(&self, key: &CacheHashKey) -> RelationComparisonResult {
        self.results.get(key).copied().unwrap_or_default()
    }

    pub fn set(&mut self, key: CacheHashKey, result: RelationComparisonResult) {
        self.results.insert(key, result);
    }

    pub fn size(&self) -> usize {
        self.results.len()
    }

    pub fn clear(&mut self) {
        self.results.clear();
    }
}
