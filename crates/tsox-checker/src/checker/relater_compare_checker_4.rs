#![allow(unused_imports)]

use crate::checker::relater_compare::*;
use tsox_frontend::ast::SyntaxKind;

pub const RELATION_REPORTS_UNMEASURABLE: u8 = 1 << 0;
pub const RELATION_REPORTS_UNRELIABLE: u8 = 1 << 1;

impl Checker {
    pub fn find_most_overlappy_type(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let ui = union_target.as_union_or_intersection()?;
        let excluded = TypeFlags::from_bits_truncate(
            TYPE_FLAGS_PRIMITIVE.bits()
                | TypeFlags::Index.bits()
                | TypeFlags::TemplateLiteral.bits()
                | TypeFlags::StringMapping.bits(),
        );
        if source.flags.intersects(excluded) {
            return None;
        }
        let mut best: Option<Arc<Type>> = None;
        let mut matching_count = 0usize;
        let source_idx = self.get_index_type(source);
        let source_ids = unit_member_ids(&source_idx);
        for t in &ui.types {
            if t.flags.intersects(excluded) {
                continue;
            }
            let target_idx = self.get_index_type(t);
            if Arc::ptr_eq(&source_idx, &target_idx)
                && source_idx.flags.contains(TypeFlags::Index)
            {
                return Some(Arc::clone(t));
            }
            let target_ids = unit_member_ids(&target_idx);
            let length = source_ids.intersection(&target_ids).count();
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
        if self.is_array_type(t)
            && !matches!(t.data, TypeData::Tuple(_))
            && let Some(elem) = self.get_element_type_of_array_type(t)
        {
            let inner = self.fully_qualified_type_string(&elem);
            let prefix = if t
                .object_flags
                .contains(crate::checker::types::ObjectFlags::IsReadonlyArray)
            {
                "readonly "
            } else {
                ""
            };
            return format!("{prefix}{inner}[]");
        }
        if let TypeData::Tuple(tuple) = &t.data {
            let rendered: Vec<String> = tuple
                .element_infos
                .iter()
                .map(|info| {
                    let inner = info
                        .type_
                        .as_ref()
                        .map(|e| self.fully_qualified_type_string(e))
                        .unwrap_or_default();
                    if info.flags.contains(ElementFlags::Rest) {
                        format!("...{inner}[]")
                    } else if info.flags.contains(ElementFlags::Optional) {
                        format!("{inner}?")
                    } else {
                        inner
                    }
                })
                .collect();
            let prefix = if tuple.readonly { "readonly " } else { "" };
            return format!("{prefix}[{}]", rendered.join(", "));
        }
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
            && !alias_sym.name.starts_with('\u{FE}')
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
        if let Some(sym) = &t.symbol && !sym.name.starts_with('\u{FE}') {
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
        if t.flags.contains(TypeFlags::Boolean) {
            return false;
        }
        if t.flags.intersects(TypeFlags::Union | TypeFlags::Intersection) {
            if let Some(members) = t.types() {
                return members
                    .iter()
                    .any(|m| self.type_could_have_top_level_singleton_types(m));
            }
            return false;
        }
        if t.flags.intersects(
            TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Conditional,
        ) {
            let constraint = if t.flags.contains(TypeFlags::TypeParameter) {
                self.get_constraint_of_type_parameter(t)
            } else {
                self.get_base_constraint_of_type(t)
            };
            if let Some(c) = constraint
                && !Arc::ptr_eq(&c, t)
            {
                return self.type_could_have_top_level_singleton_types(&c);
            }
        }
        t.flags.intersects(
            TYPE_FLAGS_UNIT | TypeFlags::TemplateLiteral | TypeFlags::StringMapping,
        )
    }

    pub(crate) fn new_marker_type_parameter(constraint: Option<Arc<Type>>) -> Arc<Type> {
        Arc::new(Type {
            flags: TypeFlags::TypeParameter,
            object_flags: ObjectFlags::None,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::TypeParameter(crate::checker::types::TypeParameterData {
                constrained: crate::checker::types::ConstrainedTypeData::default(),
                constraint,
                target: None,
                mapper: None,
                is_this_type: false,
                resolved_default_type: OnceLock::new(),
            }),
        })
    }

    pub(crate) fn marker_super(&self) -> Arc<Type> {
        Arc::clone(self.marker_super_type.get_or_init(|| Self::new_marker_type_parameter(None)))
    }

    pub(crate) fn marker_sub(&self) -> Arc<Type> {
        Arc::clone(self.marker_sub_type.get_or_init(|| {
            let sup = Arc::clone(
                self.marker_super_type
                    .get_or_init(|| Self::new_marker_type_parameter(None)),
            );
            Self::new_marker_type_parameter(Some(sup))
        }))
    }

    pub(crate) fn marker_other(&self) -> Arc<Type> {
        Arc::clone(self.marker_other_type.get_or_init(|| Self::new_marker_type_parameter(None)))
    }

    fn is_variance_marker(&self, t: &Arc<Type>) -> bool {
        self.marker_super_type.get().is_some_and(|m| Arc::ptr_eq(t, m))
            || self.marker_sub_type.get().is_some_and(|m| Arc::ptr_eq(t, m))
            || self.marker_other_type
                .get()
                .is_some_and(|m| Arc::ptr_eq(t, m))
    }

    fn any_variance_marker_in_types(&mut self, types: &[Arc<Type>], depth: usize) -> bool {
        types
            .iter()
            .any(|t| self.contains_variance_marker(t, depth + 1))
    }

    fn contains_variance_marker(&mut self, t: &Arc<Type>, depth: usize) -> bool {
        if depth > 100 {
            return false;
        }
        let mut children: Vec<Arc<Type>> = Vec::new();
        match &t.data {
            TypeData::TypeParameter(_) => return self.is_variance_marker(t),
            TypeData::Object(o) => children.extend(o.type_arguments.iter().cloned()),
            TypeData::Mapped(m) => {
                children.extend(m.object.type_arguments.iter().cloned());
                children.extend(m.type_parameter.iter().cloned());
                children.extend(m.constraint_type.iter().cloned());
                children.extend(m.name_type.iter().cloned());
                children.extend(m.template_type.iter().cloned());
                // 实例化壳（object.target+mapper，constraint_type 空）：marker
                // 驻留在按 mapper 实例化的惰性约束里（同 Go reportUnreliableMapper
                // 经 instantiateType 的遍历）
                if m.constraint_type.is_none() {
                    let shell = Arc::clone(t);
                    if let Some(resolved) = self.get_constraint_type_from_mapped_type(&shell) {
                        children.push(resolved);
                    }
                }
            }
            TypeData::Tuple(tu) => {
                children.extend(tu.interface_data.object.type_arguments.iter().cloned());
                children.extend(tu.element_infos.iter().filter_map(|e| e.type_.clone()));
            }
            TypeData::Union(u) => {
                return self.any_variance_marker_in_types(&u.union_or_intersection.types, depth)
            }
            TypeData::Intersection(i) => {
                return self.any_variance_marker_in_types(&i.union_or_intersection.types, depth)
            }
            TypeData::IndexedAccess(ia) => {
                children.extend(ia.object_type.iter().cloned());
                children.extend(ia.index_type.iter().cloned());
            }
            TypeData::Conditional(c) => {
                children.extend(c.check_type.iter().cloned());
                children.extend(c.extends_type.iter().cloned());
                children.extend(c.resolved_true_type.get().cloned());
                children.extend(c.resolved_false_type.get().cloned());
            }
            TypeData::TemplateLiteral(tl) => children.extend(tl.types.iter().cloned()),
            TypeData::StringMapping(sm) => children.extend(sm.target.iter().cloned()),
            TypeData::Substitution(s) => children.extend(s.base_type.iter().cloned()),
            TypeData::Index(i) => children.extend(i.target.iter().cloned()),
            _ => {}
        }
        self.any_variance_marker_in_types(&children, depth)
    }

    pub fn report_unreliable_markers(&mut self, t: &Arc<Type>) {
        if self.contains_variance_marker(t, 0) {
            self.reliability_flags |= RELATION_REPORTS_UNRELIABLE;
        }
    }

    pub fn report_unmeasurable_markers(&mut self, t: &Arc<Type>) {
        if self.contains_variance_marker(t, 0) {
            self.reliability_flags |= RELATION_REPORTS_UNMEASURABLE;
        }
    }

    pub fn get_alias_variances(&mut self, symbol: &Arc<Symbol>) -> Vec<VarianceFlags> {
        if let Some(links) = self.variance_links.get(symbol)
            && !links.variances.is_empty()
        {
            return links.variances.clone();
        }
        let key = Arc::as_ptr(symbol) as usize;
        if self.variance_stack.contains(&key) {
            return Vec::new();
        }
        self.variance_stack.push(key);
        let (tp_symbols, _) = self.collect_alias_type_params_and_body(symbol);
        let mut variances = Vec::with_capacity(tp_symbols.len());
        for tp_sym in &tp_symbols {
            let tp = self.get_type_parameter_from_symbol(tp_sym);
            let super_marker = self.marker_super();
            let sub_marker = self.marker_sub();
            let with_super = self.create_marker_type(symbol, &tp, &super_marker);
            let with_sub = self.create_marker_type(symbol, &tp, &sub_marker);
            let mut variance = VarianceFlags::None;
            if let (Some(w_sub), Some(w_super)) = (&with_sub, &with_super) {
                let saved_reliability = self.reliability_flags;
                self.reliability_flags = 0;
                if self.is_type_assignable_to(w_sub, w_super) {
                    variance |= VarianceFlags::Covariant;
                }
                if self.is_type_assignable_to(w_super, w_sub) {
                    variance |= VarianceFlags::Contravariant;
                }
                if variance == (VarianceFlags::Covariant | VarianceFlags::Contravariant) {
                    let other_marker = self.marker_other();
                    if let Some(w_other) = self.create_marker_type(symbol, &tp, &other_marker)
                        && self.is_type_assignable_to(&w_other, w_super)
                    {
                        variance = VarianceFlags::Independent;
                    }
                }
                if self.reliability_flags & RELATION_REPORTS_UNRELIABLE != 0 {
                    variance |= VarianceFlags::Unreliable;
                }
                if self.reliability_flags & RELATION_REPORTS_UNMEASURABLE != 0 {
                    variance |= VarianceFlags::Unmeasurable;
                }
                self.reliability_flags = saved_reliability;
            }
            variances.push(variance);
            if self
                .variance_links
                .get(symbol)
                .is_some_and(|l| !l.variances.is_empty())
            {
                break;
            }
        }
        self.variance_stack.pop();
        self.variance_links.get_or_default(symbol).variances = variances.clone();
        variances
    }

    pub fn create_marker_type(
        &mut self,
        symbol: &Arc<Symbol>,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let (tp_symbols, _) = self.collect_alias_type_params_and_body(symbol);
        if tp_symbols.is_empty() {
            return self.create_marker_type_reference(symbol, source, target);
        }
        let mut args = Vec::with_capacity(tp_symbols.len());
        for tp_sym in &tp_symbols {
            let tp = self.get_type_parameter_from_symbol(tp_sym);
            if Arc::ptr_eq(&tp, source) {
                args.push(Arc::clone(target));
            } else {
                args.push(tp);
            }
        }
        let result = self.instantiate_alias_from_types(symbol, args);
        if crate::checker::utilities::is_type_error(&result) {
            return Some(result);
        }
        self.marker_types.insert(result.id);
        Some(result)
    }

    fn create_marker_type_reference(
        &mut self,
        symbol: &Arc<Symbol>,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let is_interface = symbol
            .flags
            .intersects(tsox_frontend::ast::SymbolFlags::Interface)
            && symbol
                .declarations
                .iter()
                .any(|d| matches!(d.data, tsox_frontend::ast::NodeData::InterfaceDeclaration(_)));
        let class_node = if is_interface {
            None
        } else {
            symbol
                .flags
                .intersects(tsox_frontend::ast::SymbolFlags::Class)
                .then(|| {
                    symbol
                        .declarations
                        .iter()
                        .find(|d| matches!(d.data, tsox_frontend::ast::NodeData::ClassDeclaration(_)))
                        .cloned()
                })
                .flatten()
        };
        if !is_interface && class_node.is_none() {
            return None;
        }
        let type_parameters = self.declared_type_parameter_types(symbol);
        if type_parameters.is_empty() {
            return None;
        }
        let args: Vec<Arc<Type>> = type_parameters
            .iter()
            .map(|tp| {
                if Arc::ptr_eq(tp, source) {
                    Arc::clone(target)
                } else {
                    Arc::clone(tp)
                }
            })
            .collect();
        let result = if is_interface {
            self.resolve_interface_type_ex(symbol, Some(args))
        } else {
            let class_node = class_node?;
            let class_tps: Vec<Arc<Symbol>> = match &class_node.data {
                tsox_frontend::ast::NodeData::ClassDeclaration(cd) => match &cd.type_parameters {
                    Some(tps) => tps
                        .iter()
                        .filter_map(|tp| {
                            self.program.symbol_map().symbol_of(tp).map(Arc::clone)
                        })
                        .collect(),
                    None => Vec::new(),
                },
                _ => Vec::new(),
            };
            self.instantiate_class_instance_type(&class_node, symbol, &class_tps, &args)
        };
        if crate::checker::utilities::is_type_error(&result) {
            return Some(result);
        }
        self.marker_types.insert(result.id);
        Some(result)
    }

    pub fn get_type_parameter_modifiers(
        &mut self,
        tp: &Arc<Type>,
    ) -> tsox_frontend::ast::ModifierFlags {
        let mut flags = tsox_frontend::ast::ModifierFlags::empty();
        if let Some(symbol) = tp.symbol.as_ref() {
            for d in &symbol.declarations {
                flags |= d.syntactic_modifier_flags();
            }
        }
        flags & (tsox_frontend::ast::ModifierFlags::In | tsox_frontend::ast::ModifierFlags::Out)
    }

    pub fn has_covariant_void_argument(
        &mut self,
        type_arguments: &[Arc<Type>],
        variances: &[VarianceFlags],
    ) -> bool {
        use crate::checker::types_type_flags_instantiable_non_primitive::VARIANCE_FLAGS_VARIANCE_MASK;
        variances.iter().zip(type_arguments.iter()).any(|(v, t)| {
            (*v & VARIANCE_FLAGS_VARIANCE_MASK) == VarianceFlags::Covariant
                && t.flags.intersects(TypeFlags::Void)
        })
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

    pub fn get_known_keys_of_tuple_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let tuple = match &t.data {
            TypeData::Tuple(tuple) => tuple,
            _ => return None,
        };
        let mut keys: Vec<Arc<Type>> = Vec::with_capacity(tuple.fixed_length + 1);
        for i in 0..tuple.fixed_length {
            keys.push(self.get_string_literal_type(&i.to_string()));
        }
        let element = self.get_any_type();
        let array = self.create_array_type_ex(element, tuple.readonly);
        keys.push(self.get_index_type(&array));
        Some(self.get_union_type(keys))
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

fn unit_member_ids(t: &Arc<Type>) -> std::collections::HashSet<crate::checker::types::TypeId> {
    let mut set = std::collections::HashSet::new();
    if crate::checker::utilities::is_unit_type(t) {
        set.insert(t.id);
    } else if t.flags.contains(TypeFlags::Union) {
        if let Some(ui) = t.as_union_or_intersection() {
            for c in &ui.types {
                if crate::checker::utilities::is_unit_type(c) {
                    set.insert(c.id);
                }
            }
        }
    }
    set
}
