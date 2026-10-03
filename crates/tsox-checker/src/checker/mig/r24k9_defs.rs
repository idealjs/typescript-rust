#![allow(unused_imports)]
#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use crate::checker::types::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
use std::sync::{Arc, OnceLock};

static RESOLVING_DEFAULT_TYPE: OnceLock<Arc<Type>> = OnceLock::new();

impl Checker {
    pub fn resolving_default_type(&mut self) -> Arc<Type> { ::tsox_core::fntrace::enter("resolving_default_type"); 
        if let Some(t) = RESOLVING_DEFAULT_TYPE.get() {
            return t.clone();
        }
        let t = self.new_anonymous_type(
            &self.unknown_symbol(),
            SymbolTable::new(),
            vec![],
            vec![],
            vec![],
        );
        let _ = RESOLVING_DEFAULT_TYPE.set(t.clone());
        t
    }

    pub fn create_promise_type(&mut self, promised_type: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("create_promise_type"); 
        match self.get_promise_type() {
            Some(global_promise_type) if global_promise_type.id != self.empty_generic_type().id => {
                let unwrapped = self.unwrap_awaited_type(promised_type);
                let promised = self
                    .get_awaited_type_no_alias(&unwrapped)
                    .unwrap_or_else(|| self.unknown_type());
                self.create_type_reference(&global_promise_type, &[promised])
            }
            _ => self.unknown_type(),
        }
    }

