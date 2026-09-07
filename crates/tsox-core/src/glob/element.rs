#[derive(Clone, Debug)]
pub(crate) enum Element {
    Slash,

    Literal(String),

    Star,

    AnyChar,

    StarStar,

    Group(Vec<super::Glob>),

    CharRange { negate: bool, low: char, high: char },
}
