use crate::diagnostics::Message;

impl Message {
    pub(crate) fn elided_in_compatibility_pyramid(&self) -> bool { crate::fntrace::enter("elided_in_compatibility_pyramid"); 
        self.elided_in_compatibility_pyramid
    }
}
