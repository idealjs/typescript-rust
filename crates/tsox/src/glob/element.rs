#[derive(Clone, Debug)]
pub(super) enum Element {
    Slash,

    Literal(String),

    Star,

    AnyChar,

    StarStar,

    Group(Vec<super::Glob>),

    CharRange { negate: bool, low: char, high: char },
}
