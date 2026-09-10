#![allow(unused_imports)]

use crate::checker::flow_union_ops::*;
use tsox_frontend::ast::CheckFlags;

impl Checker {
    /// Go getPropertyOfUnionOrIntersectionType：在联合/交集各成分中查同名属性，
    /// 多成分命中时合成带 containingType 的属性符号（声明合并、类型取并/交）
    pub(crate) fn get_union_or_intersection_property(
        &mut self,
        containing_type: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let types: Vec<Arc<Type>> = containing_type.types()?.to_vec();
        let is_union = containing_type.is_union();

        let mut single_prop: Option<Arc<Symbol>> = None;
        let mut prop_flags = SymbolFlags::Property;
        let mut found: Vec<Arc<Symbol>> = Vec::new();
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
                    if !found.iter().any(|p| Arc::ptr_eq(p, &prop)) {
                        found.push(prop);
                    }
                }
                None => {
                    if is_union {
                        return None;
                    }
                }
            }
        }

        let single = single_prop?;
        if found.len() == 1 {
            return Some(single);
        }

        let mut declarations: Vec<Arc<Node>> = Vec::new();
        let mut prop_types: Vec<Arc<Type>> = Vec::new();
        let mut first_parent: Option<Arc<Symbol>> = None;
        for prop in &found {
            for d in &prop.declarations {
                if !declarations.iter().any(|x| Arc::ptr_eq(x, d)) {
                    declarations.push(Arc::clone(d));
                }
            }
            if first_parent.is_none() {
                first_parent = self
                    .parent_symbol_of_declaration_chain(prop)
                    .or_else(|| prop.parent.clone());
            }
            prop_types.push(self.get_type_of_symbol(prop));
        }

        let mut result = Symbol::new(prop_flags, name.to_string());
        result.check_flags = CheckFlags::SyntheticProperty;
        result.declarations = declarations;
        result.parent = first_parent;
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
        let parent_node = decl.parent.as_ref()?;
        self.program
            .symbol_map()
            .symbol_of(parent_node)
            .map(Arc::clone)
    }

    pub fn is_error_type(&self, t: &Arc<Type>) -> bool {
        t.intrinsic_name() == Some("error")
    }
}
