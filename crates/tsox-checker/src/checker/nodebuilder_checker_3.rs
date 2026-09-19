#![allow(unused_imports)]

use crate::checker::nodebuilder::*;

impl Checker {
    pub(crate) fn object_literal_to_string(
        &mut self,
        _t: &Arc<Type>,
        structured: &StructuredTypeData,
        flags: TypeFormatFlags,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        for sig in structured.call_signatures() {
            let params: Vec<String> = sig
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    let name = param.name.clone();
                    let param_type = self
                        .signature_instantiated_param_type(sig, i)
                        .unwrap_or_else(|| self.get_type_of_symbol(param));
                    let type_str = self.type_to_string_ex(&param_type, flags);
                    if param.flags.contains(tsox_frontend::ast::SymbolFlags::Optional) {
                        format!("{}?: {}", name, type_str)
                    } else {
                        format!("{}: {}", name, type_str)
                    }
                })
                .collect();
            let ret_type = sig
                .resolved_return_type
                .get()
                .cloned()
                .unwrap_or_else(|| self.any_type());
            let ret_str = self.type_to_string_ex(&ret_type, flags);
            let tp = self.signature_type_param_prefix(sig);
            parts.push(format!("{tp}({}) => {}", params.join(", "), ret_str));
        }

        for sig in structured.construct_signatures() {
            let params: Vec<String> = sig
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    let param_type = self
                        .signature_instantiated_param_type(sig, i)
                        .unwrap_or_else(|| self.get_type_of_symbol(param));
                    format!(
                        "{}: {}",
                        param.name,
                        self.type_to_string_ex(&param_type, flags)
                    )
                })
                .collect();
            let ret_type = sig
                .resolved_return_type
                .get()
                .cloned()
                .unwrap_or_else(|| self.any_type());
            let ret_str = self.type_to_string_ex(&ret_type, flags);
            let tp = self.signature_type_param_prefix(sig);
            parts.push(format!("new {tp}({}) => {}", params.join(", "), ret_str));
        }

        for prop in &structured.properties {
            // Go symbolToString：well-known symbol 内部名 __@x 渲染为 [Symbol.x]
            let name = if let Some(stripped) = prop.name.strip_prefix("__@") {
                format!("[Symbol.{stripped}]")
            } else {
                prop.name.clone()
            };

            let name = if prop.declarations.iter().any(|d| {
                d.name()
                    .is_some_and(|n| n.kind == SyntaxKind::StringLiteral)
            }) {
                format!("\"{name}\"")
            } else {
                name
            };
            let prop_type = self.get_type_of_symbol(prop);

            // Go writer：optional 函数属性渲染为方法语法 `name?(): ret`
            if prop.flags.contains(SymbolFlags::Optional) {
                let structured = prop_type.as_structured();
                let is_function = structured.is_some_and(|s| {
                    s.call_signature_count == 1
                        && s.construct_signatures().is_empty()
                        && s.properties.is_empty()
                });
                if is_function {
                    let sig = structured
                        .and_then(|s| s.call_signatures().first())
                        .expect("checked single call signature above");
                    let params: Vec<String> = sig
                        .parameters
                        .iter()
                        .enumerate()
                        .map(|(i, param)| {
                            let param_type = self
                                .signature_instantiated_param_type(sig, i)
                                .unwrap_or_else(|| self.get_type_of_symbol(param));
                            let q = if param.flags.contains(SymbolFlags::Optional) {
                                "?"
                            } else {
                                ""
                            };
                            format!(
                                "{}{q}: {}",
                                param.name,
                                self.type_to_string_ex(&param_type, flags)
                            )
                        })
                        .collect();
                    let ret_type = sig
                        .resolved_return_type
                        .get()
                        .cloned()
                        .unwrap_or_else(|| self.any_type());
                    let ret_str = self.type_to_string_ex(&ret_type, flags);
                    let tp = self.signature_type_param_prefix(sig);
                    parts.push(format!(
                        "{name}?{tp}({}): {ret_str}",
                        params.join(", ")
                    ));
                    continue;
                }
            }

            let type_str = self.type_to_string_ex(&prop_type, flags);
            let readonly = prop.check_flags.contains(tsox_frontend::ast::CheckFlags::Readonly);
            if prop.flags.contains(SymbolFlags::Optional) {
                let ro = if readonly { "readonly " } else { "" };
                parts.push(format!("{ro}{}?: {}", name, type_str));
            } else if readonly {
                parts.push(format!("readonly {}: {}", name, type_str));
            } else {
                parts.push(format!("{}: {}", name, type_str));
            }
        }

        for info in &structured.index_infos {
            let key_str = info
                .key_type
                .as_ref()
                .map(|k| self.type_to_string_ex(k, flags))
                .unwrap_or_else(|| "string".to_string());
            let val_str = info
                .value_type
                .as_ref()
                .map(|v| self.type_to_string_ex(v, flags))
                .unwrap_or_else(|| "any".to_string());

            let key_name = info
                .declaration
                .as_ref()
                .and_then(|d| {
                    let NodeData::IndexSignatureDeclaration(sd) = &d.data else {
                        return None;
                    };
                    sd.parameters.iter().next().and_then(|p| match &p.data {
                        NodeData::ParameterDeclaration(pd) => Some(pd.name.text().to_string()),
                        _ => None,
                    })
                })
                .unwrap_or_else(|| "x".to_string());
            let readonly = if info.is_readonly { "readonly " } else { "" };
            parts.push(format!("{readonly}[{key_name}: {key_str}]: {val_str}"));
        }

        if parts.is_empty() {
            "{}".to_string()
        } else if structured.properties.is_empty()
            && structured.call_signatures().is_empty()
            && structured.construct_signatures().len() == 1
        {
            parts.join("")
        } else {
            format!("{{ {} }}", format!("{};", parts.join("; ")))
        }
    }

    pub(crate) fn symbol_type_to_string(
        &mut self,
        t: &Arc<Type>,
        sym: &Arc<Symbol>,
        flags: TypeFormatFlags,
    ) -> String {
        if sym.flags.contains(SymbolFlags::ENUM) {
            return sym.name.clone();
        }

        let obj_data = match &t.data {
            TypeData::Object(o) => Some(o),
            TypeData::Interface(i) => Some(&i.object),
            _ => None,
        };

        if let Some(obj) = obj_data {
            let mut arg_count = obj.type_arguments.len();
            // Go nodebuilder：Iterable/IterableIterator/AsyncIterable/
            // AsyncIterableIterator 的尾随默认实参在显示中省略
            if matches!(
                sym.name.as_str(),
                "Iterable" | "IterableIterator" | "AsyncIterable" | "AsyncIterableIterator"
            ) {
                let defaults = self.interface_default_type_arguments(&Arc::clone(sym));
                if defaults.len() == arg_count {
                    while arg_count > 0 {
                        let arg_str = self.type_to_string_ex(&obj.type_arguments[arg_count - 1], flags);
                        let default_str = self.type_to_string_ex(&defaults[arg_count - 1], flags);
                        if arg_str != default_str {
                            break;
                        }
                        arg_count -= 1;
                    }
                }
            }
            if arg_count > 0 {
                let args: Vec<String> = obj.type_arguments[..arg_count]
                    .iter()
                    .map(|ty| self.type_to_string_ex(ty, flags))
                    .collect();
                return format!("{}<{}>", sym.name, args.join(", "));
            }
        }

        if sym.flags.contains(SymbolFlags::Class) {
            if let Some(structured) = t.as_structured() {
                if !structured.construct_signatures().is_empty() {
                    return format!("typeof {}", sym.name);
                }
            }
            // 类实例（含与命名空间合并的类）：typeof 前缀只给静态侧（构造签名
            // 所在），实例侧按符号名显示（Go typeToString 同）
            return sym.name.clone();
        }

        if sym.flags.contains(SymbolFlags::ValueModule) {
            if sym
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::SourceFile)
            {
                return format!("typeof import(\"{}\")", module_specifier_of_name(&sym.name));
            }
            for d in &sym.declarations {
                if let NodeData::ModuleDeclaration(md) = &d.data
                    && md.name.kind == SyntaxKind::StringLiteral
                {
                    return format!(
                        "typeof import(\"{}\")",
                        md.name.text().trim_matches(['"', '\''])
                    );
                }
            }
            return format!("typeof {}", sym.name);
        }

        sym.name.clone()
    }

    pub(crate) fn needs_parens_in_union(&mut self, t: &Arc<Type>) -> bool {
        if let Some(structured) = t.as_structured() {
            if structured.call_signature_count > 0 && t.symbol.is_none() {
                return true;
            }
        }

        false
    }

    pub(crate) fn needs_parens_as_array_element(&mut self, t: &Arc<Type>) -> bool {
        if t.is_union() || t.is_intersection() {
            return true;
        }
        if matches!(&t.data, TypeData::Conditional(_) | TypeData::Index(_)) {
            return true;
        }
        self.needs_parens_in_union(t)
    }

    pub(crate) fn maybe_parenthesize_array_element_ex(
        &mut self,
        elem: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> String {
        let s = self.type_to_string_ex(elem, flags);
        if self.needs_parens_as_array_element(elem) {
            format!("({})", s)
        } else {
            s
        }
    }

    pub fn type_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        self.type_to_type_node_worker(t)
    }

}
