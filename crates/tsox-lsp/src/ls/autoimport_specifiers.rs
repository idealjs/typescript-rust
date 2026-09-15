
use crate::ls::autoimport_export::Export;
use crate::ls::autoimport_view::View;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ResultKind {
    #[default]
    None,
    Ambient,
    Relative,
    NodeModules,
}

impl View {
    pub fn get_module_specifier(
        &self,
        _export: &Export,
        _user_preferences: &tsox_tsoptions::modulespecifiers::UserPreferences,
    ) -> (String, ResultKind) {
        todo!(
            "get_module_specifier requires registry entrypoints and modulespecifiers infrastructure"
        )
    }
}
