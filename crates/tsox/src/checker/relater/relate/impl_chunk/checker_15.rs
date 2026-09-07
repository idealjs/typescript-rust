#![allow(unused_imports)]

use super::*;

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
                    crate::diagnostics::messages_generated::
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
        let params: Vec<String> = sig
            .parameters
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let param_type = self
                    .signature_instantiated_param_type(sig, i)
                    .unwrap_or_else(|| self.get_type_of_symbol(param));

                let optional = param.flags.contains(SymbolFlags::Optional)
                    || param.declarations.iter().any(|d| {
                        matches!(
                            &d.data,
                            crate::ast::NodeData::ParameterDeclaration(pd)
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
            .collect();
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
                    crate::diagnostics::messages_generated::
                        TYPE_0_PROVIDES_NO_MATCH_FOR_THE_SIGNATURE_1,
                    vec![source_str, sig_str],
                );
            }
            return false;
        }
        let related = self
            .signatures_related_to(source, target, SignatureKind::Construct, relation)
            .is_true();
        if !related && self.relater_chain_active {
            let source_sigs = self.get_signatures_of_type(source, SignatureKind::Construct);
            let target_sigs = self.get_signatures_of_type(target, SignatureKind::Construct);
            if let (Some(ss), Some(ts)) = (source_sigs.first(), target_sigs.first())
                && ss.min_argument_count.max(0) as usize > ts.parameters.len()
            {
                let s_str = self.signature_display_arrow(ss, "new ");
                let t_str = self.signature_display_arrow(ts, "new ");
                self.relater_report_error(
                    crate::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                    vec![s_str, t_str],
                );
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TYPES_OF_CONSTRUCT_SIGNATURES_ARE_INCOMPATIBLE,
                    vec![],
                );
            }
        }
        related
    }

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
