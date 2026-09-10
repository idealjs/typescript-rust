#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;

impl Checker {
    pub(crate) fn union_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
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
        let parts: Vec<String> = ordered
            .into_iter()
            .map(|ty| {
                let s = self.type_to_string_ex(ty, flags);

                if self.needs_parens_in_union(ty) {
                    format!("({})", s)
                } else {
                    s
                }
            })
            .collect();
        parts.join(" | ")
    }

    pub(crate) fn intersection_to_string(
        &mut self,
        t: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> String {
        let types = t.types().unwrap_or(&[]);
        let parts: Vec<String> = types
            .iter()
            .map(|ty| {
                let s = self.type_to_string_ex(ty, flags);
                if self.needs_parens_in_union(ty) {
                    format!("({})", s)
                } else {
                    s
                }
            })
            .collect();
        parts.join(" & ")
    }

    pub(crate) fn type_parameter_to_string(&mut self, t: &Arc<Type>) -> String {
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
    ) -> String {
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
    ) -> String {
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

    pub(crate) fn tuple_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
        let TypeData::Tuple(tuple) = &t.data else {
            return "[]".to_string();
        };
        let readonly_prefix = if tuple.readonly { "readonly " } else { "" };
        if tuple.element_infos.is_empty() {
            return format!("{readonly_prefix}[]");
        }
        let parts: Vec<String> = tuple
            .element_infos
            .iter()
            .map(|elem| {
                let ty_str = elem
                    .type_
                    .as_ref()
                    .map(|ty| self.type_to_string_ex(ty, flags))
                    .unwrap_or_else(|| "any".to_string());
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

    pub(crate) fn reference_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
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
            if symbol_name == "ReadonlyArray" {
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

        let qualified = t
            .symbol
            .as_ref()
            .filter(|s| {
                s.parent
                    .as_ref()
                    .is_some_and(|p| p.flags.contains(tsox_frontend::ast::SymbolFlags::ValueModule))
            })
            .map(|s| {
                // Go getSymbolChain：符号是父模块的 export=（自身隔代父），
                // 链退化为模块限定名（别名 a），不再追加符号名
                let is_export_equals = s
                    .parent
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

        if obj_data.type_arguments.is_empty() {
            return qualified;
        }

        let args: Vec<String> = obj_data
            .type_arguments
            .iter()
            .map(|ty| self.type_to_string_ex(ty, flags))
            .collect();
        format!("{}<{}>", qualified, args.join(", "))
    }

    pub(crate) fn signature_instantiated_param_type(
        &self,
        sig: &Signature,
        i: usize,
    ) -> Option<Arc<Type>> {
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
    ) -> String {
        let sigs = structured.call_signatures();
        if sigs.is_empty() {
            return "() => unknown".to_string();
        }

        let sig = &sigs[0];
        let params: Vec<String> = sig
            .parameters
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let name = param.name.clone();

                let param_type = self
                    .signature_instantiated_param_type(sig, i)
                    .unwrap_or_else(|| self.get_type_of_symbol(param));
                let type_str = self
                    .annotated_param_type_text(param, &param_type)
                    .unwrap_or_else(|| self.type_to_string_ex(&param_type, flags));
                let prefix = if i + 1 == sig.parameters.len() && sig.has_rest_parameter() {
                    "..."
                } else {
                    ""
                };
                if param
                    .flags
                    .contains(tsox_frontend::ast::SymbolFlags::Optional)
                {
                    format!("{prefix}{name}?: {type_str}")
                } else {
                    format!("{prefix}{name}: {type_str}")
                }
            })
            .collect();
        let ret_type = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        let ret_str = self.type_to_string_ex(&ret_type, flags);

        let tp_prefix = self.signature_type_param_prefix(sig);
        format!("{tp_prefix}({}) => {}", params.join(", "), ret_str)
    }

    pub(crate) fn signature_type_param_prefix(&self, sig: &Arc<Signature>) -> String {
        if sig.type_parameters.is_empty() {
            return String::new();
        }
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
    }
}
