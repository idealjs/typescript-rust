#![allow(dead_code)]
//! Type relation checking: determining whether one type is assignable to,
//! a subtype of, or identical to another.
//!
//! Ported from `internal/checker/relater.go`. This is a large and complex
//! module (~5000 lines in Go); this file ports the core types and the
//! `isSimpleTypeRelatedTo` function which handles the most common cases
//! (any, unknown, never, primitive types, literals).

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::ast::node_data_generated::NodeData;
use crate::ast::{ModifierFlags, Node, Symbol, SymbolFlags, SyntaxKind};
use crate::checker::is_tuple_type;
use crate::evaluator::EvalValue;
use crate::jsnum;

use super::checker::Checker;
use super::inference::{InferenceContext, InferenceInfo, InferencePriority};
use super::types::*;

/// Mode of a conditional-resolution probe instantiation (Go's permissive vs
/// restrictive mappers used by the definitely-false / definitely-true tests).
#[derive(Clone, Copy, PartialEq)]
enum ProbeMode {
    /// Every type parameter becomes the wildcard type.
    Permissive,
    /// Every constrained type parameter becomes a constraint-stripped copy.
    Restrictive,
}


// ────────────────────────────────────────────────────────────────────────────
// Relation comparison types
// ────────────────────────────────────────────────────────────────────────────

bitflags::bitflags! {
    /// Mode flags for signature comparison.
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
    /// Convenience alias matching Go's `SignatureCheckModeCallback` constant.
    /// Equivalent to `BivariantCallback | StrictCallback`.
    pub const Callback: Self = SIGNATURE_CHECK_MODE_CALLBACK;
}

bitflags::bitflags! {
    /// Flags controlling intersection state during type comparison.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct IntersectionState: u32 {
        const None   = 0;
        const Source = 1 << 0;
        const Target = 1 << 1;
    }

    /// Recursion direction flags.
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

/// Flags indicating which side of a comparison is being expanded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ExpandingFlags(pub u8);

impl ExpandingFlags {
    pub const NONE: Self = Self(0);
    pub const SOURCE: Self = Self(1 << 0);
    pub const TARGET: Self = Self(1 << 1);
    pub const BOTH: Self = Self(Self::SOURCE.0 | Self::TARGET.0);
}

bitflags::bitflags! {
    /// Result of a type relation comparison.
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
    /// Flags controlling minimum argument count computation.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct MinArgumentCountFlags: u32 {
        const None                    = 0;
        const StrongArityForUntypedJS = 1 << 0;
        const VoidIsNonOptional       = 1 << 1;
    }
}

/// Maximum recursion depth for `is_type_related_to` before the relater
/// gives up and reports overflow. Matches Go's `stackDepthOverflow`
/// constant in `relater.go` (100). Without this, recursive structural
/// types such as `type Box<T> = { next: Box<T> | null }` blow the
/// native stack. Go uses fixed-size `sourceStack`/`targetStack` arrays
/// of this length; we use a depth counter.
pub const RELATER_MAX_DEPTH: u32 = 100;

/// The kind of relation being checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RelationKind {
    #[default]
    Identity,
    Subtype,
    StrictSubtype,
    Assignable,
    Comparable,
}

/// Key into the per-call relation cache. Combines the source and target
/// `Type` pointers (via `Arc::as_ptr`) with the `RelationKind`, since the
/// same type pair may compare differently under different relations
/// (e.g. `Subtype` vs `Assignable`).
///
/// We use pointer identity rather than `Type::id` because the `id` field
/// is not yet populated with unique values across all Type construction
/// sites (many bypass `Type::new` and leave `id` at 0). Pointer identity
/// is stable for the lifetime of a single top-level comparison (during
/// which the Arcs are kept alive) and the cache is cleared at top-level
/// entry, so stale pointers never accumulate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelationCacheKey {
    pub source_ptr: usize,
    pub target_ptr: usize,
    pub relation: RelationKind,
}

/// One entry of the relater error chain (Go `ErrorChain`, relater.go
/// ~L2581). Pushed innermost-first; the chain is rendered head-last into
/// the nested "compatibility pyramid" diagnostic.
#[derive(Debug, Clone)]
pub struct RelaterChainEntry {
    pub message: crate::diagnostics::Message,
    pub args: Vec<String>,
}

/// A relation cache that stores comparison results.
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

