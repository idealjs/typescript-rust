#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub(crate) fn is_object_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        source_is_primitive: bool,
    ) -> bool {
        // 未解析的接口壳（自引用重建实例，members 空）：先解析成完整实例再比较
        // （tsc type reference 的成员延迟解析语义）
        if let Some(sym) = target.symbol.as_ref()
            && sym
                .declarations
                .iter()
                .any(|d| matches!(d.data, tsox_frontend::ast::NodeData::InterfaceDeclaration(_)))
            && target.as_structured().is_some_and(|s| s.members.entries.is_empty())
            && !self
                .pending_interface_shells
                .contains_key(&(Arc::as_ptr(sym) as *const tsox_frontend::ast::Symbol as usize))
        {
            let args = target.as_object().map(|o| o.type_arguments.clone());
            let resolved = self.resolve_interface_type_ex(sym, args);
            if !Arc::ptr_eq(&resolved, target)
                && resolved
                    .as_structured()
                    .is_some_and(|s| !s.members.entries.is_empty())
            {
                return self.is_object_type_related_to(source, &resolved, relation, source_is_primitive);
            }
        }
        // 源侧接口实例同样可能带退化构建窗口的残缺成员表（部分基类尚为壳时
        // 合并的实例被调用方缓存）：重新解析取完整实例再比较（与上方 target
        // 侧对称，Go 成员解析同步幂等无此问题）
        if let Some(sym) = source.symbol.as_ref()
            && sym
                .declarations
                .iter()
                .any(|d| matches!(d.data, tsox_frontend::ast::NodeData::InterfaceDeclaration(_)))
            && !self
                .pending_interface_shells
                .contains_key(&(Arc::as_ptr(sym) as *const tsox_frontend::ast::Symbol as usize))
        {
            let args = source.as_object().map(|o| o.type_arguments.clone());
            let resolved = self.resolve_interface_type_ex(sym, args);
            let src_members = source.as_structured().map(|s| s.members.entries.len()).unwrap_or(0);
            if !Arc::ptr_eq(&resolved, source)
                && resolved
                    .as_structured()
                    .is_some_and(|s| s.members.entries.len() > src_members)
            {
                return self.is_object_type_related_to(&resolved, target, relation, source_is_primitive);
            }
        }
        let source_struct = match source.as_structured() {
            Some(s) => s,
            None => return false,
        };
        let target_struct = match target.as_structured() {
            Some(t) => t,
            None => return false,
        };

        // Go propertiesRelatedTo/signaturesRelatedTo/indexSignaturesRelatedTo
        // 的 identity 分派：属性数一致 + 成分直比，签名/索引签名走各自
        // identical 变体，跳过可赋值导向的弱类型/缺属性启发
        if relation == RelationKind::Identity {
            return self.properties_identical_to(source, target)
                && self
                    .signatures_related_to(source, target, SignatureKind::Call, relation)
                    .is_true()
                && self
                    .signatures_related_to(source, target, SignatureKind::Construct, relation)
                    .is_true()
                && self.index_signatures_identical_to(source, target).is_true();
        }

        if relation != RelationKind::Comparable
            && self.relater_intersection_target_depth == 0
            && !source_struct.properties.is_empty()
            && self.is_weak_type(target)
            && !self.has_common_properties(source, target, false)
        {
            let has_calls = !source_struct.call_signatures().is_empty();
            let has_constructs = !source_struct.construct_signatures().is_empty();
            if self.relater_chain_active {
                let source_str = self.type_to_string(source);
                let target_str = self.type_to_string(target);
                if has_calls || has_constructs {
                    self.relater_report_error(
                        tsox_core::diagnostics::messages_generated::
                            VALUE_OF_TYPE_0_HAS_NO_PROPERTIES_IN_COMMON_WITH_TYPE_1_DID_YOU_MEAN_TO_CALL_IT,
                        vec![source_str, target_str],
                    );
                } else {
                    self.relater_report_error(
                        tsox_core::diagnostics::messages_generated::
                            TYPE_0_HAS_NO_PROPERTIES_IN_COMMON_WITH_TYPE_1,
                        vec![source_str, target_str],
                    );
                }
            }
            return false;
        }

        if self.is_array_type(target)
            && target_struct.properties.is_empty()
            && !self.is_array_type(source)
            && !self.is_tuple_type(source)
            && !source.object_flags.contains(ObjectFlags::EvolvingArray)
        {
            let mut missing: Vec<String> = Vec::new();
            for prop in self.declared_array_member_symbols() {
                if prop.flags.contains(SymbolFlags::Optional) {
                    continue;
                }
                let found = source_struct.members.get(&prop.name).is_some()
                    || (!source_struct.call_signatures().is_empty()
                        && self
                            .global_interface_member_symbol("Function", &prop.name)
                            .is_some())
                    || self
                        .global_interface_member_symbol("Object", &prop.name)
                        .is_some();
                if !found {
                    missing.push(prop.name.clone());
                }
            }
            if !missing.is_empty() {
                if self.should_report_unmatched_property_error(source, target) {
                    let source_str = self.type_to_string(source);
                    let target_str = self.type_to_string(target);
                    if missing.len() == 1 {
                        self.relater_report_error(
                            tsox_core::diagnostics::messages_generated::
                                PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                            vec![missing[0].clone(), source_str, target_str],
                        );
                    } else if missing.len() <= 5 {
                        self.relater_report_error(
                            tsox_core::diagnostics::messages_generated::
                                TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                            vec![source_str, target_str, missing.join(", ")],
                        );
                    } else {
                        self.relater_report_error(
                            tsox_core::diagnostics::messages_generated::
                                TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE,
                            vec![
                                source_str,
                                target_str,
                                missing[..4].join(", "),
                                (missing.len() - 4).to_string(),
                            ],
                        );
                    }
                }
                return false;
            }
        }

        if matches!(relation, RelationKind::Subtype | RelationKind::StrictSubtype)
            && self.is_empty_object_type(target)
            && target.object_flags.contains(ObjectFlags::FreshLiteral)
            && !self.is_empty_object_type(source)
        {
            return false;
        }

        let mut missing_props: Vec<String> = Vec::new();
        let mut missing_prop_syms: Vec<Option<Arc<tsox_frontend::ast::Symbol>>> = Vec::new();

        // Go 元组的表面成员含 Array<any> 的 length/push 等（getPropertiesOfType
        // 对元组并入数组接口成员）；裸元组（members 空）与裸数组同样回退
        let source_is_bare_array = (self.is_array_type(source)
            || self.is_tuple_type(source)
            || source.object_flags.contains(ObjectFlags::EvolvingArray))
            && source_struct.members.is_empty();
        // Go requireOptionalProperties：Subtype/StrictSubtype 下源须持有目标
        // 全部属性（含可选/Partial 映射属性），对象字面量/元组/空数组字面量除外
        let require_optional_properties = matches!(
            relation,
            RelationKind::Subtype | RelationKind::StrictSubtype
        ) && !crate::checker::utilities_token_is_identifier_or_keyword::is_object_literal_type(
            source,
        ) && !self.is_empty_array_literal_type(source)
            && !self.is_tuple_type(source);
        for target_prop in &target_struct.properties {
            let source_declares_locally = source_struct.members.get(&target_prop.name).is_some();
            let mut source_prop = source_struct.members.get(&target_prop.name).cloned();
            if source_prop.is_none() {
                source_prop = self.get_property_of_type(source, &target_prop.name);
            }
            let source_prop = match source_prop {
                Some(p) => p,
                None => {
                    if source_is_bare_array
                        && let Some(p) = self.declared_array_member_symbol(&target_prop.name)
                    {
                        p
                    } else {
                        if target_prop.flags.contains(SymbolFlags::Optional)
                            && !require_optional_properties
                        {
                            continue;
                        }
                        missing_props.push(target_prop.name.clone());
                        missing_prop_syms.push(Some(Arc::clone(target_prop)));
                        continue;
                    }
                }
            };

            if target_prop.name.starts_with('[') {
                continue;
            }
            // 源未本地声明且目标是 Object 原型成员名时，按 Go getPropertyOfType
            // 的解析结果（全局 Object 接口成员）参与真实类型比较而非跳过
            //（{} → Boolean 的 valueOf: boolean 冲突由此报出）
            let mut source_prop = source_prop;
            if !source_declares_locally
                && let Some(obj_member) = self
                    .global_interface_member_symbol("Object", &target_prop.name)
            {
                source_prop = obj_member;
            }

            {
                let src_mod = crate::checker::exports::get_declaration_modifier_flags_from_symbol(
                    &source_prop,
                );
                let tgt_mod = crate::checker::exports::get_declaration_modifier_flags_from_symbol(
                    target_prop,
                );
                if src_mod.intersects(ModifierFlags::Private)
                    || tgt_mod.intersects(ModifierFlags::Private)
                {
                    let decl_of = |s: &Arc<tsox_frontend::ast::Symbol>| {
                        s.value_declaration
                            .clone()
                            .or_else(|| s.declarations.first().cloned())
                    };
                    let same_decl = Arc::ptr_eq(&source_prop, target_prop)
                        || match (decl_of(&source_prop), decl_of(target_prop)) {
                            (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
                            _ => false,
                        };
                    if !same_decl {
                        if src_mod.intersects(ModifierFlags::Private)
                            && tgt_mod.intersects(ModifierFlags::Private)
                        {
                            self.relater_report_error(
                                tsox_core::diagnostics::messages_generated::
                                    TYPES_HAVE_SEPARATE_DECLARATIONS_OF_A_PRIVATE_PROPERTY_0,
                                vec![target_prop.name.clone()],
                            );
                        } else {
                            let private_side = if src_mod.intersects(ModifierFlags::Private) {
                                let norm = self.single_base_for_non_augmenting_subtype(source);
                                self.type_to_string(&norm)
                            } else {
                                let norm = self.single_base_for_non_augmenting_subtype(target);
                                self.type_to_string(&norm)
                            };
                            let public_side = if src_mod.intersects(ModifierFlags::Private) {
                                self.type_to_string(target)
                            } else {
                                self.type_to_string(source)
                            };
                            self.relater_report_error(
                                tsox_core::diagnostics::messages_generated::
                                    PROPERTY_0_IS_PRIVATE_IN_TYPE_1_BUT_NOT_IN_TYPE_2,
                                vec![target_prop.name.clone(), private_side, public_side],
                            );
                        }
                        return false;
                    }
                } else if src_mod.intersects(ModifierFlags::Protected)
                    && !tgt_mod.intersects(ModifierFlags::Protected)
                {
                    let src_str = self.type_to_string(source);
                    let tgt_str = self.type_to_string(target);
                    self.relater_report_error(
                        tsox_core::diagnostics::messages_generated::
                            PROPERTY_0_IS_PROTECTED_IN_TYPE_1_BUT_PUBLIC_IN_TYPE_2,
                        vec![target_prop.name.clone(), src_str, tgt_str],
                    );
                    return false;
                }
            }

            // Go isPropertyRelatedTo：strictSubtype 下 readonly 源对 mutable
            // 目标不成立（mutable→readonly 成立），关系有序化使 union 子型
            // 归约与声明顺序无关
            if relation == RelationKind::StrictSubtype
                && self.symbol_is_readonly(&source_prop)
                && !self.symbol_is_readonly(target_prop)
            {
                return false;
            }

            let source_type = if source_is_bare_array {
                self.instantiate_array_member_type(source, &source_prop)
                    .unwrap_or_else(|| self.get_type_of_symbol(&source_prop))
            } else {
                self.substituted_member_type_of(source, &source_prop)
            };

            let source_type = self.erase_bare_generic_params(source, &source_type);
            let target_type = self.substituted_member_type_of(target, target_prop);
            let target_type = self.erase_bare_generic_params(target, &target_type);
            if !self.is_type_related_to(&source_type, &target_type, relation) {
                let prop_source_str = self.type_to_string(&source_type);
                let prop_target_str = self.type_to_string(&target_type);
                self.relater_report_error(
                    tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                    vec![prop_source_str, prop_target_str],
                );
                self.relater_report_error(
                    tsox_core::diagnostics::messages_generated::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                    vec![self.chain_property_arg_name(target_prop)],
                );
                return false;
            }
        }

        if !missing_props.is_empty() {
            if !self.should_report_unmatched_property_error(source, target) {
                return false;
            }
            let (source_str, target_str) = self.get_type_names_for_error_display(source, target);
            if missing_props.len() == 1 {
                let display =
                    crate::checker::property_name_for_display(&missing_props[0]);
                self.relater_report_error_with_related(
                    tsox_core::diagnostics::messages_generated::
                        PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![display.clone(), source_str, target_str],
                    missing_prop_syms[0].as_ref().and_then(|sym| {
                        sym.declarations.first().map(|d| {
                            crate::checker::relater_relation::ChainRelated {
                                file: self.get_source_file_of_node(d),
                                loc: d.loc,
                                message: tsox_core::diagnostics::messages_generated::
                                    X_0_IS_DECLARED_HERE,
                                args: vec![display],
                            }
                        })
                    }),
                );
            } else if missing_props.len() <= 5 {
                self.relater_report_error(
                    tsox_core::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![
                        source_str,
                        target_str,
                        crate::checker::property_names_for_display(&missing_props),
                    ],
                );
            } else {
                self.relater_report_error(
                    tsox_core::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE,
                    vec![
                        source_str,
                        target_str,
                        crate::checker::property_names_for_display(&missing_props[..4]),
                        (missing_props.len() - 4).to_string(),
                    ],
                );
            }
            return false;
        }

        if self.is_tuple_type(target)
            && !self.is_array_type(source)
            && source.object_flags.contains(ObjectFlags::EvolvingArray) == false
            && let TypeData::Tuple(tup) = &target.data
        {
            for (i, ei) in tup.element_infos.iter().enumerate() {
                let Some(elem_type) = &ei.type_ else { continue };
                let name = i.to_string();
                let Some(source_prop) = source_struct.members.get(&name) else {
                    let optional = ei.flags.contains(ElementFlags::Optional);
                    if optional {
                        continue;
                    }
                    return false;
                };
                let source_type = self.get_type_of_symbol(source_prop);
                if !self.is_type_related_to(&source_type, elem_type, relation) {
                    return false;
                }
            }
        }

        if !self.is_call_signatures_related_to(source, target, relation) {
            return false;
        }

        if !self.is_construct_signatures_related_to(source, target, relation) {
            return false;
        }

        if !self.is_index_signatures_related_to(source, target, relation, source_is_primitive) {
            return false;
        }

        true
    }
}
