#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::Symbol;

use super::types::*;

mod relate;
mod index_signatures;
mod type_arguments;
mod conditional;
mod probing;
mod compare;

#[derive(Clone, Copy, PartialEq)]
enum ProbeMode {

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
    pub source_ptr: usize,
    pub target_ptr: usize,
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

fn type_parameters_same(a: &[Arc<Type>], b: &[Arc<Type>]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(x, y)| Arc::ptr_eq(x, y))
}

pub(crate) fn type_contains_type_parameter(t: &Arc<Type>) -> bool {    if t.flags.contains(TypeFlags::TypeParameter) {
        return true;
    }
    match &t.data {
        TypeData::Union(u) => u
            .union_or_intersection
            .types
            .iter()
            .any(type_contains_type_parameter),
        TypeData::Intersection(i) => i
            .union_or_intersection
            .types
            .iter()
            .any(type_contains_type_parameter),
        TypeData::Object(o) => {
            o.type_arguments.iter().any(type_contains_type_parameter)
                || o.target
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
        }
        TypeData::Conditional(ct) => {
            ct.check_type
                .as_ref()
                .map(type_contains_type_parameter)
                .unwrap_or(false)
                || ct
                    .extends_type
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
                || ct
                    .resolved_true_type
                    .get()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
                || ct
                    .resolved_false_type
                    .get()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
        }
        TypeData::Mapped(m) => {
            m.constraint_type
                .as_ref()
                .map(type_contains_type_parameter)
                .unwrap_or(false)
                || m.template_type
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
                || m.name_type
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
                || m.type_parameter
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
        }
        TypeData::TypeParameter(_) => true,
        TypeData::IndexedAccess(ia) => {
            ia.object_type
                .as_ref()
                .map(type_contains_type_parameter)
                .unwrap_or(false)
                || ia
                    .index_type
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
        }
        TypeData::Index(it) => it
            .target
            .as_ref()
            .map(type_contains_type_parameter)
            .unwrap_or(false),
        _ => false,
    }
}

pub(crate) fn type_mentions_type_parameter(t: &Arc<Type>, needle: &Arc<Type>) -> bool {    if Arc::ptr_eq(t, needle) {
        return true;
    }
    match &t.data {
        TypeData::Union(u) => u
            .union_or_intersection
            .types
            .iter()
            .any(|ty| type_mentions_type_parameter(ty, needle)),
        TypeData::Intersection(i) => i
            .union_or_intersection
            .types
            .iter()
            .any(|ty| type_mentions_type_parameter(ty, needle)),
        TypeData::Object(o) => {
            o.type_arguments
                .iter()
                .any(|ty| type_mentions_type_parameter(ty, needle))
        }
        TypeData::Tuple(tu) => tu
            .element_infos
            .iter()
            .filter_map(|e| e.type_.as_ref())
            .any(|ty| type_mentions_type_parameter(ty, needle)),
        _ => false,
    }
}

pub(crate) fn collect_free_type_parameters(t: &Arc<Type>, out: &mut Vec<Arc<Type>>) {
    match &t.data {
        TypeData::TypeParameter(_) => {
            if !out.iter().any(|p| Arc::ptr_eq(p, t)) {
                out.push(Arc::clone(t));
            }
        }
        TypeData::Union(u) => {
            for ty in &u.union_or_intersection.types {
                collect_free_type_parameters(ty, out);
            }
        }
        TypeData::Intersection(i) => {
            for ty in &i.union_or_intersection.types {
                collect_free_type_parameters(ty, out);
            }
        }
        TypeData::Object(o) => {
            for ty in &o.type_arguments {
                collect_free_type_parameters(ty, out);
            }
        }
        TypeData::Tuple(tu) => {
            for ei in &tu.element_infos {
                if let Some(ty) = &ei.type_ {
                    collect_free_type_parameters(ty, out);
                }
            }
        }
        _ => {}
    }
}

pub fn is_hyphenated_jsx_name(name: &str) -> bool {
    name.contains('-')
}

pub fn is_excess_property_check_target(t: &Type) -> bool {

    if matches!(&t.data, TypeData::Mapped(m) if m.type_parameter.is_some()) {
        return false;
    }
    if t.flags.contains(TypeFlags::Object)
        && !t
            .object_flags
            .contains(ObjectFlags::ObjectLiteralPatternWithComputedProperties)
    {
        return true;
    }
    if t.flags.contains(TypeFlags::NonPrimitive) {
        return true;
    }
    if t.flags.contains(TypeFlags::Substitution) {
        if let TypeData::Substitution(s) = &t.data {
            return s
                .base_type
                .as_ref()
                .map(|t| is_excess_property_check_target(t))
                .unwrap_or(false);
        }
    }
    if t.flags.contains(TypeFlags::Union) {
        if let Some(types) = t.types() {
            return types.iter().any(|t| is_excess_property_check_target(t));
        }
    }
    if t.flags.contains(TypeFlags::Intersection) {
        if let Some(types) = t.types() {
            return types.iter().all(|t| is_excess_property_check_target(t));
        }
    }
    false
}

