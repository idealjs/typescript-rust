pub struct Options {
    pub indent: String,
}

pub fn jsontext_with_indent(indent: String) -> Options { crate::fntrace::enter("jsontext_with_indent"); 
    Options { indent }
}

pub fn with_indent(indent: impl Into<String>) -> Options { crate::fntrace::enter("with_indent"); 
    jsontext_with_indent(indent.into())
}
