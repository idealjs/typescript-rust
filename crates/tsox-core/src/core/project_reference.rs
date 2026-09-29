#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectReference {
    pub path: String,

    pub original_path: String,

    pub circular: bool,
}