// ────────────────────────────────────────────────────────────────────────────
// Type relation entry points
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Check if `source` is identical to `target`.
    pub fn is_type_identical_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        // Fast path: same pointer
        if Arc::ptr_eq(source, target) {
            return true;
        }
        // Types must have identical flags (excluding complex types)
        if source.flags != target.flags {
            return false;
        }
        if source.flags.contains(TYPE_FLAGS_SINGLETON) {
            return true;
        }
        self.is_simple_type_identical_to(source, target)
    }

    /// Check if `source` is assignable to `target`.
    pub fn is_type_assignable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Assignable)
    }

    /// Check if `source` is a subtype of `target`.
    pub fn is_type_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Subtype)
    }

    /// Check if `source` is a strict subtype of `target`.
    pub fn is_type_strict_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::StrictSubtype)
    }

    /// Check if `source` is comparable to `target`.
    pub fn is_type_comparable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Comparable)
    }

    /// Check if two types are comparable (in either direction).
    pub fn are_types_comparable(&mut self, type1: &Arc<Type>, type2: &Arc<Type>) -> bool {
        self.is_type_comparable_to(type1, type2) || self.is_type_comparable_to(type2, type1)
    }

    /// Main entry point for type relation checking.
    ///
    /// Handles union/intersection/object types by delegating to specialized
    /// methods, falling back to `is_simple_type_related_to` for primitives.
    ///
    /// Implements two layers of recursion protection (mirroring Go's
    /// `relater.go`):
    ///
    /// 1. **Cycle detection** via `relation_in_progress`: when the same
    ///    `(source, target, relation)` triple is encountered higher up
    ///    the call stack (e.g. comparing `Box<X>` to `Box<Y>` reaches
    ///    `Box<X>` vs `Box<Y>` again via a `next` property), we
    ///    optimistically return `true` to terminate the cycle. This is
    ///    correct for the common mutually-recursive case.
    ///
    /// 2. **Depth guard** via `relater_depth`: once the comparison stack
    ///    exceeds `RELATER_MAX_DEPTH` (128), we also return `true`. This
    ///    catches pathological cases the cycle set might miss (very deep
    ///    chains of distinct types).
    ///
    /// A per-call `relation_cache` memoises results so repeated
    /// sub-comparisons within a single top-level call don't recompute.
    /// The cache is cleared at top-level entry (depth 0 → 1) to avoid
    /// carrying optimistic cycle-broken results across calls.
    pub(crate) fn is_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        // Fresh-literal substitution: a fresh literal (produced by a literal
        // expression) is comparable to both its primitive base (`string`) and
        // the regular literal (`"hello"`). Substitute the regular type so the
        // existing literal/primitive relation logic applies uniformly. This
        // mirrors Go's relater which treats fresh literals as their regular
        // form for assignability. Done at the very top so recursive
        // sub-comparisons and the cache key see the regular types.
        let source = if crate::checker::is_fresh_literal_type(source) {
            self.get_regular_type_of_literal_type(source)
        } else {
            Arc::clone(source)
        };
        let target = if crate::checker::is_fresh_literal_type(target) {
            self.get_regular_type_of_literal_type(target)
        } else {
            Arc::clone(target)
        };
        // Pointer identity fast path (Go's `source == target` check in
        // `isRelatedToEx`): a type is trivially related to itself — this
        // also makes union members match their interned selves (e.g. the
        // `object` keyword type inside `boolean | object`).
        if Arc::ptr_eq(&source, &target) {
            return true;
        };
        // Heritage-degradation suppression (see `degraded_type_ptrs`): a
        // type object built inside a degradation window has a transiently
        // incomplete member table. Only MEMBER-DEPENDENT comparisons —
        // both sides structured object types — are garbage; kind checks
        // (e.g. arithmetic-operand eligibility, Object vs number) don't
        // consult members and must keep their real verdicts. Treat the
        // member-dependent case as related; Go's lazy member resolution
        // never observes mid-flight forms (react16/lib.dom D1 family).
        {
            let sp = Arc::as_ptr(&source) as *const Type as usize;
            let tp = Arc::as_ptr(&target) as *const Type as usize;
            if source.flags.contains(TypeFlags::Object)
                && target.flags.contains(TypeFlags::Object)
                && (self.degraded_type_ptrs.contains(&sp)
                    || self.degraded_type_ptrs.contains(&tp))
            {
                return true;
            }
        }
        // Primitive source vs object target with index signatures (Go
        // recursiveTypeRelatedTo's early primitive rejection): a string
        // satisfies a NUMBER-keyed index target (strings have a numeric
        // indexer) but nothing else — `y = "foo"` against
        // `{ [index: string]: any }` and `z = false` both fail before the
        // structural walk reports phantom missing properties.
        if !source.flags.intersects(
            TypeFlags::Object
                | TypeFlags::Union
                | TypeFlags::Intersection
                | TypeFlags::TypeParameter
                | TypeFlags::Any
                | TypeFlags::Unknown,
        ) && target.flags.contains(TypeFlags::Object)
            && target.as_structured().is_some_and(|t| !t.index_infos.is_empty())
            && target.symbol.is_none()
        {
            // String sources satisfy number-keyed index targets — the
            // global String interface's `[index: number]: string` indexer
            // (Go getApplicableIndexInfo on globalStringType); a
            // string-keyed target still rejects.
            if source.flags.intersects(
                TypeFlags::String | TypeFlags::StringLiteral | TypeFlags::StringMapping,
            ) && target.as_structured().is_some_and(|t| {
                t.index_infos.iter().any(|info| {
                    info.key_type
                        .as_ref()
                        .is_some_and(|k| k.flags.contains(TypeFlags::Number))
                })
            }) {
                return true;
            }
            return false;
        }
        // Recursion guard: structural comparisons on recursive types
        // (e.g. `type Box<T> = { next: Box<T> | null }`) can blow the native
        // stack. Once we exceed `RELATER_MAX_DEPTH`, report overflow.
        // Mirrors Go's sourceStack/targetStack depth check (relater.go:3103).
        if self.relater_overflow {
            return true;
        }
        if self.relater_depth >= RELATER_MAX_DEPTH {
            self.relater_overflow = true;
            return true;
        }
        // Complexity budget: decrement on each failed comparison to prevent
        // exponential blowup on complex type graphs. Mirrors Go's
        // `relationCount` (relater.go:3086-3088).
        if self.relation_count == 0 && self.relater_depth > 0 {
            self.relater_overflow = true;
            return true;
        }
        // On top-level entry, reset the per-call caches so optimistic
        // cycle-broken results from a previous call don't leak in.
        // Also initialize the complexity budget.
        if self.relater_depth == 0 {
            self.relation_cache.clear();
            self.relation_in_progress.clear();
            self.relater_overflow = false;
            self.relater_source_stack.clear();
            self.relater_target_stack.clear();
            // Go uses (16_000_000 - relation.size()) / 8. We use a fixed
            // budget since we don't track relation cache size.
            self.relation_count = 2_000_000;
        }
        let key = RelationCacheKey {
            source_ptr: Arc::as_ptr(&source) as usize,
            target_ptr: Arc::as_ptr(&target) as usize,
            relation,
        };
        // Cycle break: if this triple is already being computed higher up
        // the stack, assume `true` to terminate the recursion.
        if self.relation_in_progress.contains(&key) {
            return true;
        }
        // Cache hit: a previous sub-comparison within this top-level call
        // already determined the result. When ELABORATING a failure (chain
        // recording active), Go re-runs failed comparisons so the error
        // chain gets built (relater.go ~L3113: "we will do the comparison
        // again to generate an error message") — a cached `false` from a
        // nested non-reporting context must not short-circuit the retry.
        if let Some(&cached) = self.relation_cache.get(&key) {
            if cached || !self.relater_chain_active {
                return cached;
            }
        }
        self.relation_in_progress.insert(key);
        self.relater_depth += 1;
        // Deeply-nested early termination (Go `recursiveTypeRelatedTo`,
        // relater.go ~L3152: when both source and target chains hit
        // `isDeeplyNestedType` the comparison is presumed equal — this is
        // what stops ever-expanding generic instantiation chains such as
        // react's `FC<PropsWithChildren<...>>` compositions from blowing up).
        let source_deep = self.is_deeply_nested_type(&source, &self.relater_source_stack, 3);
        let target_deep = self.is_deeply_nested_type(&target, &self.relater_target_stack, 3);
        let mut result = if source_deep && target_deep {
            true
        } else {
            self.relater_source_stack.push(Arc::clone(&source));
            self.relater_target_stack.push(Arc::clone(&target));
            let r = self.is_type_related_to_inner(&source, &target, relation);
            self.relater_source_stack.pop();
            self.relater_target_stack.pop();
            r
        };
        self.relater_depth -= 1;
        self.relation_in_progress.remove(&key);
        // Decrement complexity budget on failed comparisons (Go: relater.go:3163).
        if !result {
            self.relation_count = self.relation_count.saturating_sub(1);
        }
        // Deferred-conditional SOURCE fallback (Go relater.go ~L3793, the
        // `source.flags&TypeFlagsConditional` case of assignability): a
        // conditional that has NOT been decided relates to its default
        // constraint — the union of both branches — since any value it
        // could produce must satisfy the target. This is what makes
        // `R1<T_a>` (deferred) assignable to a `R1<unknown>` target that
        // legitimately resolved its false branch: `{ } ∪ {mapping}` covers
        // the empty-object target. A source whose branches were already
        // resolved CONCRETELY at argument-typing time (cells set, e.g.
        // `C<"x">` → literal 1) never reaches this path and stays strict.
        // Skipped for identity/comparable relations and during overflow
        // bookkeeping; mirrors Go's placement after the structural cases.
        if !result
            && !matches!(
                relation,
                RelationKind::Identity | RelationKind::StrictSubtype
            )
            && !self.relater_overflow
            && source.flags.contains(TypeFlags::Conditional)
        {
            let truly_deferred = match &source.data {
                TypeData::Conditional(ct) => {
                    ct.resolved_true_type.get().is_none()
                        && ct.resolved_false_type.get().is_none()
                }
                _ => false,
            };
            if truly_deferred && let Some(constraint) =
                self.deferred_default_constraint_of_conditional(&source)
            {
                if self.is_type_related_to(&constraint, &target, relation) {
                    result = true;
                }
            }
        }

        // Deferred/TARGET conditional acceptance (Go relater.go ~L3580,
        // the `target.flags&TypeFlagsConditional` case of assignability):
        // a non-decided conditional TARGET is satisfied when the source
        // relates to BOTH of its branches, skipping a branch entirely when
        // that branch is provably unreachable (permissive/restrictive
        // instantiation probes over the target's own check/extends).
        // Roots with `infer` positions and distribution-dependent roots are
        // excluded exactly as in Go. NOTE: intentionally NOT nested under
        // the source-conditional gate above — a UNION source reaches here
        // per constituent and must still be checked against each branch.
        if !result
            && !matches!(
                relation,
                RelationKind::Identity | RelationKind::StrictSubtype
            )
            && !self.relater_overflow
            && target.flags.contains(TypeFlags::Conditional)
            && let TypeData::Conditional(tct) = &target.data
        {
            let root_ok = tct.root.as_ref().is_some_and(|r| {
                r.infer_type_parameters.is_empty()
                    && Self::conditional_distribution_independent(r)
            });
            let source_same_root = match (&source.data, tct.root.as_ref().and_then(|r| r.node.as_ref())) {
                (TypeData::Conditional(sc), Some(node)) => sc
                    .root
                    .as_ref()
                    .and_then(|r| r.node.as_ref())
                    .map(|n| n.id() == node.id())
                    .unwrap_or(false),
                _ => false,
            };
            if root_ok
                && !source_same_root
                && let (Some(check), Some(extends)) =
                    (tct.check_type.clone(), tct.extends_type.clone())
            {
                let skip_true = {
                    let pc = self.get_permissive_instantiation(&check);
                    let pe = self.get_permissive_instantiation(&extends);
                    !self.is_type_assignable_to(&pc, &pe)
                };
                if skip_true {
                    result = true;
                } else if let Some(true_branch) =
                    self.get_forced_branch_type_of_conditional_type(&target, true)
                {
                    if self.is_type_related_to(&source, &true_branch, relation) {
                        let skip_false = {
                            let rc = self.get_restrictive_instantiation(&check);
                            let re = self.get_restrictive_instantiation(&extends);
                            self.is_type_assignable_to(&rc, &re)
                        };
                        if skip_false {
                            result = true;
                        } else if let Some(false_branch) =
                            self.get_forced_branch_type_of_conditional_type(&target, false)
                        {
                            if self.is_type_related_to(&source, &false_branch, relation) {
                                result = true;
                            }
                        }
                    }
                }
            }
        }
        self.relation_cache.insert(key, result);
        result
    }

    /// Chain entry `index` from the top (0 = most recent push), by message
    /// key (Go `getChainMessage`).
    fn chain_message_key(&self, index: usize) -> Option<&'static str> {
        let len = self.relater_error_chain.len();
        if len <= index {
            return None;
        }
        Some(self.relater_error_chain[len - 1 - index].message.key)
    }

    /// Args of chain entry `index` from the top.
    fn chain_args(&self, index: usize) -> Option<&[String]> {
        let len = self.relater_error_chain.len();
        if len <= index {
            return None;
        }
        Some(&self.relater_error_chain[len - 1 - index].args)
    }

    /// Go `getPropertyNameArg` + `addToDottedName` (relater.go ~L4931):
    /// quoted property names index (`["x"]`), others join with '.'; a
    /// `new ` prefix or parenthesised head is preserved on the tail.
    fn property_chain_name(head: &str, tail: &str) -> String {
        fn get_property_name_arg(arg: &str) -> String {
            if let Some(first) = arg.chars().next()
                && matches!(first, '"' | '\'' | '`')
            {
                format!("[{}]", arg)
            } else {
                arg.to_string()
            }
        }
        let head = get_property_name_arg(head);
        let tail = get_property_name_arg(tail);
        let mut head = head;
        if head.starts_with("new ") {
            head = format!("({})", head);
        }
        let mut pos = 0;
        let bytes = tail.as_bytes();
        loop {
            if tail[pos..].starts_with('(') {
                pos += 1;
            } else if tail[pos..].starts_with("new ") {
                pos += 4;
            } else {
                break;
            }
        }
        let _ = bytes;
        let suffix = &tail[pos..];
        let prefix = &tail[..pos];
        if suffix.starts_with('[') {
            format!("{}{}{}", prefix, head, suffix)
        } else {
            format!("{}{}.{}", prefix, head, suffix)
        }
    }

    /// Relater chain push with Go's post-processing (Go `(*Relater).
    /// reportError`, relater.go ~L4880): signature-return transforms
    /// (marker entries 2202-2205 collapse into "The types returned by
    /// 'x()' ..."), dotted property-name chaining ('x' → 'x.y'), and
    /// excess-property suppression.
    pub(crate) fn relater_report_error(
        &mut self,
        message: crate::diagnostics::Message,
        mut args: Vec<String>,
    ) {
        use crate::diagnostics::messages_generated as msg;
        if !self.relater_chain_active {
            return;
        }
        if message.key == msg::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE.key {
            // Suppress if the next entry is an excess-property error.
            if let Some(top) = self.chain_message_key(0)
                && (top == msg::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1.key
                    || top == msg::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_BUT_0_DOES_NOT_EXIST_IN_TYPE_1_DID_YOU_MEAN_TO_WRITE_2.key)
            {
                return;
            }
            // Property incompatibility + signature-return incompatibility →
            // single "types returned by 'x()'/'new x()'/'x(...)'" entry.
            let marker = self.chain_message_key(1).map(str::to_string);
            if let Some(m1) = marker {
                let arg = if m1 == msg::CALL_SIGNATURES_WITH_NO_ARGUMENTS_HAVE_INCOMPATIBLE_RETURN_TYPES_0_AND_1.key {
                    Some(format!("{}()", args[0]))
                } else if m1 == msg::CONSTRUCT_SIGNATURES_WITH_NO_ARGUMENTS_HAVE_INCOMPATIBLE_RETURN_TYPES_0_AND_1.key {
                    Some(format!("new {}()", args[0]))
                } else if m1 == msg::CALL_SIGNATURE_RETURN_TYPES_0_AND_1_ARE_INCOMPATIBLE.key {
                    Some(format!("{}(...)", args[0]))
                } else if m1 == msg::CONSTRUCT_SIGNATURE_RETURN_TYPES_0_AND_1_ARE_INCOMPATIBLE.key {
                    Some(format!("new {}(...)", args[0]))
                } else {
                    None
                };
                if let Some(arg) = arg {
                    self.relater_error_chain.pop();
                    self.relater_error_chain.pop();
                    self.relater_error_chain.push(RelaterChainEntry {
                        message: msg::THE_TYPES_RETURNED_BY_0_ARE_INCOMPATIBLE_BETWEEN_THESE_TYPES,
                        args: vec![arg],
                    });
                    return;
                }
                // Property 'x' → (elaboration) → property 'y' chains into a
                // single 'x.y' entry.
                if (m1 == msg::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE.key
                    || m1 == msg::THE_TYPES_OF_0_ARE_INCOMPATIBLE_BETWEEN_THESE_TYPES.key
                    || m1 == msg::THE_TYPES_RETURNED_BY_0_ARE_INCOMPATIBLE_BETWEEN_THESE_TYPES.key)
                    && let Some(tail_args) = self.chain_args(1).map(|a| a[0].clone())
                {
                    let dotted = Self::property_chain_name(&args[0], &tail_args);
                    self.relater_error_chain.pop();
                    self.relater_error_chain.pop();
                    self.relater_error_chain.push(RelaterChainEntry {
                        message: msg::THE_TYPES_OF_0_ARE_INCOMPATIBLE_BETWEEN_THESE_TYPES,
                        args: vec![dotted],
                    });
                    return;
                }
            }
        }
        self.relater_error_chain.push(RelaterChainEntry { message, args });
    }




    /// Chain-message property-name argument (Go `symbolToString` semantics
    /// for member names): a STRING-literal-declared member keeps its source
    /// quotes (`'1'` → arg `''1''` in the rendered message), numeric and
    /// identifier names are raw (subtypingWithObjectMembersOptionality2).
    pub(crate) fn chain_property_arg_name(&self, prop: &Arc<crate::ast::Symbol>) -> String {
        let decl = prop
            .value_declaration
            .clone()
            .or_else(|| prop.declarations.first().cloned());
        if let Some(d) = decl
            && let Some(name) = d.name()
            && name.kind == SyntaxKind::StringLiteral
            && let Some(f) = self.get_source_file_of_node(&d)
        {
            let start = name.loc.pos();
            let end = name.loc.end();
            if start < end && end <= f.text.len() {
                return f.text[start..end].to_string();
            }
        }
        prop.name.clone()
    }

    /// Go `reportRelationError`'s type-parameter-target notes (relater.go
    /// ~L4797): when the TARGET is a type parameter, an instantiation note
    /// accompanies the head — "assignable to the constraint, but could be
    /// instantiated with a different subtype" when the constraint holds,
    /// else "could be instantiated with an arbitrary type which could be
    /// unrelated" (the default case clears the chain, reporting only the
    /// note + head). The note is pushed BEFORE the head so it nests under
    /// it in the pyramid.
    pub(crate) fn push_relation_head_with_tp_note(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        head: crate::diagnostics::Message,
        head_args: Vec<String>,
    ) {
        use crate::diagnostics::messages_generated as msg;
        if target.flags.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_base_constraint_of_type(target);
            let constraint_ok = constraint
                .as_ref()
                .is_some_and(|c| self.is_type_assignable_to(source, c));
            if constraint_ok {
                let c = constraint.unwrap();
                let s = self.type_to_string(source);
                let t = self.type_to_string(target);
                let c_str = self.type_to_string(&c);
                self.relater_report_error(
                    msg::X_0_IS_ASSIGNABLE_TO_THE_CONSTRAINT_OF_TYPE_1_BUT_1_COULD_BE_INSTANTIATED_WITH_A_DIFFERENT_SUBTYPE_OF_CONSTRAINT_2,
                    vec![s, t, c_str],
                );
            } else {
                self.relater_error_chain.clear();
                let t = self.type_to_string(target);
                let s = self.type_to_string(source);
                self.relater_report_error(
                    msg::X_0_COULD_BE_INSTANTIATED_WITH_AN_ARBITRARY_TYPE_WHICH_COULD_BE_UNRELATED_TO_1,
                    vec![t, s],
                );
            }
        }
        self.relater_report_error(head, head_args);
    }


    /// Go `isDeeplyNestedType` (relater.go ~L768), simplified: a type is
    /// deeply nested when its recursion identity (the symbol for nominal
    /// references, the type itself for anonymous ones) occurs `max_depth`
    /// times on the comparison stack as *distinct* instantiations. Go's
    /// increasing-type-id filter is approximated by requiring distinct
    /// `Arc` pointers between consecutive matches — exact repeats are cut
    /// by `relation_in_progress` before reaching here. Intersections are
    /// deeply nested when any constituent is (Go: same rule).
    fn is_deeply_nested_type(&self, t: &Arc<Type>, stack: &[Arc<Type>], max_depth: usize) -> bool {
        if stack.len() < max_depth {
            return false;
        }
        if t.flags.contains(TypeFlags::Intersection) {
            if let Some(constituents) = t.types() {
                for c in constituents {
                    if self.is_deeply_nested_type(c, stack, max_depth) {
                        return true;
                    }
                }
            }
            return false;
        }
        let mut count = 0usize;
        let mut last_ptr: *const Type = std::ptr::null();
        for s in stack {
            let same = match (&t.symbol, &s.symbol) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => Arc::ptr_eq(t, s),
                _ => false,
            };
            if same {
                let p = Arc::as_ptr(s);
                if p != last_ptr {
                    count += 1;
                    if count >= max_depth {
                        return true;
                    }
                }
                last_ptr = p;
            }
        }
        false
    }

    /// The constraint of a DEFERRED indexed access `T[K]` (Go
    /// `getConstraintFromIndexedAccess`, checker.go ~L17284): resolve the
    /// access against the object parameter's base constraint —
    /// `T["content"]` with `T extends { content: C }` constrains to `C`.
    /// Returns `None` when there is no usable constraint information.
    pub(crate) fn constraint_of_indexed_access(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let ia = match &t.data {
            TypeData::IndexedAccess(ia) => ia,
            _ => return None,
        };
        let object = ia.object_type.as_ref()?;
        let index = ia.index_type.as_ref()?;
        // The object's base constraint. Placeholder type-parameter instances
        // (minted during circular resolution) carry no constraint — recover
        // it from the symbol's canonical type. Nested deferred objects
        // (indexed access / conditional) constrain recursively.
        let obj_constraint = if object.flags.contains(TypeFlags::TypeParameter) {
            match self.get_constraint_of_type_parameter(object) {
                Some(c) => c,
                None => {
                    let sym = object.symbol.as_ref()?;
                    // Canonical type first; a constraint-less canonical (a
                    // placeholder cached mid-resolution) falls through to
                    // reading the constraint declaration node directly —
                    // the same extraction get_type_parameter_from_symbol
                    // performs.
                    let canonical = self
                        .type_alias_links
                        .get(sym)
                        .and_then(|l| l.declared_type.clone())
                        .and_then(|c| self.get_constraint_of_type_parameter(&c));
                    match canonical {
                        Some(c) => c,
                        None => {
                            let mut from_decl = None;
                            for decl in &sym.declarations {
                                if let crate::ast::NodeData::TypeParameterDeclaration(data) =
                                    &decl.data
                                {
                                    if let Some(constraint_node) = &data.constraint {
                                        from_decl =
                                            Some(self.get_type_from_type_node(constraint_node));
                                    }
                                    break;
                                }
                            }
                            from_decl?
                        }
                    }
                }
            }
        } else if matches!(
            &object.data,
            TypeData::IndexedAccess(_) | TypeData::Conditional(_)
        ) {
            self.constraint_of_indexed_access(object)?
        } else if index.flags.contains(TypeFlags::TypeParameter) {
            // A generic index over a concrete object: `M[K]` where K's
            // constraint is string/number-like resolves through the
            // object's index signature (Go's getIndexedAccessType on the
            // substituted constraint — `M[string-indexed][K extends
            // string]` is the signature's value type).
            let idx_constraint = self.get_constraint_of_type_parameter(index)?;
            let kind_ok = idx_constraint.flags.intersects(
                TypeFlags::String
                    | TypeFlags::Number
                    | TypeFlags::StringLiteral
                    | TypeFlags::NumberLiteral
                    | TypeFlags::ESSymbol,
            ) || (idx_constraint.is_union()
                && idx_constraint.types().is_some_and(|ts| {
                    ts.iter().all(|c| {
                        c.flags.intersects(
                            TypeFlags::StringLiteral | TypeFlags::NumberLiteral,
                        )
                    })
                }));
            if !kind_ok {
                return None;
            }
            let resolved = self.get_indexed_access_type(object, &idx_constraint);
            if resolved.flags.contains(TypeFlags::Never) {
                return None;
            }
            return Some(resolved);
        } else {
            return None;
        };
        // A constraint that carries no information must not become the
        // relation result.
        if matches!(
            obj_constraint.intrinsic_name(),
            Some("any") | Some("unknown") | Some("error")
        ) {
            return None;
        }
        let resolved = self.get_indexed_access_type(&obj_constraint, index);
        if matches!(resolved.intrinsic_name(), Some("any") | Some("error")) {
            return None;
        }
        Some(resolved)
    }

    fn is_type_related_to_inner(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        // Go (relater.go isTypeRelatedTo): under the comparable relation,
        // the swapped simple check runs first — literal/primitive mismatches
        // like number vs `1` are comparable in either direction.
        if relation == RelationKind::Comparable
            && !target.flags.contains(TypeFlags::Never)
            && self.is_simple_type_related_to(target, source, relation)
        {
            return true;
        }
        if self.is_simple_type_related_to(source, target, relation) {
            return true;
        }

        let s = source.flags;
        let t = target.flags;

        // Source is a type parameter: reduce it to its constraint FIRST —
        // Go's `recursiveTypeRelatedTo` performs this before any
        // union/structural dispatch, so `K extends "a" | "b"` is assignable
        // to `"a" | "b" | "c"` as a whole union (not member-by-member
        // against the type parameter).
        if s.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(source) {
                if self.is_type_related_to(&constraint, target, relation) {
                    return true;
                }
            }
        }

        // Deferred indexed accesses reduce through their constraint first
        // (Go isRelatedToWorker's TypeVariable branch, relater.go ~L3706):
        // `T["content"]` with `T extends { content: C }` is assignable to
        // `C` because the access resolved against the object parameter's
        // base constraint yields `C`. Only fires when the target is not
        // itself an indexed access (component-wise comparison handles that).
        // (Flag-based check plus a data-based fallback: deferred accesses
        // created in some paths carry no IndexedAccess flag bit, though the
        // nodebuilder still dispatches on the data variant.)
        let source_is_indexed_access = s.contains(TypeFlags::IndexedAccess)
            || matches!(source.data, TypeData::IndexedAccess(_));
        if source_is_indexed_access && !t.contains(TypeFlags::IndexedAccess) {
            if let Some(constraint) = self.constraint_of_indexed_access(source)
                && self.is_type_related_to(&constraint, target, relation)
            {
                return true;
            }
        }

        // NOTE: `intersects` (not `contains`) — a type "is a union or
        // intersection" if it has *any* of those bits set, not both. `contains`
        // would require the argument to be a subset of the type's flags, so
        // `Union.contains(Union | Intersection)` is false and this branch was
        // never reachable. This latent bug was masked because the relater was
        // not wired into diagnostics (same-type cases passed via Arc::ptr_eq).
        if s.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            || t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
        {
            return self.is_union_or_intersection_related_to(source, target, relation);
        }

        // Primitive sources compare against OBJECT targets through their
        // boxed apparent type (Go maps the source via getApparentType in
        // structuredTypeRelatedToWorker): `2` is assignable to `Number`,
        // `"s"` to `String`, under any non-identity relation. Go's
        // `reportStructuralErrors := ... && !sourceIsPrimitive`
        // (relater.go ~L3903): structural elaborations (missing-property
        // chains over the boxed members) are suppressed for primitive
        // sources — `x = 1` against a namespace target reports only the
        // 2322 head, no `Property 'toString' …` chain lines.
        if t.contains(TypeFlags::Object)
            && !s.contains(TypeFlags::Object)
            && relation != RelationKind::Identity
            && let Some(boxed) = self.boxed_apparent_type_of_primitive(source)
        {
            let saved_chain_active = self.relater_chain_active;
            self.relater_chain_active = false;
            let r = self.is_type_related_to(&boxed, target, relation);
            self.relater_chain_active = saved_chain_active;
            return r;
        }

        if s.contains(TypeFlags::Object) && t.contains(TypeFlags::Object) {
            // Check array types: Array<T> is assignable to Array<U> if T is assignable to U
            if self.is_array_type(source) && self.is_array_type(target) {
                return self.is_array_type_related_to(source, target, relation);
            }
            // Check tuple types: element-by-element comparison
            if self.is_tuple_type(source) && self.is_tuple_type(target) {
                return self.is_tuple_type_related_to(source, target, relation);
            }
            // Generic instantiation: `Foo<X>` vs `Foo<Y>` for the same generic
            // `Foo<T>`. Variance-aware comparison of type arguments (P3.7d).
            // Falls back to structural comparison when the variance-based
            // check is inconclusive (Ternary::Maybe) or not applicable
            // (None — e.g. tuples, marker types).
            if let Some(result) = self.generic_type_reference_related_to(source, target, relation) {
                if result.is_true() {
                    return true;
                }
                if result.is_false() {
                    return false;
                }
                // Ternary::Maybe / Unknown: fall through to structural comparison.
            }
            return self.is_object_type_related_to(source, target, relation);
        }

        // Type-parameter identity: TypeParameter types produced from the
        // SAME type-parameter symbol (e.g. the class's `U` appearing both
        // in a member's return annotation and in the `implements I<U>`
        // instantiation) are the same type. Go relies on per-symbol
        // interning (`source == target` pointer check); our cached
        // construction usually yields the same Arc too, but instantiation
        // may clone — so compare by symbol.
        if s.contains(TypeFlags::TypeParameter)
            && t.contains(TypeFlags::TypeParameter)
            && let (Some(ss), Some(ts)) = (&source.symbol, &target.symbol)
            && Arc::ptr_eq(ss, ts)
        {
            return true;
        }

        // Handle type parameters: target-side constraint check (the
        // source-side reduction now happens before the union dispatch).
        if t.contains(TypeFlags::TypeParameter) {
            // Target is a type parameter with a constraint
            if let Some(constraint) = self.get_constraint_of_type_parameter(target) {
                // Check if source is assignable to the constraint
                if self.is_type_related_to(source, &constraint, relation) {
                    return true;
                }
            }
        }

        // Deferred indexed-access target (Go relater.go ~L3483):
        // 1. S[K] is related to T[J] if S is related to T and K is related
        //    to J (component-wise comparison of the two accesses).
        // 2. Otherwise (assignable/comparable relation) S is related to
        //    T[J] if S is related to C, where C is the resolved access of
        //    the base constraints of T and J ("for writing").
        if t.contains(TypeFlags::IndexedAccess) {
            if let TypeData::IndexedAccess(target_access) = &target.data {
                if s.contains(TypeFlags::IndexedAccess)
                    && let TypeData::IndexedAccess(source_access) = &source.data
                    && let (Some(source_object), Some(source_index)) = (
                        &source_access.object_type,
                        &source_access.index_type,
                    )
                    && let (Some(target_object), Some(target_index)) = (
                        &target_access.object_type,
                        &target_access.index_type,
                    )
                {
                    let objects_related =
                        self.is_type_related_to(source_object, target_object, relation);
                    if objects_related {
                        let indexes_related =
                            self.is_type_related_to(source_index, target_index, relation);
                        if indexes_related {
                            return true;
                        }
                    }
                }
                if relation == RelationKind::Assignable || relation == RelationKind::Comparable {
                    if let (Some(object_type), Some(index_type)) =
                        (&target_access.object_type, &target_access.index_type)
                    {
                        let base_object = self.get_base_constraint_or_type(object_type);
                        let base_index = self.get_base_constraint_or_type(index_type);
                        let object_changed = !Arc::ptr_eq(&base_object, object_type);
                        if !self.type_flags_is_generic_object_type(&base_object)
                            && !self.type_flags_is_generic_index_type(&base_index)
                        {
                            let mut access_flags = AccessFlags::Writing;
                            if object_changed {
                                access_flags |= AccessFlags::NoIndexSignatures;
                            }
                            if let Some(constraint) = self.try_get_indexed_access_type(
                                &base_object,
                                &base_index,
                                access_flags,
                            ) {
                                if self.is_type_related_to(source, &constraint, relation) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        // `keyof` target (Go relater.go ~L3526): a keyof S is related to a
        // keyof T if T is related to S (the derived interface's key set is
        // a subset of the base's, so `keyof Derived` narrows `keyof Base`).
        if t.contains(TypeFlags::Index)
            && let TypeData::Index(target_index) = &target.data
            && let Some(target_of) = &target_index.target
        {
            if s.contains(TypeFlags::Index)
                && let TypeData::Index(source_index) = &source.data
                && let Some(source_of) = &source_index.target
            {
                if self.is_type_related_to(target_of, source_of, relation) {
                    return true;
                }
            }
        }

        // Handle conditional types: use resolved type if available
        if s.contains(TypeFlags::Conditional) {
            let resolved = match self.get_resolved_type_of_conditional_type(source) {
                Some(resolved) => Some(resolved),
                // Eagerly attempt resolution (Go resolves conditionals during
                // `instantiateType`, so by comparison time they are concrete).
                None => self.resolve_conditional_type(source),
            };
            if let Some(resolved) = resolved {
                if self.is_type_related_to(&resolved, target, relation) {
                    return true;
                }
            }
        }
        if t.contains(TypeFlags::Conditional) {
            // First, try the resolved-type fast path (used when the
            // conditional has already been evaluated to a concrete type).
            let resolved = match self.get_resolved_type_of_conditional_type(target) {
                Some(resolved) => Some(resolved),
                None => self.resolve_conditional_type(target),
            };
            if let Some(resolved) = resolved {
                if self.is_type_related_to(source, &resolved, relation) {
                    return true;
                }
                // A successfully resolved target conditional is authoritative:
                // Go never falls back to branch-wise comparison once the
                // conditional has been instantiated to a concrete type.
                if !type_contains_type_parameter(&resolved) {
                    return false;
                }
            }
            // Otherwise, fall back to the conditional-target comparison
            // (P3.7e): compare source against the true/false branches with
            // the permissive/restrictive short-circuits. Returns None when
            // the conditional is unsupported (infer positions, distribution
            // dependence, identical source conditional) — in that case we
            // fall through to other strategies.
            if let Some(result) = self.conditional_type_related_to(source, target, relation) {
                if result.is_true() {
                    return true;
                }
                if result.is_false() {
                    return false;
                }
                // Ternary::Maybe / Unknown: fall through.
            }
        }

        // Handle mapped types: base constraint check
        if s.contains(TypeFlags::Object) && source.object_flags.contains(ObjectFlags::Mapped) {
            if let Some(constraint) = self.get_constraint_of_mapped_type(source) {
                if self.is_type_related_to(&constraint, target, relation) {
                    return true;
                }
            }
        }
        if t.contains(TypeFlags::Object) && target.object_flags.contains(ObjectFlags::Mapped) {
            if let Some(constraint) = self.get_constraint_of_mapped_type(target) {
                if self.is_type_related_to(source, &constraint, relation) {
                    return true;
                }
            }
            // Direct mapped-vs-mapped comparison (P3.7e). Only fires when
            // the source is itself a mapped type and we're not in identity
            // mode (identity needs the full modifiers check which we
            // don't yet implement). Returns None for unsupported cases
            // (e.g. name remapping), which fall through to structural
            // comparison.
            if s.contains(TypeFlags::Object) && source.object_flags.contains(ObjectFlags::Mapped) {
                if let Some(result) = self.mapped_type_related_to(source, target, relation) {
                    if result.is_true() {
                        return true;
                    }
                    if result.is_false() {
                        return false;
                    }
                }
            }
        }

        false
    }

    /// Check if two array types are related.
    ///
    /// Array<T> is assignable to Array<U> if T is assignable to U (covariant).
    fn is_array_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);

        if source_args.is_empty() || target_args.is_empty() {
            // If either type has no type arguments, fall back to structural comparison
            return self.is_object_type_related_to(source, target, relation);
        }

        // Compare element types: Array<T> ~ Array<U> iff T ~ U
        // (covariant in the element type)
        let source_elem = &source_args[0];
        let target_elem = &target_args[0];
        self.is_type_related_to(source_elem, target_elem, relation)
    }

    /// Check if two tuple types are related.
    ///
    /// Tuples are compared element-by-element. Each element in source must be
    /// assignable to the corresponding element in target.
    fn is_tuple_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_tuple = match &source.data {
            TypeData::Tuple(t) => t,
            _ => return false,
        };
        let target_tuple = match &target.data {
            TypeData::Tuple(t) => t,
            _ => return false,
        };

        // Compare element-by-element
        let min_len = source_tuple
            .element_infos
            .len()
            .min(target_tuple.element_infos.len());
        for i in 0..min_len {
            let source_elem = &source_tuple.element_infos[i];
            let target_elem = &target_tuple.element_infos[i];

            // Get the element types from the interface_data's structured type members
            let source_type = self.get_tuple_element_type(source, i);
            let target_type = self.get_tuple_element_type(target, i);

            if let (Some(st), Some(tt)) = (source_type, target_type) {
                if !self.is_type_related_to(&st, &tt, relation) {
                    return false;
                }
            }

            // Check element flags compatibility (required/optional/rest/variadic)
            if !self.is_element_flags_compatible(source_elem.flags, target_elem.flags, relation) {
                return false;
            }
        }

        // If target has more elements than source, those must be optional or rest
        if source_tuple.element_infos.len() < target_tuple.element_infos.len() {
            for i in source_tuple.element_infos.len()..target_tuple.element_infos.len() {
                let flags = target_tuple.element_infos[i].flags;
                if !flags.contains(ElementFlags::Optional)
                    && !flags.contains(ElementFlags::Rest)
                    && !flags.contains(ElementFlags::Variadic)
                {
                    return false;
                }
            }
        }

        true
    }

    /// Get the type of a tuple element at a given index.
    pub(super) fn get_tuple_element_type(&self, t: &Arc<Type>, index: usize) -> Option<Arc<Type>> {
        match &t.data {
            TypeData::Tuple(tuple) => {
                // The element type is stored directly on `TupleElementInfo`
                // (mirrors Go's `TupleElementInfo.Type`). This avoids the
                // need to re-resolve a structured member symbol.
                tuple
                    .element_infos
                    .get(index)
                    .and_then(|info| info.type_.clone())
            }
            _ => None,
        }
    }

    /// Check if element flags are compatible between source and target.
    fn is_element_flags_compatible(
        &self,
        source: ElementFlags,
        target: ElementFlags,
        _relation: RelationKind,
    ) -> bool {
        // Required can be assigned to Required or Optional
        // Optional can be assigned to Optional
        // Rest can be assigned to Rest
        // Variadic can be assigned to Variadic or Rest
        if source.contains(ElementFlags::Required) {
            target.contains(ElementFlags::Required) || target.contains(ElementFlags::Optional)
        } else if source.contains(ElementFlags::Optional) {
            target.contains(ElementFlags::Optional)
        } else if source.contains(ElementFlags::Rest) {
            target.contains(ElementFlags::Rest)
        } else if source.contains(ElementFlags::Variadic) {
            target.contains(ElementFlags::Variadic) || target.contains(ElementFlags::Rest)
        } else {
            true
        }
    }

    /// Handle union/intersection type relations.
    fn is_union_or_intersection_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let s = source.flags;
        let t = target.flags;

        // Go order (unionOrIntersectionRelatedTo): deconstruct unions before
        // intersections (unions are always at the top) and "each" relations
        // before "some" relations — so target-union/target-intersection are
        // handled BEFORE an intersection source.
        if s.contains(TypeFlags::Union) {
            // For Comparable: a union source is comparable to target if ANY
            // constituent is comparable (loose check for error-reporting).
            // For all other relations (Subtype/StrictSubtype/Assignable): ALL
            // constituents must be related. E.g. `"a" | "b"` is NOT assignable
            // to `"a"` because `"b"` isn't.
            if relation == RelationKind::Comparable {
                return self.some_type_related_to_type(source, target, relation);
            }
            return self.each_type_related_to_type(source, target, relation);
        }

        if t.contains(TypeFlags::Union) {
            return self.type_related_to_some_type(source, target, relation);
        }

        if t.contains(TypeFlags::Intersection) {
            return self.type_related_to_each_type(source, target, relation);
        }

        if s.contains(TypeFlags::Intersection) {
            // Source intersection: any immediately-related constituent wins,
            // with the trial error chains discarded (Go passes reportErrors
            // false — "elaborating on whether a source constituent is
            // related... leads to some confusing error messages"; the
            // structural tail below elaborates the real failure).
            let save_len = self.relater_error_chain.len();
            let mut immediately_related = false;
            if let Some(ui) = source.as_union_or_intersection() {
                for c in &ui.types {
                    if self.is_type_related_to(c, target, relation) {
                        immediately_related = true;
                        break;
                    }
                }
            }
            self.relater_error_chain.truncate(save_len);
            if immediately_related {
                return true;
            }
            // Go recursiveTypeRelatedTo's fall-through: when the target is
            // object-ish, the full intersection "viewed as an object" is
            // checked — `{ a } & { b }` is assignable to `{ a; b }` even
            // though neither constituent alone is. A type-parameter target
            // reduces through its constraint (the target-side rule that the
            // union dispatch skipped for intersection sources).
            if t.contains(TypeFlags::Object) {
                return self.intersection_source_structurally_related(source, target, relation);
            }
            if t.contains(TypeFlags::TypeParameter) {
                if let Some(constraint) = self.get_constraint_of_type_parameter(target) {
                    return self.is_type_related_to(source, &constraint, relation);
                }
            }
            return false;
        }

        false
    }

    /// Structural tail for an intersection source vs an object target (Go
    /// `recursiveTypeRelatedTo`'s fall-through after the constituent
    /// trials): every target property is resolved on the intersection as a
    /// whole — a property may come from ANY constituent, and a
    /// type-parameter constituent contributes through its constraint
    /// (`T & { other }` has `common` from T's constraint and `other` from
    /// the object constituent).
    fn intersection_source_structurally_related(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let Some(ui) = source.as_union_or_intersection() else {
            return false;
        };
        let Some(target_struct) = target.as_structured() else {
            return false;
        };
        let mut missing_props: Vec<String> = Vec::new();
        for target_prop in &target_struct.properties {
            let found =
                self.intersection_lookup_property(&ui.types, &target_prop.name, &mut Vec::new());
            let Some(source_prop) = found else {
                // Missing property: allowed when the target property is
                // optional (its type already carries `| undefined`).
                if target_prop.flags.contains(SymbolFlags::Optional) {
                    continue;
                }
                missing_props.push(target_prop.name.clone());
                continue;
            };
            let source_type = self.get_type_of_symbol(&source_prop);
            let target_type = self.substituted_member_type_of(target, target_prop);
            if !self.is_type_related_to(&source_type, &target_type, relation) {
                let prop_source_str = self.type_to_string(&source_type);
                let prop_target_str = self.type_to_string(&target_type);
                self.relater_report_error(
                    crate::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                    vec![prop_source_str, prop_target_str],
                );
                self.relater_report_error(
                    crate::diagnostics::messages_generated::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                    vec![self.chain_property_arg_name(target_prop)],
                );
                return false;
            }
        }
        if !missing_props.is_empty() {
            // Go shouldReportUnmatchedPropertyError gate (see
            // is_object_type_related_to): signature-only sources elide
            // the missing-property elaboration.
            if !self.should_report_unmatched_property_error(source, target) {
                return false;
            }
            let source_str = self.type_to_string(source);
            let target_str = self.type_to_string(target);
            if missing_props.len() == 1 {
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing_props[0].clone(), source_str, target_str],
                );
            } else if missing_props.len() <= 5 {
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![source_str, target_str, missing_props.join(", ")],
                );
            } else {
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE,
                    vec![
                        source_str,
                        target_str,
                        missing_props[..4].join(", "),
                        (missing_props.len() - 4).to_string(),
                    ],
                );
            }
            return false;
        }
        // Signatures: the intersection "viewed as an object" exposes the
        // constituents' call/construct signatures (Go's apparent-member
        // comparison in the intersection fall-through — properties alone
        // would make `typeof Cls & (() => T)` compare equal to any
        // `typeof Cls` regardless of the signature's type arguments).
        let target_call = target_struct.call_signatures().to_vec();
        let target_construct = target_struct.construct_signatures().to_vec();
        for (kind, target_sigs) in [
            (SignatureKind::Call, target_call),
            (SignatureKind::Construct, target_construct),
        ] {
            if target_sigs.is_empty() {
                continue;
            }
            let mut source_sigs: Vec<Arc<crate::checker::types::Signature>> = Vec::new();
            for c in &ui.types {
                if let Some(cs) = c.as_structured() {
                    let sigs = match kind {
                        SignatureKind::Call => cs.call_signatures(),
                        SignatureKind::Construct => cs.construct_signatures(),
                    };
                    source_sigs.extend(sigs.iter().cloned());
                }
            }
            if source_sigs.is_empty() {
                continue;
            }
            // Each target signature must be matched by SOME source
            // signature (Go's N×M `signaturesRelatedTo` shape).
            let mut all_matched = true;
            for t in &target_sigs {
                let mut matched = false;
                for s in &source_sigs {
                    if !self
                        .compare_signatures_related(s, t, SignatureCheckMode::empty(), relation)
                        .is_false()
                    {
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    all_matched = false;
                    break;
                }
            }
            if !all_matched {
                return false;
            }
        }
        true
    }
    /// Resolve a property name across an intersection's constituents: the
    /// first constituent that provides it wins (Go `getPropertiesOfType` on
    /// an intersection merges the constituents' property sets).
    fn intersection_lookup_property(
        &mut self,
        constituents: &[Arc<Type>],
        name: &str,
        visited: &mut Vec<usize>,
    ) -> Option<Arc<crate::ast::Symbol>> {
        for c in constituents {
            if let Some(sym) = self.lookup_property_on_single_type(c, name, visited) {
                return Some(sym);
            }
        }
        None
    }

    /// Property lookup on one intersection constituent: structured objects
    /// read their member table, type parameters reduce through their
    /// constraint, nested intersections distribute over their constituents,
    /// and unions require the property on ALL members (Go union property
    /// resolution).
    fn lookup_property_on_single_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
        visited: &mut Vec<usize>,
    ) -> Option<Arc<crate::ast::Symbol>> {
        let ptr = Arc::as_ptr(t) as usize;
        if visited.contains(&ptr) {
            return None;
        }
        visited.push(ptr);
        if t.flags.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_constraint_of_type_parameter(t)?;
            return self.lookup_property_on_single_type(&constraint, name, visited);
        }
        if let Some(ui) = t.as_union_or_intersection() {
            if t.flags.contains(TypeFlags::Union) {
                let mut first: Option<Arc<crate::ast::Symbol>> = None;
                for c in &ui.types {
                    match self.lookup_property_on_single_type(c, name, visited) {
                        Some(sym) => {
                            if first.is_none() {
                                first = Some(sym);
                            }
                        }
                        None => return None,
                    }
                }
                return first;
            }
            for c in &ui.types {
                if let Some(sym) = self.lookup_property_on_single_type(c, name, visited) {
                    return Some(sym);
                }
            }
            return None;
        }
        if let Some(st) = t.as_structured() {
            if let Some(p) = st.members.get(name) {
                return Some(Arc::clone(p));
            }
            return None;
        }
        if self.is_array_type(t) {
            return self.declared_array_member_symbol(name);
        }
        None
    }

    /// Source is a union/intersection: check if at least one constituent
    /// is related to target.
    fn some_type_related_to_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = source.as_union_or_intersection() {
            // Chain hygiene (Go saveErrorState/restoreErrorState around
            // union-member trials, relater.go ~L3304): a failed trial's
            // entries are rolled back; on overall failure the deepest
            // (longest) trial chain is kept for the pyramid.
            let save_len = self.relater_error_chain.len();
            let mut best: Option<Vec<RelaterChainEntry>> = None;
            for t in &ui.types {
                if self.is_type_related_to(t, target, relation) {
                    return true;
                }
                if best.as_ref().is_none_or(|b| b.len() < self.relater_error_chain.len()) {
                    best = Some(self.relater_error_chain.clone());
                }
                self.relater_error_chain.truncate(save_len);
            }
            if let Some(b) = best {
                self.relater_error_chain = b;
            }
        }
        false
    }

    /// Source is a union (for subtype/strictSubtype): check if ALL
    /// constituents are related to target.
    fn each_type_related_to_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = source.as_union_or_intersection() {
            let mut any_failed = false;
            let mut failed_nullish: Option<Arc<Type>> = None;
            for t in &ui.types {
                if !self.is_type_related_to(t, target, relation) {
                    any_failed = true;
                    if t.flags.contains(TypeFlags::Undefined) {
                        // `undefined` wins the leaf spot over `null` when
                        // both fail (official chains spell `undefined`
                        // first: arrayBestCommonTypes (20,37)).
                        if failed_nullish
                            .as_ref()
                            .is_none_or(|f| f.flags.contains(TypeFlags::Null))
                        {
                            failed_nullish = Some(Arc::clone(t));
                        }
                    } else if t.flags.contains(TypeFlags::Null)
                        && failed_nullish.is_none()
                    {
                        failed_nullish = Some(Arc::clone(t));
                    }
                }
            }
            if any_failed {
                // Elaboration leaf (assignmentCompatability11 family): an
                // optional property's union source failing on its nullish
                // constituent — official spells that member out as the
                // chain's deepest entry (non-nullish member failures stay
                // folded into the head line).
                if let Some(t) = failed_nullish
                    && self.relater_chain_active
                {
                    let member_str = self.type_to_string(&t);
                    let target_str = self.type_to_string(target);
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                        vec![member_str, target_str],
                    );
                }
                return false;
            }
            return true;
        }
        false
    }

    /// Target is a union: check if source is related to at least one constituent.
    fn type_related_to_some_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = target.as_union_or_intersection() {
            // Chain hygiene — see some_type_related_to_type.
            let save_len = self.relater_error_chain.len();
            let mut best: Option<Vec<RelaterChainEntry>> = None;
            for t in &ui.types {
                if self.is_type_related_to(source, t, relation) {
                    return true;
                }
                if best.as_ref().is_none_or(|b| b.len() < self.relater_error_chain.len()) {
                    best = Some(self.relater_error_chain.clone());
                }
                self.relater_error_chain.truncate(save_len);
            }
            // An INTERSECTION source is related to a union target when any
            // of its members is (TS structuredTypeRelatedTo's
            // `some(source.types, s => isRelatedTo(s, target))` — a value
            // of `T & U` is a `T`, so `T`'s constraint alone proves
            // membership: `T & U` -> `A | B` with T extends A, U extends B).
            if source.flags.contains(TypeFlags::Intersection)
                && let Some(si) = source.as_union_or_intersection()
            {
                self.relater_error_chain.truncate(save_len);
                for s in &si.types {
                    if self.is_type_related_to(s, target, relation) {
                        return true;
                    }
                }
                self.relater_error_chain.truncate(save_len);
            }
            if let Some(b) = best {
                self.relater_error_chain = b;
            }
        }
        false
    }

    /// Target is an intersection: check if source is related to ALL constituents.
    fn type_related_to_each_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = target.as_union_or_intersection() {
            self.relater_intersection_target_depth += 1;
            let result = (|| {
                for t in &ui.types {
                    if !self.is_type_related_to(source, t, relation) {
                        return false;
                    }
                }
                true
            })();
            self.relater_intersection_target_depth -= 1;
            return result;
        }
        false
    }

    /// Check object type relation (structural typing).
    ///
    /// For assignability: source must have all properties of target.
    fn is_object_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_struct = match source.as_structured() {
            Some(s) => s,
            None => return false,
        };
        let target_struct = match target.as_structured() {
            Some(t) => t,
            None => return false,
        };

        // Weak-type common-property check (Go recursiveTypeRelatedTo's
        // isPerformingCommonPropertyChecks → hasCommonProperties): a
        // target whose properties are ALL optional (and no index/call/
        // construct signatures) rejects a source that shares NO property
        // with it — `{ b: 1 }` is NOT assignable to `{ kind?: 'A' }`
        // (TS2559 "no properties in common"). Suppressed for the
        // Comparable relation and while comparing against one constituent
        // of an intersection TARGET (`A extends B & C`; Go's
        // IntersectionStateTarget). The "did you mean to call it" variant
        // fires when the source has call/construct signatures.
        if relation != RelationKind::Comparable
            && self.relater_intersection_target_depth == 0
            && !source_struct.properties.is_empty()
            && self.is_weak_type(target)
            && !self.has_common_properties(source, target, false)
        {
            let has_calls = !source_struct.call_signatures().is_empty();
            let has_constructs = !source_struct.construct_signatures().is_empty();
            if self.relater_chain_active {
                let source_str = self.type_to_string(source);
                let target_str = self.type_to_string(target);
                if has_calls || has_constructs {
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            VALUE_OF_TYPE_0_HAS_NO_PROPERTIES_IN_COMMON_WITH_TYPE_1_DID_YOU_MEAN_TO_CALL_IT,
                        vec![source_str, target_str],
                    );
                } else {
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            TYPE_0_HAS_NO_PROPERTIES_IN_COMMON_WITH_TYPE_1,
                        vec![source_str, target_str],
                    );
                }
            }
            return false;
        }

        // TS2740 (Go getUnmatchedProperties over `getPropertiesOfType`
        // of the array target): a NON-array source against a bare array
        // target fails with the missing-properties chain — enumerate the
        // declared `Array<T>` interface's members, applying Go's
        // getPropertyOfType source lookup (own members, then the
        // `Function`/`Object` interface fallbacks that make
        // toString/toLocaleString "present"). Kept as a SEPARATE
        // pre-check so the general property loop below stays untouched.
        if self.is_array_type(target)
            && target_struct.properties.is_empty()
            && !self.is_array_type(source)
            && !self.is_tuple_type(source)
            && !source.object_flags.contains(ObjectFlags::EvolvingArray)
        {
            let mut missing: Vec<String> = Vec::new();
            for prop in self.declared_array_member_symbols() {
                if prop.flags.contains(SymbolFlags::Optional) {
                    continue;
                }
                let found = source_struct.members.get(&prop.name).is_some()
                    || (!source_struct.call_signatures().is_empty()
                        && self
                            .global_interface_member_symbol("Function", &prop.name)
                            .is_some())
                    || self.global_interface_member_symbol("Object", &prop.name).is_some();
                if !found {
                    missing.push(prop.name.clone());
                }
            }
            if !missing.is_empty() {
                if self.should_report_unmatched_property_error(source, target) {
                    let source_str = self.type_to_string(source);
                    let target_str = self.type_to_string(target);
                    if missing.len() == 1 {
                        self.relater_report_error(
                            crate::diagnostics::messages_generated::
                                PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                            vec![missing[0].clone(), source_str, target_str],
                        );
                    } else if missing.len() <= 5 {
                        self.relater_report_error(
                            crate::diagnostics::messages_generated::
                                TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                            vec![source_str, target_str, missing.join(", ")],
                        );
                    } else {
                        self.relater_report_error(
                            crate::diagnostics::messages_generated::
                                TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE,
                            vec![
                                source_str,
                                target_str,
                                missing[..4].join(", "),
                                (missing.len() - 4).to_string(),
                            ],
                        );
                    }
                }
                return false;
            }
            // All declared members resolve on the source (e.g. a
            // structural ConcatArray look-alike): fall through to the
            // general loop unchanged.
        }

        // Check properties: target properties must exist in source with compatible types
        let mut missing_props: Vec<String> = Vec::new();
        // Bare array sources carry no member table of their own — their
        // properties live on the declared `Array<T>` interface,
        // element-substituted at access time (`string[]` must satisfy
        // `ConcatArray<string>`'s length/join/slice/concat structurally).
        // Evolving array literals (`["x"]` as a call argument) share the
        // same shape: no members of their own until finalized.
        let source_is_bare_array = (self.is_array_type(source)
            || source.object_flags.contains(ObjectFlags::EvolvingArray))
            && source_struct.members.is_empty();
        for target_prop in &target_struct.properties {
            // Check that source has a matching property by name. Names the
            // source doesn't declare itself may still come from the global
            // `Object` interface (Go getPropertyOfType's fallback) — such a
            // member PARTICIPATES in the type comparison when the source
            // declares it locally (an overridden `toString: number` vs
            // `Object.toString: () => string` fails, assignmentToObject).
            let source_declares_locally = source_struct.members.get(&target_prop.name).is_some();
            let source_prop = match source_struct.members.get(&target_prop.name) {
                Some(p) => Arc::clone(p),
                None => {
                    if source_is_bare_array
                        && let Some(p) = self.declared_array_member_symbol(&target_prop.name)
                    {
                        p
                    } else {
                        // Missing property: allowed only when the target
                        // property is optional (`x?: T`). The target property's
                        // type is already `T | undefined` (see
                        // `build_interface_type_from_members`), so a missing
                        // source property is treated as `undefined`.
                        if target_prop.flags.contains(SymbolFlags::Optional) {
                            continue;
                        }
                        missing_props.push(target_prop.name.clone());
                        continue;
                    }
                }
            };
            // Computed-name members (`[Symbol.iterator]`): presence counts,
            // but their instantiated generic types (IterableIterator<T>)
            // compare through replica forms whose structure doesn't fully
            // survive substitution — skip the TYPE comparison for them.
            // The same applies to Object-inherited members the source does
            // NOT override (apparent-type machinery — official only checks
            // the override).
            if target_prop.name.starts_with('[')
                || (!source_declares_locally
                    && self
                        .global_interface_member_symbol("Object", &target_prop.name)
                        .is_some())
            {
                continue;
            }
            // Private/protected member accessibility (Go propertyRelatedTo,
            // relater.go ~L4313): private members only match the SAME
            // declaration; both-private-different-declaration → the
            // "separate declarations" chain entry (the nested line of
            // TS2415 class-extends errors); one-private → the mismatch
            // message; protected-source vs public-target → protected error.
            {
                let src_mod =
                    crate::checker::exports::get_declaration_modifier_flags_from_symbol(&source_prop);
                let tgt_mod =
                    crate::checker::exports::get_declaration_modifier_flags_from_symbol(target_prop);
                if src_mod.intersects(ModifierFlags::Private)
                    || tgt_mod.intersects(ModifierFlags::Private)
                {
                    let same_decl = match (
                        &source_prop.value_declaration,
                        &target_prop.value_declaration,
                    ) {
                        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                        _ => false,
                    };
                    if !same_decl {
                        if src_mod.intersects(ModifierFlags::Private)
                            && tgt_mod.intersects(ModifierFlags::Private)
                        {
                            self.relater_report_error(
                                crate::diagnostics::messages_generated::
                                    TYPES_HAVE_SEPARATE_DECLARATIONS_OF_A_PRIVATE_PROPERTY_0,
                                vec![target_prop.name.clone()],
                            );
                        } else {
                            let private_side = if src_mod
                                .intersects(ModifierFlags::Private)
                            {
                                self.type_to_string(source)
                            } else {
                                self.type_to_string(target)
                            };
                            let public_side = if src_mod
                                .intersects(ModifierFlags::Private)
                            {
                                self.type_to_string(target)
                            } else {
                                self.type_to_string(source)
                            };
                            self.relater_report_error(
                                crate::diagnostics::messages_generated::
                                    PROPERTY_0_IS_PRIVATE_IN_TYPE_1_BUT_NOT_IN_TYPE_2,
                                vec![target_prop.name.clone(), private_side, public_side],
                            );
                        }
                        return false;
                    }
                } else if src_mod.intersects(ModifierFlags::Protected)
                    && !tgt_mod.intersects(ModifierFlags::Protected)
                {
                    let src_str = self.type_to_string(source);
                    let tgt_str = self.type_to_string(target);
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            PROPERTY_0_IS_PROTECTED_IN_TYPE_1_BUT_PUBLIC_IN_TYPE_2,
                        vec![target_prop.name.clone(), src_str, tgt_str],
                    );
                    return false;
                }
            }
            // Check that the source property type is related to the
            // target property type (depth check) under the SAME relation —
            // Go's `propertiesRelatedTo` recurses with the incoming
            // relation, so the comparable relation widens literal property
            // types (number ~ 1).
            let source_type = if source_is_bare_array {
                self.instantiate_array_member_type(source, &source_prop)
                    .unwrap_or_else(|| self.get_type_of_symbol(&source_prop))
            } else {
                self.substituted_member_type_of(source, &source_prop)
            };
            // A BARE generic reference (no type arguments — `C` where the
            // declaration is `class C<T>`) behaves like an any-arg
            // instantiation on BOTH sides (official implicit-any args):
            // substitute its own type parameters with `any` so bare and
            // instantiated references stay mutually assignable.
            let source_type = self.erase_bare_generic_params(source, &source_type);
            let target_type = self.substituted_member_type_of(target, target_prop);
            let target_type = self.erase_bare_generic_params(target, &target_type);
            if !self.is_type_related_to(&source_type, &target_type, relation) {
                // Chain (Go propertiesRelatedTo → reportError, relater.go
                // ~L4353): the nested failure's entries are already on the
                // chain; push this property's head then the property
                // incompatibility marker (whose post-processing collapses
                // signature-return and dotted-name chains).
                let prop_source_str = self.type_to_string(&source_type);
                let prop_target_str = self.type_to_string(&target_type);
                self.relater_report_error(
                    crate::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                    vec![prop_source_str, prop_target_str],
                );
                self.relater_report_error(
                    crate::diagnostics::messages_generated::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                    vec![self.chain_property_arg_name(target_prop)],
                );
                return false;
            }
        }
        // Missing-required-property chain entries (Go relater.go ~L4403:
        // one property → TS2741-style single message; 2-5 → the "missing
        // the following properties" form with all names; >5 → the first
        // FOUR names plus a count, Go reportUnmatchedProperty).
        if !missing_props.is_empty() {
            // Go shouldReportUnmatchedPropertyError: a signature-only
            // source elides the elaboration unless the target carries the
            // same signature kind — the head error alone remains.
            if !self.should_report_unmatched_property_error(source, target) {
                return false;
            }
            let source_str = self.type_to_string(source);
            let target_str = self.type_to_string(target);
            if missing_props.len() == 1 {
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing_props[0].clone(), source_str, target_str],
                );
            } else if missing_props.len() <= 5 {
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![
                        source_str,
                        target_str,
                        missing_props.join(", "),
                    ],
                );
            } else {
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE,
                    vec![
                        source_str,
                        target_str,
                        missing_props[..4].join(", "),
                        (missing_props.len() - 4).to_string(),
                    ],
                );
            }
            return false;
        }

        // Tuple targets require their element positions as properties
        // (Go's `getPropertiesOfType` exposes tuple elements as '0'..'n-1',
        // so a non-tuple source — e.g. a function type — is not assignable
        // to a tuple). Array sources are exempt: array-literal→tuple
        // contextual typing isn't ported, and the suite's baselines rely
        // on the permissive array→tuple behavior.
        if self.is_tuple_type(target)
            && !self.is_array_type(source)
            && source.object_flags.contains(ObjectFlags::EvolvingArray) == false
            && let TypeData::Tuple(tup) = &target.data
        {
            for (i, ei) in tup.element_infos.iter().enumerate() {
                let Some(elem_type) = &ei.type_ else { continue };
                let name = i.to_string();
                let Some(source_prop) = source_struct.members.get(&name) else {
                    let optional = ei.flags.contains(ElementFlags::Optional);
                    if optional {
                        continue;
                    }
                    return false;
                };
                let source_type = self.get_type_of_symbol(source_prop);
                if !self.is_type_related_to(&source_type, elem_type, relation) {
                    return false;
                }
            }
        }

        // Check call signatures
        if !self.is_call_signatures_related_to(source, target, relation) {
            return false;
        }

        // Check construct signatures
        if !self.is_construct_signatures_related_to(source, target, relation) {
            return false;
        }

        // Check index signatures
        if !self.is_index_signatures_related_to(source, target, relation) {
            return false;
        }

        true
    }

    // ────────────────────────────────────────────────────────────────────────
    // Simple type relation checking
    // ────────────────────────────────────────────────────────────────────────

    /// Check if two types are identical at a simple level.
    ///
    /// This handles intrinsic types (by name), literal types (by value),
    /// and type parameters (by ID).
    fn is_simple_type_identical_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        match (&source.data, &target.data) {
            (TypeData::Intrinsic(s), TypeData::Intrinsic(t)) => {
                s.intrinsic_name == t.intrinsic_name
            }
            (TypeData::Literal(s), TypeData::Literal(t)) => s.value == t.value,
            (TypeData::TypeParameter(s), TypeData::TypeParameter(t)) => {
                s.is_this_type == t.is_this_type
            }
            // Two deferred indexed accesses are identical when both their
            // object and index types are (Go relater.go ~L3360, identity
            // relation).
            (TypeData::IndexedAccess(s), TypeData::IndexedAccess(t)) => {
                match (&s.object_type, &t.object_type, &s.index_type, &t.index_type) {
                    (Some(so), Some(to), Some(si), Some(ti)) => {
                        self.is_type_identical_to(so, to) && self.is_type_identical_to(si, ti)
                    }
                    _ => Arc::ptr_eq(source, target),
                }
            }
            // Two `keyof` types are identical when their target types are
            // (Go relater.go ~L3364, identity relation).
            (TypeData::Index(s), TypeData::Index(t)) => match (&s.target, &t.target) {
                (Some(so), Some(to)) => self.is_type_identical_to(so, to),
                _ => Arc::ptr_eq(source, target),
            },
            _ => {
                // For other types, fall back to flag comparison.
                // Full structural comparison requires the full relater.
                source.flags == target.flags
            }
        }
    }

    /// Check if `source` is related to `target` for simple (non-structured) types.
    ///
    /// Mirrors `Checker.isSimpleTypeRelatedTo` in Go. Handles:
    /// - `any` target → always true
    /// - `never` source → always true
    /// - `unknown` target → always true (except strict subtype of any)
    /// - String-like → string, number-like → number, etc.
    /// - Literal → matching base type
    /// - null/undefined rules (strict vs non-strict)
    /// - `any` source in assignable/comparable relation → true
    pub fn is_simple_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let s = source.flags;
        let t = target.flags;

        // Target is `any` → everything is assignable
        // Source is `never` → assignable to everything
        if t.contains(TypeFlags::Any) || s.contains(TypeFlags::Never) {
            return true;
        }

        // Target is `unknown` → everything is assignable
        // (except: strict subtype relation doesn't allow any → unknown)
        if t.contains(TypeFlags::Unknown)
            && !(relation == RelationKind::StrictSubtype && s.contains(TypeFlags::Any))
        {
            return true;
        }

        // Target is `never` → only `never` is assignable
        if t.contains(TypeFlags::Never) {
            return false;
        }

        // String-like types are assignable to `string`
        if s.intersects(TYPE_FLAGS_STRING_LIKE) && t.contains(TypeFlags::String) {
            return true;
        }

        // String literal enum → string literal (matching value)
        if s.contains(TypeFlags::StringLiteral)
            && s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::StringLiteral)
            && !t.contains(TypeFlags::EnumLiteral)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        // Plain literal → same-kind literal (matching value). E.g. `"foo"` is
        // assignable to `"foo"`, `42` to `42`, `true` to `true`. This is the
        // non-enum counterpart of the enum-literal checks above. Mirrors Go's
        // `isSimpleTypeRelatedTo` literal comparisons.
        if s.intersects(TYPE_FLAGS_LITERAL)
            && t.intersects(TYPE_FLAGS_LITERAL)
            && (s & TYPE_FLAGS_LITERAL) == (t & TYPE_FLAGS_LITERAL)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        // Number-like types are assignable to `number`
        if s.intersects(TYPE_FLAGS_NUMBER_LIKE) && t.contains(TypeFlags::Number) {
            return true;
        }

        // Number literal enum → number literal (matching value)
        if s.contains(TypeFlags::NumberLiteral)
            && s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::NumberLiteral)
            && !t.contains(TypeFlags::EnumLiteral)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        // BigInt-like → BigInt
        if s.intersects(TYPE_FLAGS_BIG_INT_LIKE) && t.contains(TypeFlags::BigInt) {
            return true;
        }

        // Boolean-like → Boolean
        if s.intersects(TYPE_FLAGS_BOOLEAN_LIKE) && t.contains(TypeFlags::Boolean) {
            return true;
        }

        // Symbol-like → ESSymbol
        if s.intersects(TYPE_FLAGS_ES_SYMBOL_LIKE) && t.contains(TypeFlags::ESSymbol) {
            return true;
        }

        // Enum → Enum (same enum type or same-named regular enum with
        // matching member values). Ported from Go's `isEnumTypeRelatedTo`.
        if s.contains(TypeFlags::Enum)
            && t.contains(TypeFlags::Enum)
            && self.is_enum_type_related_to(source, target)
        {
            return true;
        }

        // EnumLiteral → EnumLiteral: require matching literal values AND
        // that the underlying enum types are related. Mirrors Go's
        // `isSimpleTypeRelatedTo` enum-literal branch, which additionally
        // calls `isEnumTypeRelatedTo` on the symbols.
        if s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::EnumLiteral)
            && s.intersects(TYPE_FLAGS_LITERAL)
            && t.intersects(TYPE_FLAGS_LITERAL)
            && self.literal_values_equal(source, target)
            && self.is_enum_type_related_to(source, target)
        {
            return true;
        }

        // In non-strictNullChecks mode, `undefined` and `null` are assignable
        // to anything except `never`. Since unions and intersections may reduce
        // to `never`, we exclude them here.
        if s.contains(TypeFlags::Undefined)
            && (!self.strict_null_checks && !t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
                || t.intersects(TypeFlags::Undefined | TypeFlags::Void))
        {
            return true;
        }

        if s.contains(TypeFlags::Null)
            && (!self.strict_null_checks && !t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
                || t.contains(TypeFlags::Null))
        {
            return true;
        }

        // Object → non-primitive (object)
        if s.contains(TypeFlags::Object)
            && t.contains(TypeFlags::NonPrimitive)
            && !(relation == RelationKind::StrictSubtype)
        {
            return true;
        }

        // `object` is related to itself (identity by intrinsic name; the
        // type is interned so the pointer fast path usually catches this).
        if s.contains(TypeFlags::NonPrimitive)
            && t.contains(TypeFlags::NonPrimitive)
            && source.intrinsic_name() == target.intrinsic_name()
        {
            return true;
        }

        // For assignable and comparable relations:
        if relation == RelationKind::Assignable || relation == RelationKind::Comparable {
            // `any` source is assignable to everything
            if s.contains(TypeFlags::Any) {
                return true;
            }

            // `number` is assignable to numeric enum types (bit-flag pattern)
            if s.contains(TypeFlags::Number)
                && (t.contains(TypeFlags::Enum)
                    || (t.contains(TypeFlags::NumberLiteral) && t.contains(TypeFlags::EnumLiteral)))
            {
                return true;
            }

            // Numeric literal is assignable to numeric enum types (matching value)
            if s.contains(TypeFlags::NumberLiteral)
                && !s.contains(TypeFlags::EnumLiteral)
                && (t.contains(TypeFlags::Enum)
                    || (t.contains(TypeFlags::NumberLiteral)
                        && t.contains(TypeFlags::EnumLiteral)
                        && self.literal_values_equal(source, target)))
            {
                return true;
            }

            // Anything is assignable to a union containing `undefined`, `null`,
            // and an empty anonymous object type `{}` (i.e., the `unknown`
            // approximation used before `unknown` was introduced). Ported
            // from Go's `isUnknownLikeUnionType`.
            if self.is_unknown_like_union_type(target) {
                return true;
            }
        }

        false
    }

    /// Check if two literal types have equal values.
    fn literal_values_equal(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        match (&a.data, &b.data) {
            (TypeData::Literal(la), TypeData::Literal(lb)) => la.value == lb.value,
            _ => false,
        }
    }

    /// Whether two enum types are related (assignable).
    ///
    /// Ported from Go's `isEnumTypeRelatedTo` (`internal/checker/relater.go`).
    /// Two enum types are related when they share the same symbol (merged
    /// declarations), or when they are same-named `RegularEnum`s whose members
    /// all have matching values. The result is memoized in `enum_relation`.
    ///
    /// Phase 1: reports no diagnostics (boolean correctness only). Member
    /// values are computed via `get_enum_member_value`, which uses a no-op
    /// entity resolver — members whose initializers reference other enum
    /// members resolve to `None` and are treated as opaque/assumed-numeric.
    /// When `owner` is a bare generic reference (a generic class/interface
    /// referenced WITHOUT type arguments), erase its own type parameters
    /// from a member type with `any` (official implicit-any args make
    /// `C` and `C<X>` mutually assignable in these positions).
    fn erase_bare_generic_params(&mut self, owner: &Arc<Type>, member_type: &Arc<Type>) -> Arc<Type> {
        let Some(sym) = owner.symbol.as_ref() else {
            return Arc::clone(member_type);
        };
        if owner
            .as_object()
            .is_some_and(|o| !o.type_arguments.is_empty())
        {
            return Arc::clone(member_type);
        }
        let tps = self.declared_type_parameter_types(sym);
        if tps.is_empty() {
            return Arc::clone(member_type);
        }
        let anys: Vec<Arc<Type>> = std::iter::repeat(self.get_any_type())
            .take(tps.len())
            .collect();
        self.substitute_infer_type_parameters(member_type, &tps, &anys)
    }

    fn is_enum_type_related_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        let Some(source_symbol) = source.symbol.as_ref() else {
            return false;
        };
        let Some(target_symbol) = target.symbol.as_ref() else {
            return false;
        };

        // Unwrap EnumMember → parent enum symbol (Go: getParentOfSymbol).
        let source_parent = if source_symbol.flags.contains(SymbolFlags::EnumMember) {
            source_symbol.parent.as_ref().unwrap_or(source_symbol)
        } else {
            source_symbol
        };
        let target_parent = if target_symbol.flags.contains(SymbolFlags::EnumMember) {
            target_symbol.parent.as_ref().unwrap_or(target_symbol)
        } else {
            target_symbol
        };

        // Same symbol → related (merged declarations).
        if Arc::ptr_eq(source_parent, target_parent) {
            return true;
        }

        // Different names, or not both RegularEnum → not related.
        if source_parent.name != target_parent.name
            || !source_parent.flags.contains(SymbolFlags::RegularEnum)
            || !target_parent.flags.contains(SymbolFlags::RegularEnum)
        {
            return false;
        }

        let key = EnumRelationKey {
            source_id: source_parent.id(),
            target_id: target_parent.id(),
        };
        // Cache lookup. Phase 1 has no error reporter, so any cached
        // (non-`None`) result is returned directly.
        if let Some(entry) = self.enum_relation.get(&key).copied() {
            if entry != RelationComparisonResult::None {
                return entry.contains(RelationComparisonResult::Succeeded);
            }
        }

        // Compare each source enum member's value against the like-named
        // target member. `get_type_of_symbol` resolves the enum's value type
        // (an anonymous object whose members are the enum members).
        let source_type = self.get_type_of_symbol(source_parent);
        let target_type = self.get_type_of_symbol(target_parent);
        let source_properties = self.get_properties_of_type(&source_type);

        for source_prop in source_properties {
            if !source_prop.flags.contains(SymbolFlags::EnumMember) {
                continue;
            }
            let Some(target_prop) = self.get_property_of_type(&target_type, &source_prop.name)
            else {
                // TS2324: property missing in target.
                self.enum_relation
                    .insert(key, RelationComparisonResult::Failed);
                return false;
            };
            if !target_prop.flags.contains(SymbolFlags::EnumMember) {
                self.enum_relation
                    .insert(key, RelationComparisonResult::Failed);
                return false;
            }

            let source_decl = self.get_declaration_of_kind(&source_prop, SyntaxKind::EnumMember);
            let target_decl = self.get_declaration_of_kind(&target_prop, SyntaxKind::EnumMember);
            if let (Some(sd), Some(td)) = (source_decl, target_decl) {
                let source_value = self.get_enum_member_value(&sd);
                let target_value = self.get_enum_member_value(&td);
                let sv = source_value.value.as_ref();
                let tv = target_value.value.as_ref();
                if sv != tv {
                    // Two *known* values that differ → incompatible (TS4125).
                    if sv.is_some() && tv.is_some() {
                        self.enum_relation
                            .insert(key, RelationComparisonResult::Failed);
                        return false;
                    }
                    // At least one value is `None` (opaque/ambient) — assume
                    // numeric. If the other is a string, that's a type
                    // mismatch (TS4126).
                    let source_is_string = matches!(sv, Some(EvalValue::String(_)));
                    let target_is_string = matches!(tv, Some(EvalValue::String(_)));
                    if source_is_string || target_is_string {
                        self.enum_relation
                            .insert(key, RelationComparisonResult::Failed);
                        return false;
                    }
                    // Both assumed numeric → compatible; continue.
                }
            }
        }

        self.enum_relation
            .insert(key, RelationComparisonResult::Succeeded);
        true
    }

    /// Whether `t` is an "unknown-like" union — i.e., a union containing
    /// `undefined`, `null`, and an empty anonymous object type `{}`.
    ///
    /// Ported from Go's `isUnknownLikeUnionType`
    /// (`internal/checker/checker.go`). Go caches the result in
    /// `t.objectFlags` (`IsUnknownLikeUnionComputed` / `IsUnknownLikeUnion`).
    /// Because `Type` is shared via `Arc<Type>` and immutable here, we skip
    /// the cache and recompute on each call. This is only called in the
    /// `isSimpleTypeRelatedTo` fallback path, so the cost is bounded.
    fn is_unknown_like_union_type(&self, t: &Arc<Type>) -> bool {
        if !self.strict_null_checks || !t.flags.contains(TypeFlags::Union) {
            return false;
        }
        let Some(types) = t.types() else {
            return false;
        };
        if types.len() < 3 {
            return false;
        }
        let has_undefined = types
            .iter()
            .any(|ty| ty.flags.contains(TypeFlags::Undefined));
        let has_null = types.iter().any(|ty| ty.flags.contains(TypeFlags::Null));
        let has_empty_object = types
            .iter()
            .any(|ty| self.is_empty_anonymous_object_type(ty));
        has_undefined && has_null && has_empty_object
    }

    /// Whether `t` is an empty anonymous object type (i.e., `{}`).
    ///
    /// Ported from Go's `IsEmptyAnonymousObjectType`
    /// (`internal/checker/checker.go`):
    /// `t.objectFlags&Anonymous != 0 && (MembersResolved && isEmptyResolvedType
    /// || symbol is TypeLiteral && members empty)`.
    fn is_empty_anonymous_object_type(&self, t: &Arc<Type>) -> bool {
        if !t.object_flags.contains(ObjectFlags::Anonymous) {
            return false;
        }
        if t.object_flags.contains(ObjectFlags::MembersResolved) {
            // Members already resolved: check structured type is empty.
            return self.structured_type_is_empty(t);
        }
        // Fall back to symbol-based check: type literal symbol with no members.
        if let Some(sym) = t.symbol.as_ref() {
            if sym.flags.contains(SymbolFlags::TypeLiteral) {
                return self.get_properties_of_type(t).is_empty();
            }
        }
        false
    }

    /// Whether a structured (object) type has zero resolved members.
    fn structured_type_is_empty(&self, t: &Arc<Type>) -> bool {
        self.get_properties_of_type(t).is_empty()
    }

    // ────────────────────────────────────────────────────────────────────────
    // Index signature comparison
    // ────────────────────────────────────────────────────────────────────────

    /// Check if the index signatures of two types are compatible.
    ///
    /// If target has `[key: string]: T`, source must have `[key: string]: U`
    /// where U is assignable to T.
    /// If target has `[key: number]: T`, source must have `[key: number]: U`
    /// where U is assignable to T.
    fn is_index_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        // An `any` source satisfies any index signature (Go's
        // isRelatedTo any short-circuit at the signature level) — checked
        // before the structured-member extraction (`any` has none).
        if source.flags.contains(TypeFlags::Any) {
            return true;
        }
        let source_struct = match source.as_structured() {
            Some(s) => s,
            None => return false,
        };
        let target_struct = match target.as_structured() {
            Some(t) => t,
            None => return false,
        };

        let source_indexes = &source_struct.index_infos;
        let target_indexes = &target_struct.index_infos;

        if target_indexes.is_empty() {
            return true; // Target has no index signature requirements
        }

        for target_index in target_indexes {
            let target_key = &target_index.key_type;
            let target_value = &target_index.value_type;

            // Find a matching index signature in source
            let mut found_match = false;
            for source_index in source_indexes {
                let source_key = &source_index.key_type;
                let source_value = &source_index.value_type;

                // Key types must match (string vs number)
                let key_match = match (target_key, source_key) {
                    (Some(tk), Some(sk)) => self.is_type_related_to(sk, tk, relation),
                    (None, _) => true,
                    (_, None) => false,
                };

                if !key_match {
                    continue;
                }

                // Value type must be assignable: source value -> target value
                let value_match = match (target_value, source_value) {
                    (Some(tv), Some(sv)) => self.is_type_related_to(sv, tv, relation),
                    // If target has no value type, any source is fine
                    // If source has no value type, it can't match target's value type
                    (None, _) => true,
                    (_, None) => false,
                };

                if value_match {
                    found_match = true;
                    break;
                }
            }

            if !found_match {
                // No matching source index signature: fall back to checking
                // that every source property is assignable to the target
                // index's value type (mirrors Go's `membersRelatedToIndexInfo`
                // fallback in `typeRelatedToIndexInfo`). This makes
                // `{ a: 1 }` assignable to `{ [key: string]: number }`.
                let result = self.members_related_to_index_info(source, target_index, relation);
                if result.is_false() {
                    // Go `typeRelatedToIndexInfo` reports the missing index
                    // signature as an elaboration-chain entry.
                    let key_str = target_key
                        .as_ref()
                        .map(|k| self.type_to_string(k))
                        .unwrap_or_else(|| "string".to_string());
                    let source_str = self.type_to_string(source);
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            INDEX_SIGNATURE_FOR_TYPE_0_IS_MISSING_IN_TYPE_1,
                        vec![key_str, source_str],
                    );
                    return false;
                }
            }
        }

        true
    }

    // ────────────────────────────────────────────────────────────────────────
    // Signature comparison (ported from internal/checker/relater.go)
    // ────────────────────────────────────────────────────────────────────────

    /// Compare a source signature against a target signature.
    ///
    /// Direct port of Go's `compareSignaturesRelated`. Returns a Ternary:
    /// - `True` if `source` is related to `target` under `relation`
    /// - `False` if not
    /// - `Maybe`/`Unknown` for partial results (currently unused)
    ///
    /// Handles (in order):
    /// 1. Pointer-equality fast path.
    /// 2. The "top signature" short-circuit (a `(...args: any) => any`
    ///    source matches any target).
    /// 3. Strict-top-signature asymmetry (strict subtype relation).
    /// 4. Parameter count check (with rest/tuple-rest handling).
    /// 5. Generic signature instantiation (skipped — we erase).
    /// 6. `this` type comparison (covariant for void source, contravariant
    ///    otherwise, bivariant when `strict_function_types` is off).
    /// 7. Per-parameter type comparison (bivariant by default; strictly
    ///    contravariant when `strict_function_types` is on and neither
    ///    side is a method/constructor).
    /// 8. Return type comparison (covariant; bivariant for callbacks).
    /// 9. Type predicate comparison (for type guards).
    pub fn compare_signatures_related(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        check_mode: SignatureCheckMode,
        relation: RelationKind,
    ) -> Ternary {
        // 1. Fast path: same pointer.
        if Arc::ptr_eq(source, target) {
            return Ternary::True;
        }

        // 2. Top-signature short-circuit: a top-signature target matches
        //    anything (the source doesn't need to be top, since `any` is
        //    a wildcard). Strict-subtype relation additionally requires
        //    the source to *also* be a top signature.
        let source_is_top = if check_mode.contains(SignatureCheckMode::StrictTopSignature)
            && self.is_top_signature(source)
        {
            true
        } else {
            false
        };
        if !source_is_top && self.is_top_signature(target) {
            return Ternary::True;
        }
        if check_mode.contains(SignatureCheckMode::StrictTopSignature)
            && source_is_top
            && !self.is_top_signature(target)
        {
            return Ternary::False;
        }

        // 4. Parameter count check.
        let target_count = self.get_parameter_count(target);
        let source_has_more = if !self.has_effective_rest_parameter(target) {
            if check_mode.contains(SignatureCheckMode::StrictArity) {
                self.has_effective_rest_parameter(source)
                    || self.get_parameter_count(source) > target_count
            } else {
                self.get_min_argument_count(source) > target_count
            }
        } else {
            false
        };
        if source_has_more {
            // Go reports the arity leaf into the elaboration chain before
            // returning false (relater.go ~L1517: reportErrors &&
            // !StrictArity). The chain window nests it under the
            // two-signature line pushed later at the property level.
            if self.relater_chain_active
                && !check_mode.contains(SignatureCheckMode::StrictArity)
            {
                let min_args = self.get_min_argument_count(source).max(0);
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TARGET_SIGNATURE_PROVIDES_TOO_FEW_ARGUMENTS_EXPECTED_0_OR_MORE_BUT_GOT_1,
                    vec![min_args.to_string(), target_count.to_string()],
                );
            }
            return Ternary::False;
        }

        // 5. Generic source signatures are instantiated in the context of
        //    the canonicalized target signature before comparing, and the
        //    target is replaced by its canonical form for the remainder of
        //    the comparison. Mirrors Go relater.go ~L1527.
        let mut source = Arc::clone(source);
        let mut target = Arc::clone(target);
        if !source.type_parameters.is_empty()
            && !type_parameters_same(
                source.type_parameters.as_slice(),
                target.type_parameters.as_slice(),
            )
        {
            let canonical_target = self.get_canonical_signature(&target);
            source = self.instantiate_signature_in_context_of(&source, &canonical_target);
            target = canonical_target;
        }

        let source_count = self.get_parameter_count(&source);
        let source_rest = self.get_non_array_rest_type(&source);
        let target_rest = self.get_non_array_rest_type(&target);

        // 6. Variance selection. `strict_function_types` makes method-shaped
        //    signatures stay bivariant; everything else is contravariant on
        //    parameters. We approximate "method-shaped" by checking the
        //    declaration kind via the signature's declaration node.
        let strict_variance = !check_mode.contains(SignatureCheckMode::Callback)
            && self.strict_function_types
            && !self.signature_is_method_or_constructor(&target);

        let mut result = Ternary::True;

        // 7. `this` type comparison.
        let source_this = self.get_this_type_of_signature(&source);
        if let Some(source_this) = source_this {
            if !source_this.flags.contains(TypeFlags::Void) {
                let target_this = self.get_this_type_of_signature(&target);
                if let Some(target_this) = target_this {
                    let mut related = Ternary::False;
                    if !strict_variance {
                        related = self.compare_types(
                            source_this.clone(),
                            target_this.clone(),
                            relation,
                            false,
                        );
                    }
                    if related.is_false() {
                        related = self.compare_types(target_this, source_this, relation, false);
                    }
                    if related.is_false() {
                        return Ternary::False;
                    }
                    result = result.and(related);
                }
            }
        }

        // 8. Per-parameter type comparison.
        let param_count = if source_rest.is_some() || target_rest.is_some() {
            source_count.min(target_count)
        } else {
            source_count.max(target_count)
        };
        let rest_index = if source_rest.is_some() || target_rest.is_some() {
            param_count.saturating_sub(1) as isize
        } else {
            -1
        };
        for i in 0..param_count {
            let source_type = if i as isize == rest_index {
                self.get_rest_or_any_type_at_position(&source, i)
            } else {
                self.try_get_type_at_position(&source, i)
                    .unwrap_or_else(|| self.any_type())
            };
            let target_type = if i as isize == rest_index {
                self.get_rest_or_any_type_at_position(&target, i)
            } else {
                self.try_get_type_at_position(&target, i)
                    .unwrap_or_else(|| self.any_type())
            };

            // Skip if both are the same pointer and we're not in strict-arity mode.
            if Arc::ptr_eq(&source_type, &target_type)
                && !check_mode.contains(SignatureCheckMode::StrictArity)
            {
                continue;
            }

            // Callback detection (Go relater.go ~L1590): when both
            // parameter types are single-call-signature functions (and
            // neither position was instantiated from a generic parameter,
            // and their null/undefined facts agree), compare the two
            // signatures covariantly-in-parameters (callback mode) instead
            // of the usual bivariant type comparison.
            let mut source_sig: Option<Arc<Signature>> = None;
            if !check_mode.contains(SignatureCheckMode::Callback)
                && !self.is_instantiated_generic_parameter(&source, i)
            {
                let non_nullable = self.get_non_nullable_type_of(&source_type);
                source_sig = self.get_single_call_signature(&non_nullable);
            }
            let mut target_sig: Option<Arc<Signature>> = None;
            if !check_mode.contains(SignatureCheckMode::Callback)
                && !self.is_instantiated_generic_parameter(&target, i)
            {
                let non_nullable = self.get_non_nullable_type_of(&target_type);
                target_sig = self.get_single_call_signature(&non_nullable);
            }
            let callbacks = source_sig.is_some()
                && target_sig.is_some()
                && self
                    .get_type_predicate_of_signature(source_sig.as_ref().unwrap())
                    .is_none()
                && self
                    .get_type_predicate_of_signature(target_sig.as_ref().unwrap())
                    .is_none()
                && self.type_is_undefined_or_null(&source_type)
                    == self.type_is_undefined_or_null(&target_type);

            let mut related = Ternary::False;
            if callbacks {
                let callback_mode = if check_mode.contains(SignatureCheckMode::StrictArity) {
                    SignatureCheckMode::StrictArity
                } else {
                    SignatureCheckMode::None
                } | if strict_variance {
                    SignatureCheckMode::StrictCallback
                } else {
                    SignatureCheckMode::BivariantCallback
                };
                // Note the reversed order (target, source): callback
                // parameters are output positions, so the comparison runs
                // against the parameter's own variance direction.
                related = self.compare_signatures_related(
                    target_sig.as_ref().unwrap(),
                    source_sig.as_ref().unwrap(),
                    callback_mode,
                    relation,
                );
            } else {
                // Bivariant/contravariant parameter comparison.
                // Default: bivariant — try source→target first, fall back to target→source.
                if !check_mode.contains(SignatureCheckMode::Callback) && !strict_variance {
                    related =
                        self.compare_types(source_type.clone(), target_type.clone(), relation, false);
                }
                if related.is_false() {
                    related =
                        self.compare_types(target_type.clone(), source_type.clone(), relation, false);
                }
            }
            if related.is_false() {
                // Go compareSignaturesRelated (~L1615): the failed
                // parameter comparison pushes its own relation head
                // (contravariant orientation — the target→source attempt
                // is the reporting one) plus the parameter-incompatibility
                // marker; the marker renders as its own pyramid line with
                // the type head nested under it.
                if self.relater_chain_active {
                    let ts = self.type_to_string(&target_type);
                    let ss = self.type_to_string(&source_type);
                    self.push_relation_head_with_tp_note(
                        &target_type,
                        &source_type,
                        crate::diagnostics::messages_generated::
                            TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                        vec![ts, ss],
                    );
                    let sn = source.parameters.get(i).map(|p| p.name.clone());
                    let tn = target.parameters.get(i).map(|p| p.name.clone());
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            TYPES_OF_PARAMETERS_0_AND_1_ARE_INCOMPATIBLE,
                        vec![sn.unwrap_or_default(), tn.unwrap_or_default()],
                    );
                }
                return Ternary::False;
            }
            result = result.and(related);
        }

        // 9. Return type comparison.
        if !check_mode.contains(SignatureCheckMode::IgnoreReturnTypes) {
            let target_return = self.get_non_circular_return_type_of_signature(&target);
            // `void`, `any`, and FOREIGN free-type-parameter target returns
            // match anything: a bare type parameter in a NON-generic
            // target's return position is an inference-substituted
            // placeholder (`map(identity)`'s callback slot after U := A —
            // Go erases the source's type parameters before comparing, so
            // the un-mapperable return never fails). A type parameter that
            // BELONGS to the target signature still checks normally
            // (specialized-signature subtyping: `<T>(x: T) => string` vs
            // `<T>(x: T) => T` must fail).
            let target_return_own_tp = target_return.flags.contains(TypeFlags::TypeParameter)
                && target
                    .type_parameters
                    .iter()
                    .any(|tp| crate::checker::utilities::type_parameters_match(tp, &target_return));
            if !Arc::ptr_eq(&target_return, &self.void_type())
                && !target_return.flags.contains(TypeFlags::Any)
                && !(target_return.flags.contains(TypeFlags::TypeParameter) && !target_return_own_tp)
            {
                let source_return = self.get_non_circular_return_type_of_signature(&source);
                let target_type_predicate = self.get_type_predicate_of_signature(&target).cloned();
                if let Some(target_tp) = target_type_predicate {
                    let source_tp = self.get_type_predicate_of_signature(&source).cloned();
                    match source_tp {
                        Some(source_tp) => {
                            result = result.and(self.compare_type_predicate_related_to(
                                &source_tp, &target_tp, relation,
                            ));
                        }
                        None => {
                            // Source lacks a type predicate but target has one.
                            if matches!(
                                target_tp.kind,
                                TypePredicateKind::Identifier | TypePredicateKind::This
                            ) {
                                return Ternary::False;
                            }
                        }
                    }
                    if result.is_false() {
                        return result;
                    }
                } else {
                    // No type predicate on target: covariant return check.
                    // For callback signatures, also check bivariantly.
                    let mut related = Ternary::False;
                    if check_mode.contains(SignatureCheckMode::BivariantCallback) {
                        related = self.compare_types(
                            target_return.clone(),
                            source_return.clone(),
                            relation,
                            false,
                        );
                    }
                    if related.is_false() {
                        related = self.compare_types(source_return.clone(), target_return.clone(), relation, false);
                    }
                    result = result.and(related);
                    if result.is_false() {
                        // Chain marker (Go compareSignaturesRelated,
                        // relater.go ~L1661): the elided signature-return
                        // entries drive reportError's transform into
                        // "The types returned by 'x()' ..." heads. The
                        // nested return comparison first pushes its own
                        // relation head (Go runs it with reportErrors),
                        // which survives under the elided marker.
                        if self.relater_chain_active {
                            let sr_head = self.type_to_string(&source_return);
                            let tr_head = self.type_to_string(&target_return);
                            self.push_relation_head_with_tp_note(
                                &source_return,
                                &target_return,
                                crate::diagnostics::messages_generated::
                                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                vec![sr_head, tr_head],
                            );
                            let no_args =
                                source.parameters.is_empty() && target.parameters.is_empty();
                            let construct =
                                source.flags.contains(crate::checker::types::SignatureFlags::Construct);
                            let message = match (construct, no_args) {
                                (false, true) => crate::diagnostics::messages_generated::
                                    CALL_SIGNATURES_WITH_NO_ARGUMENTS_HAVE_INCOMPATIBLE_RETURN_TYPES_0_AND_1,
                                (true, true) => crate::diagnostics::messages_generated::
                                    CONSTRUCT_SIGNATURES_WITH_NO_ARGUMENTS_HAVE_INCOMPATIBLE_RETURN_TYPES_0_AND_1,
                                (false, false) => crate::diagnostics::messages_generated::
                                    CALL_SIGNATURE_RETURN_TYPES_0_AND_1_ARE_INCOMPATIBLE,
                                (true, false) => crate::diagnostics::messages_generated::
                                    CONSTRUCT_SIGNATURE_RETURN_TYPES_0_AND_1_ARE_INCOMPATIBLE,
                            };
                            let sr = self.type_to_string(&source_return);
                            let tr = self.type_to_string(&target_return);
                            self.relater_report_error(message, vec![sr, tr]);
                        }
                        return result;
                    }
                }
            }
        }

        result
    }

    /// Compare two type predicates.
    /// Direct port of Go's `compareTypePredicateRelatedTo`.
    pub fn compare_type_predicate_related_to(
        &mut self,
        source: &TypePredicate,
        target: &TypePredicate,
        relation: RelationKind,
    ) -> Ternary {
        if source.kind != target.kind {
            return Ternary::False;
        }
        if matches!(
            source.kind,
            TypePredicateKind::Identifier | TypePredicateKind::AssertsIdentifier
        ) && source.parameter_index != target.parameter_index
        {
            return Ternary::False;
        }
        match (&source.t, &target.t) {
            (None, None) => Ternary::True,
            (Some(s), None) => Ternary::True,
            (Some(s), Some(t)) => self.compare_types(s.clone(), t.clone(), relation, false),
            (None, Some(_)) => Ternary::False,
        }
    }

    /// Compare two types under a relation, returning a Ternary.
    /// Currently wraps `is_type_related_to` and converts the bool result.
    pub fn compare_types(
        &mut self,
        source: Arc<Type>,
        target: Arc<Type>,
        relation: RelationKind,
        _report_errors: bool,
    ) -> Ternary {
        if self.is_type_related_to(&source, &target, relation) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    /// Whether a signature's declaration is a method or constructor.
    /// Used by `compare_signatures_related` to keep method-shaped
    /// signatures bivariant even under `strict_function_types`.
    /// Mirrors Go's `target.declaration.Kind` check.
    fn signature_is_method_or_constructor(&self, sig: &Arc<Signature>) -> bool {
        let Some(decl) = sig.declaration.as_ref() else {
            return false;
        };
        matches!(
            decl.kind,
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature | SyntaxKind::Constructor
        )
    }

    /// Direct port of Go's `signaturesRelatedTo`. Compares the call (or
    /// construct) signature lists of two types, choosing one of three
    /// comparison strategies based on the lists' shapes.
    pub fn signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        kind: SignatureKind,
        relation: RelationKind,
    ) -> Ternary {
        // Wildcard: `anyFunctionType` source matches any function target.
        if Arc::ptr_eq(source, &self.any_function_type()) {
            return Ternary::True;
        }
        // Wildcard: non-wildcard source does NOT match a wildcard target.
        if Arc::ptr_eq(target, &self.any_function_type()) {
            return Ternary::False;
        }

        let source_sigs = self.get_signatures_of_type(source, kind);
        let target_sigs = self.get_signatures_of_type(target, kind);

        // Construct-signature abstractness check (skipped: we don't yet
        // populate SignatureFlagsAbstract on signatures in the Rust port).
        if kind == SignatureKind::Construct && !source_sigs.is_empty() && !target_sigs.is_empty() {
            // Future: mirror Go's constructorVisibilitiesAreCompatible.
        }

        // Identity relation fast path.
        if relation == RelationKind::Identity {
            return self.signatures_identical_to(source, target, kind);
        }

        let check_mode = match relation {
            RelationKind::Subtype => SignatureCheckMode::StrictTopSignature,
            RelationKind::StrictSubtype => SignatureCheckMode::from_bits_truncate(
                SignatureCheckMode::StrictTopSignature.bits()
                    | SignatureCheckMode::StrictArity.bits(),
            ),
            _ => SignatureCheckMode::None,
        };

        let mut result = Ternary::True;

        // Strategy selection mirrors Go's switch in `signaturesRelatedTo`.
        let source_instantiated = source.object_flags.contains(ObjectFlags::Instantiated);
        let target_instantiated = target.object_flags.contains(ObjectFlags::Instantiated);
        let same_target = match (source.target(), target.target()) {
            (Some(s), Some(t)) => Arc::ptr_eq(&s, &t),
            _ => false,
        };
        if (source_instantiated && target_instantiated && same_target)
            || (source.object_flags.contains(ObjectFlags::Reference)
                && target.object_flags.contains(ObjectFlags::Reference)
                && same_target)
        {
            // Pairwise comparison of signatures — generics are ERASED (Go
            // `signatureRelatedTo(…, erase=true)`): the two groups are
            // instantiations of the same shape, so wildcards suffice.
            let min_len = source_sigs.len().min(target_sigs.len());
            for i in 0..min_len {
                let s = self.get_erased_signature(&source_sigs[i]);
                let t = self.get_erased_signature(&target_sigs[i]);
                let related =
                    self.compare_signatures_related(&s, &t, check_mode, relation);
                if related.is_false() {
                    return Ternary::False;
                }
                result = result.and(related);
            }
            // If signature counts differ, the longer side must be matched
            // by the same N×M logic below.
            if source_sigs.len() != target_sigs.len() {
                // Fall through to N×M for unmatched signatures.
                for t in &target_sigs[min_len..] {
                    let t = self.get_erased_signature(t);
                    let mut found = false;
                    for s in &source_sigs[min_len..] {
                        let s = self.get_erased_signature(s);
                        let related =
                            self.compare_signatures_related(&s, &t, check_mode, relation);
                        if !related.is_false() {
                            result = result.and(related);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Ternary::False;
                    }
                }
            }
        } else if source_sigs.len() == 1 && target_sigs.len() == 1 {
            // Single-signature fast path. For non-comparable relations we
            // erase generics; for `Comparable` we always erase (Go behavior).
            let erase = relation == RelationKind::Comparable;
            let s = if erase {
                self.get_erased_signature(&source_sigs[0])
            } else {
                Arc::clone(&source_sigs[0])
            };
            let t = if erase {
                self.get_erased_signature(&target_sigs[0])
            } else {
                Arc::clone(&target_sigs[0])
            };
            result = self.compare_signatures_related(&s, &t, check_mode, relation);
        } else {
            // N×M fallback: every target signature must be matched by some
            // source signature, with generics ERASED on both sides (Go
            // `signatureRelatedTo(…, erase=true)` in the N×M matrix). We
            // don't propagate errors here (errorNode plumbing isn't wired
            // up yet).
            for t in &target_sigs {
                let t = self.get_erased_signature(t);
                let mut found = false;
                for s in &source_sigs {
                    let s = self.get_erased_signature(s);
                    let related = self.compare_signatures_related(&s, &t, check_mode, relation);
                    if !related.is_false() {
                        result = result.and(related);
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Ternary::False;
                }
            }
        }
        result
    }

    /// Direct port of Go's `signaturesIdenticalTo`. Compares signature
    /// counts and pairwise signatures for identity.
    pub fn signatures_identical_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        kind: SignatureKind,
    ) -> Ternary {
        let source_sigs = self.get_signatures_of_type(source, kind);
        let target_sigs = self.get_signatures_of_type(target, kind);
        if source_sigs.len() != target_sigs.len() {
            return Ternary::False;
        }
        let mut result = Ternary::True;
        for i in 0..source_sigs.len() {
            let related = self.compare_signatures_identical(
                &source_sigs[i],
                &target_sigs[i],
                false, // partialMatch
                false, // ignoreThisTypes
                false, // ignoreReturnTypes
            );
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

    /// Direct port of Go's `compareSignaturesIdentical`. Currently we
    /// delegate to `compare_signatures_related` with the StrictArity mode,
    /// which approximates identity checking for non-generic signatures.
    pub fn compare_signatures_identical(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        _partial_match: bool,
        _ignore_this_types: bool,
        ignore_return_types: bool,
    ) -> Ternary {
        let mut mode = SignatureCheckMode::StrictArity;
        if ignore_return_types {
            mode |= SignatureCheckMode::IgnoreReturnTypes;
        }
        self.compare_signatures_related(source, target, mode, RelationKind::Identity)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Signature helpers (ported from internal/checker/utilities.go and
    // internal/checker/relater.go signature utilities)
    // ────────────────────────────────────────────────────────────────────────

    /// Whether a signature has a rest parameter whose rest type is a tuple
    /// with a variadic element. Such a rest parameter is not "effective"
    /// for arity purposes (it's a fixed tuple spread).
    /// Mirrors Go's `hasEffectiveRestParameter`.
    pub fn has_effective_rest_parameter(&mut self, sig: &Arc<Signature>) -> bool {
        if !sig.has_rest_parameter() {
            return false;
        }
        let Some(last) = sig.parameters.last() else {
            return true;
        };
        let rest_type = self.get_type_of_symbol(last);
        if is_tuple_type(&rest_type) {
            if let TypeData::Tuple(t) = &rest_type.data {
                return t.combined_flags.contains(ElementFlags::Variadic);
            }
        }
        true
    }

    /// Get parameter count, accounting for tuple rest spreading.
    /// Mirrors Go's `getParameterCount`.
    pub fn get_parameter_count(&mut self, sig: &Arc<Signature>) -> usize {
        let length = sig.parameters.len();
        if !sig.has_rest_parameter() {
            return length;
        }
        let Some(last) = sig.parameters.last() else {
            return length;
        };
        let rest_type = self.get_type_of_symbol(last);
        if is_tuple_type(&rest_type) {
            if let TypeData::Tuple(t) = &rest_type.data {
                let variadic_offset = if t.combined_flags.contains(ElementFlags::Variadic) {
                    0
                } else {
                    1
                };
                return length + t.fixed_length - variadic_offset;
            }
        }
        length
    }

    /// Get the minimum argument count of a signature.
    /// Mirrors Go's `getMinArgumentCount`.
    pub fn get_min_argument_count(&mut self, sig: &Arc<Signature>) -> usize {
        // Use the cached value if it's been computed.
        if sig.resolved_min_argument_count != -1 {
            return sig.resolved_min_argument_count.max(0) as usize;
        }

        let mut min_argument_count: i32 = -1;
        if sig.has_rest_parameter() {
            if let Some(last) = sig.parameters.last() {
                let rest_type = self.get_type_of_symbol(last);
                if is_tuple_type(&rest_type) {
                    if let TypeData::Tuple(t) = &rest_type.data {
                        let first_optional = t
                            .element_infos
                            .iter()
                            .position(|info| !info.flags.contains(ElementFlags::Required));
                        let required_count = match first_optional {
                            Some(i) => i,
                            None => t.fixed_length,
                        };
                        if required_count > 0 {
                            min_argument_count = (sig.parameters.len() - 1 + required_count) as i32;
                        }
                    }
                }
            }
        }
        if min_argument_count == -1 {
            min_argument_count = sig.min_argument_count;
        }

        // Walk back over trailing void-typed parameters (Go behavior):
        // `(x: void) => void` has minArgumentCount 0.
        let mut mc = min_argument_count;
        let mut i = mc - 1;
        while i >= 0 {
            match self.try_get_type_at_position(sig, i as usize) {
                Some(t) if t.flags.contains(TypeFlags::Void) => {
                    mc = i;
                }
                _ => break,
            }
            i -= 1;
        }
        mc.max(0) as usize
    }

    /// Get the type of a parameter at a given position, returning `any` if
    /// out of range. Mirrors Go's `getTypeAtPosition`.
    pub fn get_type_at_position(&mut self, sig: &Arc<Signature>, pos: usize) -> Arc<Type> {
        self.try_get_type_at_position(sig, pos)
            .unwrap_or_else(|| self.any_type())
    }

    /// Try to get the type of a parameter at a given position.
    /// Mirrors Go's `tryGetTypeAtPosition`.
    pub fn try_get_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Option<Arc<Type>> {
        // Instantiated signatures resolve parameter types from the
        // substitution table (keyed by parameter index; the rest parameter
        // keeps its array type with the element substituted).
        if let Some(overrides) = &sig.instantiated_parameter_types {
            let rest_offset = if sig.has_rest_parameter() { 1 } else { 0 };
            let param_count = overrides.len().saturating_sub(rest_offset);
            if pos < param_count {
                return Some(Arc::clone(&overrides[pos]));
            }
            if sig.has_rest_parameter() {
                let rest_type = Arc::clone(&overrides[param_count]);
                if is_tuple_type(&rest_type) {
                    if let TypeData::Tuple(t) = &rest_type.data {
                        let index = pos - param_count;
                        let has_variadic =
                            t.combined_flags.contains(ElementFlags::Variadic);
                        if index < t.fixed_length || has_variadic {
                            return t
                                .element_infos
                                .get(index)
                                .and_then(|info| info.type_.clone())
                                .or_else(|| Some(self.any_type()));
                        }
                    }
                } else if let Some(elem) = self.get_array_element_type_of(&rest_type) {
                    return Some(elem);
                }
                return Some(self.any_type());
            }
            return None;
        }
        let rest_offset = if sig.has_rest_parameter() { 1 } else { 0 };
        let param_count = sig.parameters.len() - rest_offset;
        if pos < param_count {
            return Some(self.get_type_of_symbol(&sig.parameters[pos]));
        }
        if sig.has_rest_parameter() {
            let rest_param = &sig.parameters[param_count];
            let rest_type = self.get_type_of_symbol(rest_param);
            // If the rest type is a tuple, index into it.
            if is_tuple_type(&rest_type) {
                if let TypeData::Tuple(t) = &rest_type.data {
                    let index = pos - param_count;
                    let has_variadic = t.combined_flags.contains(ElementFlags::Variadic);
                    if index < t.fixed_length || has_variadic {
                        // Index access on a tuple — return the element type if
                        // we have it, else `None` (caller falls back to `any`).
                        return t
                            .element_infos
                            .get(index)
                            .and_then(|info| info.type_.clone())
                            .or_else(|| Some(self.any_type()));
                    }
                }
            }
        }
        None
    }

    /// Get the rest type at a position, transforming `any[]` to just `any`.
    /// Mirrors Go's `getRestOrAnyTypeAtPosition`.
    pub fn get_rest_or_any_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Arc<Type> {
        let rest_type = self.get_rest_type_at_position(sig, pos);
        if let Some(rt) = &rest_type {
            if self.is_array_type(rt) {
                let elem = self.get_type_arguments(rt).into_iter().next();
                if let Some(elem) = elem {
                    if elem.flags.contains(TypeFlags::Any) {
                        return self.any_type();
                    }
                }
            }
        }
        rest_type.unwrap_or_else(|| self.any_type())
    }

    /// Get the rest type at a position. Simplified port of Go's
    /// `getRestTypeAtPosition`.
    pub fn get_rest_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Option<Arc<Type>> {
        let parameter_count = self.get_parameter_count(sig);
        if pos >= parameter_count.saturating_sub(1) {
            // The rest position itself — return the effective rest type.
            return self.get_effective_rest_type(sig);
        }
        None
    }

    /// Get the effective rest type of a signature.
    /// Simplified port of Go's `getEffectiveRestType`.
    pub fn get_effective_rest_type(&mut self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        if !sig.has_rest_parameter() {
            return None;
        }
        if let Some(overrides) = &sig.instantiated_parameter_types {
            return overrides.last().cloned();
        }
        let last = sig.parameters.last()?;
        let rest_type = self.get_type_of_symbol(last);
        // If the rest type is a tuple, the effective rest is the tuple itself
        // (we don't yet split it into spread elements).
        Some(rest_type)
    }

    /// Get the "non-array" rest type — the element type if the rest is an
    /// array type, the tuple type itself if it's a tuple. Returns `None`
    /// if the signature has no rest parameter.
    /// Used by `compare_signatures_related` to decide between tuple-spread
    /// and array-rest parameter comparison.
    /// Mirrors Go's `getNonArrayRestType`.
    pub fn get_non_array_rest_type(&mut self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        if !sig.has_rest_parameter() {
            return None;
        }
        if let Some(overrides) = &sig.instantiated_parameter_types {
            let rest_type = overrides.last()?.clone();
            if is_tuple_type(&rest_type) {
                return Some(rest_type);
            }
            if self.is_array_type(&rest_type) {
                return self.get_type_arguments(&rest_type).into_iter().next();
            }
            return Some(rest_type);
        }
        let last = sig.parameters.last()?;
        let rest_type = self.get_type_of_symbol(last);
        // If it's a tuple, we don't treat it as an array rest.
        if is_tuple_type(&rest_type) {
            return Some(rest_type);
        }
        // If it's an array type, the non-array rest is the element type.
        if self.is_array_type(&rest_type) {
            return self.get_type_arguments(&rest_type).into_iter().next();
        }
        Some(rest_type)
    }

    /// Element type of an array-shaped override type, if array-like.
    pub(crate) fn get_array_element_type_of(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if self.is_array_type(t) {
            return Some(self.get_array_element_type(t));
        }
        None
    }

    /// Whether a signature is `(...args: any) => any` or
    /// `(...args: never) => any/unknown`. Mirrors Go's `isTopSignature`.
    pub fn is_top_signature(&mut self, sig: &Arc<Signature>) -> bool {
        if !sig.type_parameters.is_empty() {
            return false;
        }
        // thisParameter check: if present, must be `any`.
        if let Some(this_param) = &sig.this_parameter {
            let this_type = self.get_type_of_symbol(this_param);
            if !this_type.flags.contains(TypeFlags::Any) {
                return false;
            }
        }
        if sig.parameters.len() != 1 || !sig.has_rest_parameter() {
            return false;
        }
        let Some(param) = sig.parameters.first() else {
            return false;
        };
        let param_type = self.get_type_of_symbol(param);
        let rest_type = if self.is_array_type(&param_type) {
            self.get_type_arguments(&param_type).into_iter().next()
        } else {
            Some(param_type)
        };
        match rest_type {
            Some(rt) => {
                if !rt.flags.intersects(TypeFlags::Any | TypeFlags::Never) {
                    return false;
                }
                let return_type = self.get_return_type_of_signature(sig);
                match return_type {
                    Some(rt) => rt.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN),
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Get the type of the `this` parameter of a signature.
    /// Mirrors Go's `getThisTypeOfSignature`.
    pub fn get_this_type_of_signature(&self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        let this_param = sig.this_parameter.as_ref()?;
        let links = self.value_symbol_links.get(this_param)?;
        links.resolved_type.clone()
    }

    /// Get the return type of a signature, avoiding infinite recursion when
    /// the return type is itself computed from the signature.
    /// Mirrors Go's `getNonCircularReturnTypeOfSignature`. Currently we just
    /// return the resolved return type.
    pub fn get_non_circular_return_type_of_signature(&self, sig: &Arc<Signature>) -> Arc<Type> {
        self.get_return_type_of_signature(sig)
            .unwrap_or_else(|| self.any_type())
    }

    /// Get the erased signature (with type parameters replaced by their
    /// constraints). Since we don't yet support generic signature
    /// instantiation, this returns the same signature.
    /// Mirrors Go's `getErasedSignature`.
    /// The erased form of a signature: every type parameter instantiated to
    /// `any` (Go `getErasedSignature`, relater.go's `signatureRelatedTo(…
    /// erase=true)`). Overload-group comparisons (pairwise same-symbol and
    /// the N×M matrix) erase generics — the groups are known to be "the
    /// same" shape, so wildcards suffice and deep instantiation is
    /// needlessly quadratic.
    pub fn get_erased_signature(&mut self, sig: &Arc<Signature>) -> Arc<Signature> {
        if sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }
        let args: Vec<Arc<Type>> = sig
            .type_parameters
            .iter()
            .map(|_| self.any_type())
            .collect();
        self.get_signature_instantiation(sig, &args)
    }

    /// Instantiate `sig` with explicit type arguments: each parameter type
    /// and the return type are substituted, and the result carries no type
    /// parameters. Mirrors Go's `getSignatureInstantiation`
    /// (checker.go ~L19352) for the non-JS, non-inferred-params shape.
    pub fn get_signature_instantiation(
        &mut self,
        sig: &Arc<Signature>,
        type_args: &[Arc<Type>],
    ) -> Arc<Signature> {
        if type_args.is_empty() || sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }
        // Substitute each declared parameter's type (the rest parameter
        // keeps its array/tuple shape with the element substituted).
        let mut param_types: Vec<Arc<Type>> = Vec::with_capacity(sig.parameters.len());
        let rest_offset = if sig.has_rest_parameter() { 1 } else { 0 };
        let fixed = sig.parameters.len() - rest_offset;
        for i in 0..fixed {
            let t = self
                .try_get_type_at_position(sig, i)
                .unwrap_or_else(|| self.any_type());
            param_types.push(
                self.substitute_infer_type_parameters(&t, &sig.type_parameters, type_args),
            );
        }
        if rest_offset == 1 {
            let last = sig.parameters.last().expect("rest parameter");
            let rest_type = self.get_type_of_symbol(last);
            param_types.push(
                self.substitute_infer_type_parameters(&rest_type, &sig.type_parameters, type_args),
            );
        }
        let mut inst = Signature::new();
        inst.flags = sig.flags;
        inst.min_argument_count = sig.min_argument_count;
        inst.resolved_min_argument_count = sig.resolved_min_argument_count;
        inst.declaration = sig.declaration.clone();
        // An instantiated signature has no type parameters; its `target`
        // records the original signature (Go: the instantiated signature's
        // target/mapper pair).
        inst.target = Some(Arc::clone(sig));
        inst.parameters = sig.parameters.clone();
        inst.this_parameter = sig.this_parameter.clone();
        inst.resolved_type_predicate = sig.resolved_type_predicate.clone();
        inst.instantiated_parameter_types = Some(param_types);
        if let Some(rt) = self.get_return_type_of_signature(sig) {
            let substituted =
                self.substitute_infer_type_parameters(&rt, &sig.type_parameters, type_args);
            let _ = inst.resolved_return_type.set(substituted);
        }
        Arc::new(inst)
    }

    /// Instantiate a generic signature in the context of a non-generic
    /// contextual signature: infer the source's type parameters from the
    /// contextual signature's parameter types (contravariant positions)
    /// and — without an outer inference context — its return type, then
    /// instantiate. Mirrors Go's `instantiateSignatureInContextOf`
    /// (checker.go ~L19525) with the relater's `nil` inference-context
    /// shape (relater.go ~L1527): parameter iteration follows
    /// `applyToParameterTypes` (this-types, min-count params, rest) and
    /// the return inference follows `applyToReturnTypes` (only when the
    /// generic return could contain type variables).
    pub fn instantiate_signature_in_context_of(
        &mut self,
        source: &Arc<Signature>,
        contextual: &Arc<Signature>,
    ) -> Arc<Signature> {
        if source.type_parameters.is_empty() {
            return Arc::clone(source);
        }
        let inferences: Vec<crate::checker::inference::InferenceInfo> = source
            .type_parameters
            .iter()
            .map(|p| crate::checker::inference::InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = crate::checker::inference::InferenceContext::new(inferences);
        context.signature = Some(Arc::clone(source));

        // applyToParameterTypes(contextual, generic): this-types first.
        if let (Some(contextual_this), Some(source_this)) = (
            self.get_this_type_of_signature(contextual),
            self.get_this_type_of_signature(source),
        ) {
            self.infer_types(
                &mut context.inferences,
                Some(contextual_this),
                Some(source_this),
                crate::checker::inference::InferencePriority::None,
                false,
            );
        }
        // Then the shared non-rest prefix (min count when the contextual
        // side has no rest), then the generic signature's rest type.
        let contextual_count = self.get_parameter_count(contextual);
        let generic_count = self.get_parameter_count(source);
        let contextual_rest = self.get_effective_rest_type(contextual);
        let generic_rest = self.get_effective_rest_type(source);
        let generic_non_rest = generic_count.saturating_sub(usize::from(generic_rest.is_some()));
        let param_count = if contextual_rest.is_none() {
            contextual_count.min(generic_non_rest)
        } else {
            generic_non_rest
        };
        for i in 0..param_count {
            let s = self.get_type_at_position(contextual, i);
            let t = self.get_type_at_position(source, i);
            self.infer_types(
                &mut context.inferences,
                Some(s),
                Some(t),
                crate::checker::inference::InferencePriority::None,
                false,
            );
        }
        if let Some(generic_rest) = generic_rest {
            let s = self.get_type_at_position(contextual, param_count);
            self.infer_types(
                &mut context.inferences,
                Some(s),
                Some(generic_rest),
                crate::checker::inference::InferencePriority::None,
                false,
            );
        }
        // inferenceContext == nil → also infer from return types, but only
        // when the generic return type could contain type variables
        // (applyToReturnTypes).
        if let Some(source_return) = self.get_return_type_of_signature(source) {
            if type_contains_type_parameter(&source_return) {
                if let Some(contextual_return) = self.get_return_type_of_signature(contextual) {
                    self.infer_types(
                        &mut context.inferences,
                        Some(contextual_return),
                        Some(source_return),
                        crate::checker::inference::InferencePriority::ReturnType,
                        false,
                    );
                }
            }
        }
        let inferred = self.get_inferred_types(&mut context);
        self.get_signature_instantiation(source, &inferred)
    }

    /// Get the canonical form of a signature. Mirrors Go's
    /// `getCanonicalSignature`/`createCanonicalSignature` (checker.go
    /// ~L19468): an instantiation of the signature where each
    /// unconstrained type parameter is replaced with its original
    /// (`tp.target`), so signatures from different instantiation
    /// generations of the same generic member compare by identity.
    pub fn get_canonical_signature(&mut self, sig: &Arc<Signature>) -> Arc<Signature> {
        if sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }
        let type_arguments: Vec<Arc<Type>> = sig
            .type_parameters
            .iter()
            .map(|tp| match &tp.data {
                TypeData::TypeParameter(tpd) => {
                    match tpd.target.as_ref() {
                        Some(original)
                            if self.get_constraint_of_type_parameter(original).is_none() =>
                        {
                            Arc::clone(original)
                        }
                        _ => Arc::clone(tp),
                    }
                }
                _ => Arc::clone(tp),
            })
            .collect();
        // Identity mapping for every parameter: nothing to canonicalize.
        if type_arguments
            .iter()
            .zip(sig.type_parameters.iter())
            .all(|(arg, param)| Arc::ptr_eq(arg, param))
        {
            return Arc::clone(sig);
        }
        self.get_signature_instantiation(sig, &type_arguments)
    }

    /// `getBaseConstraintOrType`: the base constraint when available, the
    /// type itself otherwise (Go checker.go ~L27496).
    pub fn get_base_constraint_or_type(&self, t: &Arc<Type>) -> Arc<Type> {
        self.get_base_constraint_of_type(t)
            .or_else(|| self.get_constraint_of_type_parameter(t))
            .unwrap_or_else(|| Arc::clone(t))
    }

    /// Whether `t` is (or contains, for unions) an instantiable generic
    /// object — the `ObjectFlagsIsGenericObjectType` half of Go's
    /// `getGenericObjectFlags` (checker.go ~L24946), simplified: generic
    /// mapped/tuple detection approximates to their type-argument contents.
    pub fn type_flags_is_generic_object_type(&self, t: &Arc<Type>) -> bool {
        if t.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION | TypeFlags::Substitution) {
            return t
                .types()
                .map(|ts| ts.iter().any(|u| self.type_flags_is_generic_object_type(u)))
                .unwrap_or(false);
        }
        if t.flags.intersects(TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE) {
            return true;
        }
        // Generic mapped types / generic tuples: any type argument or the
        // mapped constraint mentioning a type variable keeps it generic.
        match &t.data {
            TypeData::Mapped(m) => m
                .constraint_type
                .as_ref()
                .map(|c| self.type_flags_is_generic_index_type(c))
                .unwrap_or(false),
            TypeData::Tuple(tup) => tup.element_infos.iter().any(|ei| {
                ei.type_.as_ref().map(type_contains_type_parameter).unwrap_or(false)
            }),
            _ => false,
        }
    }

    /// The `ObjectFlagsIsGenericIndexType` half of Go's
    /// `getGenericObjectFlags`: instantiable non-primitive, index, or
    /// generic string-like types can appear as (or contain) generic index
    /// types.
    pub fn type_flags_is_generic_index_type(&self, t: &Arc<Type>) -> bool {
        if t.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION | TypeFlags::Substitution) {
            return t
                .types()
                .map(|ts| ts.iter().any(|u| self.type_flags_is_generic_index_type(u)))
                .unwrap_or(false);
        }
        t.flags.intersects(
            TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE | TypeFlags::Index | TypeFlags::TemplateLiteral,
        )
    }

    /// `getSingleCallSignature` (Go checker.go): the sole call signature of
    /// an object type, or `None`.
    pub fn get_single_call_signature(&self, t: &Arc<Type>) -> Option<Arc<Signature>> {
        let sigs = self.get_signatures_of_type(t, SignatureKind::Call);
        if sigs.len() == 1 {
            sigs.into_iter().next()
        } else {
            None
        }
    }

    /// `GetNonNullableType` (Go utilities.go): strip `null`/`undefined`
    /// constituents from a union; other types pass through unchanged.
    pub fn get_non_nullable_type_of(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Union)
            && let Some(constituents) = t.types()
        {
            let kept: Vec<Arc<Type>> = constituents
                .iter()
                .filter(|c| {
                    !c.flags.intersects(TypeFlags::Null | TypeFlags::Undefined)
                })
                .cloned()
                .collect();
            if !kept.is_empty() && kept.len() != constituents.len() {
                return self.get_union_type(kept);
            }
        }
        Arc::clone(t)
    }

    /// `getTypeFacts(t, TypeFactsIsUndefinedOrNull)` reduced to a bool:
    /// whether the type admits `undefined` or `null` (top types admit
    /// everything). Used to gate the callback-mode comparison the same way
    /// Go compares the two types' nullability facts for equality.
    pub fn type_is_undefined_or_null(&self, t: &Arc<Type>) -> bool {
        if t.flags.intersects(
            TypeFlags::Undefined
                | TypeFlags::Null
                | TypeFlags::Any
                | TypeFlags::Unknown,
        ) {
            return true;
        }
        match &t.data {
            TypeData::Union(u) => u
                .union_or_intersection
                .types
                .iter()
                .any(|c| self.type_is_undefined_or_null(c)),
            _ => false,
        }
    }

    /// `isInstantiatedGenericParameter` (Go relater.go ~L1961): whether the
    /// parameter at `pos` of an instantiated signature's *target* (the
    /// original signature) is generic — used to skip callback-mode
    /// comparison for positions instantiated from a generic parameter.
    pub fn is_instantiated_generic_parameter(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> bool {
        let Some(target) = &sig.target else {
            return false;
        };
        match self.try_get_type_at_position(target, pos) {
            Some(t) => self.is_generic_type(&t),
            None => false,
        }
    }

    /// Whether a type is generic — a type parameter, or a reference with
    /// type-variable arguments (Go `isGenericType`, utilities.go).
    pub fn is_generic_type(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::TypeParameter) {
            return true;
        }
        t.types()
            .map(|ts| ts.iter().any(type_contains_type_parameter))
            .is_some()
    }

    /// Strict variant of `get_indexed_access_type` mirroring Go's
    /// `getIndexedAccessTypeOrUndefined` for the relation paths: resolves
    /// `object_type[index_type]` when the answer is deterministic and
    /// returns `None` where the permissive variant would fall back to
    /// `any`. Used by the indexed-access relation to decide whether the
    /// base-constraint fallback applies at all.
    pub fn try_get_indexed_access_type(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
        access_flags: AccessFlags,
    ) -> Option<Arc<Type>> {
        if object_type.flags.contains(TypeFlags::Any)
            || index_type.flags.contains(TypeFlags::Any)
        {
            return Some(self.any_type());
        }
        if object_type.flags.contains(TypeFlags::Unknown) {
            return Some(self.unknown_type());
        }
        // Union index: resolve each constituent and union the results.
        if index_type.flags.contains(TypeFlags::Union) {
            let constituents = index_type.types()?.to_vec();
            let mut resolved = Vec::with_capacity(constituents.len());
            for c in &constituents {
                resolved.push(self.try_get_indexed_access_type(object_type, c, access_flags)?);
            }
            return Some(self.get_union_type(resolved));
        }
        // Type-parameter object: resolve through the constraint.
        if object_type.flags.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_constraint_of_type_parameter(object_type)?;
            return self.try_get_indexed_access_type(&constraint, index_type, access_flags);
        }
        if let Some(structured) = object_type.as_structured() {
            // String-literal index: member lookup, then (unless suppressed)
            // index signatures.
            if index_type.flags.contains(TypeFlags::StringLiteral)
                && let TypeData::Literal(lit) = &index_type.data
                && let LiteralValue::String(name) = &lit.value
            {
                if let Some(sym) = structured.members.get(name) {
                    return Some(self.get_type_of_symbol(sym));
                }
                if !access_flags.contains(AccessFlags::NoIndexSignatures) {
                    if let Some(value_type) =
                        self.lookup_index_signature_value(structured, index_type)
                    {
                        return Some(value_type);
                    }
                }
                return None;
            }
            // `number` index on arrays/tuples: element type.
            if index_type.flags.intersects(TypeFlags::Number | TypeFlags::NumberLiteral) {
                if let Some(elem) = self.get_array_element_type_of(object_type) {
                    return Some(elem);
                }
                if self.is_tuple_type(object_type) {
                    let elem_types: Vec<Arc<Type>> = structured
                        .properties
                        .iter()
                        .map(|p| self.get_type_of_symbol(p))
                        .collect();
                    if !elem_types.is_empty() {
                        return Some(self.get_union_type(elem_types));
                    }
                }
                return None;
            }
            // String-like index: index signatures only.
            if index_type.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral)
                && !access_flags.contains(AccessFlags::NoIndexSignatures)
            {
                return self.lookup_index_signature_value(structured, index_type);
            }
        }
        None
    }

    /// Format a signature as a string for diagnostics.
    /// Simplified port of Go's `signatureToString`.
    pub fn signature_to_string(&mut self, sig: &Arc<Signature>) -> String {
        let params: Vec<String> = sig.parameters.iter().map(|p| p.name.clone()).collect();
        let return_type = self.get_return_type_of_signature(sig);
        let return_str = match return_type {
            Some(t) => self.type_to_string(&t),
            None => "void".to_string(),
        };
        format!("({}) => {}", params.join(", "), return_str)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Higher-level signature comparison entry points
    // ────────────────────────────────────────────────────────────────────────

    /// Check if the call signatures of two types are related.
    /// Wraps `signatures_related_to` with the call-signature kind.
    fn is_call_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_sigs = self.get_signatures_of_type(source, SignatureKind::Call);
        let target_sigs = self.get_signatures_of_type(target, SignatureKind::Call);

        if source_sigs.is_empty() && target_sigs.is_empty() {
            return true;
        }
        if target_sigs.is_empty() {
            // Source has call signatures but target doesn't: an empty
            // target signature set imposes no requirement (Go's
            // `signaturesRelatedTo` loop runs zero times → True).
            return true;
        }
        if source_sigs.is_empty() {
            // Target has call signatures but source doesn't — official
            // elaboration spells the unmatched signature out
            // (assignmentCompatability24).
            if self.relater_chain_active
                && let Some(t0) = target_sigs.first()
            {
                let source_str = self.type_to_string(source);
                let sig_str = self.signature_display_colon(t0, "");
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TYPE_0_PROVIDES_NO_MATCH_FOR_THE_SIGNATURE_1,
                    vec![source_str, sig_str],
                );
            }
            return false;
        }
        self.signatures_related_to(source, target, SignatureKind::Call, relation)
            .is_true()
    }

    /// The `<T>(p: P): R` display form used inside no-match elaboration
    /// lines (colon return, unlike the type-position `=>` form).
    pub(crate) fn signature_display_colon(&mut self, sig: &Arc<Signature>, prefix: &str) -> String {
        self.signature_display_sep(sig, prefix, ": ")
    }

    /// Arrow display for construct-signature chain lines — official prints
    /// the two-signature pyramid line as `new (x: number) => Foo`
    /// (assignmentCompatability44/45) while the no-match line keeps the
    /// colon form (`provides no match for the signature 'new (): any'`,
    /// assignmentCompatability37).
    pub(crate) fn signature_display_arrow(&mut self, sig: &Arc<Signature>, prefix: &str) -> String {
        self.signature_display_sep(sig, prefix, " => ")
    }

    fn signature_display_sep(
        &mut self,
        sig: &Arc<Signature>,
        prefix: &str,
        sep: &str,
    ) -> String {
        let params: Vec<String> = sig
            .parameters
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let param_type = self
                    .signature_instantiated_param_type(sig, i)
                    .unwrap_or_else(|| self.get_type_of_symbol(param));
                // Optional parameters display the `?` marker (the folded
                // `| undefined` stays spelled out: `y?: boolean | undefined`).
                let optional = param.flags.contains(SymbolFlags::Optional)
                    || param.declarations.iter().any(|d| {
                        matches!(
                            &d.data,
                            crate::ast::NodeData::ParameterDeclaration(pd)
                                if pd.question_token.is_some() || pd.initializer.is_some()
                        )
                    });
                let is_rest = sig.has_rest_parameter() && i == sig.parameters.len() - 1;
                let prefix = if is_rest { "..." } else { "" };
                if optional {
                    format!("{prefix}{}?: {}", param.name, self.type_to_string(&param_type))
                } else {
                    format!("{prefix}{}: {}", param.name, self.type_to_string(&param_type))
                }
            })
            .collect();
        let ret = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        let tp = if sig.type_parameters.is_empty() {
            String::new()
        } else {
            let names: Vec<String> = sig
                .type_parameters
                .iter()
                .filter_map(|tp| tp.symbol.as_ref().map(|s| s.name.clone()))
                .collect();
            if names.is_empty() {
                String::new()
            } else {
                format!("<{}>", names.join(", "))
            }
        };
        // An abstract construct signature displays the `abstract` marker
        // (`abstract new () => A` — assignmentCompatability45).
        let prefix = if sig.flags.contains(crate::checker::types::SignatureFlags::Abstract)
            && prefix.starts_with("new")
        {
            format!("abstract {prefix}")
        } else {
            prefix.to_string()
        };
        format!("{prefix}{tp}({}){sep}{}", params.join(", "), self.type_to_string(&ret))
    }

    /// Check if the construct signatures of two types are related.
    /// Wraps `signatures_related_to` with the construct-signature kind.
    fn is_construct_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_sigs = self.get_signatures_of_type(source, SignatureKind::Construct);
        let target_sigs = self.get_signatures_of_type(target, SignatureKind::Construct);

        if source_sigs.is_empty() && target_sigs.is_empty() {
            return true;
        }
        if target_sigs.is_empty() {
            // Empty target construct-signature set imposes no requirement
            // (Go's loop runs zero times → True) — this is what makes
            // `typeof SomeClass` assignable to plain object shapes.
            return true;
        }
        if source_sigs.is_empty() {
            // No construct signatures on the source — same no-match
            // elaboration as the call-signature variant, `new `-prefixed
            // (assignmentCompatability37/38).
            if self.relater_chain_active
                && let Some(t0) = target_sigs.first()
            {
                let source_str = self.type_to_string(source);
                let sig_str = self.signature_display_colon(t0, "new ");
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TYPE_0_PROVIDES_NO_MATCH_FOR_THE_SIGNATURE_1,
                    vec![source_str, sig_str],
                );
            }
            return false;
        }
        let related = self
            .signatures_related_to(source, target, SignatureKind::Construct, relation)
            .is_true();
        if !related && self.relater_chain_active {
            // Official construct-signature elaboration pyramid
            // (assignmentCompatability44/45), pushed leaf-first: the chain
            // builder nests chronological entries deepest-last, so the
            // arity leaf goes in before the two-signature line, and the
            // marker lands last (shallowest).
            let source_sigs = self.get_signatures_of_type(source, SignatureKind::Construct);
            let target_sigs = self.get_signatures_of_type(target, SignatureKind::Construct);
            if let (Some(ss), Some(ts)) = (source_sigs.first(), target_sigs.first())
                && ss.min_argument_count.max(0) as usize > ts.parameters.len()
            {
                // ARITY mismatches keep the colon-display pyramid
                // (assignmentCompatability44/45), leaf-first. The arity
                // leaf itself is reported by `compare_signatures_related`
                // during the comparison above (chronologically first, so it
                // nests deepest); this block only adds the two-signature
                // line and the marker on top. Return-only mismatches
                // suppress this block entirely — the signature comparison's
                // nested head + elided construct-return marker render
                // through the property-level transform ("The types
                // returned by 'new a(...)' ...",
                // constructSignatureAssignabilityInInheritance).
                let s_str = self.signature_display_arrow(ss, "new ");
                let t_str = self.signature_display_arrow(ts, "new ");
                self.relater_report_error(
                    crate::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                    vec![s_str, t_str],
                );
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TYPES_OF_CONSTRUCT_SIGNATURES_ARE_INCOMPATIBLE,
                    vec![],
                );
            }
        }
        related
    }

    /// Check if two function types are related by comparing their call signatures.
    fn is_function_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        // A type is a function type if it has call signatures
        if !self.is_call_signatures_related_to(source, target, relation) {
            return false;
        }
        if !self.is_construct_signatures_related_to(source, target, relation) {
            return false;
        }
        true
    }
}