    pub fn get_global_record_symbol(&mut self) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_record_symbol"); 
        self.get_global_type_alias_symbol("Record", 2, true)
    }

    pub fn get_global_omit_symbol(&mut self) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_omit_symbol"); 
        self.get_global_type_alias_symbol("Omit", 2, true)
    }

    pub fn unwrap_return_type(
        &mut self,
        return_type: &Arc<Type>,
        function_flags: tsox_frontend::ast::mig::m3e::FunctionFlags,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("unwrap_return_type"); 
        use tsox_frontend::ast::mig::m3e::FunctionFlags;
        let is_generator = function_flags.contains(FunctionFlags::GENERATOR);
        let is_async = function_flags.contains(FunctionFlags::ASYNC);
        if is_generator {
            let Some(return_iteration_type) = self.get_iteration_type_of_generator_function_return_type(
                IterationTypeKind::RETURN,
                return_type,
                is_async,
            ) else {
                return Some(self.error_type());
            };
            if is_async {
                let unwrapped = self.unwrap_awaited_type(&return_iteration_type);
                return self.get_awaited_type_no_alias(&unwrapped);
            }
            return Some(return_iteration_type);
        }
        if is_async {
            return self
                .get_awaited_type_no_alias(return_type)
                .or_else(|| Some(self.error_type()));
        }
        Some(return_type.clone())
    }

    pub fn get_contextual_return_type(
        &mut self,
        function_decl: &Arc<Node>,
        context_flags: ContextFlags,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_contextual_return_type"); 
        let _ = context_flags;
        if let Some(return_type) = self.get_return_type_from_annotation_opt(function_decl) {
            return Some(return_type);
        }
        let signature = self.get_contextual_signature_for_function_like_declaration(function_decl);
        if let Some(signature) = &signature {
            if !self.is_resolving_return_type_of_signature(signature) {
                let return_type = self.get_return_type_of_signature(signature);
                let function_flags = tsox_frontend::ast::mig::m3e::get_function_flags(Some(function_decl));
                let is_generator =
                    function_flags.contains(tsox_frontend::ast::mig::m3e::FunctionFlags::GENERATOR);
                let is_async =
                    function_flags.contains(tsox_frontend::ast::mig::m3e::FunctionFlags::ASYNC);
                if is_generator {
                    return return_type
                        .and_then(|rt| self.filter_type_for_contextual_return(&rt, function_flags, false));
                }
                if is_async {
                    return return_type
                        .and_then(|rt| self.filter_type_for_contextual_return(&rt, function_flags, true));
                }
                return return_type;
            }
        }
        None
    }

    fn get_return_type_from_annotation_opt(&mut self, declaration: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_return_type_from_annotation_opt"); 
        if is_constructor_declaration(declaration) {
            if let Some(parent_symbol) = declaration.parent().as_ref().and_then(|p| self.symbol_of_node(p)) {
                let merged = self.get_merged_symbol(&parent_symbol);
                return Some(self.get_declared_type_of_class_or_interface(&merged));
            }
            return None;
        }
        if let Some(return_type) = declaration.type_node() {
            return Some(self.get_type_from_type_node(return_type));
        }
        if is_get_accessor_declaration(declaration) && self.has_bindable_name(declaration) {
            let set_accessor = self
                .get_symbol_of_declaration_opt(declaration)
                .and_then(|s| self.get_declaration_of_kind(&s, SyntaxKind::SetAccessor));
            if let Some(set_accessor) = set_accessor {
                if let Some(t) = self.get_annotated_accessor_type(&set_accessor) {
                    return Some(t);
                }
            }
        }
        self.get_return_type_of_full_signature(declaration)
    }

    fn type_passes_contextual_return_filter(
        &mut self,
        t: &Arc<Type>,
        function_flags: tsox_frontend::ast::mig::m3e::FunctionFlags,
        is_async: bool,
    ) -> bool { ::tsox_core::fntrace::enter("type_passes_contextual_return_filter"); 
        use tsox_frontend::ast::mig::m3e::FunctionFlags;
        let predicate_flags =
            TypeFlags::ANY_OR_UNKNOWN | TypeFlags::Void | TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE;
        if t.flags.intersects(predicate_flags) {
            return true;
        }
        if is_async {
            return self.get_awaited_type_of_promise(t).is_some();
        }
        self.check_generator_instantiation_assignability_to_return_type_nil_error(t, function_flags)
    }

    fn filter_type_for_contextual_return(
        &mut self,
        t: &Arc<Type>,
        function_flags: tsox_frontend::ast::mig::m3e::FunctionFlags,
        is_async: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("filter_type_for_contextual_return"); 
        if t.flags.intersects(TypeFlags::Union) {
            let types = t.types().unwrap_or(&[]).to_vec();
            let mut kept: Vec<Arc<Type>> = Vec::new();
            for ct in &types {
                if self.type_passes_contextual_return_filter(ct, function_flags, is_async) {
                    kept.push(ct.clone());
                }
            }
            if kept.is_empty() {
                return None;
            }
            return Some(self.get_union_type(kept));
        }
        if self.type_passes_contextual_return_filter(t, function_flags, is_async) {
            return Some(t.clone());
        }
        None
    }

    fn check_generator_instantiation_assignability_to_return_type_nil_error(
        &mut self,
        return_type: &Arc<Type>,
        function_flags: tsox_frontend::ast::mig::m3e::FunctionFlags,
    ) -> bool { ::tsox_core::fntrace::enter("check_generator_instantiation_assignability_to_return_type_nil_error"); 
        use tsox_frontend::ast::mig::m3e::FunctionFlags;
        let is_async = function_flags.contains(FunctionFlags::ASYNC);
        let generator_yield_type = self
            .get_iteration_type_of_generator_function_return_type(IterationTypeKind::YIELD, return_type, is_async)
            .unwrap_or_else(|| self.any_type());
        let generator_return_type = self
            .get_iteration_type_of_generator_function_return_type(IterationTypeKind::RETURN, return_type, is_async)
            .unwrap_or_else(|| Arc::clone(&generator_yield_type));
        let generator_next_type = self
            .get_iteration_type_of_generator_function_return_type(IterationTypeKind::NEXT, return_type, is_async)
            .unwrap_or_else(|| self.unknown_type());
        let generator_instantiation =
            self.create_generator_type(&generator_yield_type, &generator_return_type, &generator_next_type, is_async);
        self.check_type_assignable_to(&generator_instantiation, return_type, None, None)
    }
}

pub(crate) fn is_resolving_default_sentinel(target_default: &Arc<Type>, _t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_resolving_default_sentinel"); 
    RESOLVING_DEFAULT_TYPE
        .get()
        .is_some_and(|m| m.id == target_default.id)
}

pub(crate) use super::r24k9b_expression_context::{is_expression_node, is_in_expression_context};

pub(crate) fn primitive_type_alias_suggestions() -> Vec<(&'static str, Arc<Symbol>)> { ::tsox_core::fntrace::enter("primitive_type_alias_suggestions"); 
    [
        ("string", "String"),
        ("number", "Number"),
        ("boolean", "Boolean"),
        ("object", "Object"),
        ("bigint", "BigInt"),
        ("symbol", "Symbol"),
    ]
    .iter()
    .map(|(primitive, builtin)| {
        (
            *builtin,
            Arc::new(Symbol::new(
                SymbolFlags::TypeAlias | SymbolFlags::Transient,
                *primitive,
            )),
        )
    })
    .collect()
}

pub(crate) fn string_mapping_key_hash(symbol: &Arc<Symbol>, t: &Arc<Type>) -> u64 { ::tsox_core::fntrace::enter("string_mapping_key_hash"); 
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write_usize(Arc::as_ptr(symbol) as *const () as usize);
    hasher.write_usize(Arc::as_ptr(t) as *const () as usize);
    hasher.finish()
}
