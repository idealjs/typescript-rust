#![allow(unused_imports)]

use super::*;

impl NameResolver {
    pub(crate) fn resolve_export_specifier_case(
        &self,
        _location: &Arc<Node>,
        _last_location: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> {
        None
    }
}