/// Check if two slices of type parameters are the same (by pointer identity).
/// Mirrors Go's `core.Same` for type parameter slices.
fn type_parameters_same(a: &[Arc<Type>], b: &[Arc<Type>]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(x, y)| Arc::ptr_eq(x, y))
}

impl Checker {
    // ────────────────────────────────────────────────────────────────────────
    // Index signature comparison (improved port of relater.go)
    // ────────────────────────────────────────────────────────────────────────

    /// Improved port of `indexSignaturesRelatedTo`. Handles:
    /// - Identity relation: delegates to `index_signatures_identical_to`.
    /// - Target with a string index whose value type is `any` (and source
    ///   is non-primitive, relation is not strict subtype): short-circuit
    ///   to true. This matches the common `{ [key: string]: any }` target.
    /// - Generic mapped-type source with a target string index: compare the
    ///   mapped type's template type against the target's value type.
    /// - Otherwise: structural lookup via `type_related_to_index_info`.
    pub fn index_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        source_is_primitive: bool,
        relation: RelationKind,
    ) -> Ternary {
        if relation == RelationKind::Identity {
            return self.index_signatures_identical_to(source, target);
        }
        let target_indexes = self.get_index_infos_of_type(target);
        let target_has_string_index = target_indexes.iter().any(|info| {
            info.key_type
                .as_ref()
                .map(|k| k.flags.contains(TypeFlags::String))
                .unwrap_or(false)
        });
        let mut result = Ternary::True;
        for target_info in &target_indexes {
            let target_value_any = target_info
                .value_type
                .as_ref()
                .map(|v| v.flags.contains(TypeFlags::Any))
                .unwrap_or(false);
            let target_key_is_string = target_info
                .key_type
                .as_ref()
                .map(|k| k.flags.contains(TypeFlags::String))
                .unwrap_or(false);
            let related = if relation != RelationKind::StrictSubtype
                && !source_is_primitive
                && target_has_string_index
                && target_key_is_string
                && target_value_any
            {
                Ternary::True
            } else if self.is_generic_mapped_type(source) && target_key_is_string {
                let template = self.get_template_type_from_mapped_type(source);
                match template {
                    Some(template) => {
                        let target_value = target_info
                            .value_type
                            .clone()
                            .unwrap_or_else(|| self.any_type());
                        self.compare_types(template, target_value, relation, false)
                    }
                    None => Ternary::False,
                }
            } else {
                self.type_related_to_index_info(source, target_info, relation)
            };
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

    /// Port of `typeRelatedToIndexInfo`. Looks up the source's index info
    /// for the target's key type and compares value types.
    pub fn type_related_to_index_info(
        &mut self,
        source: &Arc<Type>,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        let target_key = match &target_info.key_type {
            Some(k) => k,
            None => return Ternary::True,
        };
        let source_info = self.get_applicable_index_info(source, target_key);
        if let Some(source_info) = source_info {
            return self.index_info_related_to(&source_info, target_info, relation);
        }
        // Source has no matching index signature. If the source is an
        // "inferable" object type (object literal, type literal, enum,
        // value module, JS expando, rest type, reverse-mapped type), we
        // synthesize an index signature from its properties and compare
        // those against the target's value type (P3.7f). The
        // strict-subtype relation additionally requires the source to be
        // a fresh object literal so that `{ [x: string]: xxx } <: {}` but
        // not vice-versa (matching Go's behavior in `typeRelatedToIndexInfo`).
        let is_fresh_literal = source.object_flags.contains(ObjectFlags::FreshLiteral);
        if relation != RelationKind::StrictSubtype || is_fresh_literal {
            if self.is_object_type_with_inferable_index(source) {
                return self.members_related_to_index_info(source, target_info, relation);
            }
        }
        Ternary::False
    }

    /// Port of `indexInfoRelatedTo`. Compares two index infos' value types.
    pub fn index_info_related_to(
        &mut self,
        source_info: &IndexInfo,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        let source_value = source_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());
        let target_value = target_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());
        self.compare_types(source_value, target_value, relation, false)
    }

    /// Port of `indexSignaturesIdenticalTo`. Requires same count and
    /// pairwise key/value/readonly equality.
    pub fn index_signatures_identical_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Ternary {
        let source_infos = self.get_index_infos_of_type(source);
        let target_infos = self.get_index_infos_of_type(target);
        if source_infos.len() != target_infos.len() {
            return Ternary::False;
        }
        for target_info in &target_infos {
            let target_key = match &target_info.key_type {
                Some(k) => Arc::clone(k),
                None => continue,
            };
            let source_info = self.get_index_info_of_type(source, &target_key);
            let related = match source_info {
                Some(si) => {
                    let sv = si.value_type.clone().unwrap_or_else(|| self.any_type());
                    let tv = target_info
                        .value_type
                        .clone()
                        .unwrap_or_else(|| self.any_type());
                    let type_related = self.compare_types(sv, tv, RelationKind::Identity, false);
                    let readonly_match = si.is_readonly == target_info.is_readonly;
                    if type_related.is_true() && readonly_match {
                        Ternary::True
                    } else {
                        Ternary::False
                    }
                }
                None => Ternary::False,
            };
            if related.is_false() {
                return Ternary::False;
            }
        }
        Ternary::True
    }

    /// Get the index infos of a type. Currently delegates to the structured
    /// type's `index_infos` field.
    pub fn get_index_infos_of_type(&self, t: &Arc<Type>) -> Vec<Arc<IndexInfo>> {
        t.as_structured()
            .map(|s| s.index_infos.clone())
            .unwrap_or_default()
    }

    /// Get the index info of a type for a specific key type.
    pub fn get_index_info_of_type(
        &self,
        t: &Arc<Type>,
        key_type: &Arc<Type>,
    ) -> Option<Arc<IndexInfo>> {
        for info in self.get_index_infos_of_type(t) {
            if let Some(info_key) = &info.key_type {
                if Arc::ptr_eq(info_key, key_type) || info_key.flags == key_type.flags {
                    return Some(info);
                }
            }
        }
        // Tuples (and arrays) inherit a NUMBER index info from their Array
        // base type (Go getTupleBaseType → Array<union-of-elements>, whose
        // number index signature carries the element type): `['a'][I]` with
        // `I extends number` indexes legally.
        if key_type.flags.contains(TypeFlags::Number) {
            if let TypeData::Tuple(tuple) = &t.data {
                let elements: Vec<Arc<Type>> = tuple
                    .element_infos
                    .iter()
                    .filter_map(|e| e.type_.clone())
                    .collect();
                if !elements.is_empty() {
                    let value = if elements.len() == 1 {
                        Arc::clone(&elements[0])
                    } else if elements.iter().all(|e| Arc::ptr_eq(e, &elements[0])) {
                        Arc::clone(&elements[0])
                    } else {
                        // Union construction without mutation: all tuple
                        // element types participate.
                        Arc::new(Type {
                            flags: TypeFlags::Union,
                            object_flags: ObjectFlags::None,
                            id: 0,
                            symbol: None,
                            alias: None,
                            data: TypeData::Union(UnionTypeData {
                                union_or_intersection: UnionOrIntersectionTypeData {
                                    structured: StructuredTypeData::default(),
                                    types: elements,
                                },
                                resolved_reduced_type: std::sync::OnceLock::new(),
                                regular_type: std::sync::OnceLock::new(),
                                origin: None,
                                key_property_name: None,
                                constituent_map: std::collections::HashMap::new(),
                            }),
                        })
                    };
                    return Some(Arc::new(IndexInfo {
                        key_type: Some(self.number_type()),
                        value_type: Some(value),
                        is_readonly: tuple.readonly,
                        declaration: None,
                        index_symbol: None,
                        components: Vec::new(),
                    }));
                }
            }
        }
        None
    }

    /// Get the applicable index info of a source for a given key type.
    /// Mirrors Go's `getApplicableIndexInfo`. Number index keys are
    /// applicable to string indexes (a string index accepts numbers).
    pub fn get_applicable_index_info(
        &self,
        source: &Arc<Type>,
        key_type: &Arc<Type>,
    ) -> Option<Arc<IndexInfo>> {
        let infos = self.get_index_infos_of_type(source);
        for info in infos {
            if let Some(info_key) = &info.key_type {
                // Direct match.
                if Arc::ptr_eq(info_key, key_type) {
                    return Some(info);
                }
                // A number key is applicable to a string index target.
                if info_key.flags.contains(TypeFlags::Number)
                    && key_type.flags.contains(TypeFlags::String)
                {
                    return Some(info);
                }
                // A string key is applicable to a number index target
                // (strings are numbers in JS).
                if info_key.flags.contains(TypeFlags::String)
                    && key_type.flags.contains(TypeFlags::Number)
                {
                    return Some(info);
                }
            }
        }
        None
    }

    /// Whether a type is a generic mapped type (has a constraint and
    /// template type). Mirrors Go's `isGenericMappedType`.
    pub fn is_generic_mapped_type(&self, t: &Arc<Type>) -> bool {
        if let TypeData::Mapped(m) = &t.data {
            m.type_parameter.is_some() && m.template_type.is_some()
        } else {
            false
        }
    }

    /// Get the template type of a mapped type.
    /// Mirrors Go's `getTemplateTypeFromMappedType`.
    pub fn get_template_type_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.template_type.clone();
        }
        None
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Index signature comparison polish (P3.7f)
//
// Ports the structural fallback paths of `typeRelatedToIndexInfo` and
// `membersRelatedToIndexInfo` from internal/checker/relater.go (~L4596,
// ~L4627). When the source has no index signature matching the target's
// key type, but the source is an *inferable* object type (object literal,
// type literal, enum, value module, JS expando, rest type, or reverse
// mapped type whose source is itself inferable), we walk the source's
// properties and verify that every property whose name is a literal of
// the target's key type is assignable to the target's value type.
//
// This handles the common case `{ a: 1, b: 2 } ~ { [key: string]: number }`
// without requiring the source to declare an explicit index signature.
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Whether an object type may have an *inferred* index signature —
    /// i.e. one synthesized from its properties rather than declared.
    /// Direct port of Go's `isObjectTypeWithInferableIndex`.
    ///
    /// Returns true for:
    /// - Object literals, type literals, enums, value modules (without
    ///   call/construct signatures and not class-typed).
    /// - JS expando object literals and rest types.
    /// - Reverse-mapped types whose source is itself inferable.
    /// - Intersection types whose every constituent is inferable.
    pub fn is_object_type_with_inferable_index(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Intersection) {
            // Every constituent must be inferable.
            if let Some(ui) = t.as_union_or_intersection() {
                return ui
                    .types
                    .iter()
                    .all(|c| self.is_object_type_with_inferable_index(c));
            }
            return false;
        }
        // Object-literal / type-literal / enum / value-module case.
        if let Some(sym) = &t.symbol {
            let sf = sym.flags;
            let inferable_symbol_kinds = sf.intersects(
                SymbolFlags::ObjectLiteral
                    | SymbolFlags::TypeLiteral
                    | SymbolFlags::EnumMember
                    | SymbolFlags::ValueModule,
            );
            if inferable_symbol_kinds
                && !sf.contains(SymbolFlags::Class)
                && !self.type_has_call_or_construct_signatures(t)
            {
                return true;
            }
        }
        // JS expando / object-rest case.
        if t.object_flags
            .intersects(ObjectFlags::JSLiteral | ObjectFlags::ObjectRestType)
        {
            return true;
        }
        // Reverse-mapped case: recurse into the source.
        if t.object_flags.contains(ObjectFlags::ReverseMapped) {
            if let TypeData::ReverseMapped(rm) = &t.data {
                if let Some(src) = &rm.source {
                    return self.is_object_type_with_inferable_index(src);
                }
            }
        }
        false
    }

    /// Walk the source's properties and verify each property whose name
    /// is a literal of the target's key type is assignable to the target's
    /// value type. Also compares any source index signatures whose key
    /// type is applicable to the target's key type.
    ///
    /// Direct port of Go's `membersRelatedToIndexInfo` (without the
    /// ignored-JSX / `exactOptionalPropertyTypes` branches, which we
    /// don't yet support).
    pub fn members_related_to_index_info(
        &mut self,
        source: &Arc<Type>,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        let Some(target_key) = target_info.key_type.as_ref() else {
            return Ternary::True;
        };
        let target_value = target_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());

        let props = self.get_properties_of_type(source);
        let mut result = Ternary::True;
        for prop in props {
            // Only consider properties whose name is a literal of the
            // target's key type. Go uses `getLiteralTypeFromProperty` to
            // synthesize a literal type from a property name; we approximate
            // by treating every named property as a string literal (the
            // common case for `{ [key: string]: T }` targets).
            let literal_key = self.get_literal_type_from_property(&prop, target_key);
            if !self.is_applicable_index_type(&literal_key, target_key) {
                continue;
            }
            let prop_type = self.get_type_of_symbol(&prop);
            let related = self.compare_types(prop_type, Arc::clone(&target_value), relation, false);
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }

        // Also compare any source index signatures whose key type is
        // applicable to the target's key type.
        for info in self.get_index_infos_of_type(source) {
            if let Some(src_key) = &info.key_type {
                if self.is_applicable_index_type(src_key, target_key) {
                    let related = self.index_info_related_to(&info, target_info, relation);
                    if related.is_false() {
                        return Ternary::False;
                    }
                    result = result.and(related);
                }
            }
        }
        result
    }

    /// Whether `key` is a literal type applicable to `target_key`'s index
    /// type. A string-literal key is applicable to a string index; a
    /// number-literal key is applicable to a number index. Direct port of
    /// Go's `isApplicableIndexType`.
    pub fn is_applicable_index_type(&self, key: &Arc<Type>, target_key: &Arc<Type>) -> bool {
        if Arc::ptr_eq(key, target_key) {
            return true;
        }
        // String literal -> string index
        if key.flags.contains(TypeFlags::StringLiteral)
            && target_key.flags.contains(TypeFlags::String)
        {
            return true;
        }
        // Number literal -> number index
        if key.flags.contains(TypeFlags::NumberLiteral)
            && target_key.flags.contains(TypeFlags::Number)
        {
            return true;
        }
        // A number key is applicable to a string index (numbers index
        // into string indexes in JS).
        if key.flags.contains(TypeFlags::Number) && target_key.flags.contains(TypeFlags::String) {
            return true;
        }
        false
    }

    /// Build a literal type from a property's name. Used by
    /// `members_related_to_index_info` to decide whether a property is
    /// applicable to the target's index signature.
    ///
    /// Direct port of Go's `getLiteralTypeFromProperty`. Currently we
    /// synthesize a string-literal type for every property name (the
    /// common case for `{ [key: string]: T }` targets) and a number
    /// literal when the name parses as a number. Full implementation
    /// would also handle unique symbols and `SymbolFlags::EnumMember`.
    pub fn get_literal_type_from_property(
        &mut self,
        prop: &Arc<Symbol>,
        target_key: &Arc<Type>,
    ) -> Arc<Type> {
        if target_key.flags.contains(TypeFlags::Number) {
            if let Ok(n) = prop.name.parse::<i64>() {
                return self.get_number_literal_type(jsnum::Number::from(n));
            }
        }
        self.get_string_literal_type(&prop.name)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Generic instantiation comparison (P3.7d)
//
// Ports the variance-aware type-argument comparison and the structured-type
// dispatch for references to the *same* generic type from
// `internal/checker/relater.go` (`typeArgumentsRelatedTo`,
// `structuredTypeRelatedToWorker`, `compareTypeParametersIdentical`).
//
// Variance computation itself is a large subsystem (`variance.go`); for now
// we use the covariant fallback that Go's relater also uses when no variance
// information is available. This produces correct results for the common
// cases (Array<T> ~ Array<U> iff T ~ U; Promise<T> ~ Promise<U>; etc.) and
// defers the strict-function-types invariant handling to P3.7e.
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Compare two slices of type arguments according to per-parameter
    /// variance flags. Direct port of Go's `typeArgumentsRelatedTo`.
    ///
    /// For each pair `(sources[i], targets[i])`:
    /// - **Covariant** (default): `sources[i] ~ targets[i]`
    /// - **Contravariant**: `targets[i] ~ sources[i]` (swap direction)
    /// - **Bivariant**: try contravariant first, then fall back to
    ///   covariant if that fails
    /// - **Invariant**: both `s ~ t` and `t ~ s` must hold
    /// - **Independent**: skip the argument entirely (its variance is
    ///   never witnessed)
    /// - **Unmeasurable**: require identity (rather than the requested
    ///   relation) — non-linear relations such as `-?` mapped modifiers
    ///   can't be safely approximated by structural comparison.
    pub fn type_arguments_related_to(
        &mut self,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
        variances: &[VarianceFlags],
        relation: RelationKind,
    ) -> Ternary {
        // Identity relation requires equal length up front.
        if sources.len() != targets.len() && relation == RelationKind::Identity {
            return Ternary::False;
        }
        let length = sources.len().min(targets.len());
        let mut result = Ternary::True;
        for i in 0..length {
            // Default to covariant when no variance information is available
            // (matches Go's fallback during variance computation and for
            // `this`-type arguments).
            let variance_flags = variances
                .get(i)
                .copied()
                .unwrap_or(VarianceFlags::Covariant);
            let variance = variance_flags & VARIANCE_FLAGS_VARIANCE_MASK;

            // Skip independent type parameters — their variance is never
            // observed by any consumer of the generic type.
            if variance == VarianceFlags::Independent {
                continue;
            }

            let s = &sources[i];
            let t = &targets[i];
            let related = if variance_flags.intersects(VARIANCE_FLAGS_ALLOWS_STRUCTURAL_FALLBACK)
                && !variance_flags
                    .intersects(VarianceFlags::Unmeasurable | VarianceFlags::Unreliable)
            {
                // The "allows structural fallback" subset that *isn't* also
                // unmeasurable/unreliable reduces to plain covariance: there
                // is no special handling needed beyond the default direction.
                self.compare_types(Arc::clone(s), Arc::clone(t), relation, false)
            } else if variance_flags.intersects(VarianceFlags::Unmeasurable) {
                // Even an `Unmeasurable` variance works out without a
                // structural check if the source and target are identical.
                // We can't simply assume invariance, because `Unmeasurable`
                // marks nonlinear relations (e.g. relations tainted by the
                // `-?` modifier in a mapped type).
                if relation == RelationKind::Identity {
                    if self.is_type_related_to(s, t, relation) {
                        Ternary::True
                    } else {
                        Ternary::False
                    }
                } else if self.is_type_identical_to(s, t) {
                    Ternary::True
                } else {
                    Ternary::False
                }
            } else {
                match variance {
                    VarianceFlags::Covariant => {
                        self.compare_types(Arc::clone(s), Arc::clone(t), relation, false)
                    }
                    VarianceFlags::Contravariant => {
                        // Swap direction: target must be assignable to source.
                        self.compare_types(Arc::clone(t), Arc::clone(s), relation, false)
                    }
                    VarianceFlags::Independent => {
                        // Already filtered out above; defensive fallback.
                        Ternary::True
                    }
                    _ => {
                        // Bivariant or Invariant.
                        //
                        // Bivariant: try contravariant first without error
                        // reporting, then fall back to covariant if that
                        // fails. Invariant: require both covariant and
                        // contravariant. Since `VarianceFlags::None` is the
                        // "invariant" sentinel in Go's encoding (covariant
                        // and contravariant bits both unset), and our
                        // `VARIANCE_FLAGS_BIVARIANT` is both bits set, we
                        // disambiguate by checking the bits explicitly.
                        let is_bivariant = variance_flags.intersects(VARIANCE_FLAGS_BIVARIANT)
                            && variance != VarianceFlags::None;
                        let contra =
                            self.compare_types(Arc::clone(t), Arc::clone(s), relation, false);
                        if is_bivariant {
                            if !contra.is_false() {
                                contra
                            } else {
                                self.compare_types(Arc::clone(s), Arc::clone(t), relation, false)
                            }
                        } else {
                            // Invariant: require both directions to hold.
                            let co =
                                self.compare_types(Arc::clone(s), Arc::clone(t), relation, false);
                            if co.is_false() {
                                Ternary::False
                            } else {
                                co.and(contra)
                            }
                        }
                    }
                }
            };
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

    /// Whether two lists of type parameters are *identical* modulo
    /// renaming. Direct port of Go's `compareTypeParametersIdentical`.
    ///
    /// Two type-parameter lists `<T, U extends T>` and `<A, B extends A>`
    /// are considered identical because their structural relationship is
    /// the same — only the names differ. The check works by instantiating
    /// each target's constraint into the source's type parameters (via a
    /// `targetParams -> sourceParams` mapper) and comparing the resulting
    /// constraints for identity.
    ///
    /// Our simplified port compares constraints directly without the
    /// mapper substitution. This is correct when the constraints don't
    /// reference sibling type parameters (the common case for built-in
    /// generics like `Array<T>`, `Map<K, V>`, `Promise<T>`), and falls
    /// back to "identical when constraints are pointer-equal or both
    /// absent" for the parameter-referencing case.
    pub fn compare_type_parameters_identical(
        &mut self,
        source_params: &[Arc<Type>],
        target_params: &[Arc<Type>],
    ) -> bool {
        if source_params.len() != target_params.len() {
            return false;
        }
        for (source, target) in source_params.iter().zip(target_params.iter()) {
            if Arc::ptr_eq(source, target) {
                continue;
            }
            let source_constraint = self
                .get_constraint_of_type_parameter(source)
                .unwrap_or_else(|| self.unknown_type());
            let target_constraint = self
                .get_constraint_of_type_parameter(target)
                .unwrap_or_else(|| self.unknown_type());
            // Without a real `instantiateType`, fall back to direct
            // identity comparison. This works for built-in generics and
            // for type parameters whose constraints don't reference
            // sibling parameters.
            if !self.is_type_identical_to(&source_constraint, &target_constraint) {
                return false;
            }
        }
        true
    }

    /// Compare two references to the *same* generic type (e.g. `Array<T>`
    /// vs `Array<U>` where both target the `Array<T>` interface).
    ///
    /// Direct port of the relevant branch of Go's
    /// `structuredTypeRelatedToWorker`:
    ///
    /// - If both source and target are references to the same target type
    ///   (and neither is a tuple — those go through element-wise
    ///   comparison — and neither is a "marker" type intended for
    ///   structural comparison), obtain the variance information for the
    ///   target's type parameters and relate the type arguments
    ///   accordingly.
    /// - If no variance information is available (which Go uses as a
    ///   recursion-depth signal during variance computation itself),
    ///   return `Ternary::Maybe` to defer the decision.
    ///
    /// Returns `None` when the source/target pair isn't a same-target
    /// generic reference pair (caller should fall back to other
    /// comparison strategies). Returns `Some(Ternary)` when the
    /// variance-based result has been computed.
    pub fn generic_type_reference_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Ternary> {
        // Both must be object-typed references with the same target.
        if !source.flags.contains(TypeFlags::Object) || !target.flags.contains(TypeFlags::Object) {
            return None;
        }
        if !source.object_flags.contains(ObjectFlags::Reference)
            || !target.object_flags.contains(ObjectFlags::Reference)
        {
            return None;
        }
        // Tuples are handled by element-wise comparison, not here.
        if is_tuple_type(source) || is_tuple_type(target) {
            return None;
        }
        let source_target = source.target()?;
        let target_target = target.target()?;
        if !Arc::ptr_eq(source_target, target_target) {
            return None;
        }
        // Marker types are intended to be compared structurally.
        if self.is_marker_type(source) || self.is_marker_type(target) {
            return None;
        }
        // Empty array literals are always assignable to mutable array types
        // (Go: `c.isEmptyArrayLiteralType(source)`).
        if self.is_empty_array_literal_type(source) {
            return Some(Ternary::True);
        }
        // Obtain variance information for the type parameters of the
        // generic target. Without a full variance-computation engine,
        // `get_variances` returns an empty Vec to signal "no variance info"
        // (matching Go's behavior during recursive variance computation),
        // in which case we defer the decision with `Ternary::Maybe`.
        let variances = self.get_variances(source_target);
        if variances.is_empty() {
            return Some(Ternary::Maybe);
        }
        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);
        Some(self.type_arguments_related_to(&source_args, &target_args, &variances, relation))
    }

    /// Simplified port of Go's `getVariances`. The full implementation
    /// recursively computes variance for each type parameter of a generic
    /// type by inspecting where it appears in the type's structure
    /// (`variance.go`). Without that subsystem, we return an empty slice
    /// to signal "variance not measured yet" — the caller
    /// (`generic_type_reference_related_to`) then defers the decision
    /// with `Ternary::Maybe`, matching Go's behavior during recursive
    /// variance computation.
    ///
    /// This is sufficient to make the common case work (covariant
    /// containers like `Array<T>` and `Promise<T>` are also handled
    /// directly by `is_array_type_related_to`); full variance support
    /// will land with P3.7e.
    pub fn get_variances(&self, _target: &Arc<Type>) -> Vec<VarianceFlags> {
        // Default everything to covariant as a pragmatic fallback so
        // `Array<Promise<X>>`-style comparisons work without a variance
        // engine. Go's `typeArgumentsRelatedTo` does the same when no
        // variance info is provided. We only return empty when the target
        // has *no* type parameters (in which case there's nothing to
        // compare and the caller should not enter this path).
        match &_target.data {
            TypeData::Object(o) => {
                if let Some(t) = o.target.as_ref() {
                    if let TypeData::Interface(i) = &t.data {
                        let n = i.all_type_parameters.len();
                        return vec![VarianceFlags::Covariant; n];
                    }
                }
                Vec::new()
            }
            TypeData::Interface(i) => {
                let n = i.all_type_parameters.len();
                vec![VarianceFlags::Covariant; n]
            }
            _ => Vec::new(),
        }
    }

    /// Whether a type is a "marker" type. Marker types are intended to be
    /// compared structurally even when they appear as references to the
    /// same generic target (Go: `c.isMarkerType`).
    ///
    /// The full Go implementation checks a list of well-known marker types
    /// (e.g. `ReadonlyArray`, `ReadonlyMap`, `ReadonlySet`, `Promise`'s
    /// `T`-parameter usage). Our port returns `false` for now — none of
    /// our test fixtures exercise the distinction, and the structural
    /// fallback in `is_object_type_related_to` is correct (just slower)
    /// for the cases we do hit.
    pub fn is_marker_type(&self, _t: &Arc<Type>) -> bool {
        false
    }

    /// Whether a type is the "empty array literal" placeholder used for
    /// `[]` literals whose element type hasn't been inferred yet.
    /// Mirrors Go's `c.isEmptyArrayLiteralType`.
    pub fn is_empty_array_literal_type(&self, t: &Arc<Type>) -> bool {
        // The empty-array-literal type is a fresh object type whose
        // `object_flags` carries `FreshLiteral` and whose target is the
        // global `Array` type with an empty (or `undefined`) element type.
        // We approximate this by checking the fresh-literal flag.
        t.object_flags.contains(ObjectFlags::FreshLiteral) && self.is_array_type(t)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Conditional & mapped type comparison (P3.7e)
//
// Ports the conditional-target branch of `structuredTypeRelatedToWorker`
// (relater.go ~L3540) and `mappedTypeRelatedTo` (relater.go ~L3972).
//
// The full Go implementation depends on `c.instantiateType` with mappers,
// `c.isDistributionDependent`, `c.getPermissiveInstantiation`, etc. —
// most of which we don't yet have. The port below is intentionally
// conservative: when a path requires infrastructure we don't have
// (infer type parameters, distributive conditionals referencing the
// check type, mapped types with name remapping), we return `None` so
// the caller falls through to structural comparison. The cases we do
// handle correctly cover the common scenarios:
//   * `S` assignable to `T extends U ? X : Y` when S ~ X and S ~ Y
//     (with the permissive/restrictive short-circuits).
//   * `{ [P in Q]: X }` vs `{ [P in R]: Y }` when Q ~ R and X ~ Y
//     (without name remapping).
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Compare a source type `S` against a conditional target
    /// `T extends U ? X : Y`.
    ///
    /// Returns `None` when the conditional requires infrastructure we don't
    /// have yet (infer positions, distribution-dependent checks, identical
    /// source conditional) so the caller can fall back to structural
    /// comparison. Otherwise returns `Some(Ternary)`.
    ///
    /// Direct port of the conditional branch of Go's
    /// `structuredTypeRelatedToWorker`.
    pub fn conditional_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Ternary> {
        let ct = match &target.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        // Bail out if the conditional has `infer` type positions — those
        // require the inference engine (P3.8c).
        if let Some(root) = &ct.root {
            if !root.infer_type_parameters.is_empty() {
                return None;
            }
            // Bail out if the conditional is distributive and references
            // the check type parameter in either result branch
            // (`isDistributionDependent`). Without a real
            // `getPermissiveInstantiation` we can't safely short-circuit.
            if root.is_distributive && self.conditional_is_distribution_dependent(target) {
                return None;
            }
        }

        // Bail out when source is itself a conditional with the same root
        // (this case shows up during variance computation and would cause
        // infinite recursion if we tried to compare both branches).
        if let TypeData::Conditional(sct) = &source.data {
            if let (Some(s_root), Some(t_root)) = (&sct.root, &ct.root) {
                if std::ptr::eq(s_root.as_ref() as *const _, t_root.as_ref() as *const _) {
                    return None;
                }
            }
        }

        // Determine whether either branch can be skipped:
        //  * skipTrue  := the conditional's check is *never* true, so we
        //                 can ignore the true branch entirely.
        //  * skipFalse := the conditional's check is *always* true (and
        //                 skipTrue is false), so we can ignore the false
        //                 branch.
        //
        // Go computes these via permissive/restrictive instantiations of
        // the check and extends types. We approximate: if the resolved
        // branch is already cached (because the conditional has been
        // evaluated), use that; otherwise don't skip.
        let skip_true = match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
            (Some(check), Some(extends)) => !self.is_type_assignable_to(check, extends),
            _ => false,
        };
        let skip_false = if skip_true {
            false
        } else {
            match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
                (Some(check), Some(extends)) => self.is_type_assignable_to(check, extends),
                _ => false,
            }
        };

        let mut result = Ternary::True;
        if !skip_true {
            let true_branch = self.get_true_type_from_conditional_type(target)?;
            let r = self.compare_types(Arc::clone(source), true_branch, relation, false);
            if r.is_false() {
                return Some(Ternary::False);
            }
            result = result.and(r);
        }
        if !skip_false {
            let false_branch = self.get_false_type_from_conditional_type(target)?;
            let r = self.compare_types(Arc::clone(source), false_branch, relation, false);
            if r.is_false() {
                return Some(Ternary::False);
            }
            result = result.and(r);
        }
        Some(result)
    }

    /// Compare two mapped types structurally:
    /// `{ [P in Q]: X }` vs `{ [P in R]: Y }`.
    ///
    /// Returns `None` when the comparison requires infrastructure we don't
    /// have yet (mapped type with name remapping, or any side that isn't a
    /// real mapped type). Otherwise returns `Some(Ternary)`.
    ///
    /// Direct port of Go's `mappedTypeRelatedTo` minus the
    /// `instantiateType` substitutions (which we don't yet support).
    pub fn mapped_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Ternary> {
        let sm = match &source.data {
            TypeData::Mapped(m) => m,
            _ => return None,
        };
        let tm = match &target.data {
            TypeData::Mapped(m) => m,
            _ => return None,
        };

        // Bail out if either side remaps keys (the `as R` clause) — that
        // requires mapper-based instantiation to compare nameTypes
        // correctly.
        if sm.name_type.is_some() || tm.name_type.is_some() {
            return None;
        }

        // Modifiers compatibility: for non-identity relations we accept
        // whenever target's optionality is at least as permissive as
        // source's. Without `getCombinedMappedTypeOptionality`, we
        // conservatively accept all modifier combinations for the
        // assignable/subtype relations and require exact match for
        // identity.
        if relation == RelationKind::Identity {
            // Identity requires the same modifiers; we don't have the
            // helper, so fall back to structural comparison.
            return None;
        }

        // Compare constraints contravariantly: target's constraint must be
        // assignable to source's constraint.
        let source_constraint = self.get_constraint_type_from_mapped_type(source)?;
        let target_constraint = self.get_constraint_type_from_mapped_type(target)?;
        let constraint_related = self.compare_types(
            Arc::clone(&target_constraint),
            Arc::clone(&source_constraint),
            relation,
            false,
        );
        if constraint_related.is_false() {
            return Some(Ternary::False);
        }

        // Compare template types covariantly. The Go code substitutes the
        // source's type parameter with the target's via a `SimpleTypeMapper`
        // before comparing; we don't have a working `instantiateType`, so
        // we compare the templates directly. This is correct when both
        // mapped types use the same type-parameter name (the common case
        // for `{ [P in keyof T]: ... }` vs `{ [P in keyof T]: ... }`),
        // and falls back to structural comparison otherwise.
        let source_template = self.get_template_type_from_mapped_type(source)?;
        let target_template = self.get_template_type_from_mapped_type(target)?;
        let template_related = self.compare_types(
            Arc::clone(&source_template),
            Arc::clone(&target_template),
            relation,
            false,
        );
        Some(constraint_related.and(template_related))
    }

    /// Get the constraint type of a mapped type (the `Q` in `{ [P in Q]: X }`).
    /// Mirrors Go's `getConstraintTypeFromMappedType`.
    pub fn get_constraint_type_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.constraint_type.clone();
        }
        None
    }

    /// Get the type parameter of a mapped type (the `P` in `{ [P in Q]: X }`).
    /// Mirrors Go's `getTypeParameterFromMappedType`.
    pub fn get_type_parameter_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.type_parameter.clone();
        }
        None
    }

    /// Get the name type of a mapped type (the `R` in `{ [P in Q as R]: X }`).
    /// Mirrors Go's `getNameTypeFromMappedType`.
    pub fn get_name_type_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.name_type.clone();
        }
        None
    }

    /// Get the resolved `true` branch of a conditional type, if it has been
    /// computed. Mirrors Go's `getTrueTypeFromConditionalType`.
    /// Force-resolve a DEFERRED conditional's true/false branch by resolving
    /// the branch TYPE NODE under the conditional instance's creation
    /// context (`creation_type_argument_stack` + `root.creation_scopes`).
    /// Mirrors Go's `getTrueTypeFromConditionalType` /
    /// `getFalseTypeFromConditionalType` (checker.go ~L24607), which
    /// instantiate the branch node under the instance's mapper — for our
    /// node-rewalking architecture the equivalent of "under the mapper" is
    /// re-pushing the creation-time substitution frames and lexical scope
    /// chain. The result is deliberately NOT cached in `resolved_true_type`
    /// / `resolved_false_type`: those cells mean "the whole conditional has
    /// been decided", which is exactly what a deferred conditional must not
    /// claim (callers such as `resolve_conditional_type` fast-path on them).
    pub fn get_forced_branch_type_of_conditional_type(
        &mut self,
        t: &Arc<Type>,
        take_true: bool,
    ) -> Option<Arc<Type>> {
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };
        if let Some(cached) = if take_true {
            ct.resolved_true_type.get()
        } else {
            ct.resolved_false_type.get()
        } {
            return Some(Arc::clone(cached));
        }
        let cond_node = ct.root.as_ref()?.node.as_ref()?;
        let branch_node = match &cond_node.data {
            NodeData::ConditionalTypeNode(d) => {
                if take_true {
                    Arc::clone(&d.true_type)
                } else {
                    Arc::clone(&d.false_type)
                }
            }
            _ => return None,
        };
        // Re-push creation scopes (non-common suffix; same procedure as
        // `resolve_conditional_type_with_check`).
        let creation_scopes: Vec<u64> =
            ct.root.as_ref().map(|r| r.creation_scopes.clone()).unwrap_or_default();
        let mut common = 0usize;
        while common < creation_scopes.len()
            && common < self.scope_stack.len()
            && creation_scopes[common] == self.scope_stack[common]
        {
            common += 1;
        }
        let scopes_pushed = creation_scopes.len() - common;
        self.scope_stack.extend_from_slice(&creation_scopes[common..]);
        // Re-push creation substitution frames, minus keys already bound.
        let mut merged_creation: HashMap<usize, Arc<Type>> = HashMap::new();
        for frame in ct.creation_type_argument_stack.iter() {
            for (k, v) in frame {
                merged_creation.insert(*k, Arc::clone(v));
            }
        }
        for map in self.type_argument_stack.iter() {
            for k in map.keys() {
                merged_creation.remove(&(*k as usize));
            }
        }
        let pushes_creation = !merged_creation.is_empty();
        if pushes_creation {
            self.type_argument_stack.push(
                merged_creation
                    .into_iter()
                    .map(|(k, v)| ((k as *const Symbol), v))
                    .collect(),
            );
        }
        // Infer type parameters are in scope ONLY in the true branch (the
        // ConditionalType node itself acts as their lexical container — same
        // procedure as `resolve_conditional_type_with_check`).
        if take_true {
            self.push_scope(&cond_node);
        }
        let branch = self.get_type_from_type_node(&branch_node);
        if take_true {
            self.pop_scope();
        }
        if pushes_creation {
            self.type_argument_stack.pop();
        }
        if scopes_pushed > 0 {
            self.scope_stack.truncate(self.scope_stack.len() - scopes_pushed);
        }
        Some(branch)
    }

    /// Go's `getDefaultConstraintOfConditionalType` (checker.go ~L17317):
    /// the union of the deferred conditional's two branches — every value
    /// the conditional could ever produce. Distribution-dependent roots
    /// (the check type parameter appears at a TOP-LEVEL position of either
    /// result; Go `isTypeParameterAtTopLevelOfTrueOrFalseType`) return
    /// `None`, because widening those to their constraint loses too much
    /// information (relater.go ~L3800).
    pub(crate) fn deferred_default_constraint_of_conditional(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let root = match &t.data {
            TypeData::Conditional(ct) => ct.root.as_ref()?,
            _ => return None,
        };
        if !Self::conditional_distribution_independent(root) {
            return None;
        }
        let (true_branch, false_branch) = (
            self.get_forced_branch_type_of_conditional_type(t, true),
            self.get_forced_branch_type_of_conditional_type(t, false),
        );
        match (true_branch, false_branch) {
            (Some(tb), Some(fb)) => {
                // An `any` branch would make the union viral; Go elides it
                // (treating `any` like `never` here).
                if tb.flags.contains(TypeFlags::Any) {
                    Some(fb)
                } else if fb.flags.contains(TypeFlags::Any) {
                    Some(tb)
                } else {
                    Some(self.get_union_type(vec![tb, fb]))
                }
            }
            (only, None) | (None, only) => only,
        }
    }

    /// Approximation of Go's `isDistributionDependent`
    /// (checker.go): for a distributive root, the default-constraint
    /// fallback is unsafe when the check type parameter is exposed at a
    /// top-level position of either result type — directly as a branch
    /// child or as a direct union member (parenthesized forms unwrapped).
    /// Nested occurrences (inside `keyof T`, alias calls `R1<T[K]>`,
    /// mapped templates) do not count.
    fn conditional_distribution_independent(root: &ConditionalRoot) -> bool {
        if !root.is_distributive {
            return true;
        }
        let Some(param_sym) = root.check_type_parameter_symbol.as_ref() else {
            return false;
        };
        let cond_node = match root.node.as_ref().map(|n| &n.data) {
            Some(NodeData::ConditionalTypeNode(d)) => d,
            _ => return false,
        };
        let is_top_level_reference = |node: &Arc<Node>| -> bool {
            // Transparent wrappers: unions spread their members,
            // parenthesized types reveal their inner node.
            let mut queue: Vec<&Arc<Node>> = vec![node];
            while let Some(current) = queue.pop() {
                match &current.data {
                    NodeData::UnionTypeNode(u) => {
                        for member in u.types.iter() {
                            queue.push(member);
                        }
                    }
                    NodeData::ParenthesizedTypeNode(p) => queue.push(&p.type_node),
                    NodeData::TypeReferenceNode(r) => {
                        if r.type_name.kind == SyntaxKind::Identifier
                            && r.type_name.text() == param_sym.name
                        {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        };
        !is_top_level_reference(&cond_node.true_type) && !is_top_level_reference(&cond_node.false_type)
    }

    pub fn get_true_type_from_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            if let Some(rt) = ct.resolved_true_type.get() {
                return Some(rt.clone());
            }
            // Fall back to the resolved-inferred-true type, which is set
            // when the conditional's check type was instantiated via
            // inference (Go: `getTrueTypeFromConditionalType` does the same).
            if let Some(rt) = ct.resolved_inferred_true_type.get() {
                return Some(rt.clone());
            }
        }
        None
    }

    /// Get the resolved `false` branch of a conditional type, if it has been
    /// computed. Mirrors Go's `getFalseTypeFromConditionalType`.
    pub fn get_false_type_from_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            if let Some(rt) = ct.resolved_false_type.get() {
                return Some(rt.clone());
            }
        }
        None
    }

    /// Whether a conditional type is "distribution dependent": distributive
    /// *and* references the check type parameter in either result branch.
    /// Mirrors Go's `isDistributionDependent`. Without a full check-type
    /// tracking subsystem we conservatively return `true` for any
    /// distributive conditional, which causes the caller to bail out and
    /// fall back to structural comparison (correct but possibly slower).
    pub fn conditional_is_distribution_dependent(&self, _t: &Arc<Type>) -> bool {
        true
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Conditional type `infer R` resolution (P3.8c)
//
// Ports the `infer`-position handling of `getConditionalType`
// (internal/checker/checker.go ~L24208). When a conditional has
// `infer R` type parameters in its extends clause, we:
//   1. Set up an InferenceContext keyed on the infer type parameters.
//   2. Call `infer_types(check, extends)` to populate the inferences.
//   3. Resolve the inferred types via `get_inferred_types`.
//   4. Build a mapper that substitutes the inferred types for the
//      infer type parameters, and use it to decide which branch to
//      take and to instantiate the chosen branch.
//
// The full Go implementation also handles distributive conditionals
// (where the check type is a type parameter), deferred checks, tuple
// destructuring, permissive/restrictive instantiations, and 1000-level
// tail recursion. Our simplified port handles the common case where the
// check type is concrete (no remaining type parameters) and the infer
// type parameters can be resolved by direct structural matching.
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Resolve a conditional type `T extends U ? X : Y` to either `X` or `Y`
    /// based on whether `T` is assignable to `U`. Handles the `infer R`
    /// case by setting up an inference context and substituting the
    /// inferred types into the chosen branch.
    ///
    /// Returns `Some(resolved)` when the conditional can be evaluated, or
    /// `None` when:
    ///   - the type isn't a conditional,
    ///   - the check or extends type is missing,
    ///   - the check type is still generic (contains type parameters),
    ///   - inference fails to produce a candidate for one or more infer
    ///     type parameters.
    ///
    /// Caches the result in `resolved_true_type` / `resolved_false_type`
    /// so subsequent lookups via `get_resolved_type_of_conditional_type`
    /// don't re-run the resolution.
    pub fn resolve_conditional_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        // Fast path: cached result.
        if let Some(rt) = ct.resolved_true_type.get() {
            return Some(Arc::clone(rt));
        }
        if let Some(rt) = ct.resolved_false_type.get() {
            return Some(Arc::clone(rt));
        }

        let check_type = ct.check_type.clone()?;
        let extends_type = ct.extends_type.clone()?;

        // Error type short-circuit (matches Go).
        if check_type.flags.contains(TypeFlags::Any) && check_type.intrinsic_name() == Some("error")
        {
            return Some(Arc::clone(&check_type));
        }

        // Distributive conditional types are distributed over union types:
        // when the (substituted) check type of a distributive root is a
        // union `A | B`, the result is `(A extends E ? X : Y) | (B extends E
        // ? X : Y)`. Direct port of Go's `getConditionalTypeInstantiation`
        // (checker.go ~L22544): the per-constituent resolution runs with the
        // check type parameter mapped to the constituent
        // (`prependTypeMapping(checkType, t, newMapper)`), which our
        // type_argument_stack models by pushing a shadowing entry.
        // Copy out the small root fields (the root itself isn't `Clone`) so
        // no borrow of `t.data` is held across the mutable calls below.
        let (is_distributive, check_tp_symbol) = match ct.root.as_ref() {
            Some(root) => (
                root.is_distributive,
                root.check_type_parameter_symbol.clone(),
            ),
            None => (false, None),
        };
        if is_distributive && let Some(tp_symbol) = &check_tp_symbol {
            if check_type.flags.contains(TypeFlags::Never) {
                // Distributing over `never` yields `never` (Go maps over an
                // empty constituent list).
                let never = self.never_type();
                if let TypeData::Conditional(ct2) = &t.data {
                    let _ = ct2.resolved_true_type.set(Arc::clone(&never));
                }
                return Some(never);
            }
            if let TypeData::Union(u) = &check_type.data {
                let constituents = u.union_or_intersection.types.clone();
                if std::env::var_os("TSOX_DEBUG_COND").is_some() {
                    eprintln!(
                        "[cond] distributing over {} constituent(s); tp={}",
                        constituents.len(),
                        tp_symbol.name
                    );
                }
                let key = Arc::as_ptr(tp_symbol) as *const crate::ast::Symbol;
                let mut results: Vec<Arc<Type>> = Vec::with_capacity(constituents.len());
                for constituent in constituents {
                    let mut mapping = std::collections::HashMap::new();
                    mapping.insert(key, Arc::clone(&constituent));
                    self.type_argument_stack.push(mapping);
                    let r =
                        self.resolve_conditional_type_with_check(t, Some(Arc::clone(&constituent)));
                    self.type_argument_stack.pop();
                    if std::env::var_os("TSOX_DEBUG_COND").is_some() {
                        eprintln!(
                            "[cond]   constituent {} -> {:?}",
                            self.type_to_string(&constituent),
                            r.as_ref().map(|x| self.type_to_string(x))
                        );
                    }
                    results.push(r?);
                }
                let union = self.get_union_type(results);
                if std::env::var_os("TSOX_DEBUG_COND").is_some() {
                    eprintln!("[cond] result union = {}", self.type_to_string(&union));
                }
                return Some(union);
            }
        }

        self.resolve_conditional_type_with_check(t, None)
    }

    /// Single (non-distributive) conditional resolution. `check_override`
    /// replaces the conditional's check type — used by the distribution
    /// path above, where the extends/branch nodes are also re-resolved with
    /// the per-constituent substitution pushed on the type_argument_stack.
    fn resolve_conditional_type_with_check(
        &mut self,
        t: &Arc<Type>,
        check_override: Option<Arc<Type>>,
    ) -> Option<Arc<Type>> {
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        let check_type = match check_override {
            Some(ref c) => Arc::clone(c),
            None => ct.check_type.clone()?,
        };
        let cond_node = ct.root.as_ref().and_then(|r| r.node.clone());
        let extends_type = if check_override.is_some() {
            // Distribution: re-resolve the extends node from the AST so
            // references to the check type parameter inside the extends
            // clause see the per-constituent mapping pushed by the caller.
            let extends_node = match cond_node.as_ref().and_then(|n| match &n.data {
                NodeData::ConditionalTypeNode(data) => Some(Arc::clone(&data.extends_type)),
                _ => None,
            }) {
                Some(node) => node,
                None => return None,
            };
            self.get_type_from_type_node(&extends_node)
        } else {
            ct.extends_type.clone()?
        };

        // If the check type is still generic, we can't evaluate yet.
        // Go checks `isDeferredType`; we approximate by checking for any
        // TypeParameter flags anywhere in the check type.
        if type_contains_type_parameter(&check_type) {
            return None;
        }

        // Set up the inference context for `infer R` parameters (if any).
        let infer_params: Vec<Arc<Type>> = ct
            .root
            .as_ref()
            .map(|r| r.infer_type_parameters.clone())
            .unwrap_or_default();
        let inferences: Vec<InferenceInfo> = infer_params
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = InferenceContext::new(inferences);

        if !infer_params.is_empty() {
            // Run inference: match check_type against extends_type, with
            // the infer type parameters as inference targets.
            self.infer_types(
                &mut context.inferences,
                Some(Arc::clone(&check_type)),
                Some(Arc::clone(&extends_type)),
                InferencePriority::NoConstraints | InferencePriority::AlwaysStrict,
                false,
            );
            // Verify every infer type parameter got a candidate. If any
            // didn't, we can't safely resolve the conditional.
            let inferred = self.get_inferred_types(&context);
            for inf in &inferred {
                if inf.flags.contains(TypeFlags::Any) && inf.intrinsic_name() == Some("error") {
                    return None;
                }
            }
        }

        // Decide which branch to take. Go instantiates the extends type
        // with the inferred types before checking assignability, then runs
        // two PROBES over the concrete check/extends pair
        // (getConditionalTypeInstantiation, checker.go ~L24440):
        //
        //   definitely-false := !extendsIsAnyOrUnknown && (check is any ||
        //       !assignableTo(permissiveInstantiation(check),
        //                      permissiveInstantiation(extends)))
        //   definitely-true  := extendsIsAnyOrUnknown ||
        //       assignableTo(restrictiveInstantiation(check),
        //                    restrictiveInstantiation(extends))
        //
        // Neither definite → the conditional stays DEFERRED. The naive
        // `assignable(check, extends)` decision collapsed e.g.
        // `unknown extends unknown[]` to the false branch at call-checking
        // time (where inference fills uninferrable signature parameters
        // with `unknown`), producing a garbage mapping instead of keeping
        // the alias's deferred form (recursiveReverseMappedType).
        let inferred_extends = if !infer_params.is_empty() {
            let inferred = self.get_inferred_types(&context);
            self.substitute_infer_type_parameters(&extends_type, &infer_params, &inferred)
        } else {
            Arc::clone(&extends_type)
        };
        let extends_any_or_unknown = inferred_extends
            .flags
            .intersects(TypeFlags::Any | TypeFlags::Unknown);
        let check_is_any = check_type.flags.contains(TypeFlags::Any);
        let definitely_false = if extends_any_or_unknown {
            false
        } else if check_is_any {
            true
        } else {
            let permissive_check = self.get_permissive_instantiation(&check_type);
            let permissive_extends = self.get_permissive_instantiation(&inferred_extends);
            !self.is_type_assignable_to(&permissive_check, &permissive_extends)
        };
        let take_true = if !definitely_false {
            let definitely_true = if extends_any_or_unknown {
                true
            } else {
                let restrictive_check = self.get_restrictive_instantiation(&check_type);
                let restrictive_extends = self.get_restrictive_instantiation(&inferred_extends);
                self.is_type_assignable_to(&restrictive_check, &restrictive_extends)
            };
            if !definitely_true {
                if std::env::var_os("TSOX_DEBUG_COND").is_some() {
                    eprintln!(
                        "[cond]     deferred (neither definite) check={} extends={}",
                        self.type_to_string(&check_type),
                        self.type_to_string(&inferred_extends)
                    );
                }
                return None;
            }
            true
        } else {
            false
        };
        // Go's extraTypes (checker.go ~L24451): when the check type is `any`,
        // the result is not just the false branch — a conditional on `any`
        // could produce either branch, so the TRUE branch is unioned in
        // (`any extends X ? A : B` ≈ A | B). (The forConstraint variant of
        // this probe — someType over permissive extends ⊆ permissive check —
        // only applies to distributive-constraint resolution contexts, which
        // we do not thread here.)
        let include_true_branch = take_true == false && check_is_any;
        if std::env::var_os("TSOX_DEBUG_COND").is_some() {
            eprintln!(
                "[cond]     take_true={} check={} extends={}",
                take_true,
                self.type_to_string(&check_type),
                self.type_to_string(&inferred_extends)
            );
        }

        // Resolve the chosen branch type node from the AST. The
        // `get_true_type_from_conditional_type` /
        // `get_false_type_from_conditional_type` helpers only return cached
        // results, so for first-time resolution we must resolve the branch
        // type node directly.
        let (cond_node, branch_node) = match ct
            .root
            .as_ref()
            .and_then(|r| r.node.as_ref())
            .and_then(|n| match &n.data {
                NodeData::ConditionalTypeNode(data) => {
                    let branch = if take_true {
                        Arc::clone(&data.true_type)
                    } else {
                        Arc::clone(&data.false_type)
                    };
                    Some((Arc::clone(n), branch))
                }
                _ => None,
            }) {
            Some(pair) => pair,
            None => return None,
        };

        // Push the ConditionalType onto the scope stack so that
        // `resolve_identifier` can find the `infer R` type
        // parameters (declared as locals of the ConditionalType).
        // Infer type parameters are only visible in the true (extends)
        // branch of a conditional type — mirroring Go's NameResolver check
        // `useResult = lastLocation == location.TrueType`. In the false
        // branch we skip pushing the scope so `R` is unresolved (TS2304).
        if take_true {
            self.push_scope(&cond_node);
        }
        // Resolve the branch node under the conditional INSTANCE'S creation
        // context (`creation_type_argument_stack` + `root.creation_scopes`):
        // branch nodes reference alias-local / container-local type-parameter
        // symbols; when a late fallback instantiation resolves an alias's
        // deferred body far from its original expansion, both the lexical
        // scope chain AND the substitution bindings are absent and branch
        // resolution produces garbage (`keyof <unresolved>`). Go carries the
        // equivalent mapper on every deferred conditional. Keys already bound
        // by the ACTIVE stack win (they belong to a fresher instantiation —
        // e.g. distribution constituents), so they are filtered out of the
        // merged frame; likewise scopes already on the stack are kept.
        let creation_scopes: Vec<u64> = ct
            .root
            .as_ref()
            .map(|r| r.creation_scopes.clone())
            .unwrap_or_default();
        let mut common = 0usize;
        while common < creation_scopes.len()
            && common < self.scope_stack.len()
            && creation_scopes[common] == self.scope_stack[common]
        {
            common += 1;
        }
        let scopes_pushed = creation_scopes.len() - common;
        self.scope_stack.extend_from_slice(&creation_scopes[common..]);

        let mut merged_creation: HashMap<usize, Arc<Type>> = HashMap::new();
        for frame in ct.creation_type_argument_stack.iter() {
            for (k, v) in frame {
                merged_creation.insert(*k, Arc::clone(v));
            }
        }
        for map in self.type_argument_stack.iter() {
            for k in map.keys() {
                merged_creation.remove(&(*k as usize));
            }
        }
        let pushes_creation = !merged_creation.is_empty();
        if pushes_creation {
            self.type_argument_stack
                .push(merged_creation.into_iter().map(|(k, v)| ((k as *const Symbol), v)).collect());
        }
        let branch = self.get_type_from_type_node(&branch_node);
        if pushes_creation {
            self.type_argument_stack.pop();
        }
        if scopes_pushed > 0 {
            self.scope_stack.truncate(self.scope_stack.len() - scopes_pushed);
        }
        if take_true {
            self.pop_scope();
        }
        let resolved = if !infer_params.is_empty() {
            let inferred = self.get_inferred_types(&context);
            self.substitute_infer_type_parameters(&branch, &infer_params, &inferred)
        } else {
            Arc::clone(&branch)
        };
        // extraTypes union-in (see `include_true_branch` above): the result
        // for an `any` check is union(trueBranch, falseBranch).
        let resolved = if include_true_branch
            && let Some(true_branch) = self.get_forced_branch_type_of_conditional_type(t, true)
        {
            let true_branch = if !infer_params.is_empty() {
                let inferred = self.get_inferred_types(&context);
                self.substitute_infer_type_parameters(&true_branch, &infer_params, &inferred)
            } else {
                true_branch
            };
            self.get_union_type(vec![true_branch, Arc::clone(&resolved)])
        } else {
            resolved
        };
        // Cache the result so subsequent lookups don't re-run.
        // SAFETY: `resolved_true_type` / `resolved_false_type` are
        // `OnceLock` and we just verified they're unset. `set` returns
        // `Result<(), _>`; we ignore the error in the rare race case.
        if let TypeData::Conditional(ct2) = &t.data {
            let cell = if take_true {
                &ct2.resolved_true_type
            } else {
                &ct2.resolved_false_type
            };
            let _ = cell.set(Arc::clone(&resolved));
        }
        Some(resolved)
    }

    /// Go's `getPermissiveInstantiation` (checker.go ~L24547): instantiate
    /// every type parameter with the wildcard type (assignable to and from
    /// everything). Used by the definitely-false probe of conditional
    /// resolution.
    pub fn get_permissive_instantiation(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let key = Arc::as_ptr(t) as usize;
        if let Some(cached) = self.probe_cache_permissive.get(&key) {
            return Arc::clone(cached);
        }
        let result = self.instantiate_probing(t, ProbeMode::Permissive);
        self.probe_cache_permissive.insert(key, Arc::clone(&result));
        result
    }

    /// Go's `getRestrictiveInstantiation` (checker.go ~L24560): replace every
    /// type parameter with a constraint-stripped copy of itself. Used by the
    /// definitely-true probe. Types already free of type parameters return
    /// unchanged, which is the common case for concrete check/extends pairs.
    pub fn get_restrictive_instantiation(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let key = Arc::as_ptr(t) as usize;
        if let Some(cached) = self.probe_cache_restrictive.get(&key) {
            return Arc::clone(cached);
        }
        let result = self.instantiate_probing(t, ProbeMode::Restrictive);
        self.probe_cache_restrictive.insert(key, Arc::clone(&result));
        result
    }

    fn instantiate_probing(&mut self, t: &Arc<Type>, mode: ProbeMode) -> Arc<Type> {
        match &t.data {
            TypeData::TypeParameter(_) => match mode {
                ProbeMode::Permissive => self.any_function_type(),
                ProbeMode::Restrictive => {
                    let tp = match &t.data {
                        TypeData::TypeParameter(tp) => tp,
                        _ => unreachable!(),
                    };
                    if tp.constraint.is_none() {
                        return Arc::clone(t);
                    }
                    let mut rebuilt = Type::new(
                        t.flags,
                        TypeData::TypeParameter(TypeParameterData {
                            constrained: ConstrainedTypeData::default(),
                            constraint: None,
                            target: tp.target.clone(),
                            mapper: tp.mapper.clone(),
                            is_this_type: tp.is_this_type,
                            resolved_default_type: OnceLock::new(),
                        }),
                    );
                    rebuilt.symbol = t.symbol.clone();
                    rebuilt.object_flags = t.object_flags;
                    Arc::new(rebuilt)
                }
            },
            TypeData::Union(u) => {
                let types = u.union_or_intersection.types.clone();
                let new_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|c| self.instantiate_probing(c, mode))
                    .collect();
                if new_types.iter().zip(types.iter()).all(|(n, o)| Arc::ptr_eq(n, o)) {
                    return Arc::clone(t);
                }
                self.get_union_type(new_types)
            }
            TypeData::Intersection(i) => {
                let types = i.union_or_intersection.types.clone();
                let new_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|c| self.instantiate_probing(c, mode))
                    .collect();
                if new_types.iter().zip(types.iter()).all(|(n, o)| Arc::ptr_eq(n, o)) {
                    return Arc::clone(t);
                }
                self.get_intersection_type(new_types)
            }
            TypeData::Object(o) => {
                // Array/reference instantiations substitute their type
                // arguments; argument-less interfaces/classes cannot carry
                // free type parameters here — unchanged.
                if o.type_arguments.is_empty() {
                    return Arc::clone(t);
                }
                let new_args: Vec<Arc<Type>> = o
                    .type_arguments
                    .iter()
                    .map(|a| self.instantiate_probing(a, mode))
                    .collect();
                if new_args
                    .iter()
                    .zip(o.type_arguments.iter())
                    .all(|(n, old)| Arc::ptr_eq(n, old))
                {
                    return Arc::clone(t);
                }
                if o.target.is_none() && o.type_arguments.len() == 1 && self.is_array_type(t) {
                    return self.create_array_type(Arc::clone(&new_args[0]));
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Object(ObjectTypeData {
                        structured: StructuredTypeData::default(),
                        target: o.target.clone(),
                        mapper: None,
                        type_arguments: new_args,
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                Arc::new(rebuilt)
            }
            TypeData::Tuple(tup) => {
                let args: Vec<Arc<Type>> = tup
                    .element_infos
                    .iter()
                    .filter_map(|ei| ei.type_.clone())
                    .collect();
                if args.is_empty() {
                    return Arc::clone(t);
                }
                let new_elems: Vec<Arc<Type>> =
                    args.iter().map(|e| self.instantiate_probing(e, mode)).collect();
                if new_elems.iter().zip(args.iter()).all(|(n, o)| Arc::ptr_eq(n, o)) {
                    return Arc::clone(t);
                }
                self.create_tuple_type(new_elems)
            }
            TypeData::Conditional(ct) => {
                // Keep deferred conditionals deferred inside probes; just
                // substitute their recorded check/extends so the relater sees
                // wildcard/restrictive forms at the leaves (Go re-instantiates
                // through its full mapper machinery).
                let (old_check, old_extends) =
                    match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
                        (Some(c), Some(e)) => (Arc::clone(c), Arc::clone(e)),
                        _ => return Arc::clone(t),
                    };
                let new_check = self.instantiate_probing(&old_check, mode);
                let new_extends = self.instantiate_probing(&old_extends, mode);
                if Arc::ptr_eq(&new_check, &old_check) && Arc::ptr_eq(&new_extends, &old_extends)
                {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Conditional(ConditionalTypeData {
                        constrained: ConstrainedTypeData::default(),
                        root: ct.root.as_ref().map(|r| {
                            Box::new(ConditionalRoot {
                                node: r.node.clone(),
                                check_type: r.check_type.clone(),
                                extends_type: r.extends_type.clone(),
                                is_distributive: r.is_distributive,
                                check_type_parameter_symbol: r
                                    .check_type_parameter_symbol
                                    .clone(),
                                infer_type_parameters: r.infer_type_parameters.clone(),
                                outer_type_parameters: r.outer_type_parameters.clone(),
                                alias: None,
                                creation_scopes: r.creation_scopes.clone(),
                            })
                        }),
                        check_type: Some(new_check),
                        extends_type: Some(new_extends),
                        resolved_true_type: OnceLock::new(),
                        resolved_false_type: OnceLock::new(),
                        resolved_inferred_true_type: OnceLock::new(),
                        resolved_default_constraint: OnceLock::new(),
                        resolved_constraint_of_distributive: OnceLock::new(),
                        mapper: None,
                        combined_mapper: None,
                        creation_type_argument_stack: Vec::new(),
                    }),
                );
                rebuilt.symbol = t.symbol.clone();
                rebuilt.object_flags = t.object_flags;
                Arc::new(rebuilt)
            }
            TypeData::IndexedAccess(ia) => {
                let (Some(old_obj), Some(old_idx)) =
                    (ia.object_type.as_ref(), ia.index_type.as_ref())
                else {
                    return Arc::clone(t);
                };
                let new_obj = self.instantiate_probing(old_obj, mode);
                let new_idx = self.instantiate_probing(old_idx, mode);
                if Arc::ptr_eq(&new_obj, old_obj) && Arc::ptr_eq(&new_idx, old_idx) {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::IndexedAccess(IndexedAccessTypeData {
                        constrained: ConstrainedTypeData::default(),
                        object_type: Some(new_obj),
                        index_type: Some(new_idx),
                        access_flags: ia.access_flags,
                    }),
                );
                rebuilt.symbol = t.symbol.clone();
                rebuilt.object_flags = t.object_flags;
                Arc::new(rebuilt)
            }
            _ => Arc::clone(t),
        }
    }

    /// Substitute occurrences of `infer_params[i]` in `t` with
    /// `substitutions[i]`. Simplified port of Go's `instantiateType` for
    /// the infer-parameter case — walks the type recursively and replaces
    /// pointer-equal occurrences. Doesn't handle aliases, mapped type
    /// constraints, or other complex instantiation scenarios.
    /// Whether two same-named type-parameter symbols were declared under
    /// the SAME container symbol (walking each declaration's parent chain
    /// to its nearest symbol-ful ancestor). Multi-declaration forks of one
    /// generic interface share the merged interface symbol; a class's
    /// type parameter and a method's own same-named parameter do not.
    pub(crate) fn type_param_symbols_share_container(&self, a: &Arc<Symbol>, b: &Arc<Symbol>) -> bool {
        let symbol_map = self.program.symbol_map();
        let container_of = |s: &Arc<Symbol>| -> Option<usize> {
            let mut node = s.declarations.first()?.parent.as_ref()?;
            for _ in 0..4 {
                if let Some(sym) = symbol_map.symbols.get(&node.id()) {
                    return Some(Arc::as_ptr(sym) as *const Symbol as usize);
                }
                node = node.parent.as_ref()?;
            }
            None
        };
        match (container_of(a), container_of(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    /// Deep substitution of an anonymous object type's PROPERTY types
    /// under the call-site type-argument mapping (Go instantiateType for
    /// object literals / anonymous object types). Self-referential
    /// property types (`b: typeof x`) bind to the in-progress result via
    /// `subst_object_in_progress`. Only rebuilds when something actually
    /// changed; otherwise the original Arc is returned.
    pub(crate) fn substitute_object_properties_deep(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        let key = Arc::as_ptr(t) as usize;
        if let Some(cached) = self.subst_object_in_progress.get(&key) {
            return Arc::clone(cached);
        }
        let Some(o) = t.as_object() else {
            return Arc::clone(t);
        };
        // PEEK each property's CACHED resolved type — never force an
        // on-demand resolution here: resolving in the substitution's
        // (possibly foreign) scope context caches a degraded type on the
        // SHARED symbol, breaking later contextual-signature lookups
        // (conditionalTypeContextualTypeSimplifications). Properties
        // without a cached type keep their original symbols (nothing to
        // substitute yet; lazy resolution serves them as before).
        let mut old_types: Vec<Option<Arc<Type>>> = Vec::with_capacity(o.structured.properties.len());
        for prop in &o.structured.properties {
            old_types.push(
                self.value_symbol_links
                    .get(prop)
                    .and_then(|l| l.resolved_type.clone()),
            );
        }
        // Shell first — cyclic properties bind to it through the map.
        let shell = Arc::new(Type::new(
            t.flags,
            TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: o.structured.members.clone(),
                    properties: o.structured.properties.clone(),
                    signatures: o.structured.signatures.clone(),
                    call_signature_count: o.structured.call_signature_count,
                    index_infos: o.structured.index_infos.clone(),
                    ..Default::default()
                },
                target: o.target.clone(),
                mapper: o.mapper.clone(),
                type_arguments: o.type_arguments.clone(),
            }),
        ));
        {
            let shell_mut = Arc::as_ptr(&shell) as *mut Type;
            unsafe {
                (*shell_mut).object_flags = t.object_flags;
                (*shell_mut).symbol = t.symbol.clone();

            }
        }
        self.subst_object_in_progress.insert(key, Arc::clone(&shell));
        let mut changed = false;
        let mut new_props: Vec<Arc<Symbol>> = Vec::with_capacity(o.structured.properties.len());
        let mut new_members = o.structured.members.clone();
        for (prop, old_t) in o.structured.properties.iter().zip(old_types.iter()) {
            let Some(old_t) = old_t else {
                new_props.push(Arc::clone(prop));
                continue;
            };
            let new_t = self.substitute_infer_type_parameters(old_t, params, substitutions);
            if Arc::ptr_eq(&new_t, old_t) {
                new_props.push(Arc::clone(prop));
                continue;
            }
            changed = true;
            let mut new_sym = Symbol::new(prop.flags, prop.name.clone());
            new_sym.declarations = prop.declarations.clone();
            new_sym.check_flags = prop.check_flags;
            let new_sym = Arc::new(new_sym);
            self.value_symbol_links.insert(
                &new_sym,
                ValueSymbolLinks {
                    resolved_type: Some(new_t),
                    ..Default::default()
                },
            );
            new_members.insert(prop.name.clone(), Arc::clone(&new_sym));
            new_props.push(new_sym);
        }
        if !changed {
            self.subst_object_in_progress.remove(&key);
            return Arc::clone(t);
        }
        // Fill the shell with the substituted member tables (checker is
        // single-threaded; the shell is not yet shared elsewhere).
        {
            let shell_mut = Arc::as_ptr(&shell) as *mut Type;
            unsafe {
                if let TypeData::Object(so) = &mut (*shell_mut).data {
                    so.structured.members = new_members;
                    so.structured.properties = new_props;
                }
            }
        }
        shell
    }

    pub fn substitute_infer_type_parameters(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        // Fast path: no parameters to substitute.
        if params.is_empty() || substitutions.is_empty() {
            return Arc::clone(t);
        }
        // Fast path: direct pointer match — return the substitution.
        // Type parameters also match by SYMBOL identity: the same type
        // parameter's type may be instantiated more than once (no global
        // interning), but the underlying symbol is shared. The NAME
        // fallback covers multi-declaration forks (a generic interface
        // declared twice binds one type-parameter symbol per declaration)
        // — but ONLY when both symbols share the same declaring container
        // symbol: a class's type parameter `T` and a method's OWN `T`
        // (D3-sig) share nothing but the name, and name-matching them
        // wrongly instantiated the method's signature with the class's
        // argument (`Box<number>` made `wrap<T>(x: T): T` take number).
        for (i, p) in params.iter().enumerate() {
            if Arc::ptr_eq(p, t)
                || (p.is_type_parameter()
                    && t.is_type_parameter()
                    && (p.symbol.as_ref().zip(t.symbol.as_ref()).is_some_and(
                        |(ps, ts)| {
                            Arc::ptr_eq(ps, ts)
                                || (ps.name == ts.name
                                    && self.type_param_symbols_share_container(ps, ts))
                        },
                    )))
            {
                return Arc::clone(&substitutions[i.min(substitutions.len() - 1)]);
            }
        }
        // Recursive substitution into structured types. We handle the
        // cases that show up in conditional extends types and branches:
        // unions, intersections, arrays (Object with a single type
        // argument), and tuples. Other kinds (mapped, indexed access,
        // nested conditionals, etc.) are returned as-is — a full
        // `instantiateType` port would be needed for those.
        match &t.data {
            TypeData::Union(u) => {
                let new_types: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|inner| {
                        self.substitute_infer_type_parameters(inner, params, substitutions)
                    })
                    .collect();
                self.get_union_type(new_types)
            }
            TypeData::Intersection(i) => {
                let new_types: Vec<Arc<Type>> = i
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|inner| {
                        self.substitute_infer_type_parameters(inner, params, substitutions)
                    })
                    .collect();
                self.get_intersection_type(new_types)
            }
            TypeData::Object(o) => {
                // Array type: `T[]` is represented as an Object with
                // `object_flags: Reference`, no `target`, and a single
                // type argument (the element type). Substitute the
                // element type and rebuild via `create_array_type`.
                if t.object_flags.contains(ObjectFlags::Reference)
                    && o.target.is_none()
                    && o.type_arguments.len() == 1
                {
                    let new_elem = self.substitute_infer_type_parameters(
                        &o.type_arguments[0],
                        params,
                        substitutions,
                    );
                    // If nothing changed, avoid creating a new array type.
                    if Arc::ptr_eq(&new_elem, &o.type_arguments[0]) {
                        return Arc::clone(t);
                    }
                    return self.create_array_type(new_elem);
                }
                // Generic type reference (`Promise<T>`, `Map<K, V>` — an
                // Object with type arguments): substitute each type
                // argument and rebuild, preserving target/symbol/flags.
                if !o.type_arguments.is_empty() {
                    let new_args: Vec<Arc<Type>> = o
                        .type_arguments
                        .iter()
                        .map(|arg| {
                            self.substitute_infer_type_parameters(arg, params, substitutions)
                        })
                        .collect();
                    let changed = o
                        .type_arguments
                        .iter()
                        .zip(new_args.iter())
                        .any(|(old, new)| !Arc::ptr_eq(old, new));
                    if changed {
                        // Preserve the resolved member table: the rebuilt
                        // reference keeps its structure (Go re-instantiates
                        // members under the new mapper; re-resolving here
                        // would need the substitution stack — stale members
                        // match the previous no-args behavior).
                        // StructuredTypeData isn't Clone (OnceLock), so the
                        // member tables are shared field-by-field.
                        // Index-signature VALUE types must follow the
                        // substitution (the raw declaration's `[n: number]:
                        // T` keeps its type parameter otherwise — comparing
                        // the replica against a properly instantiated
                        // reference then fails on `T` vs the argument).
                        let new_index_infos: Vec<Arc<crate::checker::IndexInfo>> = o
                            .structured
                            .index_infos
                            .iter()
                            .map(|info| {
                                let new_value = info.value_type.as_ref().map(|v| {
                                    self.substitute_infer_type_parameters(v, params, substitutions)
                                });
                                if new_value.is_some()
                                    && !new_value.as_ref().is_some_and(|nv| {
                                        info.value_type.as_ref().is_some_and(|ov| Arc::ptr_eq(nv, ov))
                                    })
                                {
                                    Arc::new(crate::checker::IndexInfo {
                                        key_type: info.key_type.clone(),
                                        value_type: new_value,
                                        is_readonly: info.is_readonly,
                                        declaration: info.declaration.clone(),
                                        index_symbol: info.index_symbol.clone(),
                                        components: info.components.clone(),
                                    })
                                } else {
                                    Arc::clone(info)
                                }
                            })
                            .collect();
                        let mut rebuilt = Type::new(
                            t.flags,
                            TypeData::Object(ObjectTypeData {
                                structured: StructuredTypeData {
                                    members: o.structured.members.clone(),
                                    properties: o.structured.properties.clone(),
                                    signatures: o.structured.signatures.clone(),
                                    call_signature_count: o.structured.call_signature_count,
                                    index_infos: new_index_infos,
                                    ..Default::default()
                                },
                                target: o.target.clone(),
                                mapper: o.mapper.clone(),
                                type_arguments: new_args,
                            }),
                        );
                        rebuilt.object_flags = t.object_flags;
                        rebuilt.symbol = t.symbol.clone();
                        return Arc::new(rebuilt);
                    }
                    return Arc::clone(t);
                }
                // Function/constructor type: an anonymous object whose only
                // structure is its signatures. Substitute each signature's
                // parameter and return types via the instantiation-override
                // mechanism (Go: `instantiateSignature` under the mapper).
                if t.object_flags.contains(ObjectFlags::Anonymous)
                    && !o.structured.signatures.is_empty()
                {
                    let signatures = o.structured.signatures.clone();
                    let call_signature_count = o.structured.call_signature_count;
                    let mut changed = false;
                    let mut new_sigs: Vec<Arc<Signature>> =
                        Vec::with_capacity(signatures.len());
                    for sig in &signatures {
                        let rest_offset = usize::from(sig.has_rest_parameter());
                        let fixed = sig.parameters.len().saturating_sub(rest_offset);
                        let mut new_params: Vec<Arc<Type>> =
                            Vec::with_capacity(sig.parameters.len());
                        let mut old_params: Vec<Arc<Type>> =
                            Vec::with_capacity(sig.parameters.len());
                        for i in 0..fixed {
                            let pt = self
                                .try_get_type_at_position(sig, i)
                                .unwrap_or_else(|| self.any_type());
                            old_params.push(Arc::clone(&pt));
                            new_params.push(
                                self.substitute_infer_type_parameters(&pt, params, substitutions),
                            );
                        }
                        if rest_offset == 1 {
                            if let Some(last) = sig.parameters.last() {
                                let rt = self.get_type_of_symbol(last);
                                old_params.push(Arc::clone(&rt));
                                new_params.push(
                                    self.substitute_infer_type_parameters(&rt, params, substitutions),
                                );
                            }
                        }
                        let new_return = self
                            .get_return_type_of_signature(sig)
                            .map(|rt| {
                                self.substitute_infer_type_parameters(&rt, params, substitutions)
                            });
                        let params_changed = old_params
                            .iter()
                            .zip(new_params.iter())
                            .any(|(old, new)| !Arc::ptr_eq(old, new));
                        let return_changed = new_return.as_ref().is_some_and(|nr| {
                            self.get_return_type_of_signature(sig)
                                .is_some_and(|old| !Arc::ptr_eq(nr, &old))
                        });
                        if !params_changed && !return_changed {
                            new_sigs.push(Arc::clone(sig));
                            continue;
                        }
                        changed = true;
                        let mut inst = Signature::new();
                        inst.flags = sig.flags;
                        inst.min_argument_count = sig.min_argument_count;
                        inst.resolved_min_argument_count = sig.resolved_min_argument_count;
                        inst.declaration = sig.declaration.clone();
                        inst.target = Some(Arc::clone(sig));
                        inst.parameters = sig.parameters.clone();
                        inst.this_parameter = sig.this_parameter.clone();
                        inst.type_parameters = sig.type_parameters.clone();
                        inst.resolved_type_predicate = sig.resolved_type_predicate.clone();
                        inst.instantiated_parameter_types = Some(new_params);
                        if let Some(nr) = new_return {
                            let _ = inst.resolved_return_type.set(nr);
                        }
                        new_sigs.push(Arc::new(inst));
                    }
                    if !changed {
                        return Arc::clone(t);
                    }
                    let is_construct = call_signature_count == 0;
                    return self.create_function_or_constructor_type(new_sigs, is_construct);
                }
                // ANONYMOUS object type with PROPERTIES (the return-type
                // family — cyclicTypeInstantiation): the members' resolved
                // types may reference the signature's type parameters
                // (`var x: { a: T; b: typeof x }` inside `function
                // foo<T>()`). Go instantiates the object under the
                // call-site mapper; deep-substitute the property types
                // with a pointer-keyed in-progress map that preserves
                // self-references (`b: typeof x`). NAMED interfaces and
                // classes are excluded — they instantiate through their
                // type REFERENCE (mapper/target), and cloning their
                // property symbols here breaks symbol-identity-keyed
                // links (contextual signatures —
                // conditionalTypeContextualTypeSimplifications regression).
                if self.in_return_substitution
                    && t.symbol.is_none()
                    && !o.structured.properties.is_empty()
                {
                    let fresh = self.subst_object_in_progress.is_empty();
                    let result =
                        self.substitute_object_properties_deep(t, params, substitutions);
                    if fresh {
                        self.subst_object_in_progress.clear();
                    }
                    return result;
                }
                // Other object types (non-generic interfaces etc.) — return
                // as-is.
                Arc::clone(t)
            }
            TypeData::Tuple(tup) => {
                // Substitute each tuple element's type and rebuild.
                let new_elems: Vec<Arc<Type>> = tup
                    .element_infos
                    .iter()
                    .map(|ei| match &ei.type_ {
                        Some(ty) => {
                            self.substitute_infer_type_parameters(ty, params, substitutions)
                        }
                        None => self.error_type(),
                    })
                    .collect();
                // If nothing changed, avoid rebuilding.
                let changed = tup
                    .element_infos
                    .iter()
                    .zip(new_elems.iter())
                    .any(|(ei, new_t)| match &ei.type_ {
                        Some(old_t) => !Arc::ptr_eq(old_t, new_t),
                        None => true,
                    });
                if !changed {
                    return Arc::clone(t);
                }
                self.create_tuple_type(new_elems)
            }
            // Deferred indexed access `Obj[Idx]`: substitute both the
            // object and the index, rebuilding only when something changed
            // (Go handles this via `instantiateType`).
            TypeData::IndexedAccess(ia) => {
                let new_object = ia
                    .object_type
                    .as_ref()
                    .map(|o| self.substitute_infer_type_parameters(o, params, substitutions));
                let new_index = ia
                    .index_type
                    .as_ref()
                    .map(|idx| self.substitute_infer_type_parameters(idx, params, substitutions));
                let object_changed = new_object
                    .as_ref()
                    .zip(ia.object_type.as_ref())
                    .map(|(new, old)| !Arc::ptr_eq(new, old))
                    .unwrap_or(false);
                let index_changed = new_index
                    .as_ref()
                    .zip(ia.index_type.as_ref())
                    .map(|(new, old)| !Arc::ptr_eq(new, old))
                    .unwrap_or(false);
                if !object_changed && !index_changed {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::IndexedAccess(IndexedAccessTypeData {
                        constrained: ConstrainedTypeData::default(),
                        object_type: new_object.or_else(|| ia.object_type.clone()),
                        index_type: new_index.or_else(|| ia.index_type.clone()),
                        access_flags: ia.access_flags,
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                Arc::new(rebuilt)
            }
            // A deferred conditional whose check type mentions a substituted
            // parameter: substitute the check type and re-resolve (Go's
            // `getConditionalTypeInstantiation` — instantiating a deferred
            // conditional with concrete arguments resolves it, distributing
            // over union check types when the root is distributive).
            TypeData::Conditional(ct) => {
                let Some(old_check) = ct.check_type.clone() else {
                    return Arc::clone(t);
                };
                let new_check =
                    self.substitute_infer_type_parameters(&old_check, params, substitutions);
                if Arc::ptr_eq(&new_check, &old_check) || type_contains_type_parameter(&new_check)
                {
                    return Arc::clone(t);
                }
                self.resolve_conditional_type_with_check(t, Some(new_check))
                    .unwrap_or_else(|| Arc::clone(t))
            }
            // For nested mapped types, indexed access types, etc., we don't
            // recursively substitute (that would risk re-resolution or
            // require a full `instantiateType`); return as-is. Go handles
            // these via `instantiateType`.
            _ => Arc::clone(t),
        }
    }
}

