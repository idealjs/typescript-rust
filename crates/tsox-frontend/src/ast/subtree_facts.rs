#![allow(non_upper_case_globals)]

pub use crate::ast::mig::m3e_3::{
    propagate_binding_element_subtree_facts, propagate_eraseable_syntax_list_subtree_facts,
    propagate_eraseable_syntax_subtree_facts, propagate_modifier_list_subtree_facts,
    propagate_node_list_subtree_facts, propagate_object_binding_element_subtree_facts,
    propagate_subtree_facts, SubtreeFacts,
};

pub const SubtreeFactsNone: SubtreeFacts = SubtreeFacts(0);
pub const SubtreeContainsTypeScript: SubtreeFacts = SubtreeFacts::CONTAINS_TYPE_SCRIPT;
pub const SubtreeContainsJsx: SubtreeFacts = SubtreeFacts::CONTAINS_JSX;
pub const SubtreeContainsEsDecorators: SubtreeFacts = SubtreeFacts::CONTAINS_ES_DECORATORS;
pub const SubtreeContainsUsing: SubtreeFacts = SubtreeFacts::CONTAINS_USING;
pub const SubtreeContainsClassStaticBlocks: SubtreeFacts = SubtreeFacts::CONTAINS_CLASS_STATIC_BLOCKS;
pub const SubtreeContainsEsClassFields: SubtreeFacts = SubtreeFacts::CONTAINS_ES_CLASS_FIELDS;
pub const SubtreeContainsLogicalAssignments: SubtreeFacts = SubtreeFacts::CONTAINS_LOGICAL_ASSIGNMENTS;
pub const SubtreeContainsNullishCoalescing: SubtreeFacts = SubtreeFacts::CONTAINS_NULLISH_COALESCING;
pub const SubtreeContainsOptionalChaining: SubtreeFacts = SubtreeFacts::CONTAINS_OPTIONAL_CHAINING;
pub const SubtreeContainsMissingCatchClauseVariable: SubtreeFacts =
    SubtreeFacts::CONTAINS_MISSING_CATCH_CLAUSE_VARIABLE;
pub const SubtreeContainsEsObjectRestOrSpread: SubtreeFacts =
    SubtreeFacts::CONTAINS_ES_OBJECT_REST_OR_SPREAD;
pub const SubtreeContainsForAwaitOrAsyncGenerator: SubtreeFacts =
    SubtreeFacts::CONTAINS_FOR_AWAIT_OR_ASYNC_GENERATOR;
pub const SubtreeContainsAnyAwait: SubtreeFacts = SubtreeFacts::CONTAINS_ANY_AWAIT;
pub const SubtreeContainsExponentiationOperator: SubtreeFacts =
    SubtreeFacts::CONTAINS_EXPONENTIATION_OPERATOR;
pub const SubtreeContainsLexicalThis: SubtreeFacts = SubtreeFacts::CONTAINS_LEXICAL_THIS;
pub const SubtreeContainsLexicalSuper: SubtreeFacts = SubtreeFacts::CONTAINS_LEXICAL_SUPER;
pub const SubtreeContainsRestOrSpread: SubtreeFacts = SubtreeFacts::CONTAINS_REST_OR_SPREAD;
pub const SubtreeContainsObjectRestOrSpread: SubtreeFacts =
    SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD;
pub const SubtreeContainsAwait: SubtreeFacts = SubtreeFacts::CONTAINS_AWAIT;
pub const SubtreeContainsDynamicImport: SubtreeFacts = SubtreeFacts::CONTAINS_DYNAMIC_IMPORT;
pub const SubtreeContainsClassFields: SubtreeFacts = SubtreeFacts::CONTAINS_CLASS_FIELDS;
pub const SubtreeContainsDecorators: SubtreeFacts = SubtreeFacts::CONTAINS_DECORATORS;
pub const SubtreeContainsIdentifier: SubtreeFacts = SubtreeFacts::CONTAINS_IDENTIFIER;
pub const SubtreeContainsPrivateIdentifierInExpression: SubtreeFacts =
    SubtreeFacts::CONTAINS_PRIVATE_IDENTIFIER_IN_EXPRESSION;
pub const SubtreeContainsInvalidTemplateEscape: SubtreeFacts =
    SubtreeFacts::CONTAINS_INVALID_TEMPLATE_ESCAPE;

