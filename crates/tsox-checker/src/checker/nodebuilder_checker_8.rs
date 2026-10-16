#![allow(unused_imports)]

use crate::checker::mig::m2g::r22k9_defs::CheckerPseudoReturnDisplayExt;
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
        let ordered = self.format_union_types(t.types().unwrap_or(&[]), false);

        let parenthesize = |s: String, ty: &Arc<Type>, me: &mut Checker| {
            // Go emitUnionTypeConstituent（printer.go:2195-2198）联合成分按
            // TypePrecedenceTypeOperator 发射，仅节点优先级严格低于该级才加
            // 括号（printer.go:2463）：经别名门按 TypeReference 呈现的成分
            // 为 NonArray 档（ast/precedence.go:707）恒不加括号，门条件与
            // 渲染别名分支同源（nodebuilder_checker_9.rs renders_as_alias_reference）
            if !me.renders_as_alias_reference(ty, flags) && me.needs_parens_in_union(ty) {
                format!("({})", s)
            } else {
                s
            }
        };

        if ordered.len() > 2 && self.display_check_truncation(flags) {
            let first = self.truncated_union_member_string(&ordered[0], flags);
            let first = parenthesize(first, &ordered[0], self);
            let last = self.truncated_union_member_string(&ordered[ordered.len() - 1], flags);
            let last = parenthesize(last, &ordered[ordered.len() - 1], self);
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
                let last = self.truncated_union_member_string(&ordered[ordered.len() - 1], flags);
                parts.push(parenthesize(last, &ordered[ordered.len() - 1], self));
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
                parts.push(if !self.renders_as_alias_reference(last, flags)
                    && self.needs_parens_in_union(last)
                {
                    format!("({})", s)
                } else {
                    s
                });
                break;
            }
            let s = self.truncated_union_member_string(ty, flags);
            parts.push(if !self.renders_as_alias_reference(ty, flags)
                && self.needs_parens_in_union(ty)
            {
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
        if t.symbol.is_some() {
            // Go typeToTypeNodeHelper 型参臂（nodebuilderimpl.go:3616-3623）：
            // GenerateNamesForShadowedTypeParams 置位时引用位同经
            // typeParameterToName 取缓存名（与声明位一致），否则按符号名
            return self.type_parameter_display_name(t);
        }
        "T".to_string()
    }

    /// Go typeParameterToName（nodebuilderimpl.go:1528-1577）的串渲染版：
    /// GenerateNamesForShadowedTypeParams 下按 tp id 缓存命名；名字已被本会话
    /// 占用（typeParameterNamesByText）或阴影作用域内其他型参
    /// （typeParameterShadowsOtherTypeParameterInScope）时按 raw_i 发明后缀。
    /// 旗标未置位即 Go symbolToName 裸名
    pub(crate) fn type_parameter_display_name(&mut self, type_parameter: &Arc<Type>) -> String { ::tsox_core::fntrace::enter("type_parameter_display_name");
        let raw = type_parameter
            .symbol
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "T".to_string());
        if !self.display_generate_shadowed_names {
            return raw;
        }
        if let Some(cached) = self.display_type_parameter_names.get(&type_parameter.id) {
            return cached.clone();
        }
        let mut i = self
            .display_type_param_next_name_count
            .get(&raw)
            .copied()
            .unwrap_or(0);
        let mut text = raw.clone();
        loop {
            if !self.display_type_param_names_by_text.contains(&text)
                && !self.type_parameter_shadows_other_type_parameter_in_scope(&text, type_parameter)
            {
                break;
            }
            i += 1;
            text = format!("{raw}_{i}");
        }
        self.display_type_param_next_name_count.insert(raw, i);
        self.display_type_parameter_names
            .insert(type_parameter.id, text.clone());
        self.display_type_param_names_by_text.insert(text.clone());
        text
    }

    /// Go typeParameterShadowsOtherTypeParameterInScope
    /// （nodebuilderimpl.go:1522-1527）：名字自 enclosing 声明作用域解出的型参
    /// 符号不等于当前型参符号即阴影
    pub(crate) fn type_parameter_shadows_other_type_parameter_in_scope(
        &mut self,
        name: &str,
        type_parameter: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("type_parameter_shadows_other_type_parameter_in_scope");
        let Some(location) = self.type_render_enclosing.clone() else {
            return false;
        };
        let result = self.resolve_name(name, &location, tsox_frontend::ast::SymbolFlags::TYPE, false);
        match result {
            Some(result) if result.flags.contains(tsox_frontend::ast::SymbolFlags::TypeParameter) => {
                match &type_parameter.symbol {
                    // Go typeParameterShadowsOtherTypeParameterInScope
                    // （nodebuilderimpl.go:1522-1527）指针比较等价声明身份：
                    // 同声明不阴影、跨声明即阴影（重载签名族同名型参，
                    // functionOverloadsOnGenericArity1 第二签名 <T, U> 应呈
                    // <T_1, U>）；本仓多树副本下同声明符号存在不同 Arc 实例
                    // （explicitMembersBeforeInherited 的 >subs 应呈
                    // IndentationSubTree<L>[]），按同声明判定
                    // （type_param_symbols_same_declaration）判阴影
                    Some(tp) => !self.type_param_symbols_same_declaration(result.as_ref(), tp),
                    None => true,
                }
            }
            _ => false,
        }
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

                // Go emitIndexedAccessType（printer.go:2288）与 emitArrayType
                //（printer.go:2129）同经 emitPostfixTypeOperand
                //（printer.go:2142-2149）以 TypePrecedencePostfix 发射前段，
                // 括号判定同源（printer.go:2463 precedence < postfix）：
                // MappedType/TypeReference 档不加括号，并/交/函数/keyof 档加
                if self.needs_parens_as_array_element(t, flags) {
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
            // Go 打印机模板分支：段文本无 rawText 时经
            // escapeStringWorker(text, QuoteCharBacktick, flags) 转义
            // （printer/utilities.go:268-308）——\t/\r\n/\${/` 等呈现转义形
            result.push_str(&crate::checker::escape_string::escape_string(
                text,
                crate::checker::escape_string::QuoteChar::Backtick,
            ));
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
                    // Go emitOptionalType → emitPostfixTypeOperand
                    // （printer.go:2174/2142）：操作数按 TypePrecedencePostfix
                    // 输出，低于后缀优先级的元型（并集等）加括号——与数组
                    // 元素同一括号判据
                    if elem
                        .type_
                        .as_ref()
                        .is_some_and(|ty| self.needs_parens_as_array_element(ty, flags))
                    {
                        format!("({ty_str})?")
                    } else {
                        format!("{ty_str}?")
                    }
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
                // Go symbolToTypeNode mask=SymbolFlagsType（nodebuilderimpl.go:3297）：
                // 同名遮蔽时容器链限定兜底（needsQualification，M.P 内引用 M 的 D 呈 M.D）
                self.namespace_qualifier_of(s)
                    .or_else(|| {
                        self.container_chain_qualified_name(s, tsox_frontend::ast::SymbolFlags::TYPE)
                    })
                    .map(|q| format!("{q}.{}", s.name))
                    .unwrap_or_else(|| s.name.clone())
            })
            .unwrap_or(name);

        if let Some(sym) = t.symbol.as_ref() {
            // Go typeToTypeReferenceNode（nodebuilderimpl.go:3297）→
            // symbolToTypeNode(SymbolFlagsType, typeArguments)：引用名从渲染
            // 上下文取符号链（getSymbolChain，686-774）——就地 import 别名
            // 链=[别名] 呈裸名，链根模块符号呈 import 说明符限定。
            // .types 发射通道（type_render_enclosing = node.Parent）走链；
            // 错误消息通道维持 alias 链/父限定原行为
            let chain_name = if self.type_render_enclosing.is_some() {
                self.symbol_chain_display_name(sym, tsox_frontend::ast::SymbolFlags::TYPE)
            } else {
                self.alias_chain_qualified_type_name(sym)
            };
            if let Some(chain_name) = chain_name {
                qualified = chain_name;
            }
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
        // Go signatureToSignatureDeclarationHelper（nodebuilderimpl.go:1932）：
        // enterSignatureScope 先行，命名顺序 typeParameters（1941-1949）→
        // parameters（1963）→ this（1971-1979）→ returnType（1982）；
        // 作用域内新增名随 cleanup 丢弃，兄弟签名互不占用
        let scope = self.push_display_naming_scope();
        let tp_prefix = self.signature_type_param_prefix(sig);
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
                    let name = self.signature_parameter_display_name(param);

                    let param_type = self
                        .signature_instantiated_param_type(sig, i)
                        .unwrap_or_else(|| self.get_type_of_symbol(param));
                    // Go serializeTypeForDeclaration（nodebuilderimpl.go:2364-2368）：
                    // 参数声明 requiresAddingImplicitUndefined（emitresolver.go:629-635
                    // = isRequiredInitializedParameter 且声明类型不含 undefined）时
                    // 先 getOptionalType 并入 undefined，再参与注解复用等价——
                    // 非尾随带初始值参数呈 `string | undefined` 而非注解原文本
                    let param_type =
                        self.param_type_for_display(param, &param_type);
                    // Go symbolToParameterDeclaration（nodebuilderimpl.go:1792-1793）
                    // 无条件走 serializeTypeForDeclaration(tryReuse=true)：参数带
                    // 注解且伪等价（可选参数忽略合成 undefined，nodebuilderimpl.go
                    // :2402 的 ignoreUndefined 判据）时复用注解源文本——strict
                    // 打开的合成 `| undefined` 不进显示。复用无嵌套深度门：
                    // spread 成员内函数型参数（declarationEmitResolveTypes
                    // IfNotReusable >f o1-o4）与顶层查询同走注解复用/骨架
                    // 部分复用/解析回落三路
                    let reused =
                        self.annotated_param_type_text(param, &param_type, Self::signature_params_instantiated(sig), flags);
                    let type_str =
                        reused.unwrap_or_else(|| self.type_to_string_ex(&param_type, flags));
                    let prefix = if i + 1 == sig.parameters.len() && sig.has_rest_parameter() {
                        "..."
                    } else {
                        ""
                    };
                    if self.param_declared_optional(param) {
                        format!("{prefix}{name}?: {type_str}")
                    } else {
                        format!("{prefix}{name}: {type_str}")
                    }
                })
                .collect()
        };
        let this_param = sig
            .this_parameter
            .as_ref()
            .filter(|p| !p.name.is_empty())
            .map(|p| {
                let t = self.get_type_of_symbol(p);
                let type_str = self.type_to_string_ex(&t, flags);
                format!("this: {type_str}")
            });
        let ret_str = match self.signature_predicate_return_string(sig, flags) {
            Some(predicate_str) => predicate_str,
            None => {
                let ret_type = self.get_return_type_of_signature_lazy(sig);
                self.annotated_return_type_text(sig, &ret_type, flags)
                    .or_else(|| {
                        self.pseudo_signature_return_display_text(sig, &ret_type, flags)
                    })
                    .unwrap_or_else(|| self.type_to_string_ex(&ret_type, flags))
            }
        };
        self.pop_display_naming_scope(scope);
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

    pub(crate) fn param_type_for_display(
        &mut self,
        param: &Arc<Symbol>,
        t: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("param_type_for_display");
        let Some(decl) =
            crate::checker::mig::m2g_2::get_effective_parameter_declaration(param)
        else {
            return Arc::clone(t);
        };
        if decl.kind != tsox_frontend::ast::SyntaxKind::Parameter {
            return Arc::clone(t);
        }
        let resolver = self.get_emit_resolver();
        if resolver.requires_adding_implicit_undefined_worker(self, &decl, None) {
            self.get_optional_type(Arc::clone(t), false)
        } else {
            Arc::clone(t)
        }
    }

    pub(crate) fn signature_parameter_display_name(&self, param: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("signature_parameter_display_name");
        let Some(decl) =
            crate::checker::mig::m2g_2::get_effective_parameter_declaration(param)
        else {
            return param.name.clone();
        };
        let Some(name_node) = decl.name() else {
            return param.name.clone();
        };
        match &name_node.data {
            tsox_frontend::ast::NodeData::Identifier(_) => name_node.text().to_string(),
            tsox_frontend::ast::NodeData::QualifiedName(q) => q.right.text().to_string(),
            tsox_frontend::ast::NodeData::BindingPattern(_) => self
                .binding_pattern_display_text(&name_node)
                .unwrap_or_else(|| param.name.clone()),
            _ => self
                .node_source_text(&name_node)
                .unwrap_or_else(|| param.name.clone()),
        }
    }

    // Go parameterToParameterDeclarationName→cloneBindingName
    //（nodebuilderimpl.go:1822-1868）：绑定模式参数名按克隆节点经打印机重生成
    //（EFSingleLine；LFObjectBindingPatternElements 的 LFSpaceBetweenBraces，
    // printer.go:6659）——{a} 呈 { a }，嵌套 BindingElement 的初始化式剥除
    fn binding_pattern_display_text(&self, node: &Arc<Node>) -> Option<String> {
        let parts = match &node.data {
            tsox_frontend::ast::NodeData::BindingPattern(bp) => {
                self.binding_pattern_element_texts(&bp.elements)?
            }
            _ => return None,
        };
        match node.kind {
            SyntaxKind::ObjectBindingPattern => {
                if parts.is_empty() {
                    Some("{}".to_string())
                } else {
                    Some(format!("{{ {} }}", parts.join(", ")))
                }
            }
            SyntaxKind::ArrayBindingPattern => Some(format!("[{}]", parts.join(", "))),
            _ => None,
        }
    }

    fn binding_pattern_element_texts(&self, elements: &Arc<NodeList>) -> Option<Vec<String>> {
        let mut parts = Vec::with_capacity(elements.nodes.len());
        for el in elements.nodes.iter() {
            match &el.data {
                tsox_frontend::ast::NodeData::Token
                    if el.kind == SyntaxKind::OmittedExpression =>
                {
                    parts.push(String::new());
                }
                tsox_frontend::ast::NodeData::BindingElement(be) => {
                    let mut s = String::new();
                    if be.dot_dot_dot_token.is_some() {
                        s.push_str("...");
                    }
                    if let Some(prop) = &be.property_name {
                        s.push_str(&self.binding_property_name_text(prop));
                        s.push_str(": ");
                    }
                    match be.name.as_ref().map(|n| n.kind) {
                        Some(SyntaxKind::Identifier) => {
                            s.push_str(&be.name.as_ref().unwrap().text());
                        }
                        Some(SyntaxKind::ObjectBindingPattern)
                        | Some(SyntaxKind::ArrayBindingPattern) => {
                            s.push_str(&self.binding_pattern_display_text(
                                be.name.as_ref().unwrap(),
                            )?);
                        }
                        // 缺名元素（源侧省略位）发射空段
                        _ => {}
                    }
                    parts.push(s);
                }
                _ => return None,
            }
        }
        Some(parts)
    }

    fn binding_property_name_text(&self, prop: &Arc<Node>) -> String {
        if prop.kind == SyntaxKind::ComputedPropertyName {
            self.node_source_text(prop).unwrap_or_else(|| prop.text().to_string())
        } else {
            prop.text().to_string()
        }
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
        // Go typeParameterToDeclarationWithConstraint（nodebuilderimpl.go:1450）
        // 声明位命名同经 typeParameterToName（阴影改名会话缓存）；修饰符段
        // CreateModifiersFromModifierFlags(getTypeParameterModifiers(tp))，
        // factory 顺序 Const→In→Out（ast/mig/w2.rs:55/86/89）
        let mut s = String::new();
        let modifiers = self.get_type_parameter_modifiers(tp);
        if modifiers.intersects(tsox_frontend::ast::ModifierFlags::Const) {
            s.push_str("const ");
        }
        if modifiers.intersects(tsox_frontend::ast::ModifierFlags::In) {
            s.push_str("in ");
        }
        if modifiers.intersects(tsox_frontend::ast::ModifierFlags::Out) {
            s.push_str("out ");
        }
        s.push_str(&self.type_parameter_display_name(tp));
        let display_constraint = match tp.as_type_parameter().and_then(|d| d.constraint.clone()) {
            Some(c) => Some(c),
            None => {
                // Go typeParameterToDeclaration（nodebuilderimpl.go:1741-1747）
                // 约束取 getConstraintOfTypeParameter（checker.go:18050-18054）=
                // hasNonCircularBaseConstraint 门内 getConstraintFromTypeParameter
                // （checker.go:18074-18099）——实例化克隆型参（cloneTypeParameter
                // checker.go:23832-23836 只挂 symbol+target，约束字段恒空）沿
                // target 约束经克隆 mapper 代入解析（<T extends S>(x: T) => void
                // 实例化显示 <T extends { s: string; }>），缺臂退 <T>
                if self.has_non_circular_base_constraint(tp) {
                    self.get_constraint_from_type_parameter(tp)
                } else {
                    None
                }
            }
        };
        // Go getConstraintOfTypeParameter（checker.go:18053-18059）对环约束恒
        // None：hasNonCircularBaseConstraint（getResolvedBaseConstraint
        // checker.go:29076-29124 push/pop 判环，同符号单实例环在一层即现）——
        // 本仓同符号双实例缓存（declared_type_links 裸实例 + type_alias_links
        // 急解析实例）把环检测推迟一层，外层 resolved_base_constraint 落
        // noConstraint 门恒开，自约束 T extends T 链尾同符号实例经阴影改名呈
        // T_1（typeParameterHasSelfAsConstraint）。显示位按约束链回环补判
        let display_constraint = match (
            display_constraint,
            tp.symbol.as_ref().map(|s| Arc::as_ptr(s) as usize),
        ) {
            (Some(c), Some(k)) if self.constraint_chain_is_circular(k, &c) => None,
            (c, _) => c,
        };
        if let Some(constraint) = display_constraint {
            let c = self.constraint_display_string(tp, &constraint);
            if !c.is_empty() {
                s.push_str(" extends ");
                s.push_str(&c);
            }
        }
        if let Some(default) = self.get_default_from_type_parameter(tp) {
            // Go typeParameterToDeclaration（nodebuilderimpl.go:1737-1747）默认
            // 子句与外层同一 nodebuilder 上下文：.types 基线 harness 旗标无
            // UseAliasDefinedOutsideCurrentScope → getSymbolChain 模块根段
            // 保留（yieldModuleSymbol=true），跨包别名默认呈限定名。本仓
            // type_to_string 为错误消息入口（强制 display_yield_module_symbol
            // =false，模块根段被剔除呈裸名），发射通道改走 type_to_string_ex
            // 保持环境 yield
            let d = if self.type_render_enclosing.is_some() {
                self.type_to_string_ex(
                    &default,
                    TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE
                        .union(TypeFormatFlags::NO_TRUNCATION),
                )
            } else {
                self.type_to_string(&default)
            };
            if !d.is_empty() {
                s.push_str(" = ");
                s.push_str(&d);
            }
        }
        s
    }

    /// Go typeParameterToDeclaration（nodebuilderimpl.go:1737-1747）：约束经
    /// typeToTypeNodeHelperWithPossibleReusableTypeNode(constraint,
    /// getConstraintDeclaration(parameter)) 呈现——getTypeFromTypeNode(约束声明
    /// 节点) 与约束型一致时复用声明节点（tryReuseExistingNodeHelper，任意类型
    /// 节点种类均复用），打印节点文本而非约束型展开（T extends ChannelType 不
    /// 展开为 "email" | "text"；union 约束 T | string 保持书写序，不按成员
    /// 排序展开）。本仓 TypeLiteral 约束按 Go 复用节点的 EFSingleLine 结构化
    /// 单行重建呈 type_literal_constraint_to_string（nodecopy.go:830-840），其余
    /// 种类以声明节点源文本呈现，复用判定不成立才走展开。
    fn constraint_display_string(
        &mut self,
        tp: &Arc<Type>,
        constraint: &Arc<Type>,
    ) -> String { ::tsox_core::fntrace::enter("constraint_display_string");
        // Go 复用 visiting 对标识符段经 TrackSymbol→symbolToTypeNode 重建名
        //（nodecopy.go:689-724），型参引用位同经 typeParameterToName 取会话
        // 改名（nodebuilderimpl.go:3616-3623）；本仓源文本切片承载不了改名，
        // 约束含已改名型参时复用不忠实——跳过两处源文本/链名臂，落全序列化
        //（同一 nodebuilder 上下文，引用位经 type_parameter_to_string 改名，
        // 如第二次重载 K1_1 的 keyof T_1[K1_1]）
        let reuse_blocked = self.constraint_references_renamed_type_parameter(constraint);
        if let Some(node) = self.get_constraint_declaration(tp)
            && node.kind == tsox_frontend::ast::SyntaxKind::TypeLiteral
        {
            let resolved = self.get_type_from_type_node(&node);
            if Arc::ptr_eq(&resolved, constraint)
                && let Some(structured) = constraint.as_structured()
            {
                return self.type_literal_constraint_to_string(constraint, structured);
            }
        }
        if let Some(node) = self.get_constraint_declaration(tp) {
            let resolved = self.get_type_from_type_node(&node);
            // 等价门：Go 为对象恒等（nodebuilderimpl.go:1728-1731），本仓注解
            // 侧与型参约束侧可取同型的不同实例（annotation_type_node_is_global_
            // array_reference 同款多表示），id 相等补位
            if Arc::ptr_eq(&resolved, constraint) || resolved.id == constraint.id {
                // Go 约束经复用 visiting 呈现（typeToTypeNodeHelperWithPossible
                // ReusableTypeNode → tryReuseExistingNodeHelper，identifier 段
                // TrackSymbol 后按符号链重建名），跨文件别名约束按渲染上下文
                // 限定（index.ts 下 TBase extends import("./wrapClass")
                // .Constructor），就地可见呈裸名；裸 Identifier 名仅 .types
                // 发射通道走链（type_render_enclosing 为 Some，错误消息通道
                // 维持源文本）；Go tryVisitTypeReference（nodecopy.go:463-471）
                // 实参段 visitor.VisitNodes 原样保留，链名只承载实体名段，
                // 带实参约束不走向链（维持源文本复用，呈 IComparable<any>）
                if let tsox_frontend::ast::NodeData::TypeReferenceNode(d) = &node.data
                    && d.type_name.kind == tsox_frontend::ast::SyntaxKind::Identifier
                    && d.type_arguments.is_none()
                    && self.type_render_enclosing.is_some()
                    && !reuse_blocked
                {
                    // Go 复用 visiting（nodecopy.go:430/465
                    // trackExistingEntityName）：复用前按显示上下文重解实体名，
                    // 解不到即放弃复用落全序列化（symbolToTypeNode 链，
                    // 就地可见呈裸名=复用文本，不可见呈限定名）。约束符号取
                    // 已解型的别名符号/符号（类型引用标识符不入 binder
                    // 符号表，get_symbol_at_location 对其恒 None）
                    let type_sym = resolved
                        .alias
                        .as_ref()
                        .and_then(|a| a.symbol.clone())
                        .or_else(|| resolved.symbol.clone())
                        .or_else(|| self.get_symbol_at_location(&d.type_name));
                    if let Some(sym) = type_sym
                        && let Some(chain_name) = self.symbol_chain_display_name(
                            &sym,
                            tsox_frontend::ast::SymbolFlags::TYPE,
                        )
                    {
                        // Go 复用节点保留书写实参（tryReuseExistingTypeNode 克隆
                        // typeArguments 随节点输出，typeArgumentsNodes 走
                        // visitEachChild）：链名后接 alias 实参渲染
                        if let Some(args) = resolved
                            .alias
                            .as_ref()
                            .map(|a| a.type_arguments.as_slice())
                            .filter(|args| !args.is_empty())
                        {
                            let rendered: Vec<String> = args
                                .iter()
                                .map(|a| {
                                    self.type_to_string_ex(
                                        a,
                                        TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE
                                            .union(TypeFormatFlags::NO_TRUNCATION),
                                    )
                                })
                                .collect();
                            return format!("{chain_name}<{}>", rendered.join(", "));
                        }
                        return chain_name;
                    }
                }
                if let tsox_frontend::ast::NodeData::TypeReferenceNode(d) = &node.data
                    && d.type_name.kind == tsox_frontend::ast::SyntaxKind::Identifier
                    && d.type_arguments.is_some()
                    && self.type_render_enclosing.is_some()
                {
                    // 复用 visiting 主路（print_annotation_type_node
                    // TypeReference 臂 = trackExistingEntityName +
                    // tryVisitTypeReference 书写实参递归）：resolved 型的
                    // alias 链接不承载书写实参，下方链名+alias 实参臂呈
                    // A/C.A 丢 <B>；复用不成立才落链名臂
                    if let Some(text) = self.print_annotation_type_node(
                        &node,
                        TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE
                            .union(TypeFormatFlags::NO_TRUNCATION),
                    ) {
                        return text;
                    }
                    // Go trackExistingEntityName（nodecopy.go:346-430）：复用前
                    // 以显示语境重解实体名，解不到即 introducesError →
                    // tryVisitTypeReference（nodecopy.go:445-483）落
                    // serializeTypeName（nodebuilderimpl.go:457-474，同样判可及，
                    // 失败返 nil）→ 复用整体失败，typeToTypeNodeHelperWith
                    // PossibleReusableTypeNode（nodebuilderimpl.go:1726-1740）落
                    // typeToTypeNode 全序列化——显示语境不可见的名经别名门链呈
                    // import("@loopback/context").Constructor<any>（usage.ts 单
                    // 元行），就地可见（bindingkey.ts 单元行）维持源文本复用
                    let enclosing = self.type_render_enclosing.clone();
                    let in_scope = enclosing.is_some_and(|enc| {
                        self.resolve_name_in_enclosure(
                            &enc,
                            &d.type_name.text(),
                            tsox_frontend::ast::SymbolFlags::TYPE,
                        )
                        .is_some()
                    });
                    if !in_scope {
                        let type_sym = resolved
                            .alias
                            .as_ref()
                            .and_then(|a| a.symbol.clone())
                            .or_else(|| resolved.symbol.clone())
                            .or_else(|| self.get_symbol_at_location(&d.type_name));
                        if let Some(sym) = type_sym
                            && let Some(chain_name) = self.symbol_chain_display_name(
                                &sym,
                                tsox_frontend::ast::SymbolFlags::TYPE,
                            )
                        {
                            if let Some(args) = resolved
                                .alias
                                .as_ref()
                                .map(|a| a.type_arguments.as_slice())
                                .filter(|args| !args.is_empty())
                            {
                                let rendered: Vec<String> = args
                                    .iter()
                                    .map(|a| {
                                        self.type_to_string_ex(
                                            a,
                                            TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE
                                                .union(TypeFormatFlags::NO_TRUNCATION),
                                        )
                                    })
                                    .collect();
                                return format!("{chain_name}<{}>", rendered.join(", "));
                            }
                            return chain_name;
                        }
                    }
                }
                // Go 复用 visiting（typeToTypeNodeHelperWithPossibleReusable
                // TypeNode nodebuilderimpl.go:1731-1737 → tryReuseExistingNode
                // Helper → existingNodeTreeVisitor nodecopy.go:689-724）对约束
                // 子树中无类型注解且无初始化的参数声明合成 any 关键字呈现——
                // 函数/构造类型约束不能直取源文本（那会保留无注解参数），
                // 经注解结构化打印（参数段 print_annotation_params 同口径）
                if matches!(
                    node.kind,
                    tsox_frontend::ast::SyntaxKind::FunctionType
                        | tsox_frontend::ast::SyntaxKind::ConstructorType
                ) && let Some(s) = self.print_annotation_type_node(
                    &node,
                    TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE
                        .union(TypeFormatFlags::NO_TRUNCATION),
                )
                {
                    return s;
                }
                if !reuse_blocked
                    && let Some(decl) = node.parent()
                    && let Some(sf) = self.get_source_file_of_node(&decl)
                {
                    let text = &sf.text;
                    // Go 复用 visiting（nodecopy.go:430-471）对 union/intersection
                    // 约束节点逐成员重建：成员型节点经 getTypeFromTypeNode 取型，
                    // TypeLiteral 臂走结构化单行重建（emitTypeLiteral
                    // printer.go:2113-2121，成员分号由 emitTypeElement 产出，
                    // `{ m1; m2; }`），其余成员保持节点源文本；书写序保留
                    //（约束 union 不按成员排序展开）。整节点源文本切片会把
                    // 字面量成员呈书写形态（`{ x: string }` 无分隔号）
                    let composite = match &node.data {
                        tsox_frontend::ast::NodeData::UnionTypeNode(d) => {
                            Some((d.types.iter().collect::<Vec<_>>(), " | "))
                        }
                        tsox_frontend::ast::NodeData::IntersectionTypeNode(d) => {
                            Some((d.types.iter().collect::<Vec<_>>(), " & "))
                        }
                        _ => None,
                    };
                    if let Some((members, separator)) = composite {
                        // 结构化重建仅在无歧义形态（恰一个 TypeLiteral 成员且约束
                        // 型恰一个匿名对象成分）下进行：渲染通道不就地解析成员型
                        // 节点（解析副作用会污染 type_node_subst_cache），对象
                        // 成员取约束型既有解析产物中的匿名对象成分，其余成员保持
                        // 节点源文本；书写序保留
                        let literal_members = members
                            .iter()
                            .filter(|m| m.kind == tsox_frontend::ast::SyntaxKind::TypeLiteral)
                            .count();
                        let mut constituents: Vec<Arc<Type>> = Vec::new();
                        if constraint.flags.intersects(TypeFlags::Union) {
                            if let Some(ts) = constraint.types() {
                                constituents = ts.to_vec();
                            }
                        } else {
                            constituents.push(Arc::clone(constraint));
                        }
                        let object_constituents: Vec<Arc<Type>> = constituents
                            .into_iter()
                            .filter(|t| {
                                t.flags.contains(TypeFlags::Object)
                                    && t.object_flags
                                        .contains(crate::checker::types::ObjectFlags::Anonymous)
                            })
                            .collect();
                        let mut parts: Vec<String> = Vec::new();
                        for member in members.iter() {
                            if member.kind == tsox_frontend::ast::SyntaxKind::TypeLiteral
                                && literal_members == 1
                                && object_constituents.len() == 1
                            {
                                let object_type = &object_constituents[0];
                                if let Some(structured) = object_type.as_structured() {
                                    parts.push(self.type_literal_constraint_to_string(
                                        object_type,
                                        structured,
                                    ));
                                    continue;
                                }
                            }
                            let start = member.pos().min(text.len());
                            let end = member.end().min(text.len());
                            if start < end {
                                parts.push(text[start..end].to_string());
                            }
                        }
                        if !parts.is_empty() {
                            return parts.join(separator);
                        }
                    }
                    let start = node.pos().min(text.len());
                    let end = node.end().min(text.len());
                    if start < end {
                        return text[start..end].to_string();
                    }
                }
            }
        }
        // Go 复用判定不成立（getTypeFromTypeNode 与约束型非同一对象）时落
        // 全序列化（typeToTypeNodeHelper，同一 nodebuilder 上下文）——.types
        // 发射通道保持环境 yield（模块根段保留，跨包别名呈限定名），错误
        // 消息通道维持 type_to_string
        if self.type_render_enclosing.is_some() {
            return self.type_to_string_ex(
                constraint,
                TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE
                    .union(TypeFormatFlags::NO_TRUNCATION),
            );
        }
        self.type_to_string(constraint)
    }

    /// 约束是否引用了本显示会话中已改名的型参（typeParameterToName 去重
    /// 链，nodebuilderimpl.go:1536-1570）：是则源文本/链名复用不忠实
    pub(crate) fn constraint_references_renamed_type_parameter(
        &mut self,
        constraint: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("constraint_references_renamed_type_parameter");
        if !self.display_generate_shadowed_names {
            return false;
        }
        let mut tps: Vec<Arc<Type>> = Vec::new();
        let mut seen: std::collections::HashSet<u32> = Default::default();
        collect_type_parameters_in(constraint, &mut tps, &mut seen, 0);
        tps.iter().any(|tp| {
            let raw = tp
                .symbol
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_default();
            self.type_parameter_display_name(tp) != raw
        })
    }

    fn type_literal_constraint_to_string(
        &mut self,
        t: &Arc<Type>,
        structured: &StructuredTypeData,
    ) -> String { ::tsox_core::fntrace::enter("type_literal_constraint_to_string");
        // Go 复用节点经 EFSingleLine 单行结构化重建（nodecopy.go:830-840 →
        // emitTypeLiteral printer.go:2113-2121）：`{ m1; m2; }`，成员分号由
        // emitTypeElement 产出，非源文本切片
        let saved_yield = self.display_yield_module_symbol;
        let saved_reuse_params = self.display_constraint_reuse_params;
        self.display_constraint_reuse_params = true;
        let parts = if self.type_render_enclosing.is_some() {
            self.structured_members_to_strings(
                t,
                structured,
                TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE
                    .union(TypeFormatFlags::NO_TRUNCATION),
                true,
            )
        } else {
            self.display_yield_module_symbol = false;
            let parts = self.structured_members_to_strings(
                t,
                structured,
                TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE
                    .union(TypeFormatFlags::NO_TRUNCATION)
                    .union(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE),
                true,
            );
            self.display_yield_module_symbol = saved_yield;
            parts
        };
        self.display_constraint_reuse_params = saved_reuse_params;
        if parts.is_empty() {
            "{}".to_string()
        } else {
            format!("{{ {} }}", format!("{};", parts.join("; ")))
        }
    }

    pub(crate) fn signature_predicate_return_string(
        &mut self,
        sig: &Arc<Signature>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("signature_predicate_return_string");
        let predicate = self.get_type_predicate_of_signature(sig)?;
        let mut s = String::new();
        if matches!(
            predicate.kind,
            TypePredicateKind::AssertsThis | TypePredicateKind::AssertsIdentifier
        ) {
            s.push_str("asserts ");
        }
        if matches!(predicate.kind, TypePredicateKind::This | TypePredicateKind::AssertsThis) {
            s.push_str("this");
        } else {
            s.push_str(&predicate.parameter_name);
        }
        if let Some(t) = &predicate.t {
            s.push_str(" is ");
            s.push_str(&self.type_to_string_ex(t, flags));
        }
        Some(s)
    }
}

