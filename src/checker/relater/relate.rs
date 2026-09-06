#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{ModifierFlags, SymbolFlags, SyntaxKind};
use crate::checker::is_tuple_type;
use crate::evaluator::EvalValue;

use crate::checker::checker::Checker;
use crate::checker::types::*;

use crate::checker::relater::RelationComparisonResult;
use super::*;

impl Checker {

    pub fn is_type_identical_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {

        if Arc::ptr_eq(source, target) {
            return true;
        }

        if source.flags != target.flags {
            return false;
        }
        if source.flags.contains(TYPE_FLAGS_SINGLETON) {
            return true;
        }
        self.is_simple_type_identical_to(source, target)
    }

    pub fn is_type_assignable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Assignable)
    }

    pub fn is_type_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Subtype)
    }

    pub fn is_type_strict_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::StrictSubtype)
    }

    pub fn is_type_comparable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Comparable)
    }

    pub fn are_types_comparable(&mut self, type1: &Arc<Type>, type2: &Arc<Type>) -> bool {
        self.is_type_comparable_to(type1, type2) || self.is_type_comparable_to(type2, type1)
    }

    pub(crate) fn is_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {

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

        if Arc::ptr_eq(&source, &target) {
            return true;
        };

        {
            let sp = source.id;
            let tp = target.id;
            if source.flags.contains(TypeFlags::Object)
                && target.flags.contains(TypeFlags::Object)
                && (self.degraded_type_ptrs.contains(&sp)
                    || self.degraded_type_ptrs.contains(&tp))
            {
                return true;
            }
        }

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

        if self.relater_overflow {
            return true;
        }
        if self.relater_depth >= RELATER_MAX_DEPTH {
            self.relater_overflow = true;
            return true;
        }

        if self.relation_count == 0 && self.relater_depth > 0 {
            self.relater_overflow = true;
            return true;
        }

        if self.relater_depth == 0 {
            self.relation_cache.clear();
            self.relation_in_progress.clear();
            self.relater_overflow = false;
            self.relater_source_stack.clear();
            self.relater_target_stack.clear();

            self.relation_count = 2_000_000;
        }
        let key = RelationCacheKey {
            source_id: source.id,
            target_id: target.id,
            relation,
        };

        if self.relation_in_progress.contains(&key) {
            return true;
        }

        if let Some(&cached) = self.relation_cache.get(&key) {
            if cached || !self.relater_chain_active {
                return cached;
            }
        }
        self.relation_in_progress.insert(key);
        self.relater_depth += 1;

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

        if !result {
            self.relation_count = self.relation_count.saturating_sub(1);
        }

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

            if truly_deferred && self.deferred_constraint_depth < 100
                && let Some(constraint) = self.deferred_default_constraint_of_conditional(&source)
            {
                self.deferred_constraint_depth += 1;
                let r = self.is_type_related_to(&constraint, &target, relation);
                self.deferred_constraint_depth -= 1;
                if r {
                    result = true;
                }
            }
        }

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

    fn chain_message_key(&self, index: usize) -> Option<&'static str> {
        let len = self.relater_error_chain.len();
        if len <= index {
            return None;
        }
        Some(self.relater_error_chain[len - 1 - index].message.key)
    }

    fn chain_args(&self, index: usize) -> Option<&[String]> {
        let len = self.relater_error_chain.len();
        if len <= index {
            return None;
        }
        Some(&self.relater_error_chain[len - 1 - index].args)
    }

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

    pub(crate) fn try_elaborate_primitive_and_object(&mut self, source: &Arc<Type>, target: &Arc<Type>) {
        use crate::diagnostics::messages_generated as msg;
        if !source.flags.contains(TypeFlags::Object)
            || !target.flags.intersects(
                TypeFlags::String
                    | TypeFlags::Number
                    | TypeFlags::Boolean
                    | TypeFlags::ESSymbol,
            )
        {
            return;
        }
        let Some(sym) = source.symbol.as_ref() else {
            return;
        };
        let name = match sym.name.as_str() {
            "String" | "Number" | "Boolean" | "Symbol" => sym.name.as_str(),
            _ => return,
        };

        if self.globals.get(name).is_none() {
            return;
        }
        let matches = match name {
            "String" => target.flags.contains(TypeFlags::String),
            "Number" => target.flags.contains(TypeFlags::Number),
            "Boolean" => target.flags.contains(TypeFlags::Boolean),
            _ => target.flags.contains(TypeFlags::ESSymbol),
        };
        if !matches {
            return;
        }
        let target_str = self.type_to_string(target);
        let source_str = self.type_to_string(source);
        self.relater_report_error(
            msg::X_0_IS_A_PRIMITIVE_BUT_1_IS_A_WRAPPER_OBJECT_PREFER_USING_0_WHEN_POSSIBLE,
            vec![target_str, source_str],
        );
    }

    pub(crate) fn relater_report_error(
        &mut self,
        message: crate::diagnostics::Message,
        args: Vec<String>,
    ) {
        use crate::diagnostics::messages_generated as msg;
        if !self.relater_chain_active {
            return;
        }
        if message.key == msg::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE.key {

            if let Some(top) = self.chain_message_key(0)
                && (top == msg::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1.key
                    || top == msg::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_BUT_0_DOES_NOT_EXIST_IN_TYPE_1_DID_YOU_MEAN_TO_WRITE_2.key)
            {
                return;
            }

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

    pub(crate) fn push_relation_head_with_tp_note(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        head: crate::diagnostics::Message,
        head_args: Vec<String>,
    ) {
        use crate::diagnostics::messages_generated as msg;

        let target_flags_view = if target.flags.contains(TypeFlags::IndexedAccess)
            && !source.flags.contains(TypeFlags::IndexedAccess)
        {
            match &target.data {
                crate::checker::types::TypeData::IndexedAccess(d) => d
                    .object_type
                    .as_ref()
                    .map(|o| o.flags)
                    .unwrap_or(target.flags),
                _ => target.flags,
            }
        } else {
            target.flags
        };
        if target_flags_view.contains(TypeFlags::TypeParameter) {
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

    pub(crate) fn constraint_of_indexed_access(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let ia = match &t.data {
            TypeData::IndexedAccess(ia) => ia,
            _ => return None,
        };
        let object = ia.object_type.as_ref()?;
        let index = ia.index_type.as_ref()?;

        let obj_constraint = if object.flags.contains(TypeFlags::TypeParameter) {
            match self.get_constraint_of_type_parameter(object) {
                Some(c) => c,
                None => {
                    let sym = object.symbol.as_ref()?;

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

        if matches!(
            obj_constraint.intrinsic_name(),
            Some("any") | Some("unknown") | Some("error")
        ) {
            return None;
        }

        let effective_index = if index.flags.intersects(
            TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Index,
        ) || matches!(&index.data, TypeData::IndexedAccess(_))
        {
            match self.reduce_type_for_constraint(index, 8) {
                Some(reduced) => reduced,
                None => return None,
            }
        } else {
            Arc::clone(index)
        };
        let resolved = self.get_indexed_access_type(&obj_constraint, &effective_index);
        if matches!(resolved.intrinsic_name(), Some("any") | Some("error")) {
            return None;
        }
        Some(resolved)
    }

    fn reduce_type_for_constraint(&mut self, t: &Arc<Type>, depth: usize) -> Option<Arc<Type>> {
        if depth == 0 {
            return None;
        }
        if t.flags.contains(TypeFlags::TypeParameter) {
            if t.flags.contains(TypeFlags::Union) {
                return Some(Arc::clone(t));
            }
            let constraint = self.get_constraint_of_type_parameter(t)?;
            return self.reduce_type_for_constraint(&constraint, depth - 1);
        }
        if t.flags.contains(TypeFlags::IndexedAccess) || matches!(&t.data, TypeData::IndexedAccess(_))
        {
            return self.constraint_of_indexed_access(t);
        }
        if t.flags.contains(TypeFlags::Index) {
            if let TypeData::Index(it) = &t.data
                && let Some(target) = &it.target
            {
                let reduced = self.reduce_type_for_constraint(target, depth - 1)?;
                return Some(self.get_index_type(&reduced));
            }
            return None;
        }
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                let mut reduced_all = Vec::with_capacity(u.union_or_intersection.types.len());
                for c in &u.union_or_intersection.types {
                    reduced_all.push(self.reduce_type_for_constraint(c, depth - 1)?);
                }
                return Some(self.get_union_type(reduced_all));
            }
        }
        Some(Arc::clone(t))
    }

    pub(crate) fn constraint_of_conditional_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        if let Some(rt) = ct.resolved_true_type.get() {
            return Some(Arc::clone(rt));
        }
        if let Some(rt) = ct.resolved_false_type.get() {
            return Some(Arc::clone(rt));
        }
        let check_type = ct.check_type.clone()?;
        let tp_symbol = ct
            .root
            .as_ref()
            .filter(|r| r.is_distributive)
            .and_then(|r| r.check_type_parameter_symbol.clone())?;

        let constituents: Vec<Arc<Type>> = if check_type.flags.contains(TypeFlags::Union) {
            check_type.types()?.to_vec()
        } else if check_type.flags.contains(TypeFlags::IndexedAccess)
            || matches!(&check_type.data, TypeData::IndexedAccess(_))
        {
            let reduced = self.constraint_of_indexed_access(&check_type)?;
            if reduced.flags.contains(TypeFlags::Union) {
                reduced.types()?.to_vec()
            } else {
                vec![reduced]
            }
        } else {
            return None;
        };
        let key = Arc::as_ptr(&tp_symbol);
        let mut results: Vec<Arc<Type>> = Vec::with_capacity(constituents.len());
        for constituent in constituents {
            let mut mapping = std::collections::HashMap::new();
            mapping.insert(key, Arc::clone(&constituent));
            self.type_argument_stack.push(mapping);
            let r = self.resolve_conditional_type_with_check(t, Some(constituent));
            self.type_argument_stack.pop();
            results.push(r?);
        }
        Some(self.get_union_type(results))
    }

    fn is_type_related_to_inner(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {

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

        if s.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(source) {
                if self.is_type_related_to(&constraint, target, relation) {
                    return true;
                }
            }
        }

        let source_is_indexed_access = s.contains(TypeFlags::IndexedAccess)
            || matches!(source.data, TypeData::IndexedAccess(_));
        if source_is_indexed_access && !t.contains(TypeFlags::IndexedAccess) {
            if let Some(constraint) = self.constraint_of_indexed_access(source)
                && self.is_type_related_to(&constraint, target, relation)
            {
                return true;
            }
        }

        if s.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            || t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
        {
            return self.is_union_or_intersection_related_to(source, target, relation);
        }

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

            if let (Some(ss), Some(ts)) = (&source.symbol, &target.symbol)
                && ss.id() == ts.id()
                && ss.flags.intersects(SymbolFlags::Interface | SymbolFlags::Class)
            {
                let source_args = self.get_type_arguments(source);
                let target_args = self.get_type_arguments(target);
                if source_args.is_empty() && target_args.is_empty() {
                    return true;
                }
                if source_args.len() == target_args.len()
                    && !source_args.is_empty()
                    && source_args.iter().zip(target_args.iter()).all(|(a, b)| {
                        self.is_type_related_to(a, b, relation)
                            && self.is_type_related_to(b, a, relation)
                    })
                {
                    return true;
                }
            }

            if self.is_array_type(source) && self.is_array_type(target) {
                return self.is_array_type_related_to(source, target, relation);
            }

            if self.is_tuple_type(source) && self.is_tuple_type(target) {
                return self.is_tuple_type_related_to(source, target, relation);
            }

            if let Some(result) = self.generic_type_reference_related_to(source, target, relation) {
                if result.is_true() {
                    return true;
                }
                if result.is_false() {
                    return false;
                }

            }
            return self.is_object_type_related_to(source, target, relation);
        }

        if s.contains(TypeFlags::TypeParameter)
            && t.contains(TypeFlags::TypeParameter)
            && let (Some(ss), Some(ts)) = (&source.symbol, &target.symbol)
            && Arc::ptr_eq(ss, ts)
        {
            return true;
        }

        if t.contains(TypeFlags::TypeParameter) {

            if let Some(constraint) = self.get_constraint_of_type_parameter(target) {

                if self.is_type_related_to(source, &constraint, relation) {
                    return true;
                }
            }
        }

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

        if s.contains(TypeFlags::Conditional) {
            let resolved = match self.get_resolved_type_of_conditional_type(source) {
                Some(resolved) => Some(resolved),

                None => self.resolve_conditional_type(source),
            };
            if let Some(resolved) = resolved {
                if self.is_type_related_to(&resolved, target, relation) {
                    return true;
                }
            }
        }
        if t.contains(TypeFlags::Conditional) {

            let resolved = match self.get_resolved_type_of_conditional_type(target) {
                Some(resolved) => Some(resolved),
                None => self.resolve_conditional_type(target),
            };
            if let Some(resolved) = resolved {
                if self.is_type_related_to(source, &resolved, relation) {
                    return true;
                }

                if !type_contains_type_parameter(&resolved) {
                    return false;
                }
            }

            if let Some(result) = self.conditional_type_related_to(source, target, relation) {
                if result.is_true() {
                    return true;
                }
                if result.is_false() {
                    return false;
                }

            }
        }

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

    fn is_array_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);

        if source_args.is_empty() || target_args.is_empty() {

            return self.is_object_type_related_to(source, target, relation);
        }

        let source_elem = &source_args[0];
        let target_elem = &target_args[0];
        self.is_type_related_to(source_elem, target_elem, relation)
    }

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

        let min_len = source_tuple
            .element_infos
            .len()
            .min(target_tuple.element_infos.len());
        for i in 0..min_len {
            let source_elem = &source_tuple.element_infos[i];
            let target_elem = &target_tuple.element_infos[i];

            let source_type = self.get_tuple_element_type(source, i);
            let target_type = self.get_tuple_element_type(target, i);

            if let (Some(st), Some(tt)) = (source_type, target_type) {
                if !self.is_type_related_to(&st, &tt, relation) {
                    return false;
                }
            }

            if !self.is_element_flags_compatible(source_elem.flags, target_elem.flags, relation) {
                return false;
            }
        }

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

    pub(crate) fn get_tuple_element_type(&self, t: &Arc<Type>, index: usize) -> Option<Arc<Type>> {
        match &t.data {
            TypeData::Tuple(tuple) => {

                tuple
                    .element_infos
                    .get(index)
                    .and_then(|info| info.type_.clone())
            }
            _ => None,
        }
    }

    fn is_element_flags_compatible(
        &self,
        source: ElementFlags,
        target: ElementFlags,
        _relation: RelationKind,
    ) -> bool {

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

    fn is_union_or_intersection_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let s = source.flags;
        let t = target.flags;

        if s.contains(TypeFlags::Union) {

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

    fn some_type_related_to_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = source.as_union_or_intersection() {

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

    fn each_type_related_to_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = source.as_union_or_intersection() {
            let save_len = self.relater_error_chain.len();
            let mut any_failed = false;
            let mut failed_nullish: Option<Arc<Type>> = None;
            let mut first_failed: Option<Arc<Type>> = None;
            for t in &ui.types {
                if !self.is_type_related_to(t, target, relation) {
                    any_failed = true;
                    if first_failed.is_none() {
                        first_failed = Some(Arc::clone(t));
                    }
                    if t.flags.contains(TypeFlags::Undefined) {

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

                if self.relater_chain_active && self.speculation_depth == 0 {
                    self.relater_error_chain.truncate(save_len);
                    if let Some(t) = failed_nullish {
                        let member_str = self.type_to_string(&t);
                        let target_str = self.type_to_string(target);
                        self.relater_report_error(
                            crate::diagnostics::messages_generated::
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                            vec![member_str, target_str],
                        );
                    } else if let Some(t) = first_failed {
                        self.is_type_related_to(&t, target, relation);
                        let target_str = self.type_to_string(target);

                        let head_source =
                            if !self.type_could_have_top_level_singleton_types(target)
                                && (crate::checker::is_fresh_literal_type(&t)
                                    || t.flags.intersects(TYPE_FLAGS_LITERAL))
                            {
                                let base = self.get_base_type_of_literal_type_for_display(&t);
                                self.type_to_string(&base)
                            } else {
                                self.type_to_string(&t)
                            };

                        let mut suppress = false;
                        if let Some(entry) = self.relater_error_chain.last() {
                            let m = entry.message;
                            let a = &entry.args;
                            suppress = if m
                                == crate::diagnostics::messages_generated::
                                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2
                            {
                                a.len() == 3 && a[1] == head_source && a[2] == target_str
                            } else if m
                                == crate::diagnostics::messages_generated::
                                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2
                                || m
                                    == crate::diagnostics::messages_generated::
                                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE
                            {
                                a.len() >= 2 && a[0] == head_source && a[1] == target_str
                            } else if m
                                == crate::diagnostics::messages_generated::
                                    THE_TYPE_0_IS_READONLY_AND_CANNOT_BE_ASSIGNED_TO_THE_MUTABLE_TYPE_1
                            {
                                a.len() == 2 && a[0] == head_source && a[1] == target_str
                            } else {
                                false
                            };
                        }
                        if !suppress {
                            let msg = if head_source == target_str {
                                crate::diagnostics::messages_generated::
                                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_TWO_DIFFERENT_TYPES_WITH_THIS_NAME_EXIST_BUT_THEY_ARE_UNRELATED
                            } else {
                                crate::diagnostics::messages_generated::
                                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1
                            };
                            self.relater_report_error(msg, vec![head_source, target_str]);
                        }
                    }
                }
                return false;
            }
            return true;
        }
        false
    }

    fn type_related_to_some_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = target.as_union_or_intersection() {

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

            if self.relater_chain_active
                && self.speculation_depth == 0
                && let Some(best_t) = self.get_best_matching_type_for_error(source, target)
            {
                self.relater_error_chain.truncate(save_len);
                self.is_type_related_to(source, &best_t, relation);
                let source_str = self.type_to_string(source);
                let target_str = self.type_to_string(&best_t);
                let msg = if source_str == target_str {
                    crate::diagnostics::messages_generated::
                        TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_TWO_DIFFERENT_TYPES_WITH_THIS_NAME_EXIST_BUT_THEY_ARE_UNRELATED
                } else {
                    crate::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1
                };
                self.relater_report_error(msg, vec![source_str, target_str]);
            }
        }
        false
    }

    fn get_best_matching_type_for_error(
        &self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let ui = target.as_union_or_intersection()?;

        if source
            .object_flags
            .intersects(ObjectFlags::Reference | ObjectFlags::Anonymous)
        {
            for t in &ui.types {
                if !t.flags.contains(TypeFlags::Object) {
                    continue;
                }
                let overlap = source.object_flags & t.object_flags;
                if overlap.contains(ObjectFlags::Reference)
                    && source
                        .target()
                        .zip(t.target())
                        .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
                {
                    return Some(Arc::clone(t));
                }
                if overlap.contains(ObjectFlags::Anonymous)
                    && source
                        .alias
                        .as_ref()
                        .and_then(|a| a.symbol.as_ref())
                        .zip(t.alias.as_ref().and_then(|a| a.symbol.as_ref()))
                        .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
                {
                    return Some(Arc::clone(t));
                }
            }
        }

        if source.object_flags.contains(ObjectFlags::ObjectLiteral)
            && ui.types.iter().any(|t| self.is_array_like_type(t))
        {
            if let Some(t) = ui
                .types
                .iter()
                .find(|t| !self.is_array_like_type(t))
            {
                return Some(Arc::clone(t));
            }
        }

        if let Some(s) = source.as_structured() {
            for kind in [false , true ] {
                let has = if kind {
                    !s.construct_signatures().is_empty()
                } else {
                    !s.call_signatures().is_empty()
                };
                if has
                    && let Some(t) = ui.types.iter().find(|t| {
                        t.as_structured().is_some_and(|ts| {
                            if kind {
                                !ts.construct_signatures().is_empty()
                            } else {
                                !ts.call_signatures().is_empty()
                            }
                        })
                    })
                {
                    return Some(Arc::clone(t));
                }
            }
        }
        None
    }

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

        }

        let mut missing_props: Vec<String> = Vec::new();

        let source_is_bare_array = (self.is_array_type(source)
            || source.object_flags.contains(ObjectFlags::EvolvingArray))
            && source_struct.members.is_empty();
        for target_prop in &target_struct.properties {

            let source_declares_locally = source_struct.members.get(&target_prop.name).is_some();
            let source_prop = match source_struct.members.get(&target_prop.name) {
                Some(p) => Arc::clone(p),
                None => {
                    if source_is_bare_array
                        && let Some(p) = self.declared_array_member_symbol(&target_prop.name)
                    {
                        p
                    } else {

                        if target_prop.flags.contains(SymbolFlags::Optional) {
                            continue;
                        }
                        missing_props.push(target_prop.name.clone());
                        continue;
                    }
                }
            };

            if target_prop.name.starts_with('[')
                || (!source_declares_locally
                    && self
                        .global_interface_member_symbol("Object", &target_prop.name)
                        .is_some())
            {
                continue;
            }

            {
                let src_mod =
                    crate::checker::exports::get_declaration_modifier_flags_from_symbol(&source_prop);
                let tgt_mod =
                    crate::checker::exports::get_declaration_modifier_flags_from_symbol(target_prop);
                if src_mod.intersects(ModifierFlags::Private)
                    || tgt_mod.intersects(ModifierFlags::Private)
                {

                    let decl_of = |s: &Arc<crate::ast::Symbol>| {
                        s.value_declaration
                            .clone()
                            .or_else(|| s.declarations.first().cloned())
                    };
                    let same_decl = Arc::ptr_eq(&source_prop, target_prop)
                        || match (decl_of(&source_prop), decl_of(target_prop)) {
                            (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
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

            let source_type = if source_is_bare_array {
                self.instantiate_array_member_type(source, &source_prop)
                    .unwrap_or_else(|| self.get_type_of_symbol(&source_prop))
            } else {
                self.substituted_member_type_of(source, &source_prop)
            };

            let source_type = self.erase_bare_generic_params(source, &source_type);
            let target_type = self.substituted_member_type_of(target, target_prop);
            let target_type = self.erase_bare_generic_params(target, &target_type);
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

        if !self.is_call_signatures_related_to(source, target, relation) {
            return false;
        }

        if !self.is_construct_signatures_related_to(source, target, relation) {
            return false;
        }

        if !self.is_index_signatures_related_to(source, target, relation) {
            return false;
        }

        true
    }

    fn is_simple_type_identical_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        match (&source.data, &target.data) {
            (TypeData::Intrinsic(s), TypeData::Intrinsic(t)) => {
                s.intrinsic_name == t.intrinsic_name
            }
            (TypeData::Literal(s), TypeData::Literal(t)) => s.value == t.value,
            (TypeData::TypeParameter(s), TypeData::TypeParameter(t)) => {
                s.is_this_type == t.is_this_type
            }

            (TypeData::IndexedAccess(s), TypeData::IndexedAccess(t)) => {
                match (&s.object_type, &t.object_type, &s.index_type, &t.index_type) {
                    (Some(so), Some(to), Some(si), Some(ti)) => {
                        self.is_type_identical_to(so, to) && self.is_type_identical_to(si, ti)
                    }
                    _ => Arc::ptr_eq(source, target),
                }
            }

            (TypeData::Index(s), TypeData::Index(t)) => match (&s.target, &t.target) {
                (Some(so), Some(to)) => self.is_type_identical_to(so, to),
                _ => Arc::ptr_eq(source, target),
            },
            _ => {

                source.flags == target.flags
            }
        }
    }

    pub fn is_simple_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let s = source.flags;
        let t = target.flags;

        if t.contains(TypeFlags::Any) || s.contains(TypeFlags::Never) {
            return true;
        }

        if t.contains(TypeFlags::Unknown)
            && !(relation == RelationKind::StrictSubtype && s.contains(TypeFlags::Any))
        {
            return true;
        }

        if t.contains(TypeFlags::Never) {
            return false;
        }

        if s.intersects(TYPE_FLAGS_STRING_LIKE) && t.contains(TypeFlags::String) {
            return true;
        }

        if s.contains(TypeFlags::StringLiteral)
            && s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::StringLiteral)
            && !t.contains(TypeFlags::EnumLiteral)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        if s.intersects(TYPE_FLAGS_LITERAL)
            && t.intersects(TYPE_FLAGS_LITERAL)
            && (s & TYPE_FLAGS_LITERAL) == (t & TYPE_FLAGS_LITERAL)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        if s.intersects(TYPE_FLAGS_NUMBER_LIKE) && t.contains(TypeFlags::Number) {
            return true;
        }

        if s.contains(TypeFlags::NumberLiteral)
            && s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::NumberLiteral)
            && !t.contains(TypeFlags::EnumLiteral)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        if s.intersects(TYPE_FLAGS_BIG_INT_LIKE) && t.contains(TypeFlags::BigInt) {
            return true;
        }

        if s.intersects(TYPE_FLAGS_BOOLEAN_LIKE) && t.contains(TypeFlags::Boolean) {
            return true;
        }

        if s.intersects(TYPE_FLAGS_ES_SYMBOL_LIKE) && t.contains(TypeFlags::ESSymbol) {
            return true;
        }

        if s.contains(TypeFlags::Enum)
            && t.contains(TypeFlags::Enum)
            && self.is_enum_type_related_to(source, target)
        {
            return true;
        }

        if s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::EnumLiteral)
            && s.intersects(TYPE_FLAGS_LITERAL)
            && t.intersects(TYPE_FLAGS_LITERAL)
            && self.literal_values_equal(source, target)
            && self.is_enum_type_related_to(source, target)
        {
            return true;
        }

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

        if s.contains(TypeFlags::Object)
            && t.contains(TypeFlags::NonPrimitive)
            && !(relation == RelationKind::StrictSubtype)
        {
            return true;
        }

        if s.contains(TypeFlags::NonPrimitive)
            && t.contains(TypeFlags::NonPrimitive)
            && source.intrinsic_name() == target.intrinsic_name()
        {
            return true;
        }

        if relation == RelationKind::Assignable || relation == RelationKind::Comparable {

            if s.contains(TypeFlags::Any) {
                return true;
            }

            if s.contains(TypeFlags::Number)
                && (t.contains(TypeFlags::Enum)
                    || (t.contains(TypeFlags::NumberLiteral) && t.contains(TypeFlags::EnumLiteral)))
            {
                return true;
            }

            if s.contains(TypeFlags::NumberLiteral)
                && !s.contains(TypeFlags::EnumLiteral)
                && (t.contains(TypeFlags::Enum)
                    || (t.contains(TypeFlags::NumberLiteral)
                        && t.contains(TypeFlags::EnumLiteral)
                        && self.literal_values_equal(source, target)))
            {
                return true;
            }

            if self.is_unknown_like_union_type(target) {
                return true;
            }
        }

        false
    }

    fn literal_values_equal(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        match (&a.data, &b.data) {
            (TypeData::Literal(la), TypeData::Literal(lb)) => la.value == lb.value,
            _ => false,
        }
    }

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

        if Arc::ptr_eq(source_parent, target_parent) {
            return true;
        }

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

        if let Some(entry) = self.enum_relation.get(&key).copied() {
            if entry != RelationComparisonResult::None {
                return entry.contains(RelationComparisonResult::Succeeded);
            }
        }

        let source_type = self.get_type_of_symbol(source_parent);
        let target_type = self.get_type_of_symbol(target_parent);
        let source_properties = self.get_properties_of_type(&source_type);

        for source_prop in source_properties {
            if !source_prop.flags.contains(SymbolFlags::EnumMember) {
                continue;
            }
            let Some(target_prop) = self.get_property_of_type(&target_type, &source_prop.name)
            else {

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

                    if sv.is_some() && tv.is_some() {
                        self.enum_relation
                            .insert(key, RelationComparisonResult::Failed);
                        return false;
                    }

                    let source_is_string = matches!(sv, Some(EvalValue::String(_)));
                    let target_is_string = matches!(tv, Some(EvalValue::String(_)));
                    if source_is_string || target_is_string {
                        self.enum_relation
                            .insert(key, RelationComparisonResult::Failed);
                        return false;
                    }

                }
            }
        }

        self.enum_relation
            .insert(key, RelationComparisonResult::Succeeded);
        true
    }

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

    fn is_empty_anonymous_object_type(&self, t: &Arc<Type>) -> bool {
        if !t.object_flags.contains(ObjectFlags::Anonymous) {
            return false;
        }
        if t.object_flags.contains(ObjectFlags::MembersResolved) {

            return self.structured_type_is_empty(t);
        }

        if let Some(sym) = t.symbol.as_ref() {
            if sym.flags.contains(SymbolFlags::TypeLiteral) {
                return self.get_properties_of_type(t).is_empty();
            }
        }
        false
    }

    fn structured_type_is_empty(&self, t: &Arc<Type>) -> bool {
        self.get_properties_of_type(t).is_empty()
    }

    fn is_index_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {

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
            return true;
        }

        for target_index in target_indexes {
            let target_key = &target_index.key_type;
            let target_value = &target_index.value_type;

            let mut found_match = false;
            for source_index in source_indexes {
                let source_key = &source_index.key_type;
                let source_value = &source_index.value_type;

                let key_match = match (target_key, source_key) {
                    (Some(tk), Some(sk)) => self.is_type_related_to(sk, tk, relation),
                    (None, _) => true,
                    (_, None) => false,
                };

                if !key_match {
                    continue;
                }

                let value_match = match (target_value, source_value) {
                    (Some(tv), Some(sv)) => self.is_type_related_to(sv, tv, relation),

                    (None, _) => true,
                    (_, None) => false,
                };

                if value_match {
                    found_match = true;
                    break;
                }
            }

            if !found_match {

                let result = self.members_related_to_index_info(source, target_index, relation);
                if result.is_false() {

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

    pub fn compare_signatures_related(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        check_mode: SignatureCheckMode,
        relation: RelationKind,
    ) -> Ternary {

        if Arc::ptr_eq(source, target) {
            return Ternary::True;
        }

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

        let strict_variance = !check_mode.contains(SignatureCheckMode::Callback)
            && self.strict_function_types
            && !self.signature_is_method_or_constructor(&target);

        let mut result = Ternary::True;

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

            if Arc::ptr_eq(&source_type, &target_type)
                && !check_mode.contains(SignatureCheckMode::StrictArity)
            {
                continue;
            }

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

                related = self.compare_signatures_related(
                    target_sig.as_ref().unwrap(),
                    source_sig.as_ref().unwrap(),
                    callback_mode,
                    relation,
                );
            } else {

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

        if !check_mode.contains(SignatureCheckMode::IgnoreReturnTypes) {
            let target_return = self.get_non_circular_return_type_of_signature(&target);

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
            (Some(_s), None) => Ternary::True,
            (Some(s), Some(t)) => self.compare_types(s.clone(), t.clone(), relation, false),
            (None, Some(_)) => Ternary::False,
        }
    }

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

    fn signature_is_method_or_constructor(&self, sig: &Arc<Signature>) -> bool {
        let Some(decl) = sig.declaration.as_ref() else {
            return false;
        };
        matches!(
            decl.kind,
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature | SyntaxKind::Constructor
        )
    }

    pub fn signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        kind: SignatureKind,
        relation: RelationKind,
    ) -> Ternary {

        if Arc::ptr_eq(source, &self.any_function_type()) {
            return Ternary::True;
        }

        if Arc::ptr_eq(target, &self.any_function_type()) {
            return Ternary::False;
        }

        let source_sigs = self.get_signatures_of_type(source, kind);
        let target_sigs = self.get_signatures_of_type(target, kind);

        if kind == SignatureKind::Construct && !source_sigs.is_empty() && !target_sigs.is_empty() {

        }

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

            if source_sigs.len() != target_sigs.len() {

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
                false,
                false,
                false,
            );
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

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

    pub fn get_min_argument_count(&mut self, sig: &Arc<Signature>) -> usize {

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

    pub fn get_type_at_position(&mut self, sig: &Arc<Signature>, pos: usize) -> Arc<Type> {
        self.try_get_type_at_position(sig, pos)
            .unwrap_or_else(|| self.any_type())
    }

    pub fn try_get_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Option<Arc<Type>> {

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

            if is_tuple_type(&rest_type) {
                if let TypeData::Tuple(t) = &rest_type.data {
                    let index = pos - param_count;
                    let has_variadic = t.combined_flags.contains(ElementFlags::Variadic);
                    if index < t.fixed_length || has_variadic {

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

    pub fn get_rest_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Option<Arc<Type>> {
        let parameter_count = self.get_parameter_count(sig);
        if pos >= parameter_count.saturating_sub(1) {

            return self.get_effective_rest_type(sig);
        }
        None
    }

    pub fn get_effective_rest_type(&mut self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        if !sig.has_rest_parameter() {
            return None;
        }
        if let Some(overrides) = &sig.instantiated_parameter_types {
            return overrides.last().cloned();
        }
        let last = sig.parameters.last()?;
        let rest_type = self.get_type_of_symbol(last);

        Some(rest_type)
    }

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

        if is_tuple_type(&rest_type) {
            return Some(rest_type);
        }

        if self.is_array_type(&rest_type) {
            return self.get_type_arguments(&rest_type).into_iter().next();
        }
        Some(rest_type)
    }

    pub(crate) fn get_array_element_type_of(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if self.is_array_type(t) {
            return Some(self.get_array_element_type(t));
        }
        None
    }

    pub fn is_top_signature(&mut self, sig: &Arc<Signature>) -> bool {
        if !sig.type_parameters.is_empty() {
            return false;
        }

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

    pub fn get_this_type_of_signature(&self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        let this_param = sig.this_parameter.as_ref()?;
        let links = self.value_symbol_links.get(this_param)?;
        links.resolved_type.clone()
    }

    pub fn get_non_circular_return_type_of_signature(&self, sig: &Arc<Signature>) -> Arc<Type> {
        self.get_return_type_of_signature(sig)
            .unwrap_or_else(|| self.any_type())
    }

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

    pub fn get_signature_instantiation(
        &mut self,
        sig: &Arc<Signature>,
        type_args: &[Arc<Type>],
    ) -> Arc<Signature> {
        if type_args.is_empty() || sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }

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

        if type_arguments
            .iter()
            .zip(sig.type_parameters.iter())
            .all(|(arg, param)| Arc::ptr_eq(arg, param))
        {
            return Arc::clone(sig);
        }
        self.get_signature_instantiation(sig, &type_arguments)
    }

    pub fn get_base_constraint_or_type(&self, t: &Arc<Type>) -> Arc<Type> {
        self.get_base_constraint_of_type(t)
            .or_else(|| self.get_constraint_of_type_parameter(t))
            .unwrap_or_else(|| Arc::clone(t))
    }

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

    pub fn get_single_call_signature(&self, t: &Arc<Type>) -> Option<Arc<Signature>> {
        let sigs = self.get_signatures_of_type(t, SignatureKind::Call);
        if sigs.len() == 1 {
            sigs.into_iter().next()
        } else {
            None
        }
    }

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

    pub fn is_generic_type(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::TypeParameter) {
            return true;
        }
        t.types()
            .map(|ts| ts.iter().any(type_contains_type_parameter))
            .is_some()
    }

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

        if index_type.flags.contains(TypeFlags::Union) {
            let constituents = index_type.types()?.to_vec();
            let mut resolved = Vec::with_capacity(constituents.len());
            for c in &constituents {
                resolved.push(self.try_get_indexed_access_type(object_type, c, access_flags)?);
            }
            return Some(self.get_union_type(resolved));
        }

        if object_type.flags.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_constraint_of_type_parameter(object_type)?;
            return self.try_get_indexed_access_type(&constraint, index_type, access_flags);
        }
        if let Some(structured) = object_type.as_structured() {

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

            if index_type.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral)
                && !access_flags.contains(AccessFlags::NoIndexSignatures)
            {
                return self.lookup_index_signature_value(structured, index_type);
            }
        }
        None
    }

    pub fn signature_to_string(&mut self, sig: &Arc<Signature>) -> String {
        let params: Vec<String> = sig.parameters.iter().map(|p| p.name.clone()).collect();
        let return_type = self.get_return_type_of_signature(sig);
        let return_str = match return_type {
            Some(t) => self.type_to_string(&t),
            None => "void".to_string(),
        };
        format!("({}) => {}", params.join(", "), return_str)
    }

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

            return true;
        }
        if source_sigs.is_empty() {

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

    pub(crate) fn signature_display_colon(&mut self, sig: &Arc<Signature>, prefix: &str) -> String {
        self.signature_display_sep(sig, prefix, ": ")
    }

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

        let prefix = if sig.flags.contains(crate::checker::types::SignatureFlags::Abstract)
            && prefix.starts_with("new")
        {
            format!("abstract {prefix}")
        } else {
            prefix.to_string()
        };
        format!("{prefix}{tp}({}){sep}{}", params.join(", "), self.type_to_string(&ret))
    }

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

            return true;
        }
        if source_sigs.is_empty() {

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

            let source_sigs = self.get_signatures_of_type(source, SignatureKind::Construct);
            let target_sigs = self.get_signatures_of_type(target, SignatureKind::Construct);
            if let (Some(ss), Some(ts)) = (source_sigs.first(), target_sigs.first())
                && ss.min_argument_count.max(0) as usize > ts.parameters.len()
            {

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

    fn is_function_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {

        if !self.is_call_signatures_related_to(source, target, relation) {
            return false;
        }
        if !self.is_construct_signatures_related_to(source, target, relation) {
            return false;
        }
        true
    }
}
