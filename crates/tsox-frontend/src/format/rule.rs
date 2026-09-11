//! Go format/rule.go 的移植：规则结构与动作/旗标位掩码。

use super::rule_context::FormattingContext;
use crate::ast::SyntaxKind;

pub(crate) type ContextPredicate = fn(&FormattingContext) -> bool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuleAction(u16);

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
    pub(crate) const MODIFY_SPACE_ACTION: Self = Self(
        Self::INSERT_SPACE.0 | Self::INSERT_NEW_LINE.0 | Self::DELETE_SPACE.0,
    );
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

#[derive(Clone, Copy)]
pub(crate) struct RuleImpl {
    pub(crate) context: &'static [ContextPredicate],
    pub(crate) action: RuleAction,
    pub(crate) flags: RuleFlags,
}

impl RuleImpl {
    pub(crate) fn action(&self) -> RuleAction {
        self.action
    }
    pub(crate) fn flags(&self) -> RuleFlags {
        self.flags
    }
    pub(crate) fn context(&self) -> &'static [ContextPredicate] {
        self.context
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TokenRange {
    pub(crate) tokens: &'static [SyntaxKind],
    /// Go isSpecific：具体 token 集合（vs any 的全量排除集）；
    /// 匹配语义相同（contains），仅影响调试
    pub(crate) is_specific: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RuleSpec {
    pub(crate) left_token_range: TokenRange,
    pub(crate) right_token_range: TokenRange,
    pub(crate) rule: &'static RuleImpl,
}

pub(crate) fn rule(
    left: TokenRange,
    right: TokenRange,
    context: &'static [ContextPredicate],
    action: RuleAction,
    flags: RuleFlags,
    rule: &'static RuleImpl,
) -> RuleSpec {
    RuleSpec { left_token_range: left, right_token_range: right, rule }
}