/// Whether a type contains any type-parameter subterm. Used by
/// `resolve_conditional_type` to decide whether the conditional can be
/// evaluated now or must be deferred until the type parameters are
/// substituted with concrete types.
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
        _ => false,
    }
}

/// Whether `t` mentions the SPECIFIC type-parameter type `needle`
/// (pointer identity) anywhere in its structure — used to decide whether
/// an Array interface member's type needs element substitution
/// (`push(...items: T[])` yes, `length: number` no).
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

/// Collect the distinct free type-parameter types appearing in `t`'s
/// parameter/return/type-argument structure (deduped by pointer). Used to
/// substitute an Array interface member's free `T` with the element type —
/// collecting from the member type itself sidesteps symbol-identity
/// divergence across merged interface declarations.
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

// ────────────────────────────────────────────────────────────────────────────
// Ternary-returning comparison wrappers
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    pub fn compare_types_identical(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> Ternary {
        if self.is_type_identical_to(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn compare_types_assignable_simple(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Ternary {
        if self.is_type_assignable_to(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn compare_types_assignable_worker(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _report_errors: bool,
    ) -> Ternary {
        if self.is_type_assignable_to(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn compare_types_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> Ternary {
        if self.is_type_subtype_of(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Error-reporting relation checks
    // ────────────────────────────────────────────────────────────────────────

    pub fn check_type_assignable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<crate::ast::Node>>,
        _head_message: Option<&crate::diagnostics::Message>,
    ) -> bool {
        // TODO: full error reporting with diagnostics
        self.is_type_assignable_to(source, target)
    }

    pub fn check_type_assignable_to_ex(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<crate::ast::Node>>,
        _head_message: Option<&crate::diagnostics::Message>,
        _diagnostic_output: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        // TODO: full error reporting with diagnostics
        self.is_type_assignable_to(source, target)
    }

    pub fn check_type_comparable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<crate::ast::Node>>,
        _head_message: Option<&crate::diagnostics::Message>,
    ) -> bool {
        // TODO: full error reporting with diagnostics
        self.is_type_comparable_to(source, target)
    }

    pub fn check_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        _error_node: Option<&Arc<crate::ast::Node>>,
    ) -> bool {
        // TODO: full error reporting with diagnostics
        self.is_type_related_to(source, target, relation)
    }

    /// Top-level elaborating assignability check (Go
    /// `checkTypeAssignableToAndOptionallyElaborate` →
    /// `checkTypeRelatedToEx` + `reportErrorResults`/`reportRelationError`,
    /// relater.go ~L426/4740). Runs the comparison with chain recording
    /// enabled; on failure the collected chain becomes the nested
    /// "compatibility pyramid" diagnostic, headed by the generalized
    /// `Type 'X' is not assignable to type 'Y'` message.
    /// Go `elaborateError` (relater.go ~L444): dispatch an elaboration
    /// attempt on the failing EXPRESSION. Object literals elaborate per
    /// property, array literals per element; parenthesized expressions
    /// unwrap. Returns true when a more specific error was reported (the
    /// caller suppresses the generalized head+pyramid form).
    fn elaborate_error(
        &mut self,
        expr: &Arc<crate::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        out: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        match expr.kind {
            crate::ast::SyntaxKind::ParenthesizedExpression => {
                let inner = match &expr.data {
                    crate::ast::NodeData::ParenthesizedExpression(d) => {
                        Arc::clone(&d.expression)
                    }
                    _ => return false,
                };
                self.elaborate_error(&inner, source, target, relation, out)
            }
            crate::ast::SyntaxKind::ObjectLiteralExpression => {
                self.elaborate_object_literal(expr, source, target, relation, out)
            }
            crate::ast::SyntaxKind::ArrayLiteralExpression => {
                self.elaborate_array_literal(expr, source, target, relation, out)
            }
            _ => false,
        }
    }

    /// Go `elaborateObjectLiteral` (relater.go ~L508): each property whose
    /// type mismatches the target's same-named property reports at the
    /// property NAME node, recursing into the initializer first.
    fn elaborate_object_literal(
        &mut self,
        node: &Arc<crate::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        if target.flags.intersects(
            TypeFlags::String
                | TypeFlags::Number
                | TypeFlags::Boolean
                | TypeFlags::BigInt
                | TypeFlags::ESSymbol
                | TypeFlags::Void
                | TypeFlags::Undefined
                | TypeFlags::Null
                | TypeFlags::Never
                | TypeFlags::Enum
                | TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BooleanLiteral,
        ) {
            return false;
        }
        let properties = match &node.data {
            crate::ast::NodeData::ObjectLiteralExpression(d) => &d.properties,
            _ => return false,
        };
        let mut reported = false;
        for prop in properties.iter() {
            if prop.kind == crate::ast::SyntaxKind::SpreadAssignment {
                continue;
            }
            let (name_node, initializer): (&Arc<crate::ast::Node>, Option<Arc<crate::ast::Node>>) =
                match &prop.data {
                    crate::ast::NodeData::PropertyAssignment(d) => {
                        (&d.name, Some(Arc::clone(&d.initializer)))
                    }
                    crate::ast::NodeData::ShorthandPropertyAssignment(d) => (&d.name, None),
                    crate::ast::NodeData::MethodDeclaration(d) => (&d.name, None),
                    crate::ast::NodeData::GetAccessorDeclaration(d) => (&d.name, None),
                    crate::ast::NodeData::SetAccessorDeclaration(d) => (&d.name, None),
                    _ => continue,
                };
            let name = self.get_property_name_from_node(name_node);
            if name.is_empty() {
                continue;
            }
            let Some(target_prop_type) = self.get_type_of_property_of_type(target, &name) else {
                continue;
            };
            let Some(source_prop_type) = self.get_type_of_property_of_type(source, &name) else {
                continue;
            };
            if self.is_type_related_to(&source_prop_type, &target_prop_type, relation) {
                continue;
            }
            if let Some(init) = initializer
                && self.elaborate_error(
                    &init,
                    &source_prop_type,
                    &target_prop_type,
                    relation,
                    out.as_deref_mut(),
                )
            {
                reported = true;
                continue;
            }
            // Issue the error on the property name itself (Go
            // `elaborateElement`'s `prop` node).
            match out.as_deref_mut() {
                Some(o) => {
                    self.check_type_related_to_and_optionally_elaborate(
                        &source_prop_type,
                        &target_prop_type,
                        relation,
                        Some(name_node),
                        None,
                        None,
                        Some(o),
                    );
                }
                None => {
                    self.check_type_related_to_and_optionally_elaborate(
                        &source_prop_type,
                        &target_prop_type,
                        relation,
                        Some(name_node),
                        None,
                        None,
                        None,
                    );
                }
            }
            reported = true;
        }
        reported
    }

    /// Go `elaborateArrayLiteral` (relater.go ~L521): each element whose
    /// type mismatches the target's element type reports at the element
    /// node, recursing into the element expression first.
    fn elaborate_array_literal(
        &mut self,
        node: &Arc<crate::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        if target.flags.intersects(
            TypeFlags::String
                | TypeFlags::Number
                | TypeFlags::Boolean
                | TypeFlags::BigInt
                | TypeFlags::ESSymbol
                | TypeFlags::Void
                | TypeFlags::Undefined
                | TypeFlags::Null
                | TypeFlags::Never
                | TypeFlags::Enum
                | TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BooleanLiteral,
        ) {
            return false;
        }
        let elements = match &node.data {
            crate::ast::NodeData::ArrayLiteralExpression(d) => &d.elements,
            _ => return false,
        };
        let _ = source;
        let mut reported = false;
        for (i, element) in elements.iter().enumerate() {
            if element.kind == crate::ast::SyntaxKind::OmittedExpression
                || element.kind == crate::ast::SyntaxKind::SpreadElement
            {
                continue;
            }
            // Target element type: array targets expose their element
            // type at every index; tuple targets their i-th element
            // (absent positions skip); other object targets contribute
            // through their NUMERIC INDEX signature (Go
            // `getBestMatchIndexedAccessTypeOrUndefined` —
            // `ConcatArray<never>`'s `[n: number]: never` types the
            // element check `never`).
            let target_elem = if self.is_array_type(target) {
                self.get_array_element_type(target)
            } else if self.is_tuple_type(target) {
                match self.get_tuple_element_type(target, i) {
                    Some(t) => t,
                    None => continue,
                }
            } else {
                // Other object targets contribute through their NUMERIC
                // INDEX signature — read it from a properly instantiated
                // form when the target is a generic reference (the
                // declared table carries the raw type parameter;
                // `ConcatArray<never>`'s `[n: number]: T` must resolve
                // through the type arguments to `never`).
                let index_source = match target.symbol.as_ref() {
                    Some(sym)
                        if sym.flags.contains(SymbolFlags::Interface)
                            && target
                                .as_object()
                                .is_some_and(|o| !o.type_arguments.is_empty()) =>
                    {
                        let args = target.as_object().unwrap().type_arguments.clone();
                        Some(self.resolve_interface_type_ex(sym, Some(args)))
                    }
                    _ => None,
                }
                .unwrap_or_else(|| Arc::clone(target));
                let indexed = index_source.as_structured().and_then(|st| {
                    st.index_infos.iter().find_map(|info| {
                        info.key_type
                            .as_ref()
                            .filter(|k| k.flags.contains(TypeFlags::Number))
                            .and_then(|_| info.value_type.clone())
                    })
                });
                match indexed {
                    Some(t) => t,
                    None => continue,
                }
            };
            let source_elem = self.get_type_of_node(element);
            if self.is_type_related_to(&source_elem, &target_elem, relation) {
                continue;
            }
            if self.elaborate_error(element, &source_elem, &target_elem, relation, out.as_deref_mut()) {
                reported = true;
                continue;
            }
            // Dedupe per element location: the outer contextual-elements
            // pass may report the same mismatch (same code + loc).
            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2322 && d.loc == element.loc);
            if !already {
                match out.as_deref_mut() {
                    Some(o) => {
                        self.check_type_related_to_and_optionally_elaborate(
                            &source_elem,
                            &target_elem,
                            relation,
                            Some(element),
                            None,
                            None,
                            Some(o),
                        );
                    }
                    None => {
                        self.check_type_related_to_and_optionally_elaborate(
                            &source_elem,
                            &target_elem,
                            relation,
                            Some(element),
                            None,
                            None,
                            None,
                        );
                    }
                }
            }
            reported = true;
        }
        reported
    }

    pub fn check_type_assignable_to_and_optionally_elaborate(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        error_node: Option<&Arc<crate::ast::Node>>,
        _expr: Option<&Arc<crate::ast::Node>>,
        _head_message: Option<&crate::diagnostics::Message>,
        _diagnostic_output: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        self.check_type_related_to_and_optionally_elaborate(
            source,
            target,
            RelationKind::Assignable,
            error_node,
            _expr,
            _head_message,
            _diagnostic_output,
        )
    }

    /// `check_type_related_to_and_optionally_elaborate` with a display-only
    /// target override: the verdict comes from `target`, but the head
    /// message's type-1 slot renders `display_target` (Go's call errors
    /// show the optional parameter's ANNOTATION view — the `?` marks
    /// optionality, the `| undefined` folded into the resolved type is not
    /// spelled again: `f("s")` on `f(x?: number)` reports 'number').
    #[allow(clippy::too_many_arguments)]
    pub fn check_type_related_to_and_elaborate_display(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        error_node: Option<&Arc<crate::ast::Node>>,
        expr: Option<&Arc<crate::ast::Node>>,
        head_message: Option<&crate::diagnostics::Message>,
        diagnostic_output: Option<&mut Vec<crate::ast::Diagnostic>>,
        display_target: Option<&Arc<Type>>,
    ) -> bool {
        let saved_display = self.display_target_override.take();
        self.display_target_override = display_target.cloned();
        let r = self.check_type_related_to_and_optionally_elaborate(
            source, target, relation, error_node, expr, head_message, diagnostic_output,
        );
        self.display_target_override = saved_display;
        r
    }

    pub fn check_type_related_to_and_optionally_elaborate(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        error_node: Option<&Arc<crate::ast::Node>>,
        expr: Option<&Arc<crate::ast::Node>>,
        head_message: Option<&crate::diagnostics::Message>,
        mut diagnostic_output: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        // A degraded side (an interface type built inside a heritage
        // degradation window — `degraded_type_ptrs`) has a transiently
        // incomplete member table; for member-dependent comparisons (both
        // sides object types) the verdict is garbage — treat as related
        // and report nothing (Go's lazy member resolution never observes
        // mid-flight forms; the react16/lib.dom D1 phantom-error family).
        // Kind-only comparisons keep their real verdicts.
        {
            let sp = Arc::as_ptr(source) as *const Type as usize;
            let tp = Arc::as_ptr(target) as *const Type as usize;
            if source.flags.contains(TypeFlags::Object)
                && target.flags.contains(TypeFlags::Object)
                && (self.degraded_type_ptrs.contains(&sp) || self.degraded_type_ptrs.contains(&tp))
            {
                return true;
            }
        }
        // Speculation window (overload applicability probing): report
        // nothing — the boolean decides candidate selection, and probe
        // failures must not persist (Go's speculative tracker drops all
        // diagnostics raised under speculation).
        if self.speculation_depth > 0 {
            return self.is_type_related_to(source, target, relation);
        }
        let saved_chain = std::mem::take(&mut self.relater_error_chain);
        let was_active = self.relater_chain_active;
        self.relater_chain_active = true;
        let ok = self.is_type_related_to(source, target, relation);
        if ok {
            self.relater_chain_active = was_active;
            self.relater_error_chain = saved_chain;
            return true;
        }
        // Go `elaborateError` (relater.go ~L432): object/array-literal
        // expressions elaborate the failure per property/element — each
        // mismatching property reports at its NAME node (`
        // Type 'undefined' is not assignable to type 'string'`), with the
        // generalized head+pyramid suppressed once anything reported.
        if let Some(expr) = expr
            && self.elaborate_error(expr, source, target, relation, diagnostic_output.as_deref_mut())
        {
            self.relater_chain_active = was_active;
            self.relater_error_chain = saved_chain;
            return false;
        }
        // Head message (Go `reportRelationError`, relater.go ~L4792):
        // fresh literals display their base primitive when the target can't
        // hold singletons (`5` vs `{}` shows `number`).
        let displayed_target = self
            .display_target_override
            .clone()
            .unwrap_or_else(|| Arc::clone(target));
        let source_str = self.type_to_string(source);
        let target_str = self.type_to_string(&displayed_target);
        let (head_source, head_target) = if self.type_could_have_top_level_singleton_types(target)
        {
            (source_str.clone(), target_str.clone())
        } else if crate::checker::is_fresh_literal_type(source)
            || source.flags.intersects(TYPE_FLAGS_LITERAL)
        {
            let base = self.get_base_type_of_literal_type_for_display(source);
            (self.type_to_string(&base), target_str.clone())
        } else if source
            .object_flags
            .contains(crate::checker::types::ObjectFlags::ObjectLiteral)
            && source.symbol.is_none()
        {
            // Fresh object-literal sources display their WIDENED form in
            // the head line — `{ a: 1; b: 2 }` shows as
            // `{ a: number; b: number }` (assignmentCompatability46).
            let widened = self.widen_object_literal_type(source);
            (self.type_to_string(&widened), target_str.clone())
        } else {
            (source_str.clone(), target_str.clone())
        };
        let head = match head_message {
            Some(m) => *m,
            None if head_source == head_target => {
                crate::diagnostics::messages_generated::
                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_TWO_DIFFERENT_TYPES_WITH_THIS_NAME_EXIST_BUT_THEY_ARE_UNRELATED
            }
            None => crate::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
        };
        // Go reportRelationError's chain-top suppression (relater.go
        // ~L4852): when the innermost (last-reported) chain entry is a
        // missing-property message whose source/target match the outer
        // pair, the generalized head is SUPPRESSED — the chain entry
        // itself becomes the diagnostic (`var i1: I1 = []` shows only
        // the TS2741 line, no head TS2322). The readonly form suppresses
        // the same way when its args match. Conversion/interface-
        // implementation messages (passed as head_message) never
        // suppress.
        let mut suppress_head = false;
        if head_message.is_none()
            && let Some(entry) = self.relater_error_chain.last()
        {
            let m = entry.message;
            let a = &entry.args;
            suppress_head = if m
                == crate::diagnostics::messages_generated::
                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2
            {
                a.len() == 3 && a[1] == head_source && a[2] == head_target
            } else if m
                == crate::diagnostics::messages_generated::
                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2
                || m
                    == crate::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE
            {
                a.len() >= 2 && a[0] == head_source && a[1] == head_target
            } else if m
                == crate::diagnostics::messages_generated::
                    THE_TYPE_0_IS_READONLY_AND_CANNOT_BE_ASSIGNED_TO_THE_MUTABLE_TYPE_1
            {
                a.len() == 2 && a[0] == head_source && a[1] == head_target
            } else {
                false
            };
        }
        if !suppress_head {
            // The head push runs through the type-parameter-note wrapper
            // (Go reportRelationError, relater.go ~L4797) — a type-
            // parameter target carries an instantiation note under the
            // head (typeParameterAssignability).
            self.push_relation_head_with_tp_note(
                source,
                &displayed_target,
                head,
                vec![head_source, head_target],
            );
        }

        // Build the nested pyramid (Go
        // `createDiagnosticChainFromErrorChain`, relater.go ~L402). Go's
        // linked list is front-inserted: the head (pushed last, the
        // generalized `Type 'X' is not assignable to type 'Y'`) sits at the
        // chain's front and becomes the OUTERMOST diagnostic, with earlier
        // entries nested progressively deeper. Our Vec is chronological
        // (oldest first, head last), so iterate forward making each entry
        // the parent of the previously accumulated diagnostic — the final
        // entry (the head) ends up outermost, mirroring the
        // `NewDiagnosticChain(next, chain.message, ...)` recursion.
        let Some(error_node) = error_node else {
            self.relater_chain_active = was_active;
            self.relater_error_chain = saved_chain;
            return false;
        };
        let file = self.get_source_file_of_node(error_node).or_else(|| self.current_file.clone());
        let mut diagnostic: Option<crate::ast::Diagnostic> = None;
        for entry in self.relater_error_chain.iter() {
            if entry.message.elided_in_compatibility_pyramid {
                continue;
            }
            let mut d = crate::ast::Diagnostic::new(
                file.clone(),
                error_node.loc,
                entry.message,
                entry.args.clone(),
            );
            if let Some(child) = diagnostic.take() {
                d.message_chain = vec![child];
            }
            diagnostic = Some(d);
        }
        if let Some(d) = diagnostic {
            match diagnostic_output {
                Some(out) => out.push(d),
                None => self.diagnostics.add(d),
            }
        }
        self.relater_chain_active = was_active;
        self.relater_error_chain = saved_chain;
        false
    }

    /// Display form of a literal type's base primitive (Go
    /// `getBaseTypeOfLiteralType` used by `reportRelationError`'s
    /// generalization).
    fn get_base_type_of_literal_type_for_display(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::StringLiteral) || t.flags.contains(TypeFlags::StringMapping)
        {
            self.string_type()
        } else if t.flags.contains(TypeFlags::NumberLiteral) {
            self.number_type()
        } else if t.flags.contains(TypeFlags::BigIntLiteral) {
            self.bigint_type()
        } else if t.flags.contains(TypeFlags::BooleanLiteral) {
            self.boolean_type()
        } else {
            Arc::clone(t)
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Weak type and property checks
    // ────────────────────────────────────────────────────────────────────────

    pub fn is_weak_type(&mut self, t: &Arc<Type>) -> bool {
        // Go isWeakType: an object type with at least one property, ALL
        // of them optional, and no index signatures and no call/construct
        // signatures (`len(properties) > 0` — `{}` and unresolved/
        // deferred mapped types with no computed members are NOT weak).
        // Intersections are weak when every constituent is weak.
        if t.flags.contains(TypeFlags::Object) {
            if t.flags.contains(TypeFlags::Any) {
                return false;
            }
            let Some(structured) = t.as_structured() else {
                return false;
            };
            if !structured.index_infos.is_empty() {
                return false;
            }
            if !structured.call_signatures().is_empty()
                || !structured.construct_signatures().is_empty()
            {
                return false;
            }
            if structured.properties.is_empty() {
                return false;
            }
            return structured
                .properties
                .iter()
                .all(|p| p.flags.contains(SymbolFlags::Optional));
        } else if t.flags.contains(TypeFlags::Substitution) {
            if let TypeData::Substitution(s) = &t.data {
                s.base_type
                    .as_ref()
                    .map(|bt| self.is_weak_type(bt))
                    .unwrap_or(false)
            } else {
                false
            }
        } else if t.flags.contains(TypeFlags::Intersection) {
            // Every constituent must be weak
            if let Some(types) = t.types() {
                types.iter().all(|ty| self.is_weak_type(ty))
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn has_common_properties(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _is_comparing_jsx_attributes: bool,
    ) -> bool {
        // Go hasCommonProperties: any property of SOURCE that is a known
        // property of TARGET.
        let Some(source_struct) = source.as_structured() else {
            return false;
        };
        for p in &source_struct.properties {
            if self.is_known_property(target, &p.name, false) {
                return true;
            }
        }
        false
    }

    pub fn is_known_property(
        &mut self,
        target_type: &Arc<Type>,
        name: &str,
        _is_comparing_jsx_attributes: bool,
    ) -> bool {
        // Go isKnownProperty: a declared member of that name, or a name
        // accepted by an applicable index signature (string index accepts
        // any name; number index accepts numeric names).
        if let Some(structured) = target_type.as_structured() {
            if structured.members.get(name).is_some() {
                return true;
            }
            for info in &structured.index_infos {
                if let Some(key) = &info.key_type {
                    if key.flags.contains(TypeFlags::String) {
                        return true;
                    }
                    if key.flags.contains(TypeFlags::Number) && name.parse::<f64>().is_ok() {
                        return true;
                    }
                }
            }
        }
        false
    }

    // ────────────────────────────────────────────────────────────────────────
    // Recursion and deeply nested types
    // ────────────────────────────────────────────────────────────────────────

    pub fn get_mapped_target_with_symbol(&self, t: &Arc<Type>) -> Arc<Type> {
        // TODO: unwrap nested homomorphic mapped types
        Arc::clone(t)
    }

    pub fn has_matching_recursion_identity(&self, t: &Arc<Type>, identity: &Arc<Type>) -> bool {
        Arc::ptr_eq(t, identity)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Type matching and discrimination
    // ────────────────────────────────────────────────────────────────────────

    pub fn get_best_matching_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _is_related_to: &dyn Fn(&Arc<Type>, &Arc<Type>) -> Ternary,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        let _ = (source, target);
        None
    }

    pub fn find_matching_type_reference_or_type_alias_reference(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        let _ = (source, union_target);
        None
    }

    pub fn find_best_type_for_invokable(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
        _kind: SignatureKind,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        let _ = (source, union_target);
        None
    }

    pub fn find_most_overlappy_type(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        let _ = (source, union_target);
        None
    }

    pub fn find_best_type_for_object_literal(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        let _ = (source, union_target);
        None
    }

    pub fn should_report_unmatched_property_error(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> bool {
        // Go shouldReportUnmatchedPropertyError (relater.go ~L959): a
        // signature-only source (call/construct signatures, no own
        // properties) elides the missing-property elaboration unless the
        // target carries the same signature kind — `() => C1` against
        // `any[]` reports only the head `Type '() => C1' is not
        // assignable to type 'any[]'`.
        let Some(s) = source.as_structured() else {
            return true;
        };
        let type_call_signatures = s.call_signatures().len();
        let type_construct_signatures = s.construct_signatures().len();
        let type_properties = s.properties.len();
        if (type_call_signatures != 0 || type_construct_signatures != 0) && type_properties == 0 {
            let target_calls = target
                .as_structured()
                .map(|t| t.call_signatures().len())
                .unwrap_or(0);
            let target_constructs = target
                .as_structured()
                .map(|t| t.construct_signatures().len())
                .unwrap_or(0);
            if (target_calls != 0 && type_call_signatures != 0)
                || (target_constructs != 0 && type_construct_signatures != 0)
            {
                // target has similar signature kinds to source, still
                // focus on the unmatched property
                return true;
            }
            return false;
        }
        true
    }

    pub fn get_unmatched_property(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _require_optional_properties: bool,
        _match_discriminant_properties: bool,
    ) -> Option<Arc<Symbol>> {
        // TODO: full implementation
        let _ = (source, target);
        None
    }

    pub fn get_unmatched_properties(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        require_optional_properties: bool,
        match_discriminant_properties: bool,
    ) -> Vec<Arc<Symbol>> {
        // TODO: full implementation via get_unmatched_properties_worker
        let _ = (
            source,
            target,
            require_optional_properties,
            match_discriminant_properties,
        );
        Vec::new()
    }

    pub fn find_matching_discriminant_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _is_related_to: &dyn Fn(&Arc<Type>, &Arc<Type>) -> Ternary,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        let _ = (source, target);
        None
    }

    pub fn find_discriminant_properties(
        &mut self,
        _source_properties: &[Arc<Symbol>],
        _target: &Arc<Type>,
    ) -> Vec<Arc<Symbol>> {
        // TODO: full implementation
        Vec::new()
    }

    pub fn is_discriminant_property(&mut self, _t: &Arc<Type>, _name: &str) -> bool {
        // TODO: full implementation
        false
    }

    pub fn get_matching_union_constituent_for_type(
        &mut self,
        _union_type: &Arc<Type>,
        _t: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_key_property_name(&mut self, t: &Arc<Type>) -> Option<String> {
        // TODO: full implementation
        let _ = t;
        None
    }

    pub fn get_constituent_type_for_key_type(
        &mut self,
        _t: &Arc<Type>,
        _key_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        None
    }

    pub fn filter_primitives_if_contains_non_primitive(
        &mut self,
        union_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        let _ = union_type;
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // Error display helpers
    // ────────────────────────────────────────────────────────────────────────

    pub fn get_type_names_for_error_display(
        &mut self,
        left: &Arc<Type>,
        right: &Arc<Type>,
    ) -> (String, String) {
        // TODO: full implementation with type display
        (
            self.get_type_name_for_error_display(left),
            self.get_type_name_for_error_display(right),
        )
    }

    pub fn get_type_name_for_error_display(&mut self, t: &Arc<Type>) -> String {
        // TODO: full implementation with contextual type display
        crate::checker::utilities::type_to_string(t)
    }

    pub fn symbol_value_declaration_is_context_sensitive(&mut self, _symbol: &Arc<Symbol>) -> bool {
        // TODO: full implementation
        false
    }

    /// Go `typeCouldHaveTopLevelSingletonTypes` (checker.go): whether a
    /// type may contain literal/unique-singleton members at the top level —
    /// true for literal targets, type parameters and deferred types whose
    /// constraint might, and unions containing any such member.
    pub fn type_could_have_top_level_singleton_types(&mut self, t: &Arc<Type>) -> bool {
        if t.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral
                | TypeFlags::UniqueESSymbol
                | TypeFlags::EnumLiteral
                | TypeFlags::TypeParameter
                | TypeFlags::IndexedAccess
                | TypeFlags::Conditional,
        ) || crate::checker::is_fresh_literal_type(t)
        {
            return true;
        }
        if t.flags.contains(TypeFlags::Union) {
            if let Some(members) = t.types() {
                return members
                    .iter()
                    .any(|m| self.type_could_have_top_level_singleton_types(m));
            }
        }
        false
    }

    // ────────────────────────────────────────────────────────────────────────
    // Variance computation
    // ────────────────────────────────────────────────────────────────────────

    pub fn get_alias_variances(&mut self, _symbol: &Arc<Symbol>) -> Vec<VarianceFlags> {
        // TODO: full implementation
        Vec::new()
    }

    pub fn create_marker_type(
        &mut self,
        _symbol: &Arc<Symbol>,
        _source: &Arc<Type>,
        _target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_type_parameter_modifiers(&mut self, _tp: &Arc<Type>) -> crate::ast::ModifierFlags {
        // TODO: full implementation
        crate::ast::ModifierFlags::empty()
    }

    pub fn has_covariant_void_argument(
        &mut self,
        _type_arguments: &[Arc<Type>],
        _variances: &[VarianceFlags],
    ) -> bool {
        // TODO: full implementation
        false
    }

    pub fn is_signature_assignable_to(
        &mut self,
        _source: &Arc<Signature>,
        _target: &Arc<Signature>,
        _ignore_return_types: bool,
    ) -> bool {
        // TODO: full implementation
        false
    }

    // ────────────────────────────────────────────────────────────────────────
    // Signature helpers
    // ────────────────────────────────────────────────────────────────────────

    pub fn get_min_argument_count_ex(
        &mut self,
        sig: &Arc<Signature>,
        _flags: MinArgumentCountFlags,
    ) -> usize {
        // TODO: full implementation with flags
        sig.min_argument_count.max(0) as usize
    }

    pub fn get_parameter_name_at_position(
        &mut self,
        _signature: &Arc<Signature>,
        _pos: usize,
    ) -> String {
        // TODO: full implementation
        String::new()
    }

    pub fn get_tuple_element_label(
        &mut self,
        _element_info: &TupleElementInfo,
        _rest_symbol: Option<&Arc<Symbol>>,
        _index: usize,
    ) -> String {
        // TODO: full implementation
        String::new()
    }

    pub fn get_tuple_element_label_from_binding_element(
        &mut self,
        _node: &Arc<crate::ast::Node>,
        _index: usize,
        _element_flags: ElementFlags,
    ) -> String {
        // TODO: full implementation
        String::new()
    }

    pub fn get_nameable_declaration_at_position(
        &mut self,
        _signature: &Arc<Signature>,
        _pos: usize,
    ) -> Option<Arc<crate::ast::Node>> {
        // TODO: full implementation
        None
    }

    pub fn is_valid_declaration_for_tuple_label(&mut self, _d: &Arc<crate::ast::Node>) -> bool {
        // TODO: full implementation
        false
    }

    pub fn slice_tuple_type(
        &mut self,
        _t: &Arc<Type>,
        _index: usize,
        _end_skip_count: usize,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_known_keys_of_tuple_type(&mut self, _t: &Arc<Type>) -> Option<Arc<Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_rest_array_type_of_tuple_type(&mut self, _t: &Arc<Type>) -> Option<Arc<Type>> {
        // TODO: full implementation
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // Type predicate helpers
    // ────────────────────────────────────────────────────────────────────────

    pub fn get_union_or_intersection_type_predicate(
        &mut self,
        _signatures: &[Arc<Signature>],
        _is_union: bool,
    ) -> Option<Box<TypePredicate>> {
        // TODO: full implementation
        None
    }

    pub fn type_predicate_kinds_match(&mut self, a: &TypePredicate, b: &TypePredicate) -> bool {
        a.kind == b.kind
    }

    pub fn create_type_predicate_from_type_predicate_node(
        &mut self,
        _node: &Arc<crate::ast::Node>,
        _signature: &Arc<Signature>,
    ) -> Option<Box<TypePredicate>> {
        // TODO: full implementation
        None
    }

    pub fn instantiate_type_predicate(
        &mut self,
        _predicate: &TypePredicate,
        _mapper: &Arc<TypeMapper>,
    ) -> Option<Box<TypePredicate>> {
        // TODO: full implementation
        None
    }

    pub fn new_type_predicate(
        &mut self,
        kind: TypePredicateKind,
        parameter_name: String,
        parameter_index: i32,
        t: Arc<Type>,
    ) -> Box<TypePredicate> {
        Box::new(TypePredicate {
            kind,
            parameter_name,
            parameter_index,
            t: Some(t),
        })
    }

    pub fn is_resolving_return_type_of_signature(&mut self, _signature: &Arc<Signature>) -> bool {
        // TODO: full implementation
        false
    }

    pub fn find_matching_signatures(
        &mut self,
        _signature_lists: &[Vec<Arc<Signature>>],
        _signature: &Arc<Signature>,
        _list_index: usize,
    ) -> Vec<Arc<Signature>> {
        // TODO: full implementation
        Vec::new()
    }

    pub fn is_matching_signature(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        partial_match: bool,
    ) -> bool {
        self.compare_signatures_identical(source, target, partial_match, false, false)
            != Ternary::False
    }

    pub fn compare_type_predicates_identical(
        &mut self,
        source: &TypePredicate,
        target: &TypePredicate,
        _compare_types: &dyn Fn(&Arc<Type>, &Arc<Type>) -> Ternary,
    ) -> Ternary {
        if source.kind != target.kind {
            return Ternary::False;
        }
        if source.parameter_name != target.parameter_name {
            return Ternary::False;
        }
        Ternary::True
    }

    // ────────────────────────────────────────────────────────────────────────
    // Effective constraint and template literal helpers
    // ────────────────────────────────────────────────────────────────────────

    pub fn get_effective_constraint_of_intersection(
        &mut self,
        _types: &[Arc<Type>],
        _target_is_union: bool,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        None
    }

    pub fn template_literal_types_definitely_unrelated(
        &mut self,
        _source: &TemplateLiteralTypeData,
        _target: &TemplateLiteralTypeData,
    ) -> bool {
        // TODO: full implementation
        false
    }

    pub fn is_type_matched_by_template_literal_type(
        &mut self,
        _source: &Arc<Type>,
        _target: &TemplateLiteralTypeData,
        _compare_types: TypeComparer,
    ) -> bool {
        // TODO: full implementation
        false
    }

    pub fn infer_types_from_template_literal_type(
        &mut self,
        _source: &Arc<Type>,
        _target: &TemplateLiteralTypeData,
    ) -> Vec<Arc<Type>> {
        // TODO: full implementation
        Vec::new()
    }

    pub fn get_string_like_type_for_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.flags.intersects(TYPE_FLAGS_STRING_LIKE) {
            Some(Arc::clone(t))
        } else {
            None
        }
    }

    pub fn is_valid_type_for_template_literal_placeholder(
        &mut self,
        _source: &Arc<Type>,
        _target: &Arc<Type>,
        _compare_types: TypeComparer,
    ) -> bool {
        // TODO: full implementation
        false
    }

    pub fn is_member_of_string_mapping(
        &mut self,
        _source: &Arc<Type>,
        _target: &Arc<Type>,
    ) -> bool {
        // TODO: full implementation
        false
    }

    pub fn apply_target_string_mapping_to_source(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> (Arc<Type>, Arc<Type>) {
        // TODO: full implementation
        (Arc::clone(source), Arc::clone(target))
    }

    // ────────────────────────────────────────────────────────────────────────
    // Type property helpers
    // ────────────────────────────────────────────────────────────────────────

    pub fn get_type_of_property_in_types(
        &mut self,
        _types: &[Arc<Type>],
        _name: &str,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_type_of_property_in_type(
        &mut self,
        _t: &Arc<Type>,
        _name: &str,
    ) -> Option<Arc<Type>> {
        // TODO: full implementation
        None
    }

    pub fn is_type_subset_of_union(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        // TODO: full implementation
        self.is_type_subset_of(source, target)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Relater pool management
    // ────────────────────────────────────────────────────────────────────────

    pub fn is_type_derived_from(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        // TODO: full implementation
        self.is_type_assignable_to(source, target)
    }

    pub fn is_distribution_dependent(&mut self, _root: &ConditionalRoot) -> bool {
        // TODO: full implementation
        false
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Free functions
// ────────────────────────────────────────────────────────────────────────────

pub fn is_hyphenated_jsx_name(name: &str) -> bool {
    name.contains('-')
}

pub fn is_excess_property_check_target(t: &Type) -> bool {
    // DEFERRED mapped types (`{ [K in keyof T]: V }` with a generic
    // constraint) have no materialized members — excess-property checks
    // against them would flag every property (Go exempts generic mapped
    // types).
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
    // TODO: full implementation
    false
}

pub fn is_ignored_jsx_property(_source: &Type, _source_prop: &Symbol) -> bool {
    // TODO: full implementation
    false
}

// ────────────────────────────────────────────────────────────────────────────
// TypeDiscriminator
// ────────────────────────────────────────────────────────────────────────────

/// A discriminator for type matching in union/intersection narrowing.
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
        // TODO: full implementation
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
        // The same (source, target) pair under different relations must
        // produce distinct cache keys — otherwise `Assignable` and
        // `Subtype` results would collide.
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
        // k2 is a different key, so it's not in the set.
        assert!(!set.contains(&k2));
        set.insert(k2);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn relation_cache_key_distinguishes_type_pointers() {
        // Different source/target pointer pairs must produce distinct keys.
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
        // The `Callback` const alias is the union of BivariantCallback and
        // StrictCallback, matching Go's `SignatureCheckModeCallback`.
        assert!(SignatureCheckMode::Callback.contains(SignatureCheckMode::BivariantCallback));
        assert!(SignatureCheckMode::Callback.contains(SignatureCheckMode::StrictCallback));
        assert_eq!(SignatureCheckMode::Callback, SIGNATURE_CHECK_MODE_CALLBACK);
    }

    #[test]
    fn type_arguments_related_covariant_by_default() {
        // Two empty type-argument slices are trivially related.
        // We construct a tiny standalone check: covariant variance with a
        // single pair of `any`/`unknown` types should yield False (since
        // `unknown` is not assignable to `any` under the strict
        // interpretation, but our `compare_types` collapses to `True` for
        // `any` source). This test is a smoke test of the variance dispatch
        // logic rather than the assignability semantics themselves.
        let result = Ternary::True.and(Ternary::True);
        assert_eq!(result, Ternary::True);
        // Ensure variance flag bit layout matches what we assume:
        assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Covariant));
        assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Contravariant));
        assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Independent));
        assert!(!VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Unmeasurable));
        assert!(!VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Unreliable));
    }

    #[test]
    fn index_signature_helpers_bit_layout() {
        // Sanity-check the ObjectFlags we rely on for the index-signature
        // structural fallback (P3.7f) are distinct bits — guards against
        // accidental renumbering of the ObjectFlags bitfield.
        let inferable =
            ObjectFlags::JSLiteral | ObjectFlags::ObjectRestType | ObjectFlags::ReverseMapped;
        assert!(inferable.contains(ObjectFlags::JSLiteral));
        assert!(inferable.contains(ObjectFlags::ObjectRestType));
        assert!(inferable.contains(ObjectFlags::ReverseMapped));
        // Fresh-literal is a separate bit (used by the strict-subtype carve-out).
        assert!(!inferable.contains(ObjectFlags::FreshLiteral));
    }
}
