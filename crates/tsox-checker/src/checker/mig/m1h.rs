#![allow(unused_imports)]
#![allow(dead_code)]
use tsox_frontend::ast::{is_import_call, is_in_js_file, Diagnostic};
use super::m2a::is_thisless;
use super::w9a::new_type_mapper;
use crate::checker::types_type_flags_instantiable_non_primitive::SIGNATURE_FLAGS_PROPAGATING_FLAGS;
use crate::checker::inference_inference_key_2::InferenceFlags;
use tsox_core::diagnostics;

#[path = "r20k7_defs.rs"]
pub(crate) mod r20k7_defs;
pub(crate) use r20k7_defs::*;

#[path = "r22k7_defs.rs"]
pub(crate) mod r22k7_defs;
pub(crate) use r22k7_defs::*;

#[path = "r23k7_defs.rs"]
pub(crate) mod r23k7_defs;
pub(crate) use r23k7_defs::*;

use crate::checker::checker::*;
use std::sync::Arc;

impl Checker {
    pub fn initialize_closures(&mut self) { ::tsox_core::fntrace::enter("initialize_closures"); }

    pub fn initialize_iteration_resolvers(&mut self) { ::tsox_core::fntrace::enter("initialize_iteration_resolvers"); }

    pub fn include_mixin_type(
        &mut self,
        t: &Arc<Type>,
        types: &[Arc<Type>],
        mixin_flags: &[bool],
        index: usize,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("include_mixin_type"); 
        let mut mixed_types: Vec<Arc<Type>> = Vec::new();
        for i in 0..types.len() {
            if i == index {
                mixed_types.push(Arc::clone(t));
            } else if mixin_flags[i] {
                // Go includeMixinType（checker.go:22639）取 mixin 成分的构造
                // 签名（isMixinConstructorType 保证恰一个）；Call 签名恒空，
                // [0] 索引即 index-out-of-bounds panic
                let ctor_sigs = self.get_signatures_of_type(&types[i], SignatureKind::Construct);
                if let Some(ret) = ctor_sigs.first().and_then(|s| self.get_return_type_of_signature(s)) {
                    mixed_types.push(ret);
                }
            }
        }
        self.get_intersection_type(mixed_types)
    }

    pub fn infer_signature_instantiation_for_overload_failure(
        &mut self,
        node: &Arc<Node>,
        type_parameters: &[Arc<Type>],
        candidate: &Arc<Signature>,
        args: &[Arc<Node>],
        check_mode: CheckMode,
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("infer_signature_instantiation_for_overload_failure"); 
        let inference_flags = if is_in_js_file(node) {
            InferenceFlags::AnyDefault
        } else {
            InferenceFlags::None
        };
        let mut inference_context =
            self.new_inference_context(type_parameters, Some(Arc::clone(candidate)), inference_flags, None);
        let type_argument_types = self.infer_type_arguments(
            node,
            candidate,
            args,
            &mut inference_context,
            check_mode | CheckMode::SkipContextSensitive | CheckMode::SkipGenericFunctions,
        );
        self.create_signature_instantiation(candidate, &type_argument_types)
    }

    pub fn invocation_error(
        &mut self,
        error_target: &Arc<Node>,
        apparent_type: &Arc<Type>,
        kind: SignatureKind,
        related_information: Option<Diagnostic>,
    ) { ::tsox_core::fntrace::enter("invocation_error"); 
        let mut diagnostic = self.invocation_error_details(error_target, apparent_type, kind);
        if let Some(d) = diagnostic.as_mut() {
            if let Some(related) = related_information {
                d.add_related_info(related);
            }
        }
        if let Some(d) = diagnostic {
            let d = self.add_diagnostic(d);
            self.invocation_error_recovery(apparent_type, kind, &d);
        }
    }

