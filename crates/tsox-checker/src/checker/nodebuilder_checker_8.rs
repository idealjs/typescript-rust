#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
use crate::checker::symboltracker::DEFAULT_MAXIMUM_TRUNCATION_LENGTH;

impl Checker {
    pub(crate) fn display_check_truncation(&mut self, flags: TypeFormatFlags) -> bool { ::tsox_core::fntrace::enter("display_check_truncation"); 
        if self.display_truncating {
            return true;
        }
        let max_length = if flags.contains(TypeFormatFlags::NO_TRUNCATION) {
            usize::MAX
        } else {
            DEFAULT_MAXIMUM_TRUNCATION_LENGTH
        };
        self.display_truncating = self.display_approximate_length > max_length;
        self.display_truncating
    }

    fn truncated_union_member_string(&mut self, ty: &Arc<Type>, flags: TypeFormatFlags) -> String { ::tsox_core::fntrace::enter("truncated_union_member_string"); 
        let s = self.type_to_string_ex(ty, flags);
        if self.display_truncating
            && ty.flags.contains(TypeFlags::Object)
            && ty.symbol.is_none()
            && !s.contains("...")
        {
            self.display_approximate_length += 7;
            return "{ ...; }".to_string();
        }
        self.display_approximate_length += 2 + s.len();
        s
    }

    pub(crate) fn union_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String { ::tsox_core::fntrace::enter("union_to_string"); 
        if let TypeData::Union(u) = &t.data
            && let Some(origin) = &u.origin
        {
            return self.type_to_string_ex(origin, flags);
        }
        let types = t.types().unwrap_or(&[]);

        let mut ordered: Vec<&Arc<Type>> = Vec::with_capacity(types.len());
        let mut nulls: Vec<&Arc<Type>> = Vec::new();
        let mut undefs: Vec<&Arc<Type>> = Vec::new();
        for ty in types.iter() {
            if ty.flags.contains(TypeFlags::Undefined) {
                undefs.push(ty);
            } else if ty.flags.contains(TypeFlags::Null) {
                nulls.push(ty);
            } else {
                ordered.push(ty);
            }
        }
        ordered.extend(nulls);
        ordered.extend(undefs);

        let parenthesize = |s: String, ty: &Arc<Type>, me: &mut Checker| {
            if me.needs_parens_in_union(ty) {
                format!("({})", s)
            } else {
                s
            }
        };

        if ordered.len() > 2 && self.display_check_truncation(flags) {
            let first = self.truncated_union_member_string(ordered[0], flags);
            let first = parenthesize(first, ordered[0], self);
            let last = self.truncated_union_member_string(ordered[ordered.len() - 1], flags);
            let last = parenthesize(last, ordered[ordered.len() - 1], self);
            return format!(
                "{} | ... {} more ... | {}",
                first,
                ordered.len() - 2,
                last
            );
        }

        let mut parts: Vec<String> = Vec::with_capacity(ordered.len());
        for (i, ty) in ordered.iter().enumerate() {
            let display_index = i + 1;
            if self.display_check_truncation(flags)
                && display_index + 2 < ordered.len().saturating_sub(1)
            {
                parts.push(format!("... {} more ...", ordered.len() - display_index));
                let last = self.truncated_union_member_string(ordered[ordered.len() - 1], flags);
                parts.push(parenthesize(last, ordered[ordered.len() - 1], self));
                break;
            }
            let s = self.truncated_union_member_string(ty, flags);
            parts.push(parenthesize(s, ty, self));
        }
        parts.join(" | ")
    }

    pub(crate) fn intersection_to_string(
        &mut self,
        t: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("intersection_to_string"); 
        let types = t.types().unwrap_or(&[]);
        let mut parts: Vec<String> = Vec::with_capacity(types.len());
        for (i, ty) in types.iter().enumerate() {
            let display_index = i + 1;
            if self.display_check_truncation(flags)
                && display_index + 2 < types.len().saturating_sub(1)
            {
                parts.push(format!("... {} more ...", types.len() - display_index));
                let last = &types[types.len() - 1];
                let s = self.truncated_union_member_string(last, flags);
                parts.push(if self.needs_parens_in_union(last) {
                    format!("({})", s)
                } else {
                    s
                });
                break;
            }
            let s = self.truncated_union_member_string(ty, flags);
            parts.push(if self.needs_parens_in_union(ty) {
                format!("({})", s)
            } else {
                s
            });
        }
        parts.join(" & ")
    }

