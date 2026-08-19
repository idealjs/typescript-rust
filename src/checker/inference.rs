//! Type inference engine.
//!
//! Ported from `internal/checker/inference.go`. This module implements
//! generic type inference for TypeScript's type checker. It handles
//! inferring type arguments from function calls, contextual typing,
//! return type inference, and mapped type inference.

use std::collections::HashMap;
use std::sync::Arc;

use super::checker::Checker;
use super::types::*;
use crate::ast::SyntaxKind;

// ────────────────────────────────────────────────────────────────────────────
// InferenceKey
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InferenceKey {
    pub source: TypeId,
    pub target: TypeId,
}

// ────────────────────────────────────────────────────────────────────────────
// InferencePriority
// ────────────────────────────────────────────────────────────────────────────

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

// ────────────────────────────────────────────────────────────────────────────
// InferenceFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct InferenceFlags: u32 {
        const None                   = 0;
        const NoDefault              = 1 << 0;
        const AnyDefault             = 1 << 1;
        const SkippedGenericFunction = 1 << 2;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ExpandingFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ExpandingFlags: u8 {
        const None   = 0;
        const Source = 1 << 0;
        const Target = 1 << 1;
        const Both   = Self::Source.bits() | Self::Target.bits();
    }
}

// ────────────────────────────────────────────────────────────────────────────
// InferenceInfo
// ────────────────────────────────────────────────────────────────────────────

/// Tracks inference candidates for a single type parameter.
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

// ────────────────────────────────────────────────────────────────────────────
// InferenceContext
// ────────────────────────────────────────────────────────────────────────────

/// The context for type inference of a generic function call.
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

// ────────────────────────────────────────────────────────────────────────────
// InferenceState
// ────────────────────────────────────────────────────────────────────────────

/// Recursive inference state used during `inferFromTypes`.
struct InferenceState<'a> {
    inferences: &'a mut [InferenceInfo],
    original_source: Option<Arc<Type>>,
    original_target: Option<Arc<Type>>,
    priority: InferencePriority,
    inference_priority: InferencePriority,
    contravariant: bool,
    bivariant: bool,
    expanding_flags: ExpandingFlags,
    propagation_type: Option<Arc<Type>>,
    /// Source/target pointer pairs currently being inferred (cycle guard
    /// for `infer_from_types`; see the comment there).
    visited: HashMap<(usize, usize), InferencePriority>,
    depth: i32,
}