impl Checker {
    pub(crate) fn param_declared_optional(&mut self, param: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("param_declared_optional");
        let parameter_declaration =
            crate::checker::mig::m2g_2::get_effective_parameter_declaration(param);
        parameter_declaration.is_some_and(|d| self.is_optional_parameter(&d))
            || param
                .check_flags
                .contains(tsox_frontend::ast::CheckFlags::OptionalParameter)
    }
}

fn collect_type_parameters_in(
    t: &Arc<Type>,
    out: &mut Vec<Arc<Type>>,
    seen: &mut std::collections::HashSet<u32>,
    depth: usize,
) {
    if depth > 32 || !seen.insert(t.id) {
        return;
    }
    if let TypeData::TypeParameter(_) = &t.data {
        out.push(Arc::clone(t));
    }
    macro_rules! visit {
        ($child:expr) => {
            if let Some(c) = &$child {
                collect_type_parameters_in(c, out, seen, depth + 1);
            }
        };
    }
    match &t.data {
        TypeData::Object(o) => {
            for a in &o.type_arguments {
                collect_type_parameters_in(a, out, seen, depth + 1);
            }
            visit!(o.target);
        }
        TypeData::Interface(i) => {
            for a in &i.object.type_arguments {
                collect_type_parameters_in(a, out, seen, depth + 1);
            }
            visit!(i.object.target);
        }
        TypeData::Tuple(tup) => {
            for a in &tup.interface_data.object.type_arguments {
                collect_type_parameters_in(a, out, seen, depth + 1);
            }
            visit!(tup.interface_data.object.target);
            for ei in &tup.element_infos {
                visit!(ei.type_);
            }
        }
        TypeData::Mapped(m) => {
            visit!(m.type_parameter);
            visit!(m.constraint_type);
            visit!(m.name_type);
            visit!(m.template_type);
            visit!(m.modifiers_type);
            for a in &m.object.type_arguments {
                collect_type_parameters_in(a, out, seen, depth + 1);
            }
            visit!(m.object.target);
        }
        TypeData::ReverseMapped(r) => {
            visit!(r.source);
            visit!(r.mapped_type);
            visit!(r.constraint_type);
        }
        TypeData::EvolvingArray(e) => visit!(e.element_type),
        TypeData::InstantiationExpression(i) => {
            for a in &i.object.type_arguments {
                collect_type_parameters_in(a, out, seen, depth + 1);
            }
            visit!(i.object.target);
        }
        TypeData::Union(u) => {
            for c in &u.union_or_intersection.types {
                collect_type_parameters_in(c, out, seen, depth + 1);
            }
        }
        TypeData::Intersection(i) => {
            for c in &i.union_or_intersection.types {
                collect_type_parameters_in(c, out, seen, depth + 1);
            }
        }
        TypeData::Index(i) => visit!(i.target),
        TypeData::IndexedAccess(ia) => {
            visit!(ia.object_type);
            visit!(ia.index_type);
        }
        TypeData::TemplateLiteral(tl) => {
            for c in &tl.types {
                collect_type_parameters_in(c, out, seen, depth + 1);
            }
        }
        TypeData::StringMapping(s) => visit!(s.target),
        TypeData::Substitution(s) => {
            visit!(s.base_type);
            visit!(s.constraint);
        }
        TypeData::Conditional(c) => {
            visit!(c.check_type);
            visit!(c.extends_type);
            visit!(c.resolved_true_type.get());
            visit!(c.resolved_false_type.get());
        }
        TypeData::TypeParameter(tp) => {
            visit!(tp.constraint);
        }
        _ => {}
    }
}