    pub(crate) fn type_parameter_to_string(&mut self, t: &Arc<Type>) -> String { ::tsox_core::fntrace::enter("type_parameter_to_string"); 
        if let TypeData::TypeParameter(tp) = &t.data {
            if tp.is_this_type {
                return "this".to_string();
            }
        }
        if let Some(sym) = &t.symbol {
            return sym.name.clone();
        }
        "T".to_string()
    }

    pub(crate) fn indexed_access_to_string(
        &mut self,
        ia: &IndexedAccessTypeData,
        flags: TypeFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("indexed_access_to_string"); 
        let obj = ia
            .object_type
            .as_ref()
            .map(|t| {
                let s = self.type_to_string_ex(t, flags);

                if matches!(t.data, TypeData::Conditional(_) | TypeData::Mapped(_)) {
                    format!("({s})")
                } else {
                    s
                }
            })
            .unwrap_or_else(|| "any".to_string());
        let idx = ia
            .index_type
            .as_ref()
            .map(|t| self.type_to_string_ex(t, flags))
            .unwrap_or_else(|| "any".to_string());
        format!("{}[{}]", obj, idx)
    }

    pub(crate) fn template_literal_to_string(
        &mut self,
        tl: &TemplateLiteralTypeData,
        flags: TypeFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("template_literal_to_string"); 
        let mut result = String::new();
        for (i, text) in tl.texts.iter().enumerate() {
            result.push_str(text);
            if i < tl.types.len() {
                result.push_str("${");
                result.push_str(&self.type_to_string_ex(&tl.types[i], flags));
                result.push('}');
            }
        }
        format!("`{}`", result)
    }

    pub(crate) fn tuple_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String { ::tsox_core::fntrace::enter("tuple_to_string"); 
        let TypeData::Tuple(tuple) = &t.data else {
            return "[]".to_string();
        };
        let readonly_prefix = if tuple.readonly { "readonly " } else { "" };
        if tuple.element_infos.is_empty() {
            return format!("{readonly_prefix}[]");
        }
        let indexed: Vec<(usize, &Arc<Type>)> = tuple
            .element_infos
            .iter()
            .enumerate()
            .filter_map(|(i, elem)| elem.type_.as_ref().map(|t| (i, t)))
            .collect();
        let rendered = self.type_list_strings(
            &indexed.iter().map(|(_, t)| *t).collect::<Vec<_>>(),
            flags,
        );
        let mut elem_strs: Vec<Option<String>> = vec![None; tuple.element_infos.len()];
        for ((i, _), s) in indexed.iter().zip(rendered) {
            elem_strs[*i] = Some(s);
        }
        let parts: Vec<String> = tuple
            .element_infos
            .iter()
            .enumerate()
            .map(|(i, elem)| {
                let ty_str = elem_strs[i].clone().unwrap_or_else(|| "any".to_string());
                let label = elem.label.clone().or_else(|| {
                    elem.labeled_declaration
                        .as_ref()
                        .and_then(|d| tsox_frontend::ast::node_data_generated::node_name(d))
                        .map(|n| n.text().to_string())
                });
                let is_variable = elem.flags.contains(ElementFlags::Rest)
                    || elem.flags.contains(ElementFlags::Variadic);
                let is_optional = elem.flags.contains(ElementFlags::Optional);
                if let Some(label) = label {
                    let prefix = if is_variable { "..." } else { "" };
                    let question = if is_optional { "?" } else { "" };
                    format!("{prefix}{label}{question}: {ty_str}")
                } else if is_variable {
                    format!("...{ty_str}")
                } else if is_optional {
                    format!("{ty_str}?")
                } else {
                    ty_str
                }
            })
            .collect();
        format!("{readonly_prefix}[{}]", parts.join(", "))
    }