// ────────────────────────────────────────────────────────────────────────────
// Checker inference methods
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Entry point for type inference. Infers type arguments for a generic
    /// function call by matching source types against target types.
    pub fn infer_types(
        &mut self,
        inferences: &mut [InferenceInfo],
        original_source: Option<Arc<Type>>,
        original_target: Option<Arc<Type>>,
        priority: InferencePriority,
        contravariant: bool,
    ) {
        let mut state = InferenceState {
            inferences,
            original_source: original_source.clone(),
            original_target: original_target.clone(),
            priority,
            inference_priority: InferencePriority::MaxValue,
            contravariant,
            bivariant: false,
            expanding_flags: ExpandingFlags::None,
            propagation_type: None,
            visited: HashMap::new(),
            depth: 0,
        };
        if let (Some(source), Some(target)) = (original_source, original_target) {
            self.infer_from_types(&mut state, &source, &target);
        }
    }

    /// Main recursive inference function. Matches source type against target
    /// type and records candidates for type parameters found in target.
    fn infer_from_types(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        // Quick bail-out: if target cannot contain type variables, nothing to infer
        if !self.could_contain_type_variables(target) || self.is_no_infer_type(target) {
            return;
        }
        // Cycle guard (the `visited` map): a source/target pair re-entered
        // while already being inferred higher up the stack carries no new
        // information — without this, self-referential signatures
        // (`ReadonlyArray<T>.flatMap`'s callback parameter is
        // `ReadonlyArray<T>` again) recurse infer_types → signatures →
        // properties → infer_types without bound (stack overflow).
        let key = (Arc::as_ptr(source) as usize, Arc::as_ptr(target) as usize);
        if state.visited.contains_key(&key) {
            return;
        }
        state.visited.insert(key, state.priority);
        self.infer_from_types_inner(state, source, target);
        state.visited.remove(&key);
    }

    fn infer_from_types_inner(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        // Handle union types in target
        if target.flags.contains(TypeFlags::Union) {
            let source_types = if source.flags.contains(TypeFlags::Union) {
                source.types().unwrap_or_default().to_vec()
            } else {
                vec![Arc::clone(source)]
            };
            let target_types = target.types().unwrap_or_default().to_vec();
            let (temp_sources, temp_targets) =
                self.infer_from_matching_types(state, &source_types, &target_types, true);
            if temp_targets.is_empty() {
                return;
            }
            let target = self.get_union_type(temp_targets);
            if temp_sources.is_empty() {
                self.infer_with_priority(
                    state,
                    source,
                    &target,
                    InferencePriority::NakedTypeVariable,
                );
                return;
            }
            let source = self.get_union_type(temp_sources);
            self.infer_from_types_union(state, &source, &target);
            return;
        }

        // Handle intersection types in target
        if target.flags.contains(TypeFlags::Intersection) {
            self.infer_from_types_intersection(state, source, target);
            return;
        }

        // Handle type variable targets (type parameters)
        if target.flags.contains(TypeFlags::TypeParameter) {
            self.infer_to_type_variable(state, source, target);
            return;
        }

        // Handle object types (including arrays, tuples, etc.)
        if target.flags.contains(TypeFlags::Object) {
            self.infer_from_object_types(state, source, target);
            return;
        }
    }

    fn infer_from_types_union(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_types = if source.flags.contains(TypeFlags::Union) {
            source.types().unwrap_or_default().to_vec()
        } else {
            vec![Arc::clone(source)]
        };
        let target_types = target.types().unwrap_or_default().to_vec();
        let (sources, targets) =
            self.infer_from_matching_types(state, &source_types, &target_types, false);
        if targets.is_empty() {
            return;
        }
        let target = self.get_union_type(targets);
        if sources.is_empty() {
            self.infer_with_priority(state, source, &target, InferencePriority::NakedTypeVariable);
            return;
        }
        let source = self.get_union_type(sources);
        for t in target.types().unwrap_or(&[]) {
            self.infer_from_types(state, &source, t);
        }
    }

    fn infer_from_types_intersection(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_types = if source.flags.contains(TypeFlags::Intersection) {
            source.types().unwrap_or_default().to_vec()
        } else {
            vec![Arc::clone(source)]
        };
        let target_types = target.types().unwrap_or_default().to_vec();
        let (sources, targets) =
            self.infer_matching_types_identical(state, &source_types, &target_types);
        if sources.is_empty() || targets.is_empty() {
            return;
        }
        let source = self.get_intersection_type(sources);
        let target = self.get_intersection_type(targets);
        for t in target.types().unwrap_or(&[]) {
            self.infer_from_types(state, &source, t);
        }
    }

    fn infer_to_type_variable(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        if self.is_from_inference_blocked_source(source) {
            return;
        }
        // Find the inference index first to avoid borrow conflicts
        let inference_idx = state
            .inferences
            .iter()
            .position(|info| info.type_parameter.id == target.id);
        let Some(idx) = inference_idx else { return };

        // Capture state values before mutable borrow
        let priority = state.priority;
        let contravariant = state.contravariant;
        let bivariant = state.bivariant;
        let depth = state.depth;
        let propagation_type = state.propagation_type.clone();

        let mut cleared = false;
        let inference = &mut state.inferences[idx];
        if source.object_flags.contains(ObjectFlags::NonInferrableType) {
            return;
        }
        if !inference.is_fixed {
            let candidate = propagation_type.unwrap_or_else(|| Arc::clone(source));
            if priority.bits() < inference.priority.bits() {
                inference.candidates.clear();
                inference.candidate_depths.clear();
                inference.contra_candidates.clear();
                inference.top_level = true;
                inference.priority = priority;
                cleared = true;
            }
            if priority == inference.priority {
                if contravariant && !bivariant {
                    if !inference
                        .contra_candidates
                        .iter()
                        .any(|c| Arc::ptr_eq(c, &candidate))
                    {
                        inference.contra_candidates.push(candidate);
                        cleared = true;
                    }
                } else {
                    if !inference
                        .candidates
                        .iter()
                        .any(|c| Arc::ptr_eq(c, &candidate))
                    {
                        inference.candidates.push(candidate);
                        inference.candidate_depths.push(depth);
                        cleared = true;
                    }
                }
            }
            if !priority.contains(InferencePriority::ReturnType)
                && target.flags.contains(TypeFlags::TypeParameter)
                && inference.top_level
            {
                inference.top_level = false;
                cleared = true;
            }
        }
        drop(inference);
        if cleared {
            for info in state.inferences.iter_mut() {
                info.inferred_type = None;
            }
        }
        state.inference_priority = if state.inference_priority.bits() < state.priority.bits() {
            state.inference_priority
        } else {
            state.priority
        };
    }

    fn infer_from_object_types(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);
        if !source_args.is_empty() && !target_args.is_empty() {
            self.infer_from_type_arguments(state, &source_args, &target_args, &[]);
        }
        self.infer_from_properties(state, source, target);
        self.infer_from_signatures(state, source, target);
        self.infer_from_index_types(state, source, target);
    }

    fn infer_from_type_arguments(
        &mut self,
        state: &mut InferenceState,
        source_types: &[Arc<Type>],
        target_types: &[Arc<Type>],
        _variances: &[VarianceFlags],
    ) {
        let count = source_types.len().min(target_types.len());
        for i in 0..count {
            self.infer_from_types(state, &source_types[i], &target_types[i]);
        }
    }

    fn infer_from_properties(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_struct = source.as_structured();
        let target_struct = target.as_structured();
        if let (Some(source_s), Some(target_s)) = (source_struct, target_struct) {
            for target_prop in &target_s.properties {
                for source_prop in &source_s.properties {
                    if source_prop.name == target_prop.name {
                        let source_type = self.get_type_of_symbol(source_prop);
                        let target_type = self.get_type_of_symbol(target_prop);
                        self.infer_from_types(state, &source_type, &target_type);
                        break;
                    }
                }
            }
        }
    }

    fn infer_from_signatures(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_sigs = self.get_signatures_of_type(source, SignatureKind::Call);
        let target_sigs = self.get_signatures_of_type(target, SignatureKind::Call);
        if source_sigs.len() == 1 && target_sigs.len() == 1 {
            self.infer_from_signature(state, &source_sigs[0], &target_sigs[0]);
        }
        let source_ctors = self.get_signatures_of_type(source, SignatureKind::Construct);
        let target_ctors = self.get_signatures_of_type(target, SignatureKind::Construct);
        if source_ctors.len() == 1 && target_ctors.len() == 1 {
            self.infer_from_signature(state, &source_ctors[0], &target_ctors[0]);
        }
    }

    fn infer_from_signature(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
    ) {
        let param_count = source.parameters.len().min(target.parameters.len());
        for i in 0..param_count {
            let source_param = &source.parameters[i];
            let target_param = &target.parameters[i];
            let st = self.get_type_of_symbol(source_param);
            let tt = self.get_type_of_symbol(target_param);
            let save_contra = state.contravariant;
            let save_biv = state.bivariant;
            state.contravariant = true;
            state.bivariant = false;
            self.infer_from_types(state, &tt, &st);
            state.contravariant = save_contra;
            state.bivariant = save_biv;
        }
        if let (Some(st), Some(tt)) = (
            source.resolved_return_type.get().cloned(),
            target.resolved_return_type.get().cloned(),
        ) {
            self.infer_from_types(state, &st, &tt);
        }
    }

    fn infer_from_index_types(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_struct = source.as_structured();
        let target_struct = target.as_structured();
        if let (Some(source_s), Some(target_s)) = (source_struct, target_struct) {
            for target_index in &target_s.index_infos {
                for source_index in &source_s.index_infos {
                    let key_match = match (&target_index.key_type, &source_index.key_type) {
                        (Some(tk), Some(sk)) => self.is_type_identical_to(sk, tk),
                        _ => true,
                    };
                    if key_match {
                        if let (Some(tv), Some(sv)) =
                            (&target_index.value_type, &source_index.value_type)
                        {
                            self.infer_from_types(state, sv, tv);
                        }
                    }
                }
            }
        }
    }

    fn infer_from_matching_types(
        &mut self,
        state: &mut InferenceState,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
        use_identical: bool,
    ) -> (Vec<Arc<Type>>, Vec<Arc<Type>>) {
        let mut remaining_sources: Vec<Arc<Type>> = sources.to_vec();
        let mut remaining_targets: Vec<Arc<Type>> = targets.to_vec();
        let mut i = 0;
        while i < remaining_sources.len() {
            let mut matched = false;
            let mut j = 0;
            while j < remaining_targets.len() {
                let is_match = if use_identical {
                    self.is_type_identical_to(&remaining_sources[i], &remaining_targets[j])
                } else {
                    self.is_type_identical_to(&remaining_sources[i], &remaining_targets[j])
                };
                if is_match {
                    self.infer_from_types(state, &remaining_sources[i], &remaining_targets[j]);
                    remaining_sources.remove(i);
                    remaining_targets.remove(j);
                    matched = true;
                    break;
                }
                j += 1;
            }
            if !matched {
                i += 1;
            }
        }
        (remaining_sources, remaining_targets)
    }

    fn infer_matching_types_identical(
        &mut self,
        state: &mut InferenceState,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
    ) -> (Vec<Arc<Type>>, Vec<Arc<Type>>) {
        self.infer_from_matching_types(state, sources, targets, true)
    }

    fn infer_with_priority(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
        new_priority: InferencePriority,
    ) {
        let save = state.priority;
        state.priority = new_priority;
        self.infer_from_types(state, source, target);
        state.priority = save;
    }

    fn could_contain_type_variables(&self, t: &Type) -> bool {
        t.flags.intersects(TypeFlags::Union)
            || t.flags.intersects(TypeFlags::Intersection)
            || t.flags.intersects(TypeFlags::Object)
            || t.flags.intersects(TypeFlags::TypeParameter)
            || t.flags.intersects(TypeFlags::IndexedAccess)
            || t.flags.intersects(TypeFlags::Conditional)
            || t.flags.intersects(TypeFlags::Substitution)
    }
    fn is_no_infer_type(&self, _t: &Type) -> bool {
        false
    }

    fn is_from_inference_blocked_source(&self, _source: &Type) -> bool {
        false
    }

    /// Infer type arguments for a generic function call.
    /// Go: `inferTypeArguments` (checker.go:9366)
    pub fn infer_type_arguments(
        &mut self,
        node: &crate::ast::Node,
        signature: &Arc<Signature>,
        args: &[Arc<crate::ast::Node>],
        context: &mut InferenceContext,
    ) -> Vec<Arc<Type>> {
        // Contextual typing from return type.
        //
        // If the call expression has a contextual type (e.g., from a variable
        // annotation), infer from that contextual type to the return type of
        // the signature. For example, given:
        //   declare function wrap<T, U>(cb: (x: T) => U): (x: T) => U;
        //   let f: (x: string) => number = wrap(s => s.length);
        // we infer T=string, U=number from the declared type of `f`.
        //
        // This ports the non-JSX, non-decorator branch of Go's
        // `inferTypeArguments`. The full Go implementation also handles
        // binding-pattern inference and outer inference context cloning;
        // those require additional infrastructure and are deferred.
        if matches!(
            node.kind,
            SyntaxKind::CallExpression | SyntaxKind::NewExpression
        ) {
            // Only call/new expressions have a contextual return type path.
            // (Go also handles binary expressions and decorators elsewhere.)
            //
            // We check the parent chain directly to find a contextual type
            // (e.g., a variable declaration with a type annotation). The full
            // Go path uses `getContextualType`; here we inline a subset that
            // covers the common `let x: T = expr(...)` pattern.
            if let Some(contextual_type) = self.get_contextual_type_for_call_or_new(node) {
                if let Some(return_type) = self.get_return_type_of_signature(signature) {
                    if self.could_contain_type_variables(&return_type) {
                        self.infer_types(
                            &mut context.inferences,
                            Some(contextual_type),
                            Some(return_type),
                            InferencePriority::ReturnType,
                            false,
                        );
                    }
                }
            }
        }

        // Infer types from each argument against the parameter types.
        // Rest position mirrors Go's `getTypeAtPosition`: arguments at/after
        // the rest parameter infer against the rest ELEMENT type (one array
        // level stripped), so `new Array('hi', 'bye')` infers `T = string`
        // from `...items: T[]` instead of failing `string → T[]`.
        let has_rest = signature.has_rest_parameter();
        let rest_index = if has_rest {
            signature.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        for i in 0..args.len() {
            let param_type = if has_rest && i >= rest_index {
                let rest_type = self.get_type_of_symbol(&signature.parameters[rest_index]);
                self.get_array_element_type(&rest_type)
            } else if i < signature.parameters.len() {
                self.get_type_of_symbol(&signature.parameters[i])
            } else {
                // Beyond declared params with no rest — arity check owns it.
                continue;
            };
            if self.could_contain_type_variables(&param_type) {
                let arg_type = self.get_type_of_node(&args[i]);
                self.infer_types(
                    &mut context.inferences,
                    Some(arg_type),
                    Some(param_type),
                    InferencePriority::None,
                    false,
                );
            }
        }

        // Resolve inferred types
        self.get_inferred_types(context)
    }

    /// Resolve all inferred types from an inference context.
    /// Go: `getInferredTypes` (inference.go:1372)
    pub fn get_inferred_types(&mut self, context: &InferenceContext) -> Vec<Arc<Type>> {
        let count = context.inferences.len();
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            result.push(self.get_inferred_type(context, i));
        }
        result
    }

    /// Resolve a single inferred type from an inference context.
    /// Go: `getInferredType` (inference.go:1283)
    pub fn get_inferred_type(&mut self, context: &InferenceContext, index: usize) -> Arc<Type> {
        let inference = &context.inferences[index];
        if let Some(ref inferred) = inference.inferred_type {
            return Arc::clone(inferred);
        }

        // If the type parameter is error type, return it directly
        if inference.type_parameter.flags.contains(TypeFlags::Any)
            && inference.type_parameter.intrinsic_name() == Some("error")
        {
            return Arc::clone(&inference.type_parameter);
        }

        let mut inferred_type: Option<Arc<Type>> = None;
        let mut fallback_type: Option<Arc<Type>> = None;

        if let Some(ref signature) = context.signature {
            // Try covariant inference from candidates
            let inferred_covariant = if !inference.candidates.is_empty() {
                self.get_covariant_inference(inference, signature)
            } else {
                None
            };

            // Try contravariant inference from contra-candidates
            let inferred_contravariant = if !inference.contra_candidates.is_empty() {
                self.get_contravariant_inference(inference)
            } else {
                None
            };

            if inferred_covariant.is_some() || inferred_contravariant.is_some() {
                // Prefer covariant if it's not never, it's assignable to some contravariant candidate,
                // and no other type parameter is constrained to this one with conflicts.
                let prefer_covariant = match (&inferred_covariant, &inferred_contravariant) {
                    (Some(cov), None) => true,
                    (None, Some(_)) => false,
                    (Some(cov), Some(contra)) => {
                        let cov_not_never_or_any =
                            !cov.flags.intersects(TypeFlags::Never | TypeFlags::Any);
                        let cov_assignable_to_contra = inference
                            .contra_candidates
                            .iter()
                            .any(|t| self.is_type_assignable_to(cov, t));
                        let no_conflicting_constraints = context.inferences.iter().all(|other| {
                            let other_tp = &other.type_parameter;
                            let constraint = self.get_constraint_of_type_parameter(other_tp);
                            let is_constrained = constraint.as_ref().map_or(false, |c| {
                                if let Some(c_cons) = c.as_union_or_intersection() {
                                    c_cons
                                        .types
                                        .iter()
                                        .any(|ct| ct.id == inference.type_parameter.id)
                                } else {
                                    false
                                }
                            });
                            !is_constrained
                                || other
                                    .candidates
                                    .iter()
                                    .all(|t| self.is_type_assignable_to(t, cov))
                        });
                        cov_not_never_or_any
                            && cov_assignable_to_contra
                            && no_conflicting_constraints
                    }
                    (None, None) => false,
                };

                if prefer_covariant {
                    inferred_type = inferred_covariant;
                    fallback_type = inferred_contravariant;
                } else {
                    inferred_type = inferred_contravariant;
                    fallback_type = inferred_covariant;
                }
            } else if context.flags.contains(InferenceFlags::NoDefault) {
                inferred_type = Some(self.never_type());
            } else {
                let default_type = self.get_default_from_type_parameter(&inference.type_parameter);
                if let Some(dt) = default_type {
                    inferred_type = Some(dt);
                }
            }
        } else {
            // No signature: use union of candidates or intersection of contra-candidates
            inferred_type = self.get_type_from_inference(inference);
        }

        // Apply constraint checking
        if inferred_type.is_some() {
            let constraint = self.get_constraint_of_type_parameter(&inference.type_parameter);
            if let Some(constraint) = constraint {
                if !self.is_type_assignable_to(inferred_type.as_ref().unwrap(), &constraint) {
                    if inference.priority.contains(InferencePriority::ReturnType) {
                        // For pure return type inference, filter constituents
                        let inferred = inferred_type.as_ref().unwrap();
                        let filtered = if inferred.flags.contains(TypeFlags::Union) {
                            if let Some(types) = inferred.types() {
                                let filtered: Vec<Arc<Type>> = types
                                    .iter()
                                    .filter(|u| self.is_type_assignable_to(u, &constraint))
                                    .cloned()
                                    .collect();
                                if filtered.is_empty() {
                                    self.never_type()
                                } else if filtered.len() == 1 {
                                    filtered[0].clone()
                                } else {
                                    self.get_union_type(filtered)
                                }
                            } else {
                                self.never_type()
                            }
                        } else if self.is_type_assignable_to(inferred, &constraint) {
                            (*inferred).clone()
                        } else {
                            self.never_type()
                        };
                        if filtered.flags.contains(TypeFlags::Never) {
                            inferred_type = None;
                        } else {
                            inferred_type = Some(filtered);
                        }
                    } else {
                        inferred_type = None;
                    }
                }
            }
        }

        // Final fallback
        if inferred_type.is_none() {
            if let Some(fallback) = fallback_type {
                let constraint = self.get_constraint_of_type_parameter(&inference.type_parameter);
                if let Some(constraint) = constraint {
                    if self.is_type_assignable_to(&fallback, &constraint) {
                        inferred_type = Some(fallback);
                    } else {
                        inferred_type = Some(constraint);
                    }
                } else {
                    inferred_type = Some(fallback);
                }
            } else {
                let constraint = self.get_constraint_of_type_parameter(&inference.type_parameter);
                inferred_type = constraint;
            }
        }

        // Ensure we always have a result
        inferred_type.unwrap_or_else(|| {
            if context.flags.contains(InferenceFlags::AnyDefault) {
                self.any_type()
            } else {
                self.unknown_type()
            }
        })
    }

    /// The contextual type of a property `name` on contextual type `t`
    /// (Go `getTypeOfPropertyOfContextualTypeEx`, checker.go ~L30623):
    /// unions map over constituents, generic mapped types substitute the
    /// property-name literal into the template (`{ [K in keyof P]: P[K] }`
    /// yields `P["when"]` for property `when`), concrete properties read
    /// their symbol's type.
    pub fn get_type_of_property_of_contextual_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> {
        use crate::checker::types::TypeData;
        // A type-parameter contextual type resolves through its constraint
        // (Go apparent-izes the contextual type before property lookup):
        // `mapped1<T extends { [P in string]: TakeString }>` provides
        // property types from the mapped constraint.
        if t.flags.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_constraint_of_type_parameter(t)?;
            return self.get_type_of_property_of_contextual_type(&constraint, name);
        }
        // Union contextual types: keep the constituents that provide the
        // property (Go mapTypeEx).
        if t.flags.contains(TypeFlags::Union)
            && let TypeData::Union(u) = &t.data
        {
            let found: Vec<Arc<Type>> = u
                .union_or_intersection
                .types
                .iter()
                .filter_map(|c| self.get_type_of_property_of_contextual_type(c, name))
                .collect();
            return match found.len() {
                0 => None,
                1 => Some(found.into_iter().next().unwrap()),
                _ => {
                    let types = found;
                    if types.iter().all(|x| Arc::ptr_eq(x, &types[0])) {
                        Some(Arc::clone(&types[0]))
                    } else {
                        Some(self.get_union_type(types))
                    }
                }
            };
        }
        // Generic mapped type with plain (non-remapping) names: substitute
        // the property-name literal for the mapped type parameter (Go
        // getIndexedMappedTypeSubstitutedTypeOfContextualType +
        // substituteIndexedMappedType).
        if matches!(&t.data, TypeData::Mapped(_))
            && let TypeData::Mapped(m) = &t.data
            && m.type_parameter.is_some()
            && m.template_type.is_some()
            && m.name_type.is_none()
        {
            let constraint = m.constraint_type.clone()?;
            let name_literal = self.get_string_literal_type(name);
            // Gate (Go): the property name must be assignable to the
            // constraint (or its base constraint). `keyof P` over a type
            // parameter reduces through the parameter's constraint; an
            // UNCONSTRAINED parameter's keyof is `unknown` (always passes).
            let gate_target = self.reduced_keyof_for_contextual_gate(&constraint);
            let gate_ok = match &gate_target {
                Some(g) => self.is_type_assignable_to(&name_literal, g),
                None => {
                    if matches!(&constraint.data, TypeData::Index(idx)
                        if idx.target.as_ref().is_some_and(|tgt| {
                            tgt.flags.contains(TypeFlags::TypeParameter)
                                && self
                                    .get_constraint_of_type_parameter(tgt)
                                    .is_none()
                        }))
                    {
                        true
                    } else {
                        self.is_type_assignable_to(&name_literal, &constraint)
                    }
                }
            };
            if !gate_ok {
                return None;
            }
            let tp = m.type_parameter.clone().unwrap();
            let template = m.template_type.clone().unwrap();
            let substituted =
                self.substitute_infer_type_parameters(&template, &[tp], &[name_literal]);
            // A deferred `P["when"]` resolves through P's constraint for
            // signature extraction (`(value: string) => boolean`).
            if let TypeData::IndexedAccess(ia) = &substituted.data
                && let (Some(obj), Some(idx)) = (&ia.object_type, &ia.index_type)
            {
                let resolved = self.get_indexed_access_type(obj, idx);
                if !matches!(resolved.intrinsic_name(), Some("any") | Some("error")) {
                    return Some(resolved);
                }
            }
            return Some(substituted);
        }
        // Deferred CONDITIONAL contextual types distribute for property
        // lookup (Go apparent-izes conditionals through both branches):
        // `string extends keyof P ? A : B` provides property types from A
        // and B union'd.
        if let TypeData::Conditional(c) = &t.data {
            let mut branches: Vec<Arc<Type>> = Vec::new();
            if let Some(root) = &c.root
                && let Some(node) = &root.node
                && let crate::ast::NodeData::ConditionalTypeNode(cd) = &node.data
            {
                let true_t = self.get_type_from_type_node(&cd.true_type);
                let false_t = self.get_type_from_type_node(&cd.false_type);
                for branch in [true_t, false_t] {
                    if let Some(found) = self.get_type_of_property_of_contextual_type(&branch, name)
                    {
                        branches.push(found);
                    }
                }
            }
            return match branches.len() {
                0 => None,
                1 => Some(branches.pop().unwrap()),
                _ => {
                    if branches.iter().all(|b| Arc::ptr_eq(b, &branches[0])) {
                        Some(Arc::clone(&branches[0]))
                    } else {
                        Some(self.get_union_type(branches))
                    }
                }
            };
        }
        // Intersection contextual types: try each object constituent (Go
        // intersects the results; the common case has one provider).
        if t.flags.contains(TypeFlags::Intersection)
            && let TypeData::Intersection(i) = &t.data
        {
            for c in &i.union_or_intersection.types {
                if let Some(found) = self.get_type_of_property_of_contextual_type(c, name) {
                    return Some(found);
                }
            }
            return None;
        }
        // Concrete property of the contextual type.
        if let Some(prop) = self.get_property_of_type(t, name) {
            return Some(self.get_type_of_symbol(&prop));
        }
        // Index signatures contribute their value type.
        let name_literal = self.get_string_literal_type(name);
        if let Some(info) = self.get_applicable_index_info(t, &name_literal) {
            return info.value_type.clone();
        }
        None
    }

    /// `keyof P` (an Index type over a type parameter) reduces through the
    /// parameter's constraint for the contextual-gate check (Go
    /// `getBaseConstraintOrType` on index types).
    fn reduced_keyof_for_contextual_gate(&mut self, constraint: &Arc<Type>) -> Option<Arc<Type>> {
        use crate::checker::types::TypeData;
        let TypeData::Index(idx) = &constraint.data else {
            return None;
        };
        let target = idx.target.as_ref()?;
        if !target.flags.contains(TypeFlags::TypeParameter) {
            return None;
        }
        let target_constraint = self.get_constraint_of_type_parameter(target)?;
        Some(self.get_index_type(&target_constraint))
    }

    /// Get the contextual type for an expression node.
    /// Go: `getContextualType` (checker.go:29100)
    pub fn get_contextual_type(
        &mut self,
        node: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        let parent = match &node.parent {
            Some(p) => Arc::clone(p),
            None => return None,
        };

        match parent.kind {
            SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::BindingElement => {
                self.get_contextual_type_for_initializer_expression(node, _context_flags)
            }
            SyntaxKind::ArrowFunction | SyntaxKind::ReturnStatement => {
                self.get_contextual_type_for_return_expression(node, _context_flags)
            }
            SyntaxKind::CallExpression | SyntaxKind::NewExpression => {
                self.get_contextual_type_for_argument(&parent, node)
            }
            SyntaxKind::BinaryExpression => {
                self.get_contextual_type_for_binary_operand(node, _context_flags)
            }
            SyntaxKind::PropertyAssignment | SyntaxKind::ShorthandPropertyAssignment => {
                self.get_contextual_type_for_object_literal_element(&parent, _context_flags)
            }
            SyntaxKind::ArrayLiteralExpression => {
                self.get_contextual_type_for_array_literal_element(node, &parent, _context_flags)
            }
            _ => None,
        }
    }

    /// The contextual signature for a function-like declaration (function
    /// expression, arrow function, or object-literal method): the call
    /// signature of its contextual type that is applicable to the node's
    /// parameter list. Direct port of Go's `getContextualSignature` +
    /// `getContextualCallSignature` (checker.go ~L10314): union contextual
    /// types contribute one signature per constituent (yielding none unless
    /// they agree), and signatures with fewer parameters than the node's
    /// required-parameter count are filtered out (`isAritySmaller`).
    pub fn get_contextual_signature(
        &mut self,
        node: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Signature>> {
        let t = self.get_contextual_type(node, ContextFlags::Signature)?;
        if let TypeData::Union(u) = &t.data {
            let mut first: Option<Arc<Signature>> = None;
            for current in &u.union_or_intersection.types {
                let Some(signature) = self.get_contextual_call_signature(current, node) else {
                    continue;
                };
                match &first {
                    None => first = Some(signature),
                    Some(f) => {
                        // Go requires the collected signatures to be
                        // identical (`compareSignaturesIdentical`) else no
                        // contextual signature is used; parameter count is
                        // the decisive component for contextual parameter
                        // typing.
                        if f.parameters.len() != signature.parameters.len() {
                            return None;
                        }
                    }
                }
            }
            return first;
        }
        self.get_contextual_call_signature(&t, node)
    }

    /// If the given type has a call signature with at least as many
    /// parameters as the given function, return that signature (Go
    /// `getContextualCallSignature`; overload sets reduce to their first
    /// arity-applicable signature).
    fn get_contextual_call_signature(
        &mut self,
        t: &Arc<Type>,
        node: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Signature>> {
        let signatures = self.get_signatures_of_type(t, SignatureKind::Call);
        signatures
            .into_iter()
            .find(|s| !self.is_arity_smaller(s, node))
    }

    /// If the contextual signature has fewer parameters than the function
    /// expression, do not use it. Port of Go's `isAritySmaller`
    /// (checker.go ~L10357): counts the node's leading required parameters
    /// (stopping at the first optional/initialized/rest one, excluding a
    /// leading `this` parameter) and rejects signatures with a smaller
    /// non-rest parameter count.
    fn is_arity_smaller(
        &self,
        signature: &Arc<Signature>,
        target: &Arc<crate::ast::Node>,
    ) -> bool {
        let Some(parameters) = function_like_parameters(target) else {
            return false;
        };
        let mut target_parameter_count: i32 = 0;
        for param in parameters.iter() {
            let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if pd.initializer.is_some()
                || pd.question_token.is_some()
                || pd.dot_dot_dot_token.is_some()
            {
                break;
            }
            target_parameter_count += 1;
        }
        if let Some(first) = parameters.iter().next() {
            if is_this_parameter_node(first) {
                target_parameter_count -= 1;
            }
        }
        let has_effective_rest = signature.flags.contains(SignatureFlags::HasRestParameter);
        let parameter_count = signature.parameters.len() as i32
            - if has_effective_rest { 1 } else { 0 };
        !has_effective_rest && parameter_count < target_parameter_count
    }

    /// Contextual parameter types for an immediately-invoked function
    /// expression (`((x) => x)(1)`): a synthetic signature whose parameter
    /// types are the corresponding call arguments' types. Port of the IIFE
    /// branch of Go's `getContextuallyTypedParameterType` (checker.go
    /// ~L29273); arguments past the parameter list are ignored, missing
    /// ones contribute no entry (the parameter stays implicitly `any`).
    pub fn iife_contextual_signature(
        &mut self,
        node: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Signature>> {
        // The function expression must be the callee of an enclosing call
        // (parentheses between them are skipped, Go
        // `GetImmediatelyInvokedFunctionExpression`).
        let mut parent = node.parent.clone()?;
        while parent.kind == SyntaxKind::ParenthesizedExpression {
            parent = parent.parent.clone()?;
        }
        let crate::ast::NodeData::CallExpression(call) = &parent.data else {
            return None;
        };
        if !Arc::ptr_eq(&call.expression, node) {
            return None;
        }
        let arg_count = call.arguments.len();
        if arg_count == 0 {
            return None;
        }
        // Recursion guard: an argument whose type depends on this very
        // invocation (e.g. a self-referential recursive IIFE) must not
        // re-enter. Go swaps in `anySignature` for the duration; we skip
        // the argument and leave the parameter contextual slot empty.
        let key = node.id();
        if !self.resolving_function_like.insert(key) {
            return None;
        }
        let mut params: Vec<Arc<crate::ast::Symbol>> = Vec::with_capacity(arg_count);
        for (i, arg) in call.arguments.iter().enumerate() {
            let t = self.get_type_of_node(arg);
            let sym = Arc::new(crate::ast::Symbol::new(
                crate::ast::SymbolFlags::Property,
                format!("__iife{}", i),
            ));
            self.value_symbol_links.insert(
                &sym,
                ValueSymbolLinks {
                    resolved_type: Some(t),
                    ..Default::default()
                },
            );
            params.push(sym);
        }
        self.resolving_function_like.remove(&key);
        Some(Arc::new(Signature {
            id: 0,
            flags: SignatureFlags::None,
            min_argument_count: 0,
            resolved_min_argument_count: -1,
            declaration: None,
            type_parameters: Vec::new(),
            parameters: params,
            this_parameter: None,
            resolved_return_type: std::sync::OnceLock::new(),
            resolved_type_predicate: None,
            target: None,
            mapper: None,
            isolated_signature_type: std::sync::OnceLock::new(),
            instantiated_parameter_types: None,
        }))
    }

    /// Get the contextual type for an initializer expression.
    ///
    /// In a variable, parameter or property declaration with a type annotation,
    /// the contextual type of an initializer expression is the type of the
    /// variable, parameter or property.
    ///
    /// Go: `getContextualTypeForInitializerExpression` (checker.go:29180)
    fn get_contextual_type_for_initializer_expression(
        &mut self,
        node: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let declaration = node.parent.as_ref()?;
        // Check that node is indeed the initializer
        let is_initializer = match &declaration.data {
            NodeData::VariableDeclaration(data) => data
                .initializer
                .as_ref()
                .map_or(false, |init| Arc::ptr_eq(init, node)),
            NodeData::ParameterDeclaration(data) => data
                .initializer
                .as_ref()
                .map_or(false, |init| Arc::ptr_eq(init, node)),
            NodeData::PropertyDeclaration(data) => data
                .initializer
                .as_ref()
                .map_or(false, |init| Arc::ptr_eq(init, node)),
            NodeData::BindingElement(data) => data
                .initializer
                .as_ref()
                .map_or(false, |init| Arc::ptr_eq(init, node)),
            _ => false,
        };
        if !is_initializer {
            return None;
        }

        // Get type from the declaration's type annotation if present
        let type_node = match &declaration.data {
            NodeData::VariableDeclaration(data) => data.type_node.as_ref(),
            NodeData::ParameterDeclaration(data) => data.type_node.as_ref(),
            NodeData::PropertyDeclaration(data) => data.type_node.as_ref(),
            NodeData::PropertySignatureDeclaration(data) => Some(&data.type_node),
            NodeData::BindingElement(_) => None,
            _ => None,
        };

        if let Some(type_node) = type_node {
            return Some(self.get_type_from_type_node(type_node));
        }

        // No type annotation. If the declaration is a BindingElement, attempt
        // to derive a contextual type from the binding pattern's initializer.
        // This ports the binding-pattern branch of Go's
        // `getContextualTypeForInitializerExpression` (which calls
        // `getTypeFromBindingPattern`). We implement a simplified version that
        // walks up to the enclosing VariableDeclaration, evaluates the
        // initializer's type, and indexes into it for array/object patterns.
        if let NodeData::BindingElement(_) = &declaration.data {
            if let Some(ctx) = self.get_contextual_type_from_binding_element(declaration) {
                return Some(ctx);
            }
        }

        None
    }

    /// Derive a contextual type for a `BindingElement`'s initializer by
    /// walking up to the enclosing `VariableDeclaration` and evaluating the
    /// initializer expression's type, then indexing into it for array/object
    /// binding patterns.
    ///
    /// Simplified port of Go's `getTypeFromBindingPattern` +
    /// `getContextualTypeForBindingElement`. The full Go implementation
    /// constructs an anonymous object type from the pattern; here we return
    /// the element/property type for the specific binding element so that
    /// initializers like `const [a = 42] = arr` get `arr`'s element type as
    /// context.
    fn get_contextual_type_from_binding_element(
        &mut self,
        binding_element: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        // BindingElement → BindingPattern → VariableDeclaration
        let binding_pattern = binding_element.parent.as_ref()?;
        let var_declaration = binding_pattern.parent.as_ref()?;

        let var_data = match &var_declaration.data {
            NodeData::VariableDeclaration(d) => d,
            _ => return None,
        };

        let initializer = var_data.initializer.as_ref()?;
        let init_type = self.get_type_of_node(initializer);

        match binding_pattern.kind {
            SyntaxKind::ArrayBindingPattern => {
                // Find this element's index in the pattern.
                let elements = match &binding_pattern.data {
                    NodeData::BindingPattern(d) => &d.elements,
                    _ => return None,
                };
                let idx = elements
                    .nodes
                    .iter()
                    .position(|e| Arc::ptr_eq(e, binding_element))?;
                // Element type of the initializer (if array/tuple).
                self.get_element_type_of_array(&init_type, idx)
            }
            SyntaxKind::ObjectBindingPattern => {
                // Property name from the binding element.
                let be_data = match &binding_element.data {
                    NodeData::BindingElement(d) => d,
                    _ => return None,
                };
                let property_name = be_data.property_name.as_ref().unwrap_or_else(|| {
                    be_data
                        .name
                        .as_ref()
                        .expect("binding element has name or property_name")
                });
                let name = property_name.text();
                self.property_type_of_type(&init_type, &name)
            }
            _ => None,
        }
    }

    /// Get the element type at `index` from an array or tuple type.
    fn get_element_type_of_array(&mut self, t: &Arc<Type>, index: usize) -> Option<Arc<Type>> {
        // Tuple type: return the specific element type if in range.
        if let TypeData::Tuple(tuple) = &t.data {
            if index < tuple.element_infos.len() {
                if let Some(ref elem) = tuple.element_infos[index].type_ {
                    return Some(Arc::clone(elem));
                }
            }
        }
        // Array<T>: return T (always succeeds, defaults to `any`). Only for
        // genuine array-like types — a generic interface instantiation
        // records type arguments for display, and those must not be
        // misread as an element type.
        if self.is_array_type(t) || matches!(t.data, TypeData::EvolvingArray(_)) {
            return Some(self.get_array_element_type(t));
        }
        None
    }

    /// Get the type of a named property of an object/interface type.
    /// (Reuses `get_property_of_type` from flow.rs via `pub(super)`; defined
    /// here under a different name to avoid clashing with flow.rs's
    /// `get_property_type_of_type`.)
    fn property_type_of_type(&mut self, t: &Arc<Type>, name: &str) -> Option<Arc<Type>> {
        let prop = self.get_property_of_type(t, name)?;
        Some(self.get_type_of_symbol(&prop))
    }

    /// Inline subset of `get_contextual_type` for call/new expressions.
    ///
    /// Returns the contextual type when a call/new expression is the
    /// initializer of a variable/parameter/property declaration with a type
    /// annotation, or the return value of a function with a return type
    /// annotation. This covers the common pattern
    /// `let x: T = genericFn(...)` where T should flow into inference of
    /// genericFn's return type.
    fn get_contextual_type_for_call_or_new(
        &mut self,
        node: &crate::ast::Node,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let parent = node.parent.as_ref()?;
        match &parent.data {
            NodeData::VariableDeclaration(data) => data
                .type_node
                .as_ref()
                .map(|tn| self.get_type_from_type_node(tn)),
            NodeData::ReturnStatement(_) => {
                // return genericFn(...) — contextual type is the function's
                // declared return type.
                let fn_node = parent.parent.as_ref()?;
                self.get_return_type_annotation_of_function(fn_node)
            }
            _ => None,
        }
    }

    /// Get the declared return type annotation of a function-like node.
    fn get_return_type_annotation_of_function(
        &mut self,
        node: &crate::ast::Node,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let type_node = match &node.data {
            NodeData::FunctionDeclaration(d) => d.type_node.as_ref(),
            NodeData::FunctionExpression(d) => d.type_node.as_ref(),
            NodeData::ArrowFunction(d) => d.type_node.as_ref(),
            NodeData::MethodDeclaration(d) => d.type_node.as_ref(),
            NodeData::MethodSignatureDeclaration(d) => d.type_node.as_ref(),
            NodeData::GetAccessorDeclaration(d) => d.type_node.as_ref(),
            NodeData::SetAccessorDeclaration(_) => None,
            _ => None,
        };
        type_node.map(|tn| self.get_type_from_type_node(tn))
    }

    /// Get the contextual type for a return expression.
    ///
    /// The type `return expr;` is checked against in the containing
    /// function: its declared return-type annotation if present, else the
    /// return type of the function's contextual signature, else (for an
    /// immediately-invoked function expression) the contextual type of the
    /// invocation itself.
    ///
    /// Go: `getContextualTypeForReturnExpression` + `getContextualReturnType`
    /// (checker.go ~L29332, ~L29370)
    fn get_contextual_type_for_return_expression(
        &mut self,
        _node: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        // Walk up the parent chain to find the containing function (the
        // first function-like ancestor — nested functions are boundaries).
        let mut current = _node.parent.as_ref()?.clone();
        loop {
            match current.kind {
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => break,
                SyntaxKind::SourceFile => return None,
                _ => {
                    current = current.parent.as_ref()?.clone();
                }
            }
        }
        self.contextual_return_type_of(&current)
    }

    /// The type `return expr;` statements are checked against in `fn_node`'s
    /// body: the declared return-type annotation, else the return type of
    /// the contextual signature, else (for an IIFE) the contextual type of
    /// the invocation. Port of Go's `getContextualReturnType` (checker.go
    /// ~L29370) minus the generator/async constituent filtering (handled at
    /// the await/yield subsystem instead).
    pub fn contextual_return_type_of(
        &mut self,
        fn_node: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;
        // 1. An explicit return-type annotation wins.
        let type_node = match &fn_node.data {
            NodeData::FunctionDeclaration(data) => data.type_node.clone(),
            NodeData::FunctionExpression(data) => data.type_node.clone(),
            NodeData::ArrowFunction(data) => data.type_node.clone(),
            NodeData::MethodDeclaration(data) => data.type_node.clone(),
            NodeData::ConstructorDeclaration(data) => data.type_node.clone(),
            NodeData::GetAccessorDeclaration(data) => data.type_node.clone(),
            NodeData::SetAccessorDeclaration(data) => data.type_node.clone(),
            _ => None,
        };
        if let Some(type_node) = type_node {
            return Some(self.get_type_from_type_node(&type_node));
        }
        // 2. Otherwise, if the function is contextually typed by a function
        // type with a call signature, return statements are contextually
        // typed by that signature's return type.
        if let Some(signature) = self.get_contextual_signature(fn_node) {
            if let Some(return_type) = self.get_return_type_of_signature(&signature) {
                return Some(return_type);
            }
        }
        // 3. An immediately-invoked function expression is contextually
        // typed by its invocation's contextual type.
        let mut parent = fn_node.parent.clone()?;
        while parent.kind == SyntaxKind::ParenthesizedExpression {
            parent = parent.parent.clone()?;
        }
        if let NodeData::CallExpression(call) = &parent.data {
            if Arc::ptr_eq(&call.expression, fn_node) {
                return self.get_contextual_type(&parent, ContextFlags::None);
            }
        }
        None
    }

    /// Get the contextual type for a function argument.
    ///
    /// In a typed function call, an argument is contextually typed by the
    /// type of the corresponding parameter — for a GENERIC signature, the
    /// parameter type of the RESOLVED signature, i.e. with type arguments
    /// inferred from the (non-context-sensitive) sibling arguments already
    /// substituted. Port of Go's `getContextualTypeForArgument` +
    /// `getContextualTypeForArgumentAtIndex` (checker.go ~L29519): the
    /// signature is the resolved one, so its parameter types carry the call's
    /// inference results.
    fn get_contextual_type_for_argument(
        &mut self,
        call_node: &Arc<crate::ast::Node>,
        arg_node: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        // Get the arguments list
        let args = match &call_node.data {
            NodeData::CallExpression(data) => Some(&data.arguments),
            NodeData::NewExpression(data) => data.arguments.as_ref(),
            _ => None,
        }?;

        // Find the argument index
        let arg_index = args.iter().position(|a| Arc::ptr_eq(a, arg_node))?;

        // Get the expression type to find call signatures. `new` expressions
        // consult construct signatures; calls consult call signatures.
        let is_new = matches!(&call_node.data, NodeData::NewExpression(_));
        let expression_type = match &call_node.data {
            NodeData::CallExpression(data) => Some(self.get_type_of_node(&data.expression)),
            NodeData::NewExpression(data) => Some(self.get_type_of_node(&data.expression)),
            _ => None,
        }?;
        let kind = if is_new {
            SignatureKind::Construct
        } else {
            SignatureKind::Call
        };
        let signatures = self.get_signatures_of_type(&expression_type, kind);
        // Pick the first signature that reaches the argument position, else
        // the first signature (approximates the resolved overload).
        let sig = signatures
            .iter()
            .find(|s| s.parameters.len() > arg_index)
            .or_else(|| signatures.first())?
            .clone();

        // Get the parameter type at the argument index
        if arg_index >= sig.parameters.len() {
            return None;
        }
        let base_param_type = self.get_type_of_symbol(&sig.parameters[arg_index]);

        // Generic signature: infer type arguments from the NON-context-
        // sensitive sibling arguments (Go's first inference phase; function
        // literal arguments are context-sensitive and contribute only after
        // the mapper is fixed), then substitute them into the parameter type.
        if !sig.type_parameters.is_empty() {
            let key = call_node.id();
            if self.resolving_contextual_calls.insert(key) {
                let sibling_args: Vec<Arc<crate::ast::Node>> = args
                    .iter()
                    .enumerate()
                    .filter(|(i, a)| {
                        *i != arg_index
                            && !matches!(
                                a.kind,
                                SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression
                            )
                    })
                    .map(|(_, a)| Arc::clone(a))
                    .collect();
                let inferred =
                    self.infer_call_type_arguments(call_node, &sig, &sibling_args);
                self.resolving_contextual_calls.remove(&key);
                if !inferred.is_empty() {
                    return Some(self.substitute_infer_type_parameters(
                        &base_param_type,
                        &sig.type_parameters,
                        &inferred,
                    ));
                }
            }
        }
        Some(base_param_type)
    }

    /// Get the contextual type for a binary operand.
    ///
    /// In an assignment expression, the right operand is contextually typed
    /// by the type of the left operand.
    ///
    /// Go: `getContextualTypeForBinaryOperand` (checker.go:29566)
    fn get_contextual_type_for_binary_operand(
        &mut self,
        node: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let parent = node.parent.as_ref()?;
        let binary = match &parent.data {
            NodeData::BinaryExpression(data) => data,
            _ => return None,
        };

        // Only right operand gets contextual typing
        if !Arc::ptr_eq(node, &binary.right) {
            return None;
        }

        match binary.operator_token.kind {
            SyntaxKind::EqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken => {
                // Assignment: right operand is contextually typed by left operand
                Some(self.get_type_of_node(&binary.left))
            }
            SyntaxKind::BarBarToken | SyntaxKind::QuestionQuestionToken => {
                // || and ?? : right operand is contextually typed by left operand
                Some(self.get_type_of_node(&binary.left))
            }
            SyntaxKind::AmpersandAmpersandToken | SyntaxKind::CommaToken => {
                // && and comma: right operand is contextually typed by the parent expression
                Some(self.get_type_of_node(&binary.left))
            }
            _ => None,
        }
    }

    /// Get the contextual type for an object literal element.
    ///
    /// In an object literal contextually typed by a type T, the contextual
    /// type of a property assignment is the type of the matching property
    /// in T.
    ///
    /// Go: `getContextualTypeForObjectLiteralElement` (checker.go:29677)
    fn get_contextual_type_for_object_literal_element(
        &mut self,
        node: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        // Get the parent object literal
        let object_literal = node.parent.as_ref()?;

        // Get the contextual type of the object literal
        let contextual_type = self.get_contextual_type(object_literal, _context_flags)?;

        // Get the property name from the element
        let name = match &node.data {
            NodeData::PropertyAssignment(data) => match &data.name.data {
                NodeData::Identifier(id) => Some(id.text.clone()),
                NodeData::StringLiteral(s) => Some(s.text.clone()),
                _ => None,
            },
            NodeData::ShorthandPropertyAssignment(data) => match &data.name.data {
                NodeData::Identifier(id) => Some(id.text.clone()),
                _ => None,
            },
            _ => None,
        }?;

        // Contextual property type (Go getTypeOfPropertyOfContextualTypeEx):
        // mapped/union/deferred contextual types contribute per-property
        // types; falls back to None when the context provides nothing.
        self.get_type_of_property_of_contextual_type(&contextual_type, &name)
    }

    /// Get the contextual type for an array literal element.
    ///
    /// Returns the element type of the contextual type of the parent array
    /// literal.
    ///
    /// Go: `getContextualTypeForElementExpression` (checker.go:29729)
    fn get_contextual_type_for_array_literal_element(
        &mut self,
        _node: &crate::ast::Node,
        parent: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        // Get the contextual type of the parent array literal
        let contextual_type = self.get_contextual_type(parent, _context_flags)?;

        // For an array/object type with type arguments, return the element type
        let type_args = self.get_type_arguments(&contextual_type);
        if !type_args.is_empty() {
            return Some(Arc::clone(&type_args[0]));
        }

        // Try to get the element type from index info
        if let Some(structured) = contextual_type.as_structured() {
            for index_info in &structured.index_infos {
                if let Some(ref value_type) = index_info.value_type {
                    return Some(Arc::clone(value_type));
                }
            }
        }

        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // getCovariantInference and related helpers
    // ────────────────────────────────────────────────────────────────────────

    /// Get the covariant inference from candidates.
    fn get_covariant_inference(
        &mut self,
        inference: &InferenceInfo,
        _signature: &Arc<Signature>,
    ) -> Option<Arc<Type>> {
        if inference.candidates.is_empty() {
            return None;
        }
        let candidates = self.union_object_and_array_literal_candidates(&inference.candidates);
        let primitive_constraint = self.has_primitive_constraint(&inference.type_parameter)
            || self.is_const_type_variable(&inference.type_parameter, 0);
        let widen_literal_types = !primitive_constraint
            && inference.top_level
            && (inference.is_fixed
                || !self.is_type_parameter_at_top_level_in_return_type(
                    _signature,
                    &inference.type_parameter,
                ));
        let base_candidates: Vec<Arc<Type>> = if primitive_constraint {
            candidates
                .iter()
                .map(|t| self.get_regular_type_of_literal_type(t))
                .collect()
        } else if widen_literal_types {
            candidates
                .iter()
                .map(|t| self.get_widened_literal_type(t))
                .collect()
        } else {
            candidates
        };
        let unwidened_type = if inference
            .priority
            .contains(InferencePriority::PriorityImpliesCombination)
        {
            self.get_union_type(base_candidates)
        } else {
            self.get_common_supertype(&base_candidates)
        };
        Some(self.get_widened_type(&unwidened_type))
    }

    /// Get the contravariant inference from contra-candidates.
    fn get_contravariant_inference(&mut self, inference: &InferenceInfo) -> Option<Arc<Type>> {
        if inference.contra_candidates.is_empty() {
            return None;
        }
        if inference
            .priority
            .contains(InferencePriority::PriorityImpliesCombination)
        {
            Some(self.get_intersection_type(inference.contra_candidates.clone()))
        } else {
            Some(self.get_common_subtype(&inference.contra_candidates))
        }
    }

    /// Union object and array literal candidates.
    fn union_object_and_array_literal_candidates(
        &self,
        candidates: &[Arc<Type>],
    ) -> Vec<Arc<Type>> {
        if candidates.len() > 1 {
            let object_literals: Vec<Arc<Type>> = candidates
                .iter()
                .filter(|t| self.is_object_or_array_literal_type(t))
                .cloned()
                .collect();
            if !object_literals.is_empty() {
                let literals_type = self.create_union_type(object_literals);
                let non_literal_types: Vec<Arc<Type>> = candidates
                    .iter()
                    .filter(|t| !self.is_object_or_array_literal_type(t))
                    .cloned()
                    .collect();
                let mut result = non_literal_types;
                result.push(literals_type);
                return result;
            }
        }
        candidates.to_vec()
    }

    /// Check if a type parameter has a primitive constraint.
    fn has_primitive_constraint(&self, t: &Arc<Type>) -> bool {
        let constraint = self.get_constraint_of_type_parameter(t);
        if let Some(constraint) = constraint {
            let c = if constraint.flags.contains(TypeFlags::Conditional) {
                self.get_default_constraint_of_conditional_type(&constraint)
            } else {
                Some(constraint)
            };
            if let Some(c) = c {
                return self.maybe_type_of_kind(
                    &c,
                    TypeFlags::String
                        | TypeFlags::Number
                        | TypeFlags::BigInt
                        | TypeFlags::Boolean
                        | TypeFlags::ESSymbol
                        | TypeFlags::Enum
                        | TypeFlags::Index
                        | TypeFlags::TemplateLiteral
                        | TypeFlags::StringMapping,
                );
            }
        }
        false
    }

    /// Check if a type parameter is at the top level in a type.
    fn is_type_parameter_at_top_level(&self, t: &Type, tp: &Type, depth: i32) -> bool {
        if t.id == tp.id {
            return true;
        }
        if t.flags.contains(TypeFlags::Union | TypeFlags::Intersection) {
            if let Some(types) = t.types() {
                return types
                    .iter()
                    .any(|tt| self.is_type_parameter_at_top_level(tt, tp, depth));
            }
        }
        false
    }

    /// Check if a type parameter is at the top level in the return type of a signature.
    fn is_type_parameter_at_top_level_in_return_type(
        &self,
        signature: &Arc<Signature>,
        type_parameter: &Type,
    ) -> bool {
        if let Some(return_type) = signature.resolved_return_type.get() {
            return self.is_type_parameter_at_top_level(return_type, type_parameter, 0);
        }
        false
    }

    /// Get the type from inference without a signature.
    fn get_type_from_inference(&self, inference: &InferenceInfo) -> Option<Arc<Type>> {
        if !inference.candidates.is_empty() {
            Some(self.create_union_type(inference.candidates.clone()))
        } else if !inference.contra_candidates.is_empty() {
            Some(self.create_intersection_type(inference.contra_candidates.clone()))
        } else {
            None
        }
    }

    /// Get the common supertype of a list of types.
    fn get_common_supertype(&mut self, types: &[Arc<Type>]) -> Arc<Type> {
        if types.len() == 1 {
            return types[0].clone();
        }
        let primary_types: Vec<Arc<Type>> = types.to_vec();
        if self.literal_types_with_same_base_type(&primary_types) {
            return self.create_union_type(primary_types);
        }
        let supertype = self.get_single_common_supertype(&primary_types);
        let nullable_flags = self.get_combined_type_flags(types) & TYPE_FLAGS_NULLABLE;
        if nullable_flags != TypeFlags::None {
            self.get_nullable_type(&supertype, nullable_flags)
        } else {
            supertype
        }
    }

    /// Get a single common supertype from a list of types.
    fn get_single_common_supertype(&mut self, types: &[Arc<Type>]) -> Arc<Type> {
        let candidate = self.find_leftmost_type(types);
        // Check if all types are subtypes of the candidate
        let all_are_strict_subtypes = types
            .iter()
            .all(|t| Arc::ptr_eq(t, &candidate) || self.is_type_strict_subtype_of(t, &candidate));
        if all_are_strict_subtypes {
            return candidate;
        }
        // Find the leftmost type using subtype relation
        let mut candidate: Option<Arc<Type>> = None;
        for t in types {
            match &candidate {
                None => candidate = Some(t.clone()),
                Some(c) => {
                    if self.is_type_subtype_of(c, t) {
                        candidate = Some(t.clone());
                    }
                }
            }
        }
        candidate.unwrap_or_else(|| self.unknown_type())
    }

    /// Find the leftmost type in a list according to a comparison function.
    fn find_leftmost_type(&mut self, types: &[Arc<Type>]) -> Arc<Type> {
        let mut candidate: Option<Arc<Type>> = None;
        for t in types {
            match &candidate {
                None => candidate = Some(t.clone()),
                Some(c) => {
                    candidate = Some(t.clone());
                }
            }
        }
        candidate.unwrap_or_else(|| self.unknown_type())
    }

    /// Get the common subtype (intersection-like) from a list of types.
    fn get_common_subtype(&mut self, types: &[Arc<Type>]) -> Arc<Type> {
        let mut subtype: Option<Arc<Type>> = None;
        for t in types {
            match &subtype {
                None => subtype = Some(t.clone()),
                Some(s) => {
                    if self.is_type_subtype_of(t, s) {
                        subtype = Some(t.clone());
                    }
                }
            }
        }
        subtype.unwrap_or_else(|| self.unknown_type())
    }

    /// Get combined type flags from a list of types.
    fn get_combined_type_flags(&self, types: &[Arc<Type>]) -> TypeFlags {
        let mut flags = TypeFlags::None;
        for t in types {
            if t.flags.contains(TypeFlags::Union) {
                if let Some(inner_types) = t.types() {
                    flags |= self.get_combined_type_flags(inner_types);
                }
            } else {
                flags |= t.flags;
            }
        }
        flags
    }

    /// Check if all types are literal types with the same base type.
    fn literal_types_with_same_base_type(&self, types: &[Arc<Type>]) -> bool {
        let mut common_base_type: Option<Arc<Type>> = None;
        for t in types {
            if t.flags.contains(TypeFlags::Never) {
                continue;
            }
            let base_type = self.get_base_type_of_literal_type(t);
            match &common_base_type {
                None => common_base_type = Some(base_type.clone()),
                Some(cbt) => {
                    if Arc::ptr_eq(&base_type, t) || !Arc::ptr_eq(&base_type, cbt) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Check if a type is a const type variable.
    fn is_const_type_variable(&self, _t: &Type, _depth: i32) -> bool {
        false
    }

    /// Get the default constraint of a conditional type.
    fn get_default_constraint_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let Some(constraint) = self.get_constraint_of_type_parameter(t) {
            return Some(constraint);
        }
        None
    }

    /// Check if a type is of a kind specified by flags.
    fn maybe_type_of_kind(&self, t: &Type, flags: TypeFlags) -> bool {
        t.flags.intersects(flags)
    }

    /// Create a union type (simplified wrapper).
    fn create_union_type(&self, types: Vec<Arc<Type>>) -> Arc<Type> {
        let filtered: Vec<Arc<Type>> = types
            .into_iter()
            .filter(|t| !t.flags.contains(TypeFlags::Never))
            .collect();
        if filtered.is_empty() {
            return self.never_type();
        }
        if filtered.len() == 1 {
            return filtered[0].clone();
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: filtered,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    /// Create an intersection type (simplified wrapper).
    fn create_intersection_type(&self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.unknown_type();
        }
        if types.len() == 1 {
            return types[0].clone();
        }
        Arc::new(Type::new(
            TypeFlags::Intersection,
            TypeData::Intersection(IntersectionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types,
                },
                resolved_apparent_type: std::sync::OnceLock::new(),
                unique_literal_filled_instantiation: std::sync::OnceLock::new(),
            }),
        ))
    }

    // `get_widened_literal_type` and `get_regular_type_of_literal_type`
    // are implemented in `checker.rs` (real implementations backed by the
    // fresh-literal mechanism, not stubs).

    // ────────────────────────────────────────────────────────────────────────
    // Helper methods for inferTypeArguments
    // ────────────────────────────────────────────────────────────────────────

    // `get_this_type_of_signature` and `get_non_array_rest_type` are
    // implemented in `relater.rs` (real implementations, not stubs).

    /// Get the this argument type for a call node.
    fn get_this_argument_type(&self, _node: &crate::ast::Node) -> Arc<Type> {
        self.undefined_type()
    }

    /// Get the spread argument type for a list of arguments.
    fn get_spread_argument_type(
        &self,
        _args: &[Arc<crate::ast::Node>],
        _start: usize,
        _end: usize,
    ) -> Arc<Type> {
        self.unknown_type()
    }

    /// Check if a type is an object or array literal type.
    fn is_object_or_array_literal_type(&self, t: &Type) -> bool {
        t.flags.contains(TypeFlags::Object)
            && t.object_flags
                .intersects(ObjectFlags::ObjectLiteral | ObjectFlags::ArrayLiteral)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Helper functions
// ────────────────────────────────────────────────────────────────────────────

/// Parameter list of a function-like node (function expression, arrow,
/// method, accessor, function declaration). `None` for other nodes.
pub(super) fn function_like_parameters(
    node: &Arc<crate::ast::Node>,
) -> Option<Arc<crate::ast::NodeList>> {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::FunctionExpression(d) => Some(Arc::clone(&d.parameters)),
        NodeData::ArrowFunction(d) => Some(Arc::clone(&d.parameters)),
        NodeData::FunctionDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::MethodSignatureDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.parameters)),
        _ => None,
    }
}

/// Whether a parameter node is a `this` parameter (Go `ast.IsThisParameter`).
fn is_this_parameter_node(param: &Arc<crate::ast::Node>) -> bool {
    if let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data {
        return matches!(&pd.name.data, crate::ast::NodeData::Identifier(id) if id.text == "this");
    }
    false
}
