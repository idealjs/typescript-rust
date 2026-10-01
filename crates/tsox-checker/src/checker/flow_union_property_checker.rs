#![allow(unused_imports)]

use crate::checker::flow_union_ops::*;
use tsox_frontend::ast::CheckFlags;
use tsox_frontend::ast::ModifierFlags;

impl Checker {
    /// Go getPropertyOfUnionOrIntersectionType：raw 合成属性之上过滤 ReadPartial
    /// （联合读取侧不存在于全部成分的属性不可见）
    pub(crate) fn get_property_of_union_or_intersection_type(
        &mut self,
        containing_type: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let prop = self.get_union_or_intersection_property(containing_type, name)?;
        if prop
            .check_flags
            .contains(tsox_frontend::ast::CheckFlags::ReadPartial)
        {
            return None;
        }
        Some(prop)
    }

    /// Go getUnionOrIntersectionProperty（raw）：部分存在的属性合成带
    /// ReadPartial/WritePartial 的符号，由调用方决定是否过滤
    pub(crate) fn get_union_or_intersection_property(
        &mut self,
        containing_type: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        // Go 在联合/交叉型实例上挂 propertyCache（checker.go:21749-21758），
        // 合成符号单次创建指针稳定，供 == 比较复用；Rust 侧按 (type id, name)
        // 缓存持有 Arc 等价保活
        let key = (u64::from(containing_type.id), name.to_string());
        if let Some(prop) = self.union_or_intersection_property_cache.get(&key) {
            return Some(Arc::clone(prop));
        }
        let prop = self.compute_union_or_intersection_property(containing_type, name)?;
        self.union_or_intersection_property_cache
            .insert(key, Arc::clone(&prop));
        Some(prop)
    }

