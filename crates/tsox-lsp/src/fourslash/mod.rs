//! fourslash 测试框架：内容解析（parse，纯函数）、会话数据（session）、
//! 框架操作（api，自由函数 + 显式 Session 参数）。

pub mod api;
mod api_completions;
pub mod parse;
pub mod session;

pub use api::*;
pub use parse::{Marker, RangeMarker, TestData, TestFileInfo};
pub use session::Session;
