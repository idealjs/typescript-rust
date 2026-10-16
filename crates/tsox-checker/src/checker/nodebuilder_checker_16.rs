#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
use crate::checker::jsx_impl_chunk::{is_intrinsic_jsx_name, jsx_tag_name};
use std::sync::Arc;
use tsox_frontend::ast::SyntaxKind;

impl Checker {
    /// Go .types typeWriter 短路臂（tsc/internal/testutil/tsbaseline/
    /// type_symbol_baseline.go:394-404 writeTypeOrSymbol）：无错误基线且
    /// IsTypeAny(t) 且查询节点不在排除形态时直接打印 IntrinsicName。errorType
    /// 是内在名 "error" 的 Any 型（tsc/internal/checker/checker.go:990
    /// newIntrinsicType(TypeFlagsAny, "error")），同一型走 nodebuilder 臂命中
    /// Any 旗标打 any（nodebuilderimpl.go:3432-3440）——两臂分叉即
    /// `><a /> : error`（Go）与 `any`（本侧）的 .types 首分歧源
    pub(crate) fn type_writer_intrinsic_name(&self, t: &Type) -> Option<String> {
        ::tsox_core::fntrace::enter("type_writer_intrinsic_name");
        if self.had_error_baseline() {
            return None;
        }
        if !t.flags.intersects(crate::checker::types::TypeFlags::Any) {
            return None;
        }
        let node = self.access_location.as_ref()?;
        let parent = node.parent()?;
        if tsox_frontend::ast::is_binding_element(&parent)
            || tsox_frontend::ast::is_property_access_or_qualified_name(&parent)
            || tsox_frontend::ast::mig::m3g::is_label_name(node)
            || tsox_frontend::ast::is_global_scope_augmentation(&parent)
            || tsox_frontend::ast::is_meta_property(&parent)
            || Self::is_import_statement_name(node)
            || Self::is_export_statement_name(node)
            || Self::is_intrinsic_jsx_tag(node)
        {
            return None;
        }
        t.intrinsic_name().map(str::to_string)
    }

    /// Go compiler_runner.go:549 `len(c.result.Diagnostics) > 0`，经
    /// DoTypeAndSymbolBaseline 第 9 参成 typeWriterWalker.hadErrorBaseline：
    /// 任一诊断非空即整臂跳过（有错时 Go 全走 nodebuilder，errorType 打 any）。
    /// Go 门在发射前由整例结果一次性算定；本侧语义诊断入账于
    /// build_checker_internal 的一次性 checker（program_3.rs
    /// get_semantic_diagnostics），发射 walk 所在 pool checker 的查询对未解析
    /// 名静默落 errorType（get_type_of_node 表达式臂尾 error_type，不重报），
    /// 故语义分量取 get_semantic_diagnostics 在发射前缓存于 program 的
    /// 存在性（发射通道 build_and_check 先取诊断后 generate，序恒成立）
    fn had_error_baseline(&self) -> bool {
        ::tsox_core::fntrace::enter("had_error_baseline");
        !self.diagnostics.is_empty()
            || self.program.has_semantic_diagnostics()
            || !self.program.diagnostics().is_empty()
            || self
                .program
                .source_files()
                .iter()
                .any(|f| f.has_parse_diagnostics)
    }

    /// Go isIntrinsicJsxTag（type_symbol_baseline.go:497-507）：宿主为 JSX
    /// opening/closing/self-closing 元素、节点即其 tag 名、tag 文本为
    /// intrinsic jsx 名。intrinsic tag 名的型（如未解析 tag 的 errorType）
    /// 不走短路臂，保持 nodebuilder 的 any 呈现（`>a : any`）
    fn is_intrinsic_jsx_tag(node: &Arc<Node>) -> bool {
        ::tsox_core::fntrace::enter("is_intrinsic_jsx_tag");
        let Some(parent) = node.parent() else {
            return false;
        };
        if !matches!(
            parent.kind,
            SyntaxKind::JsxOpeningElement
                | SyntaxKind::JsxClosingElement
                | SyntaxKind::JsxSelfClosingElement
        ) {
            return false;
        }
        let Some(tag) = jsx_tag_name(&parent) else {
            return false;
        };
        Arc::ptr_eq(&tag, node) && is_intrinsic_jsx_name(node.text())
    }

    /// Go isImportStatementName（type_symbol_baseline.go:472-485）
    fn is_import_statement_name(node: &Arc<Node>) -> bool {
        ::tsox_core::fntrace::enter("is_import_statement_name");
        let Some(parent) = node.parent() else {
            return false;
        };
        match &parent.data {
            tsox_frontend::ast::NodeData::ImportSpecifier(d) => {
                Arc::ptr_eq(&d.name, node)
                    || d.property_name
                        .as_ref()
                        .is_some_and(|n| Arc::ptr_eq(n, node))
            }
            tsox_frontend::ast::NodeData::ImportClause(d) => {
                d.name.as_ref().is_some_and(|n| Arc::ptr_eq(n, node))
            }
            tsox_frontend::ast::NodeData::ImportEqualsDeclaration(d) => Arc::ptr_eq(&d.name, node),
            _ => false,
        }
    }

    /// Go isExportStatementName（type_symbol_baseline.go:486-495）
    fn is_export_statement_name(node: &Arc<Node>) -> bool {
        ::tsox_core::fntrace::enter("is_export_statement_name");
        let Some(parent) = node.parent() else {
            return false;
        };
        match &parent.data {
            tsox_frontend::ast::NodeData::ExportAssignment(d) => Arc::ptr_eq(&d.expression, node),
            tsox_frontend::ast::NodeData::ExportSpecifier(d) => {
                Arc::ptr_eq(&d.name, node)
                    || d.property_name
                        .as_ref()
                        .is_some_and(|n| Arc::ptr_eq(n, node))
            }
            _ => false,
        }
    }
}
