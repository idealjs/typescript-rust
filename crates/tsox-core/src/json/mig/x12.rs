pub struct Options {
    pub indent: String,
}

pub fn jsontext_with_indent(indent: String) -> Options {
    Options { indent }
}

pub fn with_indent(indent: impl Into<String>) -> Options {
    jsontext_with_indent(indent.into())
}
