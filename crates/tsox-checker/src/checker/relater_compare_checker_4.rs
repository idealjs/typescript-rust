#![allow(unused_imports)]

use crate::checker::relater_compare::*;
use tsox_frontend::ast::SyntaxKind;

impl Checker {
    // Go findMostOverlappyType：以 keyof 交集的 unit 数量选最佳匹配的联合成分
    pub fn find_most_overlappy_type(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let ui = union_target.as_union_or_intersection()?;
        let non_instantiable_primitive = TYPE_FLAGS_PRIMITIVE
            | TypeFlags::TypeParameter
            | TypeFlags::IndexedAccess
            | TypeFlags::Conditional;
        if source.flags.intersects(non_instantiable_primitive) {
            return None;
        }
        let mut best: Option<Arc<Type>> = None;
        let mut matching_count = 0usize;
        for t in &ui.types {
            if t.flags.intersects(non_instantiable_primitive) {
                continue;
            }
            let source_idx = self.get_index_type(source);
            let target_idx = self.get_index_type(t);
            if source_idx.flags.contains(TypeFlags::Index)
                && source_idx.flags == target_idx.flags
            {
                return Some(Arc::clone(t));
            }
            let source_keys = self.string_literal_values(&source_idx);
            let target_keys = self.string_literal_values(&target_idx);
            let length = source_keys
                .iter()
                .filter(|k| target_keys.contains(k))
                .count();
            if !source_keys.is_empty() && length == source_keys.len() && length == target_keys.len()
            {
                return Some(Arc::clone(t));
            }
            if length >= matching_count && length > 0 {
                best = Some(Arc::clone(t));
                matching_count = length;
            }
        }
        best
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
        // Go getTypeNamesForErrorDisplay：两侧显示名相同（同名不同型）时
        // 改用全限定形式消歧
        let left_str = self.get_type_name_for_error_display(left);
        let right_str = self.get_type_name_for_error_display(right);
        if left_str == right_str {
            return (
                self.fully_qualified_type_string(left),
                self.fully_qualified_type_string(right),
            );
        }
        (left_str, right_str)
    }

    // Go typeToString(TypeFormatFlags.UseFullyQualifiedType)：沿符号 parent
    // 链拼点分全限定名；外部模块文件符号输出 import("name")，ambient 模块
    // 名去引号
    pub fn fully_qualified_type_string(&mut self, t: &Arc<Type>) -> String {
        if t.flags.contains(TypeFlags::Union)
            && let Some(ui) = t.as_union_or_intersection()
            && ui.types.len() > 1
            && ui
                .types
                .iter()
                .all(|c| c.flags.contains(TypeFlags::EnumLiteral))
        {
            if let Some(sym) = &t.symbol {
                return self.symbol_fqn(sym);
            }
        }
        if let Some(alias) = &t.alias
            && let Some(alias_sym) = &alias.symbol
        {
            let name = self.symbol_fqn(alias_sym);
            if !alias.type_arguments.is_empty() {
                let rendered: Vec<String> = alias
                    .type_arguments
                    .iter()
                    .map(|a| self.type_to_string(a))
                    .collect();
                return format!("{name}<{}>", rendered.join(", "));
            }
            return name;
        }
        if let Some(sym) = &t.symbol {
            let name = self.symbol_fqn(sym);
            if let Some(args) = self.reference_type_arguments(t) {
                let rendered: Vec<String> = args.iter().map(|a| self.type_to_string(a)).collect();
                return format!("{name}<{}>", rendered.join(", "));
            }
            return name;
        }
        self.type_to_string(t)
    }

    fn symbol_fqn(&self, sym: &Arc<Symbol>) -> String {
        let mut parts: Vec<String> = Vec::new();
        let mut cur = Some(Arc::clone(sym));
        while let Some(s) = cur {
            if s.declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::SourceFile)
            {
                if let Some(decl) = s
                    .declarations
                    .iter()
                    .find(|d| d.kind == SyntaxKind::SourceFile)
                    && let Some(sf) = self.get_source_file_of_node(decl)
                    && (sf.external_module_indicator.is_some()
                        || sf.common_js_module_indicator.is_some())
                {
                    let module = crate::checker::nodebuilder::module_specifier_of_name(&s.name);
                    let suffix = parts.iter().rev().cloned().collect::<Vec<_>>().join(".");
                    return if suffix.is_empty() {
                        format!("import(\"{module}\")")
                    } else {
                        format!("import(\"{module}\").{suffix}")
                    };
                }
                break;
            }
            if s.declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::ModuleDeclaration)
                && s.name.starts_with('"')
            {
                let module = s.name.trim_matches('"').to_string();
                let suffix = parts.iter().rev().cloned().collect::<Vec<_>>().join(".");
                return if suffix.is_empty() {
                    module
                } else {
                    format!("{module}.{suffix}")
                };
            }
            parts.push(s.name.clone());
            cur = s.parent();
        }
        parts.reverse();
        parts.join(".")
    }

    fn reference_type_arguments(&self, t: &Arc<Type>) -> Option<Vec<Arc<Type>>> {
        match &t.data {
            TypeData::Object(o) if !o.type_arguments.is_empty() => {
                Some(o.type_arguments.iter().cloned().collect())
            }
            _ => None,
        }
    }

    pub fn get_type_name_for_error_display(&mut self, t: &Arc<Type>) -> String {
        self.type_to_string(t)
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

    pub fn get_type_parameter_modifiers(
        &mut self,
        _tp: &Arc<Type>,
    ) -> tsox_frontend::ast::ModifierFlags {
        tsox_frontend::ast::ModifierFlags::empty()
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
        _node: &Arc<tsox_frontend::ast::Node>,
        _index: usize,
        _element_flags: ElementFlags,
    ) -> String {
        String::new()
    }

    pub fn get_nameable_declaration_at_position(
        &mut self,
        _signature: &Arc<Signature>,
        _pos: usize,
    ) -> Option<Arc<tsox_frontend::ast::Node>> {
        None
    }

    pub fn is_valid_declaration_for_tuple_label(
        &mut self,
        _d: &Arc<tsox_frontend::ast::Node>,
    ) -> bool {
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
        _node: &Arc<tsox_frontend::ast::Node>,
        _signature: &Arc<Signature>,
    ) -> Option<Box<TypePredicate>> {
        None
    }
}
