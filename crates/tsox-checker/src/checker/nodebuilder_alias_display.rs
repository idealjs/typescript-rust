#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;

impl Checker {
    pub(crate) fn alias_symbol_for_type_node(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("alias_symbol_for_type_node"); 
        let mut host = node.parent()?;
        loop {
            let recurse = match &host.data {
                NodeData::ParenthesizedTypeNode(_) => true,
                NodeData::TypeOperatorNode(d) if d.operator == SyntaxKind::ReadonlyKeyword => {
                    true
                }
                _ => false,
            };
            if !recurse {
                break;
            }
            host = host.parent()?;
        }
        if matches!(&host.data, NodeData::TypeAliasDeclaration(_)) {
            return self.program.symbol_map().symbol_of(&host).map(Arc::clone);
        }
        None
    }

    pub(crate) fn attach_alias_for_type_node(&mut self, node: &Arc<Node>, result: &Arc<Type>) { ::tsox_core::fntrace::enter("attach_alias_for_type_node"); 
        let attachable = match &result.data {
            TypeData::Union(_) | TypeData::Intersection(_) | TypeData::Mapped(_) => true,
            TypeData::Conditional(c) => {
                c.resolved_true_type.get().is_none() && c.resolved_false_type.get().is_none()
            }
            TypeData::Object(_) => true,
            _ => false,
        };
        if !attachable {
            return;
        }
        let Some(alias_sym) = self.alias_symbol_for_type_node(node) else {
            return;
        };
        // Go getAliasForTypeNode（tsc/internal/checker/checker.go:25100-25108）：
        // typeArguments 统一取 getTypeArgumentsForAliasSymbol（别名声明的局部类型参数）
        let args = self.get_type_arguments_for_alias_symbol(Some(&alias_sym));
        let ptr = Arc::as_ptr(result) as *mut crate::checker::types::Type;
        unsafe {
            if (*ptr).alias.is_none() {
                (*ptr).alias = Some(Box::new(crate::checker::types::TypeAlias::new(
                    Some(alias_sym),
                    args,
                )));
            }
        }
    }

    /// Go getTypeFromArrayOrTupleTypeNode deferred 分支的 alias 附加等价物
    /// （checker.go:25560-25562 → createDeferredTypeReference
    /// checker.go:26612-26621，alias = getAliasForTypeNode）。
    /// isDeferredTypeReferenceNode 首判即 getAliasSymbolForTypeNode != nil
    /// （checker.go:24594-24596， ParenthesizedType/readonly TypeOperator 爬升后
    /// 宿主为 type alias 声明），命中即对解析产物附加别名
    pub(crate) fn attach_type_alias_from_node(
        &mut self,
        node: &Arc<Node>,
        result: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("attach_type_alias_from_node");
        let Some(alias_sym) = self.alias_symbol_for_type_node(node) else {
            return Arc::clone(result);
        };
        if result.alias.is_some() {
            return Arc::clone(result);
        }
        // Go getAliasForTypeNode（checker.go:25104-25112）：typeArguments 取
        // getTypeArgumentsForAliasSymbol（别名声明的局部类型参数）
        let args = self.get_type_arguments_for_alias_symbol(Some(&alias_sym));
        let alias = Box::new(crate::checker::types::TypeAlias::new(
            Some(alias_sym),
            args,
        ));
        match &result.data {
            // 普通元组为本次解析新建对象，原地附加
            TypeData::Tuple(_) => {
                let ptr = Arc::as_ptr(result) as *mut crate::checker::types::Type;
                unsafe {
                    (*ptr).alias = Some(alias);
                }
                Arc::clone(result)
            }
            // Go [...T] 单变长元组不进 deferred 分支（checker.go:25560 变长例外），
            // 归一化回退产物是驻留缓存里的共享数组型（create_array_type_ex
            // intern），保持无 alias；数组直 RHS（type A = T[]）按 Go deferred
            // 分支新建独立引用对象，不写共享驻留对象
            TypeData::Object(otd) => {
                if matches!(&node.data, NodeData::TupleTypeNode(_)) {
                    return Arc::clone(result);
                }
                Arc::new(crate::checker::types::Type {
                    flags: result.flags,
                    object_flags: result.object_flags,
                    id: crate::checker::types::next_type_id(),
                    symbol: result.symbol.clone(),
                    alias: Some(alias),
                    data: TypeData::Object(crate::checker::types::ObjectTypeData {
                        structured: crate::checker::types::StructuredTypeData::default(),
                        target: otd.target.clone(),
                        mapper: otd.mapper.clone(),
                        type_arguments: otd.type_arguments.clone(),
                        node: otd.node.clone(),
                    }),
                })
            }
            _ => Arc::clone(result),
        }
    }

    pub(crate) fn type_list_strings(
        &mut self,
        types: &[&Arc<Type>],
        flags: TypeFormatFlags,
    ) -> Vec<String> { ::tsox_core::fntrace::enter("type_list_strings"); 
        let rendered: Vec<String> = types
            .iter()
            .map(|t| self.type_to_string_ex(t, flags))
            .collect();
        let mut needs_fq = vec![false; types.len()];
        let mut by_name: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();
        for (i, s) in rendered.iter().enumerate() {
            if !is_bare_identifier_reference(s) {
                continue;
            }
            by_name.entry(identifier_head(s)).or_default().push(i);
        }
        for (_, idxs) in by_name {
            if idxs.len() < 2 {
                continue;
            }
            let homogeneous = idxs.windows(2).all(|w| {
                types_same_reference(types[w[0]], types[w[1]])
            });
            if !homogeneous {
                for i in idxs {
                    needs_fq[i] = true;
                }
            }
        }
        rendered
            .into_iter()
            .enumerate()
            .zip(&needs_fq)
            .map(|((i, s), fq)| {
                if *fq {
                    self.fully_qualified_type_string(types[i])
                } else {
                    s
                }
            })
            .collect()
    }
}

fn is_bare_identifier_reference(s: &str) -> bool { ::tsox_core::fntrace::enter("is_bare_identifier_reference"); 
    if let Some(pos) = s.find('<') {
        let rest = &s[pos..];
        if !rest.ends_with('>') || !balanced_angle(rest) {
            return false;
        }
        return is_identifier(&s[..pos]);
    }
    is_identifier(s)
}

fn balanced_angle(s: &str) -> bool { ::tsox_core::fntrace::enter("balanced_angle"); 
    let mut depth = 0i32;
    for ch in s.chars() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            '\'' | '"' | '`' => return false,
            _ => {}
        }
    }
    depth == 0
}

fn is_identifier(s: &str) -> bool { ::tsox_core::fntrace::enter("is_identifier"); 
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

fn identifier_head(s: &str) -> String { ::tsox_core::fntrace::enter("identifier_head"); 
    match s.find('<') {
        Some(pos) => s[..pos].to_string(),
        None => s.to_string(),
    }
}

fn types_same_reference(a: &Arc<Type>, b: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("types_same_reference"); 
    if Arc::ptr_eq(a, b) {
        return true;
    }
    if let (Some(sa), Some(sb)) = (&a.symbol, &b.symbol) {
        if Arc::ptr_eq(sa, sb) {
            return true;
        }
    }
    match (&a.alias, &b.alias) {
        (Some(x), Some(y)) => match (&x.symbol, &y.symbol) {
            (Some(pa), Some(pb)) => Arc::ptr_eq(pa, pb),
            _ => false,
        },
        _ => false,
    }
}