    fn compute_union_or_intersection_property(
        &mut self,
        containing_type: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let types: Vec<Arc<Type>> = containing_type.types()?.to_vec();
        let is_union = containing_type.is_union();

        let mut single_prop: Option<Arc<Symbol>> = None;
        let mut prop_flags = SymbolFlags::Property;
        let mut optional_flag = if is_union {
            SymbolFlags::empty()
        } else {
            SymbolFlags::Optional
        };
        let mut found: Vec<Arc<Symbol>> = Vec::new();
        let mut index_types: Vec<Arc<Type>> = Vec::new();
        let mut index_readonly = false;
        let mut read_partial = false;
        let mut write_partial = false;
        // Go createUnionOrIntersectionProperty：intersection 初始 readonly，
        // 任一成分非 readonly 清除；union 任一成分 readonly 置位
        let mut check_readonly = !is_union;
        // Go CheckFlagsContains*：按成分声明修饰符累计读/写两侧可达性
        let mut contains_public = false;
        let mut contains_protected = false;
        let mut contains_private = false;
        let mut contains_write_public = false;
        let mut contains_write_protected = false;
        let mut contains_write_private = false;
        let mut contains_static = false;
        for current in types.iter() {
            let t = self.get_apparent_type(current);
            if self.is_error_type(&t) || t.flags.contains(TypeFlags::Never) {
                continue;
            }
            let prop = self.get_property_of_type(&t, name);
            match prop {
                Some(prop) => {
                    if is_union && self.symbol_is_readonly(&prop) {
                        check_readonly = true;
                    } else if !is_union && !self.symbol_is_readonly(&prop) {
                        check_readonly = false;
                    }
                    let modifiers =
                        crate::checker::exports::get_declaration_modifier_flags_from_symbol(&prop);
                    let write_modifiers =
                        crate::checker::exports::get_declaration_modifier_flags_from_symbol_ex(
                            &prop, true,
                        );
                    if modifiers.contains(ModifierFlags::Protected)
                        && !modifiers.contains(ModifierFlags::Public)
                    {
                        contains_protected = true;
                    } else if modifiers.contains(ModifierFlags::Private)
                        && !modifiers.contains(ModifierFlags::Public)
                    {
                        contains_private = true;
                    } else {
                        contains_public = true;
                    }
                    if write_modifiers.contains(ModifierFlags::Protected)
                        && !write_modifiers.contains(ModifierFlags::Public)
                    {
                        contains_write_protected = true;
                    } else if write_modifiers.contains(ModifierFlags::Private)
                        && !write_modifiers.contains(ModifierFlags::Public)
                    {
                        contains_write_private = true;
                    } else {
                        contains_write_public = true;
                    }
                    if modifiers.contains(ModifierFlags::Static) {
                        contains_static = true;
                    }
                    if single_prop.is_none() {
                        single_prop = Some(Arc::clone(&prop));
                        prop_flags = if prop
                            .flags
                            .intersects(SymbolFlags::GetAccessor | SymbolFlags::SetAccessor)
                        {
                            prop.flags & (SymbolFlags::GetAccessor | SymbolFlags::SetAccessor)
                        } else {
                            SymbolFlags::Property
                        };
                    }
                    let class_member = prop.flags.intersects(
                        SymbolFlags::Property
                            | SymbolFlags::GetAccessor
                            | SymbolFlags::SetAccessor
                            | SymbolFlags::Method,
                    );
                    if is_union && class_member {
                        optional_flag |= prop.flags & SymbolFlags::Optional;
                    } else if !is_union && class_member {
                        optional_flag &= prop.flags;
                    }
                    if !found.iter().any(|p| Arc::ptr_eq(p, &prop)) {
                        found.push(prop);
                    }
                }
                None => {
                    if !is_union {
                        continue;
                    }
                    let name_literal = self.get_string_literal_type(name);
                    if !crate::checker::utilities_is_optional_symbol::is_late_bound_name(name)
                        && let Some(info) = self.get_applicable_index_info(&t, &name_literal)
                    {
                        index_readonly |= info.is_readonly;
                        write_partial = true;
                        let vt = if self.is_tuple_type(&t) {
                            self.tuple_rest_or_undefined(&t)
                        } else {
                            info.value_type
                                .clone()
                                .unwrap_or_else(|| self.undefined_type())
                        };
                        index_types.push(vt);
                    } else if t.object_flags.contains(ObjectFlags::ObjectLiteral)
                        && !t.object_flags.contains(ObjectFlags::ContainsSpread)
                    {
                        write_partial = true;
                        index_types.push(self.undefined_type());
                    } else {
                        read_partial = true;
                    }
                }
            }
        }

        let single = single_prop?;
        // Go createUnionOrIntersectionProperty：union 中属性在某一成分为
        // private/protected 声明且其余成分缺省或声明不同时，读侧直接不建属性
        // （TS2339），写侧按最受限成分降级
        if is_union
            && (found.len() >= 2 || read_partial || write_partial)
            && (contains_private
                || contains_protected
                || contains_write_private
                || contains_write_protected)
            && !(found.len() >= 2 && self.has_common_declaration(&found))
        {
            if contains_private || contains_protected {
                return None;
            }
            if contains_write_private {
                contains_write_public = false;
                contains_write_protected = false;
            } else if contains_write_protected {
                contains_write_public = false;
            }
        }
        if found.len() == 1 && index_types.is_empty() && !read_partial && !write_partial {
            return Some(single);
        }

        let mut declarations: Vec<Arc<Node>> = Vec::new();
        let mut prop_types: Vec<Arc<Type>> = Vec::new();
        let mut first_parent: Option<Arc<Symbol>> = None;
        let mut non_uniform = false;
        let mut has_literal = false;
        let mut first_type: Option<Arc<Type>> = None;
        for prop in &found {
            for d in &prop.declarations {
                if !declarations.iter().any(|x| Arc::ptr_eq(x, d)) {
                    declarations.push(Arc::clone(d));
                }
            }
            if first_parent.is_none() {
                first_parent = self
                    .parent_symbol_of_declaration_chain(prop)
                    .or_else(|| prop.parent().clone());
            }
            let t = self.get_type_of_symbol(prop);
            if let Some(ft) = &first_type {
                if ft.id != t.id {
                    non_uniform = true;
                }
            } else {
                first_type = Some(Arc::clone(&t));
            }
            if self.is_go_literal_type(&t) || self.is_pattern_literal_type(&t) {
                has_literal = true;
            }
            prop_types.push(t);
        }
        prop_types.extend(index_types);

        let mut result = Symbol::new(prop_flags | optional_flag, name.to_string());
        result.check_flags = CheckFlags::SyntheticProperty;
        if non_uniform {
            result.check_flags |= CheckFlags::HasNonUniformType;
        }
        if has_literal {
            result.check_flags |= CheckFlags::HasLiteralType;
        }
        if read_partial {
            result.check_flags |= CheckFlags::ReadPartial;
        }
        if write_partial {
            result.check_flags |= CheckFlags::WritePartial;
        }
        if check_readonly || index_readonly {
            result.check_flags |= CheckFlags::Readonly;
        }
        if contains_public {
            result.check_flags |= CheckFlags::ContainsPublic;
        }
        if contains_protected {
            result.check_flags |= CheckFlags::ContainsProtected;
        }
        if contains_private {
            result.check_flags |= CheckFlags::ContainsPrivate;
        }
        if contains_write_public {
            result.check_flags |= CheckFlags::ContainsWritePublic;
        }
        if contains_write_protected {
            result.check_flags |= CheckFlags::ContainsWriteProtected;
        }
        if contains_write_private {
            result.check_flags |= CheckFlags::ContainsWritePrivate;
        }
        if contains_static {
            result.check_flags |= CheckFlags::ContainsStatic;
        }
        result.declarations = declarations;
        if let Some(fp) = first_parent {
            result.set_parent(&fp);
        }
        let symbol = Arc::new(result);
        let resolved = if is_union {
            self.get_union_type(prop_types)
        } else {
            // Go getIntersectionType：相同类型成分去重（number & number → number）
            let mut deduped: Vec<Arc<Type>> = Vec::with_capacity(prop_types.len());
            for t in prop_types {
                if !deduped.iter().any(|d| d.id == t.id) {
                    deduped.push(t);
                }
            }
            self.get_intersection_type(deduped)
        };
        self.value_symbol_links.insert(
            &symbol,
            crate::checker::types::ValueSymbolLinks {
                resolved_type: Some(resolved),
                containing_type: Some(Arc::clone(containing_type)),
                ..Default::default()
            },
        );
        Some(symbol)
    }