pub const SUBTREE_FACTS_NONE: SubtreeFacts = SubtreeFacts(0);

const SUBTREE_FACTS_MASK: u32 = (1 << 26) - 1;

impl SubtreeFacts {
    pub const TypeScript: SubtreeFacts = SubtreeFacts::CONTAINS_TYPE_SCRIPT;
    pub const Using: SubtreeFacts = SubtreeFacts::CONTAINS_USING;
    pub const ClassFields: SubtreeFacts = SubtreeFacts::CONTAINS_CLASS_FIELDS;
    pub const AnyAwait: SubtreeFacts = SubtreeFacts::CONTAINS_ANY_AWAIT;
    pub const Await: SubtreeFacts = SubtreeFacts::CONTAINS_AWAIT;
    pub const LexicalThis: SubtreeFacts = SubtreeFacts::CONTAINS_LEXICAL_THIS;
    pub const LexicalSuper: SubtreeFacts = SubtreeFacts::CONTAINS_LEXICAL_SUPER;
    pub const ExponentiationOperator: SubtreeFacts = SubtreeFacts::CONTAINS_EXPONENTIATION_OPERATOR;
    pub const NullishCoalescing: SubtreeFacts = SubtreeFacts::CONTAINS_NULLISH_COALESCING;
    pub const OptionalChaining: SubtreeFacts = SubtreeFacts::CONTAINS_OPTIONAL_CHAINING;
    pub const LogicalAssignments: SubtreeFacts = SubtreeFacts::CONTAINS_LOGICAL_ASSIGNMENTS;
    pub const ForAwaitOrAsyncGenerator: SubtreeFacts =
        SubtreeFacts::CONTAINS_FOR_AWAIT_OR_ASYNC_GENERATOR;
    pub const DynamicImport: SubtreeFacts = SubtreeFacts::CONTAINS_DYNAMIC_IMPORT;
    pub const RestOrSpread: SubtreeFacts = SubtreeFacts::CONTAINS_REST_OR_SPREAD;
    pub const ObjectRestOrSpread: SubtreeFacts = SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD;
    pub const ESObjectRestOrSpread: SubtreeFacts = SubtreeFacts::CONTAINS_ES_OBJECT_REST_OR_SPREAD;
    pub const Decorators: SubtreeFacts = SubtreeFacts::CONTAINS_DECORATORS;
    pub const Identifier: SubtreeFacts = SubtreeFacts::CONTAINS_IDENTIFIER;
    pub const PrivateIdentifierInExpression: SubtreeFacts =
        SubtreeFacts::CONTAINS_PRIVATE_IDENTIFIER_IN_EXPRESSION;
    pub const InvalidTemplateEscape: SubtreeFacts = SubtreeFacts::CONTAINS_INVALID_TEMPLATE_ESCAPE;
    pub const MissingCatchClauseVariable: SubtreeFacts =
        SubtreeFacts::CONTAINS_MISSING_CATCH_CLAUSE_VARIABLE;
    pub const Computed: SubtreeFacts = SubtreeFacts::COMPUTED;

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn from_bits_truncate(bits: u32) -> SubtreeFacts {
        SubtreeFacts(bits & SUBTREE_FACTS_MASK)
    }

    pub const fn intersects(self, other: SubtreeFacts) -> bool {
        self.0 & other.0 != 0
    }

    pub const fn difference(self, other: SubtreeFacts) -> SubtreeFacts {
        SubtreeFacts(self.0 & !other.0)
    }

    pub const fn empty() -> SubtreeFacts {
        SubtreeFacts(0)
    }
}

