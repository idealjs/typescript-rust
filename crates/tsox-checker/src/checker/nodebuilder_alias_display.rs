#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;

impl Checker {
    pub(crate) fn alias_symbol_for_type_node(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
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

    pub(crate) fn attach_alias_for_type_node(&mut self, node: &Arc<Node>, result: &Arc<Type>) {
        let attachable = match &result.data {
            TypeData::Union(_) | TypeData::Intersection(_) | TypeData::Mapped(_) => true,
            TypeData::Conditional(c) => {
                c.resolved_true_type.get().is_none() && c.resolved_false_type.get().is_none()
            }
            TypeData::Object(_) => result.symbol.is_none(),
            _ => false,
        };
        if !attachable {
            return;
        }
        let Some(alias_sym) = self.alias_symbol_for_type_node(node) else {
            return;
        };
        let args = match &result.data {
            TypeData::Mapped(_) | TypeData::Conditional(_) => {
                let (tp_symbols, _) = self.collect_alias_type_params_and_body(&alias_sym);
                tp_symbols
                    .iter()
                    .map(|tp| self.get_type_parameter_from_symbol(tp))
                    .collect()
            }
            _ => Vec::new(),
        };
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

    pub(crate) fn type_list_strings(
        &mut self,
        types: &[&Arc<Type>],
        flags: TypeFormatFlags,
    ) -> Vec<String> {
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

fn is_bare_identifier_reference(s: &str) -> bool {
    if let Some(pos) = s.find('<') {
        let rest = &s[pos..];
        if !rest.ends_with('>') || !balanced_angle(rest) {
            return false;
        }
        return is_identifier(&s[..pos]);
    }
    is_identifier(s)
}

fn balanced_angle(s: &str) -> bool {
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

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

fn identifier_head(s: &str) -> String {
    match s.find('<') {
        Some(pos) => s[..pos].to_string(),
        None => s.to_string(),
    }
}

fn types_same_reference(a: &Arc<Type>, b: &Arc<Type>) -> bool {
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
