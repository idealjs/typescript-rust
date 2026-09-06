#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectReference {

    pub path: String,

    pub original_path: String,

    pub circular: bool,
}