    fn is_go_literal_type(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Boolean) {
            return true;
        }
        if t.flags.contains(TypeFlags::Union) {
            if t.flags.intersects(TypeFlags::EnumLiteral) {
                return true;
            }
            return t
                .types()
                .is_some_and(|ts| {
                    ts.iter()
                        .all(|m| crate::checker::is_unit_type(m))
                });
        }
        crate::checker::is_unit_type(t)
    }

    pub(crate) fn is_pattern_literal_type(&self, t: &Arc<Type>) -> bool {
        if let TypeData::TemplateLiteral(data) = &t.data {
            return data.types.iter().all(|p| self.is_pattern_literal_placeholder_type(p));
        }
        if let TypeData::StringMapping(data) = &t.data {
            return data
                .target
                .as_ref()
                .is_some_and(|target| self.is_pattern_literal_placeholder_type(target));
        }
        false
    }

    fn is_pattern_literal_placeholder_type(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Intersection) {
            let mut seen_placeholder = false;
            for s in t.types().into_iter().flatten() {
                if s.flags.intersects(TYPE_FLAGS_LITERAL | TYPE_FLAGS_NULLABLE)
                    || self.is_pattern_literal_placeholder_type(&s)
                {
                    seen_placeholder = true;
                } else if !s.flags.contains(TypeFlags::Object) {
                    return false;
                }
            }
            return seen_placeholder;
        }
        t.flags.intersects(
            TypeFlags::Any | TypeFlags::String | TypeFlags::Number | TypeFlags::BigInt,
        ) || self.is_pattern_literal_type(t)
    }

    fn parent_symbol_of_declaration_chain(
        &self,
        prop: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let decl = prop.value_declaration.as_ref().or(prop.declarations.first())?;
        let parent_node = decl.parent()?;
        self.program
            .symbol_map()
            .symbol_of(&parent_node)
            .map(Arc::clone)
    }

    /// Go createUnionOrIntersectionProperty 缺火成分准入：属性命中、适用
    /// 索引签名、或无 spread 的对象字面量（补 undefined）
    pub(crate) fn constituent_admits_property(&mut self, ct: &Arc<Type>, name: &str) -> bool {
        let apparent = self.get_apparent_type(ct);
        if self.get_property_of_type(&apparent, name).is_some() {
            return true;
        }
        if !crate::checker::utilities_is_optional_symbol::is_late_bound_name(name) {
            let name_literal = self.get_string_literal_type(name);
            if self.get_applicable_index_info(&apparent, &name_literal).is_some() {
                return true;
            }
        }
        apparent.object_flags.contains(ObjectFlags::ObjectLiteral)
            && !apparent.object_flags.contains(ObjectFlags::ContainsSpread)
    }

    fn tuple_rest_or_undefined(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if let TypeData::Tuple(tuple) = &t.data
            && let Some(info) = tuple.element_infos.get(tuple.fixed_length)
            && let Some(ty) = &info.type_
        {
            return Arc::clone(ty);
        }
        self.undefined_type()
    }

    pub fn is_error_type(&self, t: &Arc<Type>) -> bool {
        t.intrinsic_name() == Some("error")
    }
}
