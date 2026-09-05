#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Symbol, SymbolFlags};

use crate::checker::checker::Checker;
use crate::checker::types::*;

use super::*;

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

    pub fn check_type_assignable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<crate::ast::Node>>,
        _head_message: Option<&crate::diagnostics::Message>,
    ) -> bool {

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

        self.is_type_assignable_to(source, target)
    }

    pub fn check_type_comparable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<crate::ast::Node>>,
        _head_message: Option<&crate::diagnostics::Message>,
    ) -> bool {

        self.is_type_comparable_to(source, target)
    }

    pub fn check_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        _error_node: Option<&Arc<crate::ast::Node>>,
    ) -> bool {

        self.is_type_related_to(source, target, relation)
    }

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

            let target_elem = if self.is_array_type(target) {
                self.get_array_element_type(target)
            } else if self.is_tuple_type(target) {
                match self.get_tuple_element_type(target, i) {
                    Some(t) => t,
                    None => continue,
                }
            } else {

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

        {
            let sp = source.id;
            let tp = target.id;
            if source.flags.contains(TypeFlags::Object)
                && target.flags.contains(TypeFlags::Object)
                && (self.degraded_type_ptrs.contains(&sp) || self.degraded_type_ptrs.contains(&tp))
            {
                return true;
            }
        }

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

        if let Some(expr) = expr
            && self.elaborate_error(expr, source, target, relation, diagnostic_output.as_deref_mut())
        {
            self.relater_chain_active = was_active;
            self.relater_error_chain = saved_chain;
            return false;
        }

        self.try_elaborate_primitive_and_object(source, target);

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

            self.push_relation_head_with_tp_note(
                source,
                &displayed_target,
                head,
                vec![head_source, head_target],
            );
        }

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

    pub(crate) fn get_base_type_of_literal_type_for_display(&mut self, t: &Arc<Type>) -> Arc<Type> {
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

    pub fn is_weak_type(&mut self, t: &Arc<Type>) -> bool {

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

    pub fn get_mapped_target_with_symbol(&self, t: &Arc<Type>) -> Arc<Type> {

        Arc::clone(t)
    }

    pub fn has_matching_recursion_identity(&self, t: &Arc<Type>, identity: &Arc<Type>) -> bool {
        Arc::ptr_eq(t, identity)
    }

    pub fn get_best_matching_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _is_related_to: &dyn Fn(&Arc<Type>, &Arc<Type>) -> Ternary,
    ) -> Option<Arc<Type>> {

        let _ = (source, target);
        None
    }

    pub fn find_matching_type_reference_or_type_alias_reference(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {

        let _ = (source, union_target);
        None
    }

    pub fn find_best_type_for_invokable(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
        _kind: SignatureKind,
    ) -> Option<Arc<Type>> {

        let _ = (source, union_target);
        None
    }

    pub fn find_most_overlappy_type(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {

        let _ = (source, union_target);
        None
    }

    pub fn find_best_type_for_object_literal(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {

        let _ = (source, union_target);
        None
    }

    pub fn should_report_unmatched_property_error(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> bool {

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

        let _ = (source, target);
        None
    }

    pub fn find_discriminant_properties(
        &mut self,
        _source_properties: &[Arc<Symbol>],
        _target: &Arc<Type>,
    ) -> Vec<Arc<Symbol>> {

        Vec::new()
    }

    pub fn is_discriminant_property(&mut self, _t: &Arc<Type>, _name: &str) -> bool {

        false
    }

    pub fn get_matching_union_constituent_for_type(
        &mut self,
        _union_type: &Arc<Type>,
        _t: &Arc<Type>,
    ) -> Option<Arc<Type>> {

        None
    }

    pub fn get_key_property_name(&mut self, t: &Arc<Type>) -> Option<String> {

        let _ = t;
        None
    }

    pub fn get_constituent_type_for_key_type(
        &mut self,
        _t: &Arc<Type>,
        _key_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {

        None
    }

    pub fn filter_primitives_if_contains_non_primitive(
        &mut self,
        union_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {

        let _ = union_type;
        None
    }

    pub fn get_type_names_for_error_display(
        &mut self,
        left: &Arc<Type>,
        right: &Arc<Type>,
    ) -> (String, String) {

        (
            self.get_type_name_for_error_display(left),
            self.get_type_name_for_error_display(right),
        )
    }

    pub fn get_type_name_for_error_display(&mut self, t: &Arc<Type>) -> String {

        crate::checker::utilities::type_to_string(t)
    }

    pub fn symbol_value_declaration_is_context_sensitive(&mut self, _symbol: &Arc<Symbol>) -> bool {

        false
    }

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

    pub fn get_alias_variances(&mut self, _symbol: &Arc<Symbol>) -> Vec<VarianceFlags> {

        Vec::new()
    }

    pub fn create_marker_type(
        &mut self,
        _symbol: &Arc<Symbol>,
        _source: &Arc<Type>,
        _target: &Arc<Type>,
    ) -> Option<Arc<Type>> {

        None
    }

    pub fn get_type_parameter_modifiers(&mut self, _tp: &Arc<Type>) -> crate::ast::ModifierFlags {

        crate::ast::ModifierFlags::empty()
    }

    pub fn has_covariant_void_argument(
        &mut self,
        _type_arguments: &[Arc<Type>],
        _variances: &[VarianceFlags],
    ) -> bool {

        false
    }

    pub fn is_signature_assignable_to(
        &mut self,
        _source: &Arc<Signature>,
        _target: &Arc<Signature>,
        _ignore_return_types: bool,
    ) -> bool {

        false
    }

    pub fn get_min_argument_count_ex(
        &mut self,
        sig: &Arc<Signature>,
        _flags: MinArgumentCountFlags,
    ) -> usize {

        sig.min_argument_count.max(0) as usize
    }

    pub fn get_parameter_name_at_position(
        &mut self,
        _signature: &Arc<Signature>,
        _pos: usize,
    ) -> String {

        String::new()
    }

    pub fn get_tuple_element_label(
        &mut self,
        _element_info: &TupleElementInfo,
        _rest_symbol: Option<&Arc<Symbol>>,
        _index: usize,
    ) -> String {

        String::new()
    }

    pub fn get_tuple_element_label_from_binding_element(
        &mut self,
        _node: &Arc<crate::ast::Node>,
        _index: usize,
        _element_flags: ElementFlags,
    ) -> String {

        String::new()
    }

    pub fn get_nameable_declaration_at_position(
        &mut self,
        _signature: &Arc<Signature>,
        _pos: usize,
    ) -> Option<Arc<crate::ast::Node>> {

        None
    }

    pub fn is_valid_declaration_for_tuple_label(&mut self, _d: &Arc<crate::ast::Node>) -> bool {

        false
    }

    pub fn slice_tuple_type(
        &mut self,
        _t: &Arc<Type>,
        _index: usize,
        _end_skip_count: usize,
    ) -> Option<Arc<Type>> {

        None
    }

    pub fn get_known_keys_of_tuple_type(&mut self, _t: &Arc<Type>) -> Option<Arc<Type>> {

        None
    }

    pub fn get_rest_array_type_of_tuple_type(&mut self, _t: &Arc<Type>) -> Option<Arc<Type>> {

        None
    }

    pub fn get_union_or_intersection_type_predicate(
        &mut self,
        _signatures: &[Arc<Signature>],
        _is_union: bool,
    ) -> Option<Box<TypePredicate>> {

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

        None
    }

    pub fn instantiate_type_predicate(
        &mut self,
        _predicate: &TypePredicate,
        _mapper: &Arc<TypeMapper>,
    ) -> Option<Box<TypePredicate>> {

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

        false
    }

    pub fn find_matching_signatures(
        &mut self,
        _signature_lists: &[Vec<Arc<Signature>>],
        _signature: &Arc<Signature>,
        _list_index: usize,
    ) -> Vec<Arc<Signature>> {

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

    pub fn get_effective_constraint_of_intersection(
        &mut self,
        _types: &[Arc<Type>],
        _target_is_union: bool,
    ) -> Option<Arc<Type>> {

        None
    }

    pub fn template_literal_types_definitely_unrelated(
        &mut self,
        _source: &TemplateLiteralTypeData,
        _target: &TemplateLiteralTypeData,
    ) -> bool {

        false
    }

    pub fn is_type_matched_by_template_literal_type(
        &mut self,
        _source: &Arc<Type>,
        _target: &TemplateLiteralTypeData,
        _compare_types: TypeComparer,
    ) -> bool {

        false
    }

    pub fn infer_types_from_template_literal_type(
        &mut self,
        _source: &Arc<Type>,
        _target: &TemplateLiteralTypeData,
    ) -> Vec<Arc<Type>> {

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

        false
    }

    pub fn is_member_of_string_mapping(
        &mut self,
        _source: &Arc<Type>,
        _target: &Arc<Type>,
    ) -> bool {

        false
    }

    pub fn apply_target_string_mapping_to_source(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> (Arc<Type>, Arc<Type>) {

        (Arc::clone(source), Arc::clone(target))
    }

    pub fn get_type_of_property_in_types(
        &mut self,
        _types: &[Arc<Type>],
        _name: &str,
    ) -> Option<Arc<Type>> {

        None
    }

    pub fn get_type_of_property_in_type(
        &mut self,
        _t: &Arc<Type>,
        _name: &str,
    ) -> Option<Arc<Type>> {

        None
    }

    pub fn is_type_subset_of_union(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {

        self.is_type_subset_of(source, target)
    }

    pub fn is_type_derived_from(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {

        self.is_type_assignable_to(source, target)
    }

    pub fn is_distribution_dependent(&mut self, _root: &ConditionalRoot) -> bool {

        false
    }
}
