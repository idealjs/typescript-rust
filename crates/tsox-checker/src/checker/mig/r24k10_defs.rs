#![allow(unused_imports)]
use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntrinsicTypeKind {
    Uppercase,
    Lowercase,
    Capitalize,
    Uncapitalize,
    NoInfer,
}

pub(crate) fn intrinsic_type_kinds(name: &str) -> Option<IntrinsicTypeKind> {
    match name {
        "Uppercase" => Some(IntrinsicTypeKind::Uppercase),
        "Lowercase" => Some(IntrinsicTypeKind::Lowercase),
        "Capitalize" => Some(IntrinsicTypeKind::Capitalize),
        "Uncapitalize" => Some(IntrinsicTypeKind::Uncapitalize),
        "NoInfer" => Some(IntrinsicTypeKind::NoInfer),
        _ => None,
    }
}

impl Checker {
    pub(crate) fn get_or_init_global_this_type(&mut self) -> Option<Arc<Type>> {
        if let Some(t) = self.global_this_type.get() {
            return Some(Arc::clone(t));
        }
        // Go globalThisType 字段语义：lib ThisType<T> 接口声明型（wc3
        // initialize_checker 常规已 set；此兜底仅覆盖初始化前窗口），getThisTypeArgument
        // 以 Reference target 身份比对它识别 contextual this 标记
        let t = self.get_global_type("ThisType", 1, false);
        let _ = self.global_this_type.set(Arc::clone(&t));
        Some(t)
    }
}
