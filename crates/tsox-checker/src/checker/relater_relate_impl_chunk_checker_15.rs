#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub fn signature_to_string(&mut self, sig: &Arc<Signature>) -> String {
        let params: Vec<String> = sig.parameters.iter().map(|p| p.name.clone()).collect();
        let return_type = self.get_return_type_of_signature(sig);
        let return_str = match return_type {
            Some(t) => self.type_to_string(&t),
            None => "void".to_string(),
        };
        format!("({}) => {}", params.join(", "), return_str)
    }

    /// Go getExpandedParameters：末参是 rest 且其类型为元组时，按元组元素
    /// 展开为具名参数序列（标签取元素 label，回退 rest 符号名_i）；其余
    /// 形态返回 None
    pub(crate) fn tuple_expanded_params(
        &mut self,
        sig: &Signature,
    ) -> Option<Vec<(String, Arc<Type>, bool, bool)>> {
        if !sig.has_rest_parameter() || sig.parameters.is_empty() {
            return None;
        }
        let rest_idx = sig.parameters.len() - 1;
        let rest_sym = &sig.parameters[rest_idx];
        let rest_type = self
            .signature_instantiated_param_type(sig, rest_idx)
            .unwrap_or_else(|| self.get_type_of_symbol(rest_sym));
        let crate::checker::types::TypeData::Tuple(tup) = &rest_type.data else {
            return None;
        };
        let mut out: Vec<(String, Arc<Type>, bool, bool)> = sig.parameters[..rest_idx]
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    self.get_type_of_symbol(p),
                    p.flags.contains(SymbolFlags::Optional),
                    false,
                )
            })
            .collect();
        for (i, info) in tup.element_infos.iter().enumerate() {
            let ty = info.type_.clone().unwrap_or_else(|| self.any_type());
            let variadic = info
                .flags
                .contains(crate::checker::types::ElementFlags::Variadic);
            // Go expandSignatureParametersWithTupleMembers 仅对 Rest 元素再包
            // 数组；Variadic（...T）元素类型已是数组形态
            let ty = if info
                .flags
                .contains(crate::checker::types::ElementFlags::Rest)
            {
                self.create_array_type(ty)
            } else {
                ty
            };
            let label = info.label.clone().unwrap_or_else(|| {
                let n = rest_sym.name.trim_start_matches("...");
                let root = if n.is_empty() { "arg" } else { n };
                format!("{root}_{i}")
            });
            let optional = info
                .flags
                .contains(crate::checker::types::ElementFlags::Optional);
            out.push((label, ty, optional, variadic));
        }
        Some(out)
    }

    pub(crate) fn is_call_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_sigs = self.get_signatures_of_type(source, SignatureKind::Call);
        let target_sigs = self.get_signatures_of_type(target, SignatureKind::Call);

        if source_sigs.is_empty() && target_sigs.is_empty() {
            return true;
        }
        if target_sigs.is_empty() {
            return true;
        }
        if source_sigs.is_empty() {
            if self.relater_chain_active
                && let Some(t0) = target_sigs.first()
            {
                let source_str = self.type_to_string(source);
                let sig_str = self.signature_display_colon(t0, "");
                self.relater_report_error(
                    tsox_core::diagnostics::messages_generated::
                        TYPE_0_PROVIDES_NO_MATCH_FOR_THE_SIGNATURE_1,
                    vec![source_str, sig_str],
                );
            }
            return false;
        }
        self.signatures_related_to(source, target, SignatureKind::Call, relation)
            .is_true()
    }

    pub(crate) fn signature_display_colon(&mut self, sig: &Arc<Signature>, prefix: &str) -> String {
        self.signature_display_sep(sig, prefix, ": ")
    }

    pub(crate) fn signature_display_arrow(&mut self, sig: &Arc<Signature>, prefix: &str) -> String {
        self.signature_display_sep(sig, prefix, " => ")
    }

    pub(crate) fn signature_display_sep(
        &mut self,
        sig: &Arc<Signature>,
        prefix: &str,
        sep: &str,
    ) -> String {
        let params: Vec<String> = if let Some(expanded) = self.tuple_expanded_params(sig) {
            expanded
                .iter()
                .map(|(name, ty, optional, variadic)| {
                    let type_str = self.type_to_string(ty);
                    let prefix = if *variadic { "..." } else { "" };
                    let question = if *optional { "?" } else { "" };
                    format!("{prefix}{name}{question}: {type_str}")
                })
                .collect()
        } else {
            sig.parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    let param_type = self
                        .signature_instantiated_param_type(sig, i)
                        .unwrap_or_else(|| self.get_type_of_symbol(param));
                    let param_type = self.strip_param_optionality_undefined(param, &param_type);

                    let optional = param.flags.contains(SymbolFlags::Optional)
                        || param.declarations.iter().any(|d| {
                            matches!(
                                &d.data,
                                tsox_frontend::ast::NodeData::ParameterDeclaration(pd)
                                    if pd.question_token.is_some() || pd.initializer.is_some()
                            )
                        });
                    let is_rest = sig.has_rest_parameter() && i == sig.parameters.len() - 1;
                    let prefix = if is_rest { "..." } else { "" };
                    if optional {
                        format!(
                            "{prefix}{}?: {}",
                            param.name,
                            self.type_to_string(&param_type)
                        )
                    } else {
                        format!(
                            "{prefix}{}: {}",
                            param.name,
                            self.type_to_string(&param_type)
                        )
                    }
                })
                .collect()
        };
        let ret = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        let tp = if sig.type_parameters.is_empty() {
            String::new()
        } else {
            let names: Vec<String> = sig
                .type_parameters
                .iter()
                .filter_map(|tp| tp.symbol.as_ref().map(|s| s.name.clone()))
                .collect();
            if names.is_empty() {
                String::new()
            } else {
                format!("<{}>", names.join(", "))
            }
        };

        let prefix = if sig
            .flags
            .contains(crate::checker::types::SignatureFlags::Abstract)
            && prefix.starts_with("new")
        {
            format!("abstract {prefix}")
        } else {
            prefix.to_string()
        };
        format!(
            "{prefix}{tp}({}){sep}{}",
            params.join(", "),
            self.type_to_string(&ret)
        )
    }

    pub(crate) fn is_construct_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_sigs = self.get_signatures_of_type(source, SignatureKind::Construct);
        let target_sigs = self.get_signatures_of_type(target, SignatureKind::Construct);

        if source_sigs.is_empty() && target_sigs.is_empty() {
            return true;
        }
        if target_sigs.is_empty() {
            return true;
        }
        if source_sigs.is_empty() {
            if self.relater_chain_active
                && let Some(t0) = target_sigs.first()
            {
                let source_str = self.type_to_string(source);
                let sig_str = self.signature_display_colon(t0, "new ");
                self.relater_report_error(
                    tsox_core::diagnostics::messages_generated::
                        TYPE_0_PROVIDES_NO_MATCH_FOR_THE_SIGNATURE_1,
                    vec![source_str, sig_str],
                );
            }
            return false;
        }
        let related = self
            .signatures_related_to(source, target, SignatureKind::Construct, relation)
            .is_true();
        related
    }

    #[allow(dead_code)]
    pub(crate) fn is_function_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if !self.is_call_signatures_related_to(source, target, relation) {
            return false;
        }
        if !self.is_construct_signatures_related_to(source, target, relation) {
            return false;
        }
        true
    }
}