    pub(crate) fn reference_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String { ::tsox_core::fntrace::enter("reference_to_string"); 
        let obj_data = match &t.data {
            TypeData::Object(o) => o,
            TypeData::Interface(i) => &i.object,
            _ => return "object".to_string(),
        };

        let symbol_name = t.symbol.as_ref().map(|s| s.name.as_str()).unwrap_or("");
        let is_array = obj_data.type_arguments.len() == 1
            && (symbol_name == "Array" || symbol_name == "ReadonlyArray" || t.symbol.is_none());

        if is_array {
            let elem = &obj_data.type_arguments[0];
            let elem_str = self.type_to_string_ex(elem, flags);
            let symbol_name = t.symbol.as_ref().map(|s| s.name.as_str()).unwrap_or("");
            if symbol_name == "ReadonlyArray"
                || (symbol_name == "Array"
                    && t.object_flags.contains(
                        crate::checker::types::ObjectFlags::IsReadonlyArray,
                    ))
            {
                return format!("readonly {}[]", self.maybe_parenthesize_array_element_ex(elem, flags));
            }
            if flags.contains(TypeFormatFlags::WRITE_ARRAY_AS_GENERIC) {
                return format!("Array<{}>", elem_str);
            }
            return format!("{}[]", self.maybe_parenthesize_array_element_ex(elem, flags));
        }

        let name = t
            .symbol
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "object".to_string());

        let mut qualified = t
            .symbol
            .as_ref()
            .filter(|s| {
                s.parent()
                    .as_ref()
                    .is_some_and(|p| p.flags.contains(tsox_frontend::ast::SymbolFlags::ValueModule))
            })
            .map(|s| {
                // Go getSymbolChain：符号是父模块的 export=（自身隔代父），
                // 链退化为模块限定名（别名 a），不再追加符号名
                let is_export_equals = s
                    .parent()
                    .as_ref()
                    .and_then(|p| p.exports.get("export="))
                    .is_some_and(|exp| {
                        Arc::ptr_eq(exp, s)
                            || exp
                                .export_symbol
                                .as_ref()
                                .is_some_and(|t| Arc::ptr_eq(t, s))
                            || self
                                .follow_alias_resolving(exp)
                                .is_some_and(|target| Arc::ptr_eq(&target, s))
                    });
                if is_export_equals {
                    return self
                        .namespace_qualifier_of(s)
                        .unwrap_or_else(|| s.name.clone());
                }
                self.namespace_qualifier_of(s)
                    .map(|q| format!("{q}.{}", s.name))
                    .unwrap_or_else(|| s.name.clone())
            })
            .unwrap_or(name);

        if let Some(sym) = t.symbol.as_ref()
            && let Some(chain_name) = self.alias_chain_qualified_type_name(sym)
        {
            qualified = chain_name;
        }

        if obj_data.type_arguments.is_empty() {
            return qualified;
        }

