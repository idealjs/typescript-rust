//! Go format/rule.go 的移植：规则结构与动作/旗标位掩码。
//! 规则表在运行时构建（rules.rs::get_all_rules），故使用拥有式数据。

use super::rule_context::FormattingContext;
use crate::ast::SyntaxKind;

pub(crate) type ContextPredicate = fn(&mut FormattingContext) -> bool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuleAction(pub(crate) u16);

impl RuleAction {
    pub(crate) const NONE: Self = Self(0);
    pub(crate) const STOP_PROCESSING_SPACE_ACTIONS: Self = Self(1 << 0);
    pub(crate) const STOP_PROCESSING_TOKEN_ACTIONS: Self = Self(1 << 1);
    pub(crate) const INSERT_SPACE: Self = Self(1 << 2);
    pub(crate) const INSERT_NEW_LINE: Self = Self(1 << 3);
    pub(crate) const DELETE_SPACE: Self = Self(1 << 4);
    pub(crate) const DELETE_TOKEN: Self = Self(1 << 5);
    pub(crate) const INSERT_TRAILING_SEMICOLON: Self = Self(1 << 6);

    pub(crate) const STOP_ACTION: Self =
        Self(Self::STOP_PROCESSING_SPACE_ACTIONS.0 | Self::STOP_PROCESSING_TOKEN_ACTIONS.0);
    pub(crate) const MODIFY_SPACE_ACTION: Self =
        Self(Self::INSERT_SPACE.0 | Self::INSERT_NEW_LINE.0 | Self::DELETE_SPACE.0);
    pub(crate) const MODIFY_TOKEN_ACTION: Self =
        Self(Self::DELETE_TOKEN.0 | Self::INSERT_TRAILING_SEMICOLON.0);

    pub(crate) fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub(crate) fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuleFlags {
    None,
    CanDeleteNewLines,
}

#[derive(Clone)]
pub(crate) struct RuleImpl {
    pub(crate) debug_name: &'static str,
    pub(crate) context: Vec<ContextPredicate>,
    pub(crate) action: RuleAction,
    pub(crate) flags: RuleFlags,
}

#[derive(Clone)]
pub(crate) struct TokenRange {
    pub(crate) tokens: Vec<SyntaxKind>,
}

impl TokenRange {
    pub(crate) fn contains(&self, kind: SyntaxKind) -> bool {
        self.tokens.contains(&kind)
    }
}

#[derive(Clone)]
pub(crate) struct RuleSpec {
    pub(crate) left_token_range: TokenRange,
    pub(crate) right_token_range: TokenRange,
    pub(crate) rule: RuleImpl,
}

pub(crate) struct RuleBuilder {
    pub(crate) debug_name: &'static str,
}

impl RuleBuilder {
    /// Go rule()：left/right 接受 Kind、Vec<Kind> 或 tokenRange（此处由
    /// 调用点先归一为 TokenRange）
    pub(crate) fn rule(
        self,
        left: TokenRange,
        right: TokenRange,
        context: Vec<ContextPredicate>,
        action: RuleAction,
        flags: RuleFlags,
    ) -> RuleSpec {
        RuleSpec {
            left_token_range: left,
            right_token_range: right,
            rule: RuleImpl {
                debug_name: self.debug_name,
                context,
                action,
                flags,
            },
        }
    }
}

pub(crate) fn r(name: &'static str) -> RuleBuilder {
    RuleBuilder { debug_name: name }
}

pub(crate) fn kind_range(kind: SyntaxKind) -> TokenRange {
    TokenRange { tokens: vec![kind] }
}

pub(crate) fn kinds_range(kinds: &[SyntaxKind]) -> TokenRange {
    TokenRange { tokens: kinds.to_vec() }
}
