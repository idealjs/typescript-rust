use crate::ast::node::Node;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct FlowFlags(u32);

impl FlowFlags {
    pub const UNREACHABLE: Self = Self(1 << 0);
    pub const START: Self = Self(1 << 1);
    pub const BRANCH_LABEL: Self = Self(1 << 2);
    pub const LOOP_LABEL: Self = Self(1 << 3);
    pub const ASSIGNMENT: Self = Self(1 << 4);
    pub const TRUE_CONDITION: Self = Self(1 << 5);
    pub const FALSE_CONDITION: Self = Self(1 << 6);
    pub const SWITCH_CLAUSE: Self = Self(1 << 7);
    pub const ARRAY_MUTATION: Self = Self(1 << 8);
    pub const CALL: Self = Self(1 << 9);
    pub const REDUCE_LABEL: Self = Self(1 << 10);
    pub const REFERENCED: Self = Self(1 << 11);
    pub const SHARED: Self = Self(1 << 12);

    pub const LABEL: Self = Self(Self::BRANCH_LABEL.0 | Self::LOOP_LABEL.0);
    pub const CONDITION: Self = Self(Self::TRUE_CONDITION.0 | Self::FALSE_CONDITION.0);

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for FlowFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug)]
pub struct FlowNode {
    pub flags: FlowFlags,
    pub node: Option<Arc<Node>>,
    pub antecedent: Option<Arc<FlowNode>>,
    pub antecedents: Vec<Arc<FlowNode>>,

    pub switch_statement: Option<Arc<Node>>,

    pub clause_range: Option<(usize, usize)>,

    pub reduce_target: Option<Arc<FlowNode>>,
}

impl FlowNode {
    pub fn new(flags: FlowFlags) -> Self {
        Self {
            flags,
            node: None,
            antecedent: None,
            antecedents: Vec::new(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        }
    }
}

pub type FlowLabel = FlowNode;
