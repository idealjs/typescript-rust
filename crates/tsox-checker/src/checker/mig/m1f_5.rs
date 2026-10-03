#![allow(unused_imports)]
use crate::checker::mig::m1f::r19k9_defs::R19K9NodeExt;
use crate::checker::mig::wc3::NodeAccessExt;

#[path = "r24k10_defs.rs"]
pub mod r24k10_defs;

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;

pub(crate) use crate::checker::checker::*;
pub(crate) use super::m1f::*;
pub(crate) use super::m1f_4::*;
#[allow(unused_imports)]
use tsox_core::core::compiler_options_kinds::{ResolutionMode, JsxEmit};
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3e_4::get_this_container;
#[allow(unused_imports)]
use crate::checker::mig::w9a::new_type_mapper;
#[allow(unused_imports)]
use crate::checker::mig::m2a::is_type_reference_with_generic_arguments;
#[allow(unused_imports)]
use crate::checker::mig::wc2_2::get_type_instantiation_key;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3d_2::new_diagnostic_chain;
#[allow(unused_imports)]
use tsox_core::tspath::{is_declaration_file_name, remove_extension};
#[allow(unused_imports)]
use tsox_core::jsnum::Number;
#[allow(unused_imports)]
use tsox_core::diagnostics::Message;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3c::type_argument_list;
#[allow(unused_imports)]
use tsox_frontend::scanner::skip_trivia;
#[allow(unused_imports)]
use crate::checker::mig::m1e::r18k5_helpers::new_diagnostic;
use crate::checker::mig::m1f::r25k6_defs;
#[allow(unused_imports)]
use tsox_core::core::text::TextRange;
#[allow(unused_imports)]
use crate::checker::mig::m1e::r20k2_defs::is_in_js_file;
#[allow(unused_imports)]
use crate::checker::mig::m2e::r19k3_defs::NodeAccessExtR19k3;
#[allow(unused_imports)]
use crate::checker::string_mapping_checker::StringMappingKind;
#[allow(unused_imports)]
use r24k10_defs::{intrinsic_type_kinds, IntrinsicTypeKind};
use std::sync::Arc;

impl Checker {
    pub fn get_suggested_import_source(
        &self,
        module_reference: &str,
        ts_extension: &str,
        mode: ResolutionMode,
    ) -> String { ::tsox_core::fntrace::enter("get_suggested_import_source"); 
        let import_source_without_extension = remove_extension(module_reference, ts_extension);
        if self.module_kind.is_non_node_esm() || mode == ResolutionMode::ESNext {
            let prefer_ts = is_declaration_file_name(module_reference)
                && self.compiler_options.get_allow_importing_ts_extensions();
            let ext = match ts_extension {
                ".mts" | ".d.mts" => if prefer_ts { ".mts" } else { ".mjs" },
                ".cts" | ".d.cts" => if prefer_ts { ".cts" } else { ".cjs" },
                _ => if prefer_ts { ".ts" } else { ".js" },
            };
            return format!("{import_source_without_extension}{ext}");
        }
        import_source_without_extension
    }

    pub fn get_this_argument_of_call(&self, node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_this_argument_of_call"); 
        if node.kind == SyntaxKind::BinaryExpression {
            return Some(node.as_binary_expression().right.clone());
        }
        let expression: Arc<Node> = match node.kind {
            SyntaxKind::CallExpression => node.expression()?.clone(),
            SyntaxKind::TaggedTemplateExpression => node.as_tagged_template_expression().tag.clone(),
            SyntaxKind::Decorator if !self.legacy_decorators => node.expression()?.clone(),
            _ => return None,
        };
        let callee = skip_outer_expressions(&expression, OuterExpressionKinds::ALL);
        if is_access_expression(&callee) {
            return callee.expression().cloned();
        }
        None
    }

    pub fn get_this_type(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_this_type"); 
        let container = get_this_container(node, false, false);
        let parent = container.parent_rc();
        if parent.kind == SyntaxKind::ClassDeclaration
            || parent.kind == SyntaxKind::ClassExpression
            || parent.kind == SyntaxKind::InterfaceDeclaration
        {
            let body_descendant = is_constructor_declaration(&container)
                && container
                    .as_constructor_declaration()
                    .body
                    .as_ref()
                    .map(|b| is_node_descendant_of(node, b))
                    .unwrap_or(false);
            if !is_static(&container) || body_descendant {
                let Some(symbol) = self.get_symbol_of_declaration(&parent) else {
                    return self.error_type();
                };
                let declared = self.get_declared_type_of_class_or_interface(&symbol);
                if let TypeData::Interface(d) = &declared.data {
                    if let Some(this_type) = &d.this_type {
                        return this_type.clone();
                    }
                }
                return self.error_type();
            }
        }
        self.error_message(
            node,A_THIS_TYPE_IS_AVAILABLE_ONLY_IN_A_NON_STATIC_MEMBER_OF_A_CLASS_OR_INTERFACE,
            &[],
        );
        self.error_type()
    }