    pub fn invocation_error_recovery(
        &mut self,
        apparent_type: &Arc<Type>,
        kind: SignatureKind,
        diagnostic: &Diagnostic,
    ) { ::tsox_core::fntrace::enter("invocation_error_recovery"); 
        let symbol = match &apparent_type.symbol {
            Some(symbol) => Arc::clone(symbol),
            None => return,
        };
        let import_node = self
            .export_type_links
            .get(&symbol)
            .and_then(|l| l.originating_import.clone());
        if let Some(import_node) = import_node.filter(|n| !is_import_call(n)) {
            let target_symbol = self
                .export_type_links
                .get(&symbol)
                .and_then(|l| l.target.clone());
            let target_type = match target_symbol {
                Some(t) => self.get_type_of_symbol(&t),
                None => return,
            };
            let sigs = self.get_signatures_of_type(&target_type, kind);
            if sigs.is_empty() {
                return;
            }
            let mut d = diagnostic.clone();
            d.add_related_info(Diagnostic::new(
                None,
                import_node.loc,
                diagnostics::TYPE_ORIGINATES_AT_THIS_IMPORT_A_NAMESPACE_STYLE_IMPORT_CANNOT_BE_CALLED_OR_CONSTRUCTED_AND_WILL_CAUSE_A_FAILURE_AT_RUNTIME_CONSIDER_USING_A_DEFAULT_IMPORT_OR_IMPORT_REQUIRE_HERE_INSTEAD,
                Vec::new(),
            ));
        }
    }

    pub fn intersect_types(
        &mut self,
        type1: Option<&Arc<Type>>,
        type2: Option<&Arc<Type>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("intersect_types"); 
        match (type1, type2) {
            (None, t2) => t2.map(Arc::clone),
            (t1, None) => t1.map(Arc::clone),
            (Some(t1), Some(t2)) => {
                Some(self.get_intersection_type(vec![Arc::clone(t1), Arc::clone(t2)]))
            }
        }
    }

    pub fn instantiate_signature(
        &mut self,
        sig: &Arc<Signature>,
        m: Option<&Arc<TypeMapper>>,
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("instantiate_signature"); 
        let erase_type_parameters = m.is_some_and(|m| is_permissive_mapper(m));
        self.instantiate_signature_ex(sig, m, erase_type_parameters)
    }

