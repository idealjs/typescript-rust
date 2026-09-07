//! fourslash 会话：测试内容解析结果 + 当前文件状态（编辑作用于内存文本）。
//! 框架操作以自由函数形式提供（见 api 模块），Session 只承载数据。

use crate::fourslash::parse::{Marker, RangeMarker, TestData, parse_test_data};

#[derive(Debug)]
pub struct Session {
    pub data: TestData,
    pub active_file: String,
    pub cursor: Option<usize>,
    /// 文件名 -> 当前内容（应用过 Insert 等编辑后）
    contents: std::collections::BTreeMap<String, String>,
    pub capabilities: Option<String>,
}

pub const DEFAULT_FILE_NAME: &str = "main.ts";

impl Session {
    pub fn new(content: &str) -> Session {
        Self::new_with_capabilities(content, None)
    }

    pub fn new_with_capabilities(content: &str, capabilities: Option<String>) -> Session {
        let data = parse_test_data(content, DEFAULT_FILE_NAME);
        let mut contents = std::collections::BTreeMap::new();
        let mut active = DEFAULT_FILE_NAME.to_string();
        for f in &data.files {
            contents.insert(f.file_name.clone(), f.content.clone());
            if f.file_options.get("emitthisfile").is_none() {
                // 首个文件为活动文件
            }
        }
        if let Some(first) = data.files.first() {
            active = first.file_name.clone();
        }
        Session {
            data,
            active_file: active,
            cursor: None,
            contents,
            capabilities,
        }
    }

    pub fn file_content(&self, name: &str) -> &str {
        self.contents
            .get(name)
            .unwrap_or_else(|| panic!("文件不存在: {name}"))
    }

    pub fn set_file_content(&mut self, name: &str, content: String) {
        self.contents.insert(name.to_string(), content);
    }

    pub fn marker(&self, name: &str) -> &Marker {
        let pos = self
            .data
            .marker_positions
            .get(name)
            .unwrap_or_else(|| panic!("标记不存在: {name}"));
        self.data
            .markers
            .iter()
            .find(|m| m.position == *pos && m.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("标记不存在: {name}"))
    }

    pub fn ranges_in(&self, file: &str) -> Vec<&RangeMarker> {
        self.data
            .ranges
            .iter()
            .filter(|r| r.file_name == file)
            .collect()
    }
}
