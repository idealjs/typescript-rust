#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_external_emit_helpers(&mut self, location: &Arc<Node>, helpers: u32) {
        if !self.compiler_options.import_helpers.is_true() {
            return;
        }
        if self.ambient_context_depth > 0 {
            return;
        }
        let file_id = self.current_file_id as usize;
        let requested = self
            .requested_external_emit_helpers
            .get(&file_id)
            .copied()
            .unwrap_or(0);
        if requested & helpers == helpers {
            return;
        }
        let unchecked = helpers & !requested;
        let Some(helpers_module) = self.resolve_module_file_symbol("tslib") else {
            if self.ts2354_checked_files.insert(file_id) {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    location.loc,
                    crate::diagnostics::messages_generated::
                        THIS_SYNTAX_REQUIRES_AN_IMPORTED_HELPER_BUT_MODULE_0_CANNOT_BE_FOUND,
                    vec!["tslib".to_string()],
                ));
            }
            self.requested_external_emit_helpers
                .insert(file_id, requested | helpers);
            return;
        };
        for (bit, helper_name) in [
            (EXTERNAL_EMIT_HELPER_IMPORT_DEFAULT, "__importDefault"),
            (EXTERNAL_EMIT_HELPER_IMPORT_STAR, "__importStar"),
            (EXTERNAL_EMIT_HELPER_EXPORT_STAR, "__exportStar"),
        ] {
            if unchecked & bit == 0 {
                continue;
            }

            let found = helpers_module
                .exports
                .get(helper_name)
                .or_else(|| helpers_module.members.get(helper_name))
                .cloned()
                .or_else(|| self.ambient_namespace_local(&helpers_module, helper_name))
                .is_some_and(|s| s.flags.intersects(SymbolFlags::VALUE));
            if !found {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    location.loc,
                    crate::diagnostics::messages_generated::
                        THIS_SYNTAX_REQUIRES_AN_IMPORTED_HELPER_NAMED_1_WHICH_DOES_NOT_EXIST_IN_0_CONSIDER_UPGRADING_YOUR_VERSION_OF_0,
                    vec!["tslib".to_string(), helper_name.to_string()],
                ));
            }
        }
        self.requested_external_emit_helpers
            .insert(file_id, requested | helpers);
    }
}
