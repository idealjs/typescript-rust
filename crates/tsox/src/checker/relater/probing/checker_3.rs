#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn substitute_object_properties_deep(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        let key = t.id;
        if let Some(cached) = self.subst_object_in_progress.get(&key) {
            return Arc::clone(cached);
        }
        let Some(o) = t.as_object() else {
            return Arc::clone(t);
        };

        let mut old_types: Vec<Option<Arc<Type>>> =
            Vec::with_capacity(o.structured.properties.len());
        for prop in &o.structured.properties {
            old_types.push(
                self.value_symbol_links
                    .get(prop)
                    .and_then(|l| l.resolved_type.clone()),
            );
        }

        let shell = Arc::new(Type::new(
            t.flags,
            TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: o.structured.members.clone(),
                    properties: o.structured.properties.clone(),
                    signatures: o.structured.signatures.clone(),
                    call_signature_count: o.structured.call_signature_count,
                    index_infos: o.structured.index_infos.clone(),
                    ..Default::default()
                },
                target: o.target.clone(),
                mapper: o.mapper.clone(),
                type_arguments: o.type_arguments.clone(),
            }),
        ));
        {
            let shell_mut = Arc::as_ptr(&shell) as *mut Type;
            unsafe {
                (*shell_mut).object_flags = t.object_flags;
                (*shell_mut).symbol = t.symbol.clone();
            }
        }
        self.subst_object_in_progress
            .insert(key, Arc::clone(&shell));
        let mut changed = false;
        let mut new_props: Vec<Arc<Symbol>> = Vec::with_capacity(o.structured.properties.len());
        let mut new_members = o.structured.members.clone();
        for (prop, old_t) in o.structured.properties.iter().zip(old_types.iter()) {
            let Some(old_t) = old_t else {
                new_props.push(Arc::clone(prop));
                continue;
            };
            let new_t = self.substitute_infer_type_parameters(old_t, params, substitutions);
            if Arc::ptr_eq(&new_t, old_t) {
                new_props.push(Arc::clone(prop));
                continue;
            }
            changed = true;
            let mut new_sym = Symbol::new(prop.flags, prop.name.clone());
            new_sym.declarations = prop.declarations.clone();
            new_sym.check_flags = prop.check_flags;
            let new_sym = Arc::new(new_sym);
            self.value_symbol_links.insert(
                &new_sym,
                ValueSymbolLinks {
                    resolved_type: Some(new_t),
                    ..Default::default()
                },
            );
            new_members.insert(prop.name.clone(), Arc::clone(&new_sym));
            new_props.push(new_sym);
        }
        if !changed {
            self.subst_object_in_progress.remove(&key);
            return Arc::clone(t);
        }

        {
            let shell_mut = Arc::as_ptr(&shell) as *mut Type;
            unsafe {
                if let TypeData::Object(so) = &mut (*shell_mut).data {
                    so.structured.members = new_members;
                    so.structured.properties = new_props;
                }
            }
        }
        shell
    }
}
