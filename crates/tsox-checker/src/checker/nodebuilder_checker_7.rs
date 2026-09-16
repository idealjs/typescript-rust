#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;

impl Checker {
    pub fn type_to_string(&mut self, t: &Arc<Type>) -> String {
        self.type_to_string_ex(t, TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE)
    }

    pub fn type_to_string_ex(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
        let key = Arc::as_ptr(t) as usize;
        if self.type_print_stack.len() >= 300 || self.type_print_stack.contains(&key) {
            return "...".to_string();
        }
        if self.serialization_level >= MAX_SERIALIZATION_LEVEL {
            return "?".to_string();
        }
        self.type_print_stack.push(key);
        let result = self.type_to_string_ex_worker(t, flags);
        self.type_print_stack.pop();
        result
    }

    pub(crate) fn type_to_string_ex_worker(
        &mut self,
        t: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> String {
        if let Some(name) = t.intrinsic_name() {
            if name == "error" {
                return "any".to_string();
            }
            return name.to_string();
        }

        if t.flags.contains(TypeFlags::EnumLiteral)
            && !t.is_union()
            && let Some(sym) = &t.symbol
            && sym
                .flags
                .intersects(tsox_frontend::ast::SymbolFlags::EnumMember)
            && let Some(parent) = sym.parent()
        {
            return format!("{}.{}", parent.name, sym.name);
        }

        if let Some(val) = t.literal_value() {
            return self.literal_value_to_string(val);
        }

        if t.flags.contains(TypeFlags::UniqueESSymbol) {
            if let TypeData::UniqueESSymbol(sym) = &t.data {
                if flags.contains(TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE) {
                    return format!("unique symbol");
                }
                return format!("typeof {}", sym.name);
            }
        }

        if t.flags.contains(TypeFlags::Never) {
            return "never".to_string();
        }

        if t.is_union()
            && let Some(sym) = &t.symbol
            && sym.flags.intersects(tsox_frontend::ast::SymbolFlags::ENUM)
        {
            return sym.name.clone();
        }

        if t.is_union() {
            return self.union_to_string(t, flags);
        }

        if t.is_intersection() {
            return self.intersection_to_string(t, flags);
        }

        if t.is_type_parameter() {
            return self.type_parameter_to_string(t);
        }

        if let TypeData::IndexedAccess(ia) = &t.data {
            return self.indexed_access_to_string(ia, flags);
        }

        if let TypeData::TemplateLiteral(tl) = &t.data {
            return self.template_literal_to_string(tl, flags);
        }

        if let TypeData::Index(i) = &t.data {
            let target = i
                .target
                .as_ref()
                .map(|tt| self.type_to_string_ex(tt, flags))
                .unwrap_or_else(|| "any".to_string());
            return format!("keyof {target}");
        }
        if let TypeData::StringMapping(s) = &t.data {
            let target = s
                .target
                .as_ref()
                .map(|tt| self.type_to_string_ex(tt, flags))
                .unwrap_or_else(|| "any".to_string());
            let name = t
                .symbol
                .as_ref()
                .map(|sym| sym.name.clone())
                .unwrap_or_default();
            if name.is_empty() {
                return target;
            }
            return format!("{name}<{target}>");
        }
        if let TypeData::Mapped(m) = &t.data {
            if let Some(alias) = &t.alias
                && let Some(sym) = &alias.symbol
            {
                let args: Vec<String> = alias
                    .type_arguments
                    .iter()
                    .map(|a| self.type_to_string_ex(a, flags))
                    .collect();
                if args.is_empty() {
                    return sym.name.clone();
                }
                return format!("{}<{}>", sym.name, args.join(", "));
            }

            let mut decl_tp_name: Option<String> = None;
            let mut decl_constraint: Option<String> = None;
            if let Some(decl) = m.declaration.as_ref()
                && let tsox_frontend::ast::NodeData::MappedTypeNode(md) = &decl.data
                && let tsox_frontend::ast::NodeData::TypeParameterDeclaration(tpd) =
                    &md.type_parameter.data
            {
                decl_tp_name = Some(tpd.name.text().to_string());
                if let Some(c) = &tpd.constraint {
                    decl_constraint = self.node_source_text(c);
                }
            }
            let tp = decl_tp_name
                .filter(|n| !n.is_empty())
                .or_else(|| {
                    m.type_parameter
                        .as_ref()
                        .and_then(|tp| tp.symbol.as_ref().map(|s| s.name.clone()))
                })
                .unwrap_or_else(|| "K".to_string());
            let constraint = decl_constraint
                .filter(|c| !c.is_empty())
                .unwrap_or_else(|| {
                    m.constraint_type
                        .as_ref()
                        .map(|c| self.type_to_string_ex(c, flags))
                        .unwrap_or_else(|| "keyof any".to_string())
                });
            let as_clause = m
                .name_type
                .as_ref()
                .map(|n| format!(" as {}", self.type_to_string_ex(n, flags)))
                .unwrap_or_default();
            let template = m
                .template_type
                .as_ref()
                .map(|tt| self.type_to_string_ex(tt, flags))
                .unwrap_or_else(|| "any".to_string());
            return format!("{{ [{tp} in {constraint}{as_clause}]: {template}; }}");
        }
        if let TypeData::Substitution(sub) = &t.data {
            if let Some(base) = &sub.base_type {
                return self.type_to_string_ex(base, flags);
            }
            if let Some(c) = &sub.constraint {
                return self.type_to_string_ex(c, flags);
            }
        }
        if let TypeData::Conditional(c) = &t.data {
            // Go conditionalTypeToTypeNode：已解析/可解析的条件显示解析值，
            // 别名形态只留给泛型挂起的条件
            let resolved = c
                .resolved_true_type
                .get()
                .or_else(|| c.resolved_false_type.get())
                .cloned()
                .or_else(|| self.resolve_conditional_type(t));
            if let Some(resolved) = resolved {
                return self.type_to_string_ex(&resolved, flags);
            }
            if let Some(alias) = &t.alias
                && let Some(sym) = &alias.symbol
            {
                let args: Vec<String> = alias
                    .type_arguments
                    .iter()
                    .map(|a| self.type_to_string_ex(a, flags))
                    .collect();
                if args.is_empty() {
                    return sym.name.clone();
                }
                return format!("{}<{}>", sym.name, args.join(", "));
            }
            let root = c.root.as_ref();
            let check = root
                .and_then(|r| r.check_type.clone())
                .or_else(|| c.check_type.clone())
                .map(|ct| {
                    // Go conditionalTypeToTypeNode：函数/构造形态的 check 需要
                    // 括号分组（(...t: T) => void extends ...）
                    let s = self.type_to_string_ex(&ct, flags);
                    if ct.as_structured().is_some_and(|st| !st.signatures.is_empty()) {
                        format!("({s})")
                    } else {
                        s
                    }
                })
                .unwrap_or_else(|| "unknown".to_string());
            let extends = root
                .and_then(|r| r.extends_type.clone())
                .or_else(|| c.extends_type.clone())
                .map(|et| self.type_to_string_ex(&et, flags))
                .unwrap_or_else(|| "unknown".to_string());

            let (cond_node, true_node, false_node) = root
                .and_then(|r| r.node.as_ref())
                .map(|n| match &n.data {
                    tsox_frontend::ast::NodeData::ConditionalTypeNode(d) => (
                        Some(Arc::clone(n)),
                        Some(Arc::clone(&d.true_type)),
                        Some(Arc::clone(&d.false_type)),
                    ),
                    _ => (None, None, None),
                })
                .unwrap_or((None, None, None));
            if let Some(cn) = &cond_node {
                self.push_scope(cn);
            }
            let true_t = c
                .resolved_true_type
                .get()
                .map(|tt| self.type_to_string_ex(tt, flags))
                .or_else(|| {
                    true_node.map(|n| {
                        let t = self.get_type_from_type_node(&n);
                        self.type_to_string_ex(&t, flags)
                    })
                })
                .unwrap_or_else(|| "...".to_string());
            let false_t = c
                .resolved_false_type
                .get()
                .map(|ft| self.type_to_string_ex(ft, flags))
                .or_else(|| {
                    false_node.map(|n| {
                        let t = self.get_type_from_type_node(&n);
                        self.type_to_string_ex(&t, flags)
                    })
                })
                .unwrap_or_else(|| "...".to_string());
            if cond_node.is_some() {
                self.pop_scope();
            }
            return format!("{check} extends {extends} ? {true_t} : {false_t}");
        }

        // 泛型函数内声明的类经调用位实例化：f<string>.C 形态
        // （alias=外层函数+实参，symbol=类）
        if let Some(alias) = &t.alias
            && let Some(fn_sym) = &alias.symbol
            && fn_sym.flags.contains(tsox_frontend::ast::SymbolFlags::Function)
            && let Some(class_sym) = &t.symbol
            && class_sym.flags.contains(tsox_frontend::ast::SymbolFlags::Class)
            && !alias.type_arguments.is_empty()
        {
            let args: Vec<String> = alias
                .type_arguments
                .iter()
                .map(|a| self.type_to_string_ex(a, flags))
                .collect();
            return format!("{}<{}>.{}", fn_sym.name, args.join(", "), class_sym.name);
        }

        // Go typeToString：hover flags 带 UseAliasDefinedOutsideCurrentScope 时
        // 优先按别名符号打印（Name<args>）
        if flags.contains(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE)
            && let Some(alias) = &t.alias
            && let Some(sym) = &alias.symbol
        {
            let args: Vec<String> = alias
                .type_arguments
                .iter()
                .map(|a| self.type_to_string_ex(a, flags))
                .collect();
            if args.is_empty() {
                return sym.name.clone();
            }
            return format!("{}<{}>", sym.name, args.join(", "));
        }

        if t.object_flags.contains(ObjectFlags::Tuple) {
            return self.tuple_to_string(t, flags);
        }

        if t.object_flags.contains(ObjectFlags::Reference) {
            return self.reference_to_string(t, flags);
        }

        if let Some(structured) = t.as_structured() {
            // Go createTypeNodeFromObjectType：仅当无属性/索引签名且恰好一条调用
            // （或构造）签名时才输出裸函数形态，否则保留完整对象字面量
            if t.symbol.is_none()
                && structured.signatures.len() == 1
                && structured.properties.is_empty()
                && structured.index_infos.is_empty()
            {
                return self.function_type_to_string(t, structured, flags);
            }
        }

        if let Some(sym) = &t.symbol {
            return self.symbol_type_to_string(t, sym, flags);
        }

        if let Some(structured) = t.as_structured() {
            if !structured.properties.is_empty()
                || !structured.call_signatures().is_empty()
                || !structured.construct_signatures().is_empty()
                || !structured.index_infos.is_empty()
            {
                return self.object_literal_to_string(t, structured, flags);
            }
            // 空匿名对象字面量形态（Go createTypeNodeFromObjectType 无成员 TypeLiteral）
            if t.symbol.is_none() {
                return "{}".to_string();
            }
        }

        if t.flags.contains(TypeFlags::Object) {
            return "object".to_string();
        }
        if t.flags.contains(TypeFlags::Unknown) {
            return "unknown".to_string();
        }

        "<unknown type>".to_string()
    }

    pub(crate) fn literal_value_to_string(&mut self, val: &LiteralValue) -> String {
        match val {
            LiteralValue::String(s) => format!("\"{}\"", s),
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::BigInt(b) => format!("{}n", b.to_string()),
            LiteralValue::Boolean(true) => "true".to_string(),
            LiteralValue::Boolean(false) => "false".to_string(),
            LiteralValue::None => String::new(),
        }
    }
}