    pub fn instantiate_signature_ex(
        &mut self,
        sig: &Arc<Signature>,
        m: Option<&Arc<TypeMapper>>,
        erase_type_parameters: bool,
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("instantiate_signature_ex"); 
        let mut m = m.map(Arc::clone);
        let mut fresh_type_parameters: Vec<Arc<Type>> = Vec::new();
        if !sig.type_parameters.is_empty() && !erase_type_parameters {
            fresh_type_parameters = sig
                .type_parameters
                .iter()
                .map(|tp| self.clone_type_parameter(tp))
                .collect();
            let fresh_mapper =
                new_type_mapper(sig.type_parameters.clone(), fresh_type_parameters.clone());
            m = self.combine_type_mappers(Some(&Arc::new(fresh_mapper)), m.as_ref());
            // Go instantiateSignatureEx（checker.go:21846-21852）：fresh 型参逐个
            // 挂组合 mapper（tp.AsTypeParameter().mapper = m），约束惰性实例化
            //（getConstraintFromTypeParameter checker.go:18078-18080 tp.target
            // 约束经 tp.mapper 代换）产出的 V 约束 U[] 与签名声明 fresh U 同一
            // 对象，渲染位签名作用域内呈 U 不落阴影改名。Arc::get_mut 在
            // fresh_type_parameters.clone() 进 mapper 实参表后恒 None，静默
            // 跳过使约束代换丢 {old→fresh} 段回落原约束（旧 U），渲染位
            // 签名声明表外呈 U_1（typeParameterConstraintInstantiation 的
            // >m.map : <U extends string, V extends U_1[]> 应 U[]）——共享
            // 分配经裸指针写入（本仓 declare_symbol 等构造窗口同型先例），
            // 克隆体尚未发布、单线程构造窗内写共享分配无竞争面
            if let Some(mm) = m.as_ref() {
                for tp in &fresh_type_parameters {
                    let tp_ptr = Arc::as_ptr(tp) as *mut crate::checker::types::Type;
                    unsafe {
                        (*tp_ptr).set_type_parameter_mapper(mm);
                    }
                }
            }
        }
        let this_parameter = match &sig.this_parameter {
            Some(this) => self.instantiate_symbol(this, m.as_ref()),
            None => None,
        };
        let parameters: Vec<Arc<Symbol>> = self.instantiate_symbols(&sig.parameters, m.as_ref());
        // Go instantiateSignatureEx（checker.go:21857）resolvedReturnType 传 nil：
        // 返回型沿 target+mapper 惰性回填（getReturnTypeOfSignature
        // checker.go:21190），占位 unknown 会经 OnceLock 写死短路惰性链
        let mut result = self.new_signature_unset_return(
            sig.flags & SIGNATURE_FLAGS_PROPAGATING_FLAGS,
            sig.declaration.as_ref(),
            &fresh_type_parameters,
            this_parameter.as_ref(),
            &parameters,
            sig.min_argument_count as usize,
        );
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.target = Some(Arc::clone(sig));
            result_mut.mapper = m;
        }
        result
    }

    pub fn instantiate_index_info(
        &mut self,
        info: &Arc<IndexInfo>,
        m: Option<&Arc<TypeMapper>>,
    ) -> Arc<IndexInfo> { ::tsox_core::fntrace::enter("instantiate_index_info"); 
        let value_type = match &info.value_type {
            Some(v) => Arc::clone(v),
            None => return Arc::clone(info),
        };
        let new_value_type = self.instantiate_type(&value_type, m);
        if Arc::ptr_eq(&new_value_type, &value_type) {
            return Arc::clone(info);
        }
        let key_type = info
            .key_type
            .clone()
            .unwrap_or_else(|| Arc::clone(&self.unknown_type()));
        self.new_index_info(
            &key_type,
            &new_value_type,
            info.is_readonly,
            info.declaration.as_ref(),
            &info.components,
        )
    }

    pub fn instantiate_symbol_table(
        &mut self,
        symbols: &SymbolTable,
        m: Option<&Arc<TypeMapper>>,
    ) -> SymbolTable { ::tsox_core::fntrace::enter("instantiate_symbol_table"); 
        if symbols.is_empty() {
            return SymbolTable::new();
        }
        let mut result = SymbolTable::new();
        for (id, symbol) in symbols.iter() {
            if self.is_named_member(symbol, id) {
                if let Some(instantiated) = self.instantiate_symbol(symbol, m) {
                    result.insert(id.clone(), instantiated);
                }
            }
        }
        result
    }

    pub fn instantiate_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        m: Option<&Arc<TypeMapper>>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("instantiate_symbol"); 
        let mut symbol = Arc::clone(symbol);
        if m.is_some_and(|m| m.maps_this_only()) && is_thisless(&symbol) {
            return Some(symbol);
        }
        let (resolved_type, write_type) = match self.value_symbol_links.get(&symbol) {
            Some(l) => (l.resolved_type.clone(), l.write_type.clone()),
            None => (None, None),
        };
        if let Some(resolved_type) = &resolved_type {
            if !self.could_contain_type_variables(resolved_type) {
                if !symbol.flags.intersects(SymbolFlags::SetAccessor) {
                    return Some(symbol);
                }
                if let Some(write_type) = &write_type {
                    if !self.could_contain_type_variables(write_type) {
                        return Some(symbol);
                    }
                }
            }
        }
        let mut m = m.map(Arc::clone);
        let (links_target, links_mapper, links_name_type) = match self.value_symbol_links.get(&symbol)
        {
            Some(l) => (l.target.clone(), l.mapper.clone(), l.name_type.clone()),
            None => (None, None, None),
        };
        if symbol.check_flags.intersects(CheckFlags::Instantiated) {
            if let Some(target) = links_target {
                symbol = target;
                m = self.combine_type_mappers(links_mapper.as_ref(), m.as_ref());
            }
        }
        let mut result = Symbol::new(symbol.flags | SymbolFlags::Transient, symbol.name.clone());
        result.check_flags = CheckFlags::Instantiated
            | symbol.check_flags
                & (CheckFlags::Readonly
                    | CheckFlags::Late
                    | CheckFlags::OptionalParameter
                    | CheckFlags::RestParameter);
        result.declarations = symbol.declarations.clone();
        if let Some(parent) = symbol.parent() {
            result.set_parent(&parent);
        }
        result.value_declaration = symbol.value_declaration.clone();
        let result = Arc::new(result);
        let result_links = self.value_symbol_links.get_or_default(&result);
        result_links.target = Some(Arc::clone(&symbol));
        result_links.mapper = m;
        result_links.name_type = links_name_type;
        Some(result)
    }

    pub fn instantiate_type(
        &mut self,
        t: &Arc<Type>,
        m: Option<&Arc<TypeMapper>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("instantiate_type"); 
        self.instantiate_type_with_alias(t, m, None)
    }
}
