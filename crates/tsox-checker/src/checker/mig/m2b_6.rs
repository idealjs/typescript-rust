use crate::checker::checker::*;
use crate::checker::checker_heritage_retry_limit::TypeResolutionProperty;
use crate::checker::mig::m1d_2::get_base_type_node_of_class;
use crate::checker::mig::wc3::r26k2_defs::set_interface_resolved_base_types;
use crate::checker::types_type_flags_instantiable_non_primitive::{
    ObjectFlags, OBJECT_FLAGS_CLASS_OR_INTERFACE,
};
use std::sync::Arc;
use tsox_core::diagnostics as msg;
use tsox_frontend::ast::SymbolFlags;

impl Checker {
    pub fn get_base_types(&mut self, t: &Arc<Type>) -> Vec<Arc<Type>> {
        if !t
            .object_flags
            .intersects(OBJECT_FLAGS_CLASS_OR_INTERFACE.union(ObjectFlags::Tuple))
        {
            return Vec::new();
        }
        if let Some(interface) = t.as_interface_type()
            && interface.base_types_resolved
        {
            return interface.resolved_base_types.clone();
        }
        let Some(symbol) = t.symbol.clone() else {
            return Vec::new();
        };
        if !self.push_type_resolution(
            Arc::as_ptr(&symbol),
            TypeResolutionProperty::ResolvedBaseTypes,
        ) {
            return t
                .as_interface_type()
                .map(|i| i.resolved_base_types.clone())
                .unwrap_or_default();
        }
        if t.object_flags.contains(ObjectFlags::Tuple) {
            let tuple_base_type = self.get_tuple_base_type(t);
            set_interface_resolved_base_types(t, vec![tuple_base_type]);
        } else if symbol
            .flags
            .intersects(SymbolFlags::Class.union(SymbolFlags::Interface))
        {
            if symbol.flags.intersects(SymbolFlags::Class) {
                self.resolve_base_types_of_class(t);
            }
            if symbol.flags.intersects(SymbolFlags::Interface) {
                self.resolve_base_types_of_interface(t);
            }
        }
        let circular = !self.pop_type_resolution();
        if circular {
            for declaration in symbol.declarations.iter() {
                if tsox_frontend::ast::is_class_declaration(declaration)
                    || tsox_frontend::ast::is_interface_declaration(declaration)
                {
                    self.report_circular_base_type(declaration, t);
                }
            }
        }
        clear_members_resolved(t);
        set_base_types_resolved(t);
        t.as_interface_type()
            .map(|i| i.resolved_base_types.clone())
            .unwrap_or_default()
    }

    pub fn get_base_constructor_type_of_class(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let Some(resolved) = t
            .as_interface_type()
            .and_then(|i| i.resolved_base_constructor_type.get().cloned())
        {
            return Some(resolved);
        }
        if let Some(resolved) = self.base_ctor_type_cache.get(&t.id) {
            return Some(Arc::clone(resolved));
        }
        let base_type_node = get_base_type_node_of_class(t)?;
        let Some(symbol) = t.symbol.clone() else {
            return None;
        };
        if !self.push_type_resolution(
            Arc::as_ptr(&symbol),
            TypeResolutionProperty::ResolvedBaseConstructorType,
        ) {
            let error = self.error_type();
            self.base_ctor_type_cache.insert(t.id, Arc::clone(&error));
            return Some(error);
        }
        let expression = base_type_node.expression().unwrap();
        let base_constructor_type = self.heritage_extends_value_type(&expression);
        if base_constructor_type
            .flags
            .intersects(TypeFlags::Object | TypeFlags::Intersection)
        {
            self.resolve_structured_type_members(&base_constructor_type);
        }
        if !self.pop_type_resolution() {
            let declaration = symbol.value_declaration.clone().unwrap();
            let name = self.symbol_to_string(&symbol);
            self.error_message(
                &declaration,
                msg::X_0_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ITS_OWN_BASE_EXPRESSION,
                &[name],
            );
            set_resolved_base_constructor_type(t, self.error_type());
            let error = self.error_type();
            self.base_ctor_type_cache.insert(t.id, Arc::clone(&error));
            return t
                .as_interface_type()
                .and_then(|i| i.resolved_base_constructor_type.get().cloned())
                .or(Some(error));
        }
        if !base_constructor_type.flags.intersects(TypeFlags::Any)
            && !Arc::ptr_eq(&base_constructor_type, &self.null_widening_type())
            && !self.is_constructor_type(&base_constructor_type)
        {
            let text = self.type_to_string(&base_constructor_type);
            let mut diag = tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                expression.loc,
                msg::TYPE_0_IS_NOT_A_CONSTRUCTOR_FUNCTION_TYPE,
                vec![text],
            );
            if base_constructor_type.flags.contains(TypeFlags::TypeParameter) {
                let constraint = self.get_constraint_from_type_parameter(&base_constructor_type);
                let ctor_return = self
                    .get_signatures_of_type(&constraint, SignatureKind::Construct)
                    .into_iter()
                    .next()
                    .and_then(|sig| self.get_return_type_of_signature(&sig))
                    .map(|ty| self.type_to_string(&ty))
                    .unwrap_or_else(|| "unknown".to_string());
                if let Some(tp_symbol) = &base_constructor_type.symbol
                    && let Some(decl) = tp_symbol.declarations.first()
                {
                    let related_file = self
                        .get_source_file_of_node(decl)
                        .or_else(|| self.current_file.clone());
                    diag.related_information.push(tsox_frontend::ast::Diagnostic::new(
                        related_file,
                        decl.loc,
                        msg::DID_YOU_MEAN_FOR_0_TO_BE_CONSTRAINED_TO_TYPE_NEW_ARGS_COLON_ANY_1,
                        vec![tp_symbol.name.clone(), ctor_return],
                    ));
                }
            }
            self.diagnostics.add(diag);
            set_resolved_base_constructor_type(t, self.error_type());
            let error = self.error_type();
            self.base_ctor_type_cache.insert(t.id, Arc::clone(&error));
            return t
                .as_interface_type()
                .and_then(|i| i.resolved_base_constructor_type.get().cloned())
                .or(Some(error));
        }
        set_resolved_base_constructor_type(t, Arc::clone(&base_constructor_type));
        self.base_ctor_type_cache
            .insert(t.id, Arc::clone(&base_constructor_type));
        Some(base_constructor_type)
    }
}

fn set_base_types_resolved(t: &Arc<Type>) {
    let mut owned = Arc::clone(t);
    let Some(owned) = Arc::get_mut(&mut owned) else { return };
    match &mut owned.data {
        TypeData::Interface(interface) => interface.base_types_resolved = true,
        TypeData::Tuple(tuple) => tuple.interface_data.base_types_resolved = true,
        _ => {}
    }
}

fn clear_members_resolved(t: &Arc<Type>) {
    let mut owned = Arc::clone(t);
    let Some(owned) = Arc::get_mut(&mut owned) else { return };
    owned.object_flags.remove(ObjectFlags::MembersResolved);
}

fn set_resolved_base_constructor_type(t: &Arc<Type>, base_constructor_type: Arc<Type>) {
    if let Some(interface) = t.as_interface_type() {
        let _ = interface.resolved_base_constructor_type.set(base_constructor_type);
    }
}