    pub fn get_this_type_argument(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_this_type_argument"); 
        if t.object_flags.intersects(ObjectFlags::Reference) {
            let target = t.target()?;
            if let Some(global_this) = self.get_or_init_global_this_type() {
                if target.id == global_this.id {
                    return Some(self.get_type_arguments(t)[0].clone());
                }
            }
        }
        None
    }

    pub fn get_this_type_from_contextual_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_this_type_from_contextual_type"); 
        r25k6_defs::map_type_with_checker(self, t, &mut |c, t| {
            if t.flags.intersects(TypeFlags::Intersection) {
                for ct in t.types()? {
                    if let Some(type_arg) = c.get_this_type_argument(ct) {
                        return Some(type_arg);
                    }
                }
                None
            } else {
                c.get_this_type_argument(t)
            }
        })
        .unwrap_or_else(|| t.clone())
    }

    pub fn get_this_type_of_object_literal_from_contextual_type(
        &mut self,
        containing_literal: &Arc<Node>,
        contextual_type: Option<&Arc<Type>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_this_type_of_object_literal_from_contextual_type"); 
        let mut literal = Arc::clone(containing_literal);
        let mut t = contextual_type.cloned();
        while let Some(current) = t {
            let this_type = self.get_this_type_from_contextual_type(&current);
            if this_type.id != current.id {
                return Some(this_type);
            }
            if literal.parent_rc().kind != SyntaxKind::PropertyAssignment {
                break;
            }
            literal = literal.parent_rc().parent_rc();
            let next = self.get_apparent_type_of_contextual_type(&literal, ContextFlags::None);
            t = next;
        }
        None
    }

    pub fn get_tuple_base_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_tuple_base_type"); 
        let (type_parameters, element_infos, readonly) = match &t.data {
            TypeData::Tuple(d) => {
                let iface = &d.interface_data;
                let type_parameters: Vec<Arc<Type>> = if iface.outer_type_parameter_count != 0 {
                    iface.all_type_parameters[..iface.outer_type_parameter_count].to_vec()
                } else {
                    iface.all_type_parameters.clone()
                };
                (type_parameters, d.element_infos.clone(), d.readonly)
            }
            _ => return t.clone(),
        };
        let mut element_types: Vec<Arc<Type>> = Vec::with_capacity(type_parameters.len());
        for (i, tp) in type_parameters.iter().enumerate() {
            if element_infos
                .get(i)
                .map(|e| e.flags.intersects(ElementFlags::Variadic))
                .unwrap_or(false)
            {
                let number_type = self.number_type();
                element_types.push(self.get_indexed_access_type(tp, &number_type));
            } else {
                element_types.push(tp.clone());
            }
        }
        let union = self.get_union_type(element_types);
        self.create_array_type_ex(union, readonly)
    }

    pub fn get_tuple_element_type_out_of_start_count(
        &mut self,
        t: &Arc<Type>,
        index: Number,
        undefined_like_type: Option<&Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_tuple_element_type_out_of_start_count"); 
        r25k6_defs::map_type_with_checker(self, t, &mut |c, t| {
            let rest_type = c.get_rest_type_of_tuple_type(t);
            if let Some(undefined_like_type) = undefined_like_type {
                let total = get_total_fixed_element_count(t);
                if index >= Number::from(total as f64) {
                    return Some(c.get_union_type(vec![
                        rest_type.clone(),
                        undefined_like_type.clone(),
                    ]));
                }
            }
            Some(rest_type)
        })
        .unwrap_or_else(|| Arc::clone(t))
    }

    pub fn get_tuple_target_type(
        &mut self,
        element_infos: &[TupleElementInfo],
        readonly: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_tuple_target_type"); 
        if element_infos.len() == 1 && element_infos[0].flags.intersects(ElementFlags::Rest) {
            if readonly {
                return self.global_readonly_array_type();
            }
            return self.global_array_type();
        }
        let key = get_tuple_key(element_infos, readonly);
        if let Some(t) = self.tuple_types.get(&key) {
            return t.clone();
        }
        let t = self.create_tuple_target_type(element_infos, readonly);
        self.tuple_types.insert(key, t.clone());
        t
    }

    pub fn get_type_alias_instantiation(
        &mut self,
        symbol: &Arc<Symbol>,
        type_arguments: &[Arc<Type>],
        alias: Option<&Arc<TypeAlias>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_alias_instantiation"); 
        let t = self.get_declared_type_of_symbol(symbol);
        if t.id == self.intrinsic_marker_type().id {
            if let Some(type_kind) = intrinsic_type_kinds(&symbol.name) {
                if type_arguments.len() == 1 {
                    if type_kind == IntrinsicTypeKind::NoInfer {
                        return self.get_no_infer_type(&type_arguments[0]);
                    }
                    let string_kind = match type_kind {
                        IntrinsicTypeKind::Uppercase => StringMappingKind::Uppercase,
                        IntrinsicTypeKind::Lowercase => StringMappingKind::Lowercase,
                        IntrinsicTypeKind::Capitalize => StringMappingKind::Capitalize,
                        _ => StringMappingKind::Uncapitalize,
                    };
                    return self.get_string_mapping_type(
                        string_kind,
                        Some(symbol.clone()),
                        &type_arguments[0],
                    );
                }
            }
        }
        let type_parameters = self
            .type_alias_links
            .get(symbol)
            .map(|l| l.type_parameters.clone())
            .unwrap_or_default();
        let key = get_type_alias_instantiation_key(type_arguments, alias);
        if let Some(instantiation) = r25k6_defs::alias_instantiations_get(symbol, &key) {
            return instantiation;
        }
        let min_count = self.get_min_type_argument_count(&type_parameters);
        let in_js_file = symbol
            .value_declaration
            .as_ref()
            .map(|d| is_in_js_file(d))
            .unwrap_or(false);
        let filled =
            self.fill_missing_type_arguments(type_arguments, &type_parameters, min_count, in_js_file);
        let mapper = Arc::new(new_type_mapper(type_parameters.clone(), filled));
        let instantiation =
            self.instantiate_type_with_alias(&t, Some(&mapper), alias.map(|a| a.as_ref()));
        r25k6_defs::alias_instantiations_insert(symbol, key, instantiation.clone());
        instantiation
    }

    pub fn get_type_argument_arity_error(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        type_arguments: &[Arc<Node>],
        head_message: Option<&'static Message>,
    ) -> Diagnostic { ::tsox_core::fntrace::enter("get_type_argument_arity_error"); 
        let arg_count = type_arguments.len();
        let source_file = self.get_source_file_of_node(node).expect("source file of node");
        let type_argument_list = type_argument_list(node).expect("type argument list");
        let loc = TextRange::new(
            skip_trivia(&source_file.text, type_argument_list.loc.pos()),
            type_argument_list.loc.end(),
        );
        let diagnostic;
        if signatures.len() == 1 {
            let sig = &signatures[0];
            let min_count = self.get_min_type_argument_count(&sig.type_parameters);
            let max_count = sig.type_parameters.len();
            let expected = if min_count < max_count {
                format!("{min_count}-{max_count}")
            } else {
                format!("{min_count}")
            };
            diagnostic = new_diagnostic(
                &source_file,
                loc,
                EXPECTED_0_TYPE_ARGUMENTS_BUT_GOT_1,
                &[expected, arg_count.to_string()],
            );
        } else {
            let mut below_arg_count = i64::MIN;
            let mut above_arg_count = i64::MAX;
            for sig in signatures {
                let min_count = self.get_min_type_argument_count(&sig.type_parameters);
                let max_count = sig.type_parameters.len();
                if min_count > arg_count {
                    above_arg_count = above_arg_count.min(min_count as i64);
                } else if max_count < arg_count {
                    below_arg_count = below_arg_count.max(max_count as i64);
                }
            }
            if below_arg_count != i64::MIN && above_arg_count != i64::MAX {
                diagnostic = new_diagnostic(
                    &source_file,
                    loc,
                    NO_OVERLOAD_EXPECTS_0_TYPE_ARGUMENTS_BUT_OVERLOADS_DO_EXIST_THAT_EXPECT_EITHER_1_OR_2_TYPE_ARGUMENTS,
                    &[
                        arg_count.to_string(),
                        below_arg_count.to_string(),
                        above_arg_count.to_string(),
                    ],
                );
            } else {
                let shown = if below_arg_count == i64::MIN {
                    above_arg_count
                } else {
                    below_arg_count
                };
                diagnostic = new_diagnostic(
                    &source_file,
                    loc,
                    EXPECTED_0_TYPE_ARGUMENTS_BUT_GOT_1,
                    &[shown.to_string(), arg_count.to_string()],
                );
            }
        }
        match head_message {
            Some(head_message) => new_diagnostic_chain(Some(&diagnostic), *head_message, vec![]),
            None => diagnostic,
        }
    }

    pub fn get_symbol_of_node(&mut self, node: &Node) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_of_node"); 
        get_symbol_of_node_impl(self, node)
    }
}

