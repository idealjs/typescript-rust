//! Go format/rulesmap.go 的移植：按 (left, right) token kind 对索引的规则桶。
//!
//! 桶内规则按 Go addRule 的六段次序插入：
//! stop/specific、stop/any、context/specific、context/any、
//! no-context/specific、no-context/any；段内保持声明序。
//! processPair 逆序应用，先入桶者优先级高。

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::ast::SyntaxKind;
use crate::format::rule::{RuleAction, RuleImpl};

use super::rules::get_all_rules;

const MASK_BIT_SIZE: u16 = 5;
/// Go mask = 0b11111（5 位全 1，非「位数减一」）
const MASK: u32 = 0b11111;
const SEGMENTS: u16 = 6;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct BucketKey(u16, u16);

/// 六段子桶位图（对齐 Go rulesBucketConstructionStateList 的 5-bit 段）
struct Bucket {
    rules: Vec<RuleImpl>,
    state: u32,
}

impl Bucket {
    fn new() -> Self {
        Bucket { rules: Vec::new(), state: 0 }
    }

    /// Go getRuleInsertionIndex（position 为位偏移 = 段号 * MASK_BIT_SIZE）
    fn insertion_index(&self, bit_position: u16) -> usize {
        let mut index = 0u32;
        let mut bitmap = self.state;
        let mut pos: u16 = 0;
        while pos <= bit_position {
            index += bitmap & MASK;
            bitmap >>= MASK_BIT_SIZE;
            pos += MASK_BIT_SIZE;
        }
        index as usize
    }

    /// Go increaseInsertionIndex
    fn increase(&mut self, bit_position: u16) {
        let shift = bit_position;
        let value = ((self.state >> shift) & MASK) + 1;
        debug_assert!((value & MASK) == value);
        self.state = (self.state & !(MASK << shift)) | (value << shift);
    }

    fn add(&mut self, rule: RuleImpl, specific_tokens: bool) {
        let position = if rule.action.intersects(RuleAction::STOP_ACTION) {
            if specific_tokens { 0 } else { 1 }
        } else if !rule.context.is_empty() {
            if specific_tokens { 2 } else { 3 }
        } else if specific_tokens {
            4
        } else {
            5
        };
        debug_assert!(position < SEGMENTS);
        let bit_position = position * MASK_BIT_SIZE;
        let index = self.insertion_index(bit_position);
        self.rules.insert(index, rule);
        self.increase(bit_position);
    }
}

static RULES_MAP: OnceLock<HashMap<BucketKey, Vec<RuleImpl>>> = OnceLock::new();

fn build_rules_map() -> HashMap<BucketKey, Vec<RuleImpl>> {
    let mut raw: HashMap<BucketKey, Bucket> = HashMap::new();
    for spec in get_all_rules() {
        let specific = spec.left_token_range.is_specific && spec.right_token_range.is_specific;
        for left in &spec.left_token_range.tokens {
            for right in &spec.right_token_range.tokens {
                let key = BucketKey(*left as u16, *right as u16);
                let bucket = raw.entry(key).or_insert_with(Bucket::new);
                bucket.add(spec.rule.clone(), specific);
            }
        }
    }
    raw.into_iter()
        .map(|(k, b)| (k, b.rules))
        .collect()
}

pub(crate) fn get_rules(left: SyntaxKind, right: SyntaxKind) -> &'static [RuleImpl] {
    let map = RULES_MAP.get_or_init(build_rules_map);
    map.get(&BucketKey(left as u16, right as u16))
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}
