use crate::diagnostics::Message;

impl Message {
    pub(crate) fn elided_in_compatibility_pyramid(&self) -> bool {
        self.elided_in_compatibility_pyramid
    }
}