impl std::ops::BitOr for SubtreeFacts {
    type Output = SubtreeFacts;
    fn bitor(self, rhs: SubtreeFacts) -> SubtreeFacts { ::tsox_core::fntrace::enter("bitor"); 
        SubtreeFacts(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SubtreeFacts {
    fn bitor_assign(&mut self, rhs: SubtreeFacts) { ::tsox_core::fntrace::enter("bitor_assign"); 
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for SubtreeFacts {
    type Output = SubtreeFacts;
    fn bitand(self, rhs: SubtreeFacts) -> SubtreeFacts { ::tsox_core::fntrace::enter("bitand"); 
        SubtreeFacts(self.0 & rhs.0)
    }
}

impl std::ops::Not for SubtreeFacts {
    type Output = SubtreeFacts;
    fn not(self) -> SubtreeFacts { ::tsox_core::fntrace::enter("not"); 
        SubtreeFacts(!self.0 & SUBTREE_FACTS_MASK)
    }
}

pub const SUBTREE_EXCLUSIONS_NODE: SubtreeFacts = SubtreeFacts::COMPUTED;
pub const SUBTREE_EXCLUSIONS_OUTER_EXPRESSION: SubtreeFacts = SUBTREE_EXCLUSIONS_NODE;
pub const SUBTREE_EXCLUSIONS_PROPERTY_ACCESS: SubtreeFacts = SUBTREE_EXCLUSIONS_NODE;
pub const SUBTREE_EXCLUSIONS_ELEMENT_ACCESS: SubtreeFacts = SUBTREE_EXCLUSIONS_NODE;
pub const SUBTREE_EXCLUSIONS_ARROW_FUNCTION: SubtreeFacts = SubtreeFacts(
    SUBTREE_EXCLUSIONS_NODE.0 | SubtreeFacts::Await.0 | SubtreeFacts::ObjectRestOrSpread.0,
);
pub const SUBTREE_EXCLUSIONS_FUNCTION: SubtreeFacts = SubtreeFacts(
    SUBTREE_EXCLUSIONS_NODE.0
        | SubtreeFacts::LexicalThis.0
        | SubtreeFacts::LexicalSuper.0
        | SubtreeFacts::Await.0
        | SubtreeFacts::ObjectRestOrSpread.0,
);
pub const SUBTREE_EXCLUSIONS_CONSTRUCTOR: SubtreeFacts = SUBTREE_EXCLUSIONS_FUNCTION;
pub const SUBTREE_EXCLUSIONS_METHOD: SubtreeFacts = SUBTREE_EXCLUSIONS_FUNCTION;
pub const SUBTREE_EXCLUSIONS_ACCESSOR: SubtreeFacts = SUBTREE_EXCLUSIONS_FUNCTION;
pub const SUBTREE_EXCLUSIONS_PROPERTY: SubtreeFacts = SubtreeFacts(
    SUBTREE_EXCLUSIONS_NODE.0 | SubtreeFacts::LexicalThis.0 | SubtreeFacts::LexicalSuper.0,
);
pub const SUBTREE_EXCLUSIONS_CLASS: SubtreeFacts = SUBTREE_EXCLUSIONS_NODE;
pub const SUBTREE_EXCLUSIONS_MODULE: SubtreeFacts = SubtreeFacts(
    SUBTREE_EXCLUSIONS_NODE.0 | SubtreeFacts::LexicalThis.0 | SubtreeFacts::LexicalSuper.0,
);
pub const SUBTREE_EXCLUSIONS_OBJECT_LITERAL: SubtreeFacts = SubtreeFacts(
    SUBTREE_EXCLUSIONS_NODE.0 | SubtreeFacts::ObjectRestOrSpread.0,
);
pub const SUBTREE_EXCLUSIONS_ARRAY_LITERAL: SubtreeFacts = SUBTREE_EXCLUSIONS_NODE;
pub const SUBTREE_EXCLUSIONS_CALL: SubtreeFacts = SUBTREE_EXCLUSIONS_NODE;
pub const SUBTREE_EXCLUSIONS_NEW: SubtreeFacts = SUBTREE_EXCLUSIONS_NODE;
pub const SUBTREE_EXCLUSIONS_VARIABLE_DECLARATION_LIST: SubtreeFacts = SubtreeFacts(
    SUBTREE_EXCLUSIONS_NODE.0 | SubtreeFacts::ObjectRestOrSpread.0,
);
pub const SUBTREE_EXCLUSIONS_PARAMETER: SubtreeFacts = SUBTREE_EXCLUSIONS_NODE;
pub const SUBTREE_EXCLUSIONS_CATCH_CLAUSE: SubtreeFacts = SubtreeFacts(
    SUBTREE_EXCLUSIONS_NODE.0 | SubtreeFacts::ObjectRestOrSpread.0,
);
pub const SUBTREE_EXCLUSIONS_BINDING_PATTERN: SubtreeFacts = SubtreeFacts(
    SUBTREE_EXCLUSIONS_NODE.0 | SubtreeFacts::RestOrSpread.0,
);
