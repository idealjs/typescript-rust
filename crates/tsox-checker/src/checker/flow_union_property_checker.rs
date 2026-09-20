#![allow(unused_imports)]

use crate::checker::flow_union_ops::*;
use tsox_frontend::ast::CheckFlags;

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
        let types: Vec<Arc<Type>> = containing_type.types()?.to_vec();
        let is_union = containing_type.is_union();

        let mut single_prop: Option<Arc<Symbol>> = None;
        let mut prop_flags = SymbolFlags::Property;
        let mut optional_flag = SymbolFlags::empty();
        let mut found: Vec<Arc<Symbol>> = Vec::new();
        let mut index_types: Vec<Arc<Type>> = Vec::new();
        let mut index_readonly = false;
        let mut read_partial = false;
        let mut write_partial = false;
        for current in types.iter() {
            let t = self.get_apparent_type(current);
            if self.is_error_type(&t) || t.flags.contains(TypeFlags::Never) {
                continue;
            }
            let prop = self.get_property_of_type(&t, name);
            match prop {
                Some(prop) => {
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
                    if is_union
                        && prop.flags.intersects(
                            SymbolFlags::Property
                                | SymbolFlags::GetAccessor
                                | SymbolFlags::SetAccessor
                                | SymbolFlags::Method,
                        ) {
                        optional_flag |= prop.flags & SymbolFlags::Optional;
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
            if crate::checker::utilities_token_is_identifier_or_keyword::is_literal_type(&t)
                || t.flags.contains(TypeFlags::TemplateLiteral)
            {
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
        if index_readonly {
            result.check_flags |= CheckFlags::Readonly;
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
