//! Module specifier generation (1:1 port of Go's `internal/ls/autoimport/specifiers.go`).

use crate::modulespecifiers;

use super::export::Export;
use super::view::View;

/// The result kind of module specifier generation.
///
/// Mirrors `modulespecifiers.ResultKind` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ResultKind {
    #[default]
    None,
    Ambient,
    Relative,
    NodeModules,
}

impl View {
    /// Gets the module specifier for an export.
    ///
    /// Mirrors `(v *View) GetModuleSpecifier` in Go.
    pub fn get_module_specifier(
        &self,
        _export: &Export,
        _user_preferences: &modulespecifiers::UserPreferences,
    ) -> (String, ResultKind) {
        // Requires full module-specifier infrastructure, entrypoint resolution,
        // and registry caches — stubbed until those are ported.
        // The Go logic:
        // 1. If ambient module (bare specifier), return module ID.
        // 2. If package has entrypoints, process entrypoint endings.
        // 3. Otherwise compute relative specifier via GetModuleSpecifiersForFileWithInfo.
        todo!(
            "get_module_specifier requires registry entrypoints and modulespecifiers infrastructure"
        )
    }
}