        let args = self.type_list_strings(
            &obj_data.type_arguments.iter().collect::<Vec<_>>(),
            flags,
        );
        format!("{}<{}>", qualified, args.join(", "))
    }

    pub(crate) fn alias_chain_qualified_type_name(
        &mut self,
        symbol: &Arc<tsox_frontend::ast::Symbol>,
    ) -> Option<String> { ::tsox_core::fntrace::enter("alias_chain_qualified_type_name"); 
        use tsox_frontend::ast::SymbolFlags;
        if symbol.flags.intersects(SymbolFlags::Alias) {
            return None;
        }
        let enclosing = self.display_enclosing_node.clone();
        let chain =
            self.get_accessible_symbol_chain(symbol, enclosing.as_ref(), SymbolFlags::TYPE, false);
        if chain.len() < 2 || !chain[0].flags.intersects(SymbolFlags::Alias) {
            return None;
        }
        if chain.iter().any(|s| s.name.starts_with('\u{FE}')) {
            return None;
        }
        Some(
            chain
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join("."),
        )
    }

    pub(crate) fn signature_instantiated_param_type(
        &self,
        sig: &Signature,
        i: usize,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("signature_instantiated_param_type"); 
        let overrides = sig.instantiated_parameter_types.as_ref()?;
        let rest_offset = usize::from(sig.has_rest_parameter());
        let fixed = overrides.len().saturating_sub(rest_offset);
        if i < fixed {
            return Some(Arc::clone(&overrides[i]));
        }

        if rest_offset == 1 && i == fixed {
            return Some(Arc::clone(&overrides[fixed]));
        }
        None
    }

    pub(crate) fn function_type_to_string(
        &mut self,
        _t: &Arc<Type>,
        structured: &StructuredTypeData,
        flags: TypeFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("function_type_to_string"); 
        let new_prefix;
        let sigs = structured.call_signatures();
        let sig = if sigs.is_empty() {
            let ctors = structured.construct_signatures();
            if ctors.is_empty() {
                return "() => unknown".to_string();
            }
            new_prefix = "new ";
            &ctors[0]
        } else {
            new_prefix = "";
            &sigs[0]
        };
        // Go getExpandedParameters：末参是 rest 且其类型为元组时，按元组
        // 元素展开为具名参数序列（标签取元素 label，回退 rest 符号名_i）
        let expanded_params = self.tuple_expanded_params(sig);
        let params: Vec<String> = if let Some(expanded) = expanded_params {
            expanded
                .iter()
                .map(|(name, ty, optional, variadic)| {
                    let type_str = self.type_to_string_ex(ty, flags);
                    let prefix = if *variadic { "..." } else { "" };
                    let question = if *optional { "?" } else { "" };
                    format!("{prefix}{name}{question}: {type_str}")
                })
                .collect()
        } else {
            sig.parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    let name = param.name.clone();

                    let param_type = self
                        .signature_instantiated_param_type(sig, i)
                        .unwrap_or_else(|| self.get_type_of_symbol(param));
                    let reused = if self.display_enclosing_node.is_some() {
                        self.annotated_param_type_text(param, &param_type)
                    } else {
                        None
                    };
                    let type_str =
                        reused.unwrap_or_else(|| self.type_to_string_ex(&param_type, flags));
                    let prefix = if i + 1 == sig.parameters.len() && sig.has_rest_parameter() {
                        "..."
                    } else {
                        ""
                    };
                    if param_declared_optional(param) {
                        format!("{prefix}{name}?: {type_str}")
                    } else {
                        format!("{prefix}{name}: {type_str}")
                    }
                })
                .collect()
        };
        let ret_type = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        let ret_str = self.type_to_string_ex(&ret_type, flags);

        let tp_prefix = self.signature_type_param_prefix(sig);
        let this_param = sig
            .this_parameter
            .as_ref()
            .filter(|p| !p.name.is_empty())
            .map(|p| {
                let t = self.get_type_of_symbol(p);
                let type_str = self.type_to_string_ex(&t, flags);
                format!("this: {type_str}")
            });
        let params = match this_param {
            Some(this) => {
                let mut v = vec![this];
                v.extend(params);
                v
            }
            None => params,
        };
        format!("{new_prefix}{tp_prefix}({}) => {}", params.join(", "), ret_str)
    }

    pub(crate) fn signature_type_param_prefix(&mut self, sig: &Arc<Signature>) -> String { ::tsox_core::fntrace::enter("signature_type_param_prefix"); 
        if sig.type_parameters.is_empty() {
            return String::new();
        }
        let parts: Vec<String> = sig
            .type_parameters
            .iter()
            .map(|tp| self.type_param_decl_string(tp))
            .collect();
        if parts.is_empty() {
            String::new()
        } else {
            format!("<{}>", parts.join(", "))
        }
    }

    fn type_param_decl_string(&mut self, tp: &Arc<Type>) -> String { ::tsox_core::fntrace::enter("type_param_decl_string"); 
        let Some(sym) = &tp.symbol else {
            return "T".to_string();
        };
        let mut s = sym.name.clone();
        if let TypeData::TypeParameter(tpd) = &tp.data
            && let Some(constraint) = &tpd.constraint
        {
            let c = self.type_to_string(constraint);
            if !c.is_empty() {
                s.push_str(" extends ");
                s.push_str(&c);
            }
        }
        if let TypeData::TypeParameter(tpd) = &tp.data
            && let Some(default) = tpd.resolved_default_type.get()
        {
            let d = self.type_to_string(default);
            if !d.is_empty() {
                s.push_str(" = ");
                s.push_str(&d);
            }
        }
        s
    }
}

pub(crate) fn param_declared_optional(param: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("param_declared_optional"); 
    if param
        .flags
        .contains(tsox_frontend::ast::SymbolFlags::Optional)
    {
        return true;
    }
    param.declarations.iter().any(|d| match &d.data {
        tsox_frontend::ast::NodeData::ParameterDeclaration(pd) => {
            pd.question_token.is_some() || pd.initializer.is_some()
        }
        _ => false,
    })
}