pub(crate) fn get_relation_key(
    source: &Arc<Type>,
    target: &Arc<Type>,
    intersection_state: IntersectionState,
    is_identity: bool,
    ignore_constraints: bool,
) -> (CacheHashKey, bool) { ::tsox_core::fntrace::enter("get_relation_key"); 
    let (source, target) = if is_identity && source.id > target.id {
        (target, source)
    } else {
        (source, target)
    };
    let mut b = KeyBuilder::new();
    let constrained;
    if is_type_reference_with_generic_arguments(source) && is_type_reference_with_generic_arguments(target)
    {
        b.write_byte(b'g');
        constrained = b.write_generic_type_references(source, target, ignore_constraints);
    } else {
        b.write_byte(b's');
        b.write_type(source);
        b.write_type(target);
        constrained = false;
    }
    b.write_uint32(intersection_state.bits());
    (b.hash(), constrained)
}

pub(crate) fn get_type_alias_instantiation_key(
    type_arguments: &[Arc<Type>],
    alias: Option<&Arc<TypeAlias>>,
) -> CacheHashKey { ::tsox_core::fntrace::enter("get_type_alias_instantiation_key"); 
    get_type_instantiation_key(
        type_arguments,
        alias.map(|a| a.as_ref()),
        false,
    )
}
