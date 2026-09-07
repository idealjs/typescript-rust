#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Parse,
    Program,
    Bind,
    Check,
    CheckTypes,
    Emit,
    Session,
}

impl Phase {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Phase::Parse => "parse",
            Phase::Program => "program",
            Phase::Bind => "bind",
            Phase::Check => "check",
            Phase::CheckTypes => "checkTypes",
            Phase::Emit => "emit",
            Phase::Session => "session",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceArg {
    Int(i64),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub tid: usize,

    pub ph: &'static str,
    pub cat: &'static str,
    pub name: String,
    pub args: Vec<(String, TraceArg)>,
}
