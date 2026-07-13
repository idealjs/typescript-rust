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
    visited: HashMap<InferenceKey, InferencePriority>,
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

        // Handle union types in target
        if target.flags.contains(TypeFlags::Union) {
            let source_types = if source.flags.contains(TypeFlags::Union) {
                source.types().unwrap_or_default().to_vec()
            } else {
                vec![Arc::clone(source)]
            };
            let target_types = target.types().unwrap_or_default().to_vec();
            let (temp_sources, temp_targets) = self.infer_from_matching_types(
                state, &source_types, &target_types, true,
            );
            if temp_targets.is_empty() {
                return;
            }
            let target = self.get_union_type(temp_targets);
            if temp_sources.is_empty() {
                self.infer_with_priority(state, source, &target, InferencePriority::NakedTypeVariable);
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
        let (sources, targets) = self.infer_from_matching_types(
            state, &source_types, &target_types, false,
        );
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
        let (sources, targets) = self.infer_matching_types_identical(
            state, &source_types, &target_types,
        );
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
        let inference_idx = state.inferences.iter().position(|info| info.type_parameter.id == target.id);
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
                    if !inference.contra_candidates.iter().any(|c| Arc::ptr_eq(c, &candidate)) {
                        inference.contra_candidates.push(candidate);
                        cleared = true;
                    }
                } else {
                    if !inference.candidates.iter().any(|c| Arc::ptr_eq(c, &candidate)) {
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
        state.inference_priority = if state.inference_priority.bits() < state.priority.bits() { state.inference_priority } else { state.priority };
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
                        if let (Some(tv), Some(sv)) = (&target_index.value_type, &source_index.value_type) {
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
}

// ────────────────────────────────────────────────────────────────────────────
// Helper functions
// ────────────────────────────────────────────────────────────────────────────