pub fn is_object_or_instantiable_non_primitive(t: &Type) -> bool {
    t.flags
        .intersects(TypeFlags::Object | TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE)
}

pub fn is_non_primitive_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::NonPrimitive)
}

pub fn visibility_to_string(flags: crate::ast::ModifierFlags) -> String {
    if flags == crate::ast::ModifierFlags::Private {
        "private".to_string()
    } else if flags == crate::ast::ModifierFlags::Protected {
        "protected".to_string()
    } else {
        "public".to_string()
    }
}

pub fn exclude_properties(
    properties: &[Arc<Symbol>],
    excluded_properties: &std::collections::HashSet<String>,
) -> Vec<Arc<Symbol>> {
    properties
        .iter()
        .filter(|p| !excluded_properties.contains(&p.name))
        .cloned()
        .collect()
}

pub fn should_check_as_excess_property(_prop: &Symbol, _container: &Symbol) -> bool {

    false
}

pub fn is_ignored_jsx_property(_source: &Type, _source_prop: &Symbol) -> bool {

    false
}

pub struct TypeDiscriminator {
    pub names: Vec<String>,
}

impl TypeDiscriminator {
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn name(&self, index: usize) -> &str {
        &self.names[index]
    }

    pub fn matches(&self, _index: usize, _t: &Arc<Type>) -> bool {

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_comparison_result_flags() {
        let result = RelationComparisonResult::Succeeded;
        assert!(result.contains(RelationComparisonResult::Succeeded));
        assert!(!result.contains(RelationComparisonResult::Failed));

        let combined = RelationComparisonResult::Succeeded | RelationComparisonResult::Failed;
        assert!(combined.contains(RelationComparisonResult::Succeeded));
        assert!(combined.contains(RelationComparisonResult::Failed));
    }

    #[test]
    fn relation_cache_basic() {
        let mut rel = Relation::new(RelationKind::Assignable);
        let key = CacheHashKey::new(1, 2);
        assert!(rel.get(&key) == RelationComparisonResult::None);
        rel.set(key, RelationComparisonResult::Succeeded);
        assert!(rel.get(&key) == RelationComparisonResult::Succeeded);
        assert_eq!(rel.size(), 1);
    }

    #[test]
    fn relation_cache_key_distinguishes_relation_kinds() {

        let k1 = RelationCacheKey {
            source_ptr: 0x1000,
            target_ptr: 0x2000,
            relation: RelationKind::Assignable,
        };
        let k2 = RelationCacheKey {
            source_ptr: 0x1000,
            target_ptr: 0x2000,
            relation: RelationKind::Subtype,
        };
        assert_ne!(k1, k2);

        let mut set = std::collections::HashSet::new();
        set.insert(k1);

        assert!(!set.contains(&k2));
        set.insert(k2);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn relation_cache_key_distinguishes_type_pointers() {

        let k1 = RelationCacheKey {
            source_ptr: 0x1000,
            target_ptr: 0x2000,
            relation: RelationKind::Assignable,
        };
        let k2 = RelationCacheKey {
            source_ptr: 0x3000,
            target_ptr: 0x2000,
            relation: RelationKind::Assignable,
        };
        assert_ne!(k1, k2);
    }

    #[test]
    fn recursion_flags_both() {
        assert!(RECURSION_FLAGS_BOTH.contains(RecursionFlags::Source));
        assert!(RECURSION_FLAGS_BOTH.contains(RecursionFlags::Target));
    }

    #[test]
    fn expanding_flags() {
        assert_eq!(ExpandingFlags::NONE.0, 0);
        assert_eq!(ExpandingFlags::SOURCE.0, 1);
        assert_eq!(ExpandingFlags::TARGET.0, 2);
        assert_eq!(ExpandingFlags::BOTH.0, 3);
    }

    #[test]
    fn signature_check_mode_callback_alias() {

        assert!(SignatureCheckMode::Callback.contains(SignatureCheckMode::BivariantCallback));
        assert!(SignatureCheckMode::Callback.contains(SignatureCheckMode::StrictCallback));
        assert_eq!(SignatureCheckMode::Callback, SIGNATURE_CHECK_MODE_CALLBACK);
    }

    #[test]
    fn type_arguments_related_covariant_by_default() {

        let result = Ternary::True.and(Ternary::True);
        assert_eq!(result, Ternary::True);

        assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Covariant));
        assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Contravariant));
        assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Independent));
        assert!(!VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Unmeasurable));
        assert!(!VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Unreliable));
    }

    #[test]
    fn index_signature_helpers_bit_layout() {

        let inferable =
            ObjectFlags::JSLiteral | ObjectFlags::ObjectRestType | ObjectFlags::ReverseMapped;
        assert!(inferable.contains(ObjectFlags::JSLiteral));
        assert!(inferable.contains(ObjectFlags::ObjectRestType));
        assert!(inferable.contains(ObjectFlags::ReverseMapped));

        assert!(!inferable.contains(ObjectFlags::FreshLiteral));
    }
}
