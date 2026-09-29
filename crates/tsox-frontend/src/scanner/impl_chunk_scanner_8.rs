//! Go scanner.go 的 ReScan 系补件（格式化引擎依赖）：
//! ReScanTemplateToken / ReScanJsxToken / ReScanJsxAttributeValue

use crate::ast::SyntaxKind;
use crate::scanner::impl_chunk::*;

use super::impl_chunk::Scanner;

impl Scanner {
    /// Go ReScanTemplateToken：回退到当前 token 起点重扫模板段
    /// （`}` → TemplateMiddle/TemplateTail）
    pub fn re_scan_template_token(&mut self) -> SyntaxKind {
        self.pos = self.token_pos;
        self.token_pos = self.pos;
        self.full_start_pos = self.pos;
        self.preceding_line_break = false;
        self.token_flags = TOKEN_FLAGS_NONE;
        // 当前字符是 `}`：跳过后按模板段扫描
        if self.pos < self.end && self.text.as_bytes()[self.pos] == b'}' {
            self.pos += 1;
        }
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == '`' {
                self.pos += 1;
                self.token = SyntaxKind::TemplateTail;
                break;
            }
            if c == '$' && self.pos + 1 < self.end && self.text.as_bytes()[self.pos + 1] == b'{' {
                self.pos += 2;
                self.token = SyntaxKind::TemplateMiddle;
                break;
            }
            if c == '\\' {
                self.pos = (self.pos + 2).min(self.end);
                continue;
            }
            if c == '\n' || c == '\r' {
                self.preceding_line_break = true;
            }
            self.pos += 1;
        }
        if self.pos >= self.end {
            self.token = SyntaxKind::TemplateTail;
        }
        self.token_end = self.pos;
        self.has_preceding_line_break = self.preceding_line_break;
        self.token
    }

    /// Go ReScanJsxToken(allowMultilineJsxText=false)：格式化用，JSX 文本
    /// 逐行切分以便逐行缩进
    pub fn re_scan_jsx_token(&mut self, allow_multiline_jsx_text: bool) -> SyntaxKind {
        self.pos = self.full_start_pos;
        self.token_pos = self.full_start_pos;
        self.token = self.scan_jsx_token_ex(allow_multiline_jsx_text);
        self.token
    }

    /// Go ReScanJsxAttributeValue：回退重扫属性值（= 后的引号串）
    pub fn re_scan_jsx_attribute_value(&mut self) -> SyntaxKind {
        self.pos = self.full_start_pos;
        self.token_pos = self.full_start_pos;
        self.scan_jsx_attribute_value()
    }
}
