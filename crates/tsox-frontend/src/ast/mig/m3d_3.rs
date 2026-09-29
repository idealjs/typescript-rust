#![allow(unused_imports)]
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::ast::node::{ModifierList, Node, NodeList, SourceFile};
use crate::ast::node_data_generated::*;
use crate::ast::node_flags::{ModifierFlags, NodeFlags};
use crate::scanner::error_callback::TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::subtree_facts::*;
use crate::ast::utilities::*;
use super::m3d::*;

pub fn compute_subtree_facts_token(kind: SyntaxKind) -> SubtreeFacts {
    match kind {
        SyntaxKind::UsingKeyword => SubtreeFacts::Using,
        SyntaxKind::PublicKeyword
        | SyntaxKind::PrivateKeyword
        | SyntaxKind::ProtectedKeyword
        | SyntaxKind::ReadonlyKeyword
        | SyntaxKind::AbstractKeyword
        | SyntaxKind::DeclareKeyword
        | SyntaxKind::ConstKeyword
        | SyntaxKind::AnyKeyword
        | SyntaxKind::NumberKeyword
        | SyntaxKind::BigIntKeyword
        | SyntaxKind::NeverKeyword
        | SyntaxKind::ObjectKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::OutKeyword
        | SyntaxKind::OverrideKeyword
        | SyntaxKind::StringKeyword
        | SyntaxKind::BooleanKeyword
        | SyntaxKind::SymbolKeyword
        | SyntaxKind::VoidKeyword
        | SyntaxKind::UnknownKeyword
        | SyntaxKind::UndefinedKeyword
        | SyntaxKind::ExportKeyword => SubtreeFacts::TypeScript,
        SyntaxKind::AccessorKeyword => SubtreeFacts::ClassFields,
        SyntaxKind::AsyncKeyword => SubtreeFacts::AnyAwait,
        SyntaxKind::SuperKeyword => SubtreeFacts::LexicalSuper,
        SyntaxKind::ThisKeyword => SubtreeFacts::LexicalThis,
        SyntaxKind::AsteriskAsteriskToken | SyntaxKind::AsteriskAsteriskEqualsToken => {
            SubtreeFacts::ExponentiationOperator
        }
        SyntaxKind::QuestionQuestionToken => SubtreeFacts::NullishCoalescing,
        SyntaxKind::QuestionDotToken => SubtreeFacts::OptionalChaining,
        SyntaxKind::QuestionQuestionEqualsToken
        | SyntaxKind::BarBarEqualsToken
        | SyntaxKind::AmpersandAmpersandEqualsToken => SubtreeFacts::LogicalAssignments,
        _ => SUBTREE_FACTS_NONE,
    }
}

pub fn compute_subtree_facts_type_syntax_base() -> SubtreeFacts {
    SubtreeFacts::TypeScript
}

pub fn propagate_subtree_facts_type_syntax_base() -> SubtreeFacts {
    SubtreeFacts::TypeScript
}

pub fn compute_subtree_facts_node_default() -> SubtreeFacts {
    SUBTREE_FACTS_NONE
}

pub fn propagate_subtree_facts_node_default(facts: SubtreeFacts) -> SubtreeFacts {
    facts.difference(SUBTREE_EXCLUSIONS_NODE)
}

pub fn set_modifiers_node_default() {}

pub fn set_modifiers_modifiers_base(
    modifiers_field: &mut Option<Arc<ModifierList>>,
    modifiers: Option<Arc<ModifierList>>,
) {
    *modifiers_field = modifiers;
}

pub fn set_modifiers_named_member_base(
    modifiers_field: &mut Option<Arc<ModifierList>>,
    modifiers: Option<Arc<ModifierList>>,
) {
    *modifiers_field = modifiers;
}

pub fn subtree_facts_worker_default(compute: impl FnOnce() -> SubtreeFacts) -> SubtreeFacts {
    compute()
}

pub fn subtree_facts_worker_composite(
    facts_cell: &AtomicU32,
    compute: impl FnOnce() -> SubtreeFacts,
) -> SubtreeFacts {
    let mut facts = SubtreeFacts::from_bits_truncate(facts_cell.load(Ordering::Relaxed));
    if !facts.intersects(SubtreeFacts::Computed) {
        facts |= compute() | SubtreeFacts::Computed;
        facts_cell.store(facts.bits(), Ordering::Relaxed);
    }
    facts.difference(SubtreeFacts::Computed)
}

impl Node {
    pub fn subtree_facts(&self) -> SubtreeFacts {
        subtree_facts_worker_default(|| compute_subtree_facts_dispatch(self))
    }

    pub fn propagate_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts_node_default(self.subtree_facts())
    }
}

pub fn compute_subtree_facts_dispatch(node: &Node) -> SubtreeFacts {
    match &node.data {
        NodeData::Token => compute_subtree_facts_token(node.kind),
        NodeData::SpreadAssignment(d) => d.compute_subtree_facts(),
        NodeData::SpreadElement(d) => d.compute_subtree_facts(),
        NodeData::TaggedTemplateExpression(d) => d.compute_subtree_facts(),
        NodeData::TemplateHead(d) => d.compute_subtree_facts(),
        NodeData::TemplateMiddle(d) => d.compute_subtree_facts(),
        NodeData::TemplateTail(d) => d.compute_subtree_facts(),
        NodeData::TypeAssertion(d) => d.compute_subtree_facts(),
        NodeData::VariableDeclaration(d) => d.compute_subtree_facts(),
        NodeData::VariableDeclarationList(d) => d.compute_subtree_facts(node.flags),
        NodeData::VariableStatement(d) => d.compute_subtree_facts(),
        NodeData::YieldExpression(d) => d.compute_subtree_facts(),
        NodeData::CatchClause(d) => d.compute_subtree_facts(),
        NodeData::ClassDeclaration(d) => d.compute_subtree_facts(),
        NodeData::ClassExpression(d) => d.compute_subtree_facts(),
        NodeData::ConstructorDeclaration(d) => d.compute_subtree_facts(),
        NodeData::FunctionDeclaration(d) => d.compute_subtree_facts(),
        NodeData::FunctionExpression(d) => d.compute_subtree_facts(),
        NodeData::MethodDeclaration(d) => d.compute_subtree_facts(),
        NodeData::ModuleDeclaration(d) => d.compute_subtree_facts(),
        NodeData::ArrowFunction(d) => d.compute_subtree_facts(),
        NodeData::AsExpression(d) => d.compute_subtree_facts(),
        NodeData::SatisfiesExpression(d) => d.compute_subtree_facts(),
        NodeData::PropertyAccessExpression(d) => d.compute_subtree_facts(),
        NodeData::PropertyDeclaration(d) => d.compute_subtree_facts(),
        NodeData::CallExpression(d) => d.compute_subtree_facts(),
        NodeData::NewExpression(d) => d.compute_subtree_facts(),
        NodeData::BindingPattern(d) => d.compute_subtree_facts(node.kind),
        NodeData::ParameterDeclaration(d) => d.compute_subtree_facts(),
        _ => compute_subtree_facts_node_default(),
    }
}

impl SpreadAssignmentData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts(Some(&self.expression))
            .union(SubtreeFacts::ESObjectRestOrSpread)
            .union(SubtreeFacts::ObjectRestOrSpread)
    }
}

impl SpreadElementData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts(Some(&self.expression)).union(SubtreeFacts::RestOrSpread)
    }
}

impl TaggedTemplateExpressionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts(Some(&self.tag))
            .union(propagate_subtree_facts(self.question_dot_token.as_ref()))
            .union(propagate_eraseable_syntax_list_subtree_facts(self.type_arguments.as_deref()))
            .union(propagate_subtree_facts(Some(&self.template)))
    }
}

impl TemplateHeadData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        if self.template_flags & TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE != 0 {
            return SubtreeFacts::InvalidTemplateEscape;
        }
        SUBTREE_FACTS_NONE
    }
}

impl TemplateMiddleData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        if self.template_flags & TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE != 0 {
            return SubtreeFacts::InvalidTemplateEscape;
        }
        SUBTREE_FACTS_NONE
    }
}

impl TemplateTailData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        if self.template_flags & TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE != 0 {
            return SubtreeFacts::InvalidTemplateEscape;
        }
        SUBTREE_FACTS_NONE
    }
}

impl TypeAssertionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts(Some(&self.expression)).union(SubtreeFacts::TypeScript)
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_OUTER_EXPRESSION)
    }
}

impl VariableDeclarationData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts(Some(&self.name))
            .union(propagate_eraseable_syntax_subtree_facts(self.exclamation_token.as_ref()))
            .union(propagate_eraseable_syntax_subtree_facts(self.type_node.as_ref()))
            .union(propagate_subtree_facts(self.initializer.as_ref()))
    }
}

impl VariableDeclarationListData {
    pub fn compute_subtree_facts(&self, node_flags: NodeFlags) -> SubtreeFacts {
        let mut facts = propagate_node_list_subtree_facts(Some(&self.declarations), |n: &Arc<Node>| propagate_subtree_facts(Some(n)));
        if node_flags.intersects(NodeFlags::Using) {
            facts |= SubtreeFacts::Using;
        }
        facts
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_VARIABLE_DECLARATION_LIST)
    }
}

impl VariableStatementData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        if self
            .modifiers
            .as_ref()
            .is_some_and(|m| m.modifier_flags.intersects(ModifierFlags::Ambient))
        {
            SubtreeFacts::TypeScript
        } else {
            propagate_modifier_list_subtree_facts(self.modifiers.as_deref())
                .union(propagate_subtree_facts(Some(&self.declaration_list)))
        }
    }
}

impl YieldExpressionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts(self.expression.as_ref())
            .union(SubtreeFacts::ForAwaitOrAsyncGenerator)
    }
}

impl CatchClauseData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        let mut res = propagate_subtree_facts(self.variable_declaration.as_ref())
            .union(propagate_subtree_facts(Some(&self.block)));
        if self.variable_declaration.is_none() {
            res |= SubtreeFacts::MissingCatchClauseVariable;
        }
        res
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_CATCH_CLAUSE)
    }
}

impl ClassDeclarationData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        class_like_compute_subtree_facts(
            self.modifiers.as_deref(),
            self.name.as_ref(),
            self.type_parameters.as_deref(),
            self.heritage_clauses.as_deref(),
            &self.members,
        )
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_CLASS)
    }
}

impl ClassExpressionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        class_like_compute_subtree_facts(
            self.modifiers.as_deref(),
            self.name.as_ref(),
            self.type_parameters.as_deref(),
            self.heritage_clauses.as_deref(),
            &self.members,
        )
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_CLASS)
    }
}

fn class_like_compute_subtree_facts(
    modifiers: Option<&ModifierList>,
    name: Option<&Arc<Node>>,
    type_parameters: Option<&NodeList>,
    heritage_clauses: Option<&NodeList>,
    members: &NodeList,
) -> SubtreeFacts {
    if modifiers
        .map(|m| m.modifier_flags.intersects(ModifierFlags::Ambient))
        .unwrap_or(false)
    {
        return SubtreeFacts::TypeScript;
    }
    propagate_modifier_list_subtree_facts(modifiers)
        .union(propagate_subtree_facts(name))
        .union(propagate_eraseable_syntax_list_subtree_facts(type_parameters))
        .union(propagate_node_list_subtree_facts(heritage_clauses, |n: &Arc<Node>| propagate_subtree_facts(Some(n))))
        .union(propagate_node_list_subtree_facts(Some(members), |n: &Arc<Node>| propagate_subtree_facts(Some(n))))
}

impl ConstructorDeclarationData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        if self.body.is_none() {
            return SubtreeFacts::TypeScript;
        }
        propagate_modifier_list_subtree_facts(self.modifiers.as_deref())
            .union(propagate_eraseable_syntax_list_subtree_facts(self.type_parameters.as_deref()))
            .union(propagate_node_list_subtree_facts(Some(&self.parameters), |n: &Arc<Node>| propagate_subtree_facts(Some(n))))
            .union(propagate_eraseable_syntax_subtree_facts(self.type_node.as_ref()))
            .union(propagate_eraseable_syntax_subtree_facts(self.full_signature.as_ref()))
            .union(propagate_subtree_facts(self.body.as_ref()))
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_CONSTRUCTOR)
    }
}

impl FunctionDeclarationData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        if self.body.is_none()
            || self
                .modifiers
                .as_ref()
                .is_some_and(|m| m.modifier_flags.intersects(ModifierFlags::Ambient))
        {
            return SubtreeFacts::TypeScript;
        }
        let is_async = self
            .modifiers
            .as_ref()
            .is_some_and(|m| m.modifier_flags.intersects(ModifierFlags::Async));
        let is_generator = self.asterisk_token.is_some();
        propagate_modifier_list_subtree_facts(self.modifiers.as_deref())
            .union(propagate_subtree_facts(self.asterisk_token.as_ref()))
            .union(propagate_subtree_facts(self.name.as_ref()))
            .union(propagate_eraseable_syntax_list_subtree_facts(self.type_parameters.as_deref()))
            .union(propagate_node_list_subtree_facts(Some(&self.parameters), |n: &Arc<Node>| propagate_subtree_facts(Some(n))))
            .union(propagate_eraseable_syntax_subtree_facts(self.type_node.as_ref()))
            .union(propagate_eraseable_syntax_subtree_facts(self.full_signature.as_ref()))
            .union(propagate_subtree_facts(self.body.as_ref()))
            .union(if is_async && is_generator {
                SubtreeFacts::ForAwaitOrAsyncGenerator
            } else {
                SUBTREE_FACTS_NONE
            })
            .union(if is_async && !is_generator {
                SubtreeFacts::AnyAwait
            } else {
                SUBTREE_FACTS_NONE
            })
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_FUNCTION)
    }
}

impl FunctionExpressionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        let is_async = self
            .modifiers
            .as_ref()
            .is_some_and(|m| m.modifier_flags.intersects(ModifierFlags::Async));
        let is_generator = self.asterisk_token.is_some();
        propagate_modifier_list_subtree_facts(self.modifiers.as_deref())
            .union(propagate_subtree_facts(self.asterisk_token.as_ref()))
            .union(propagate_subtree_facts(self.name.as_ref()))
            .union(propagate_eraseable_syntax_list_subtree_facts(self.type_parameters.as_deref()))
            .union(propagate_node_list_subtree_facts(Some(&self.parameters), |n: &Arc<Node>| propagate_subtree_facts(Some(n))))
            .union(propagate_eraseable_syntax_subtree_facts(self.type_node.as_ref()))
            .union(propagate_eraseable_syntax_subtree_facts(self.full_signature.as_ref()))
            .union(propagate_subtree_facts(Some(&self.body)))
            .union(if is_async && is_generator {
                SubtreeFacts::ForAwaitOrAsyncGenerator
            } else {
                SUBTREE_FACTS_NONE
            })
            .union(if is_async && !is_generator {
                SubtreeFacts::AnyAwait
            } else {
                SUBTREE_FACTS_NONE
            })
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_FUNCTION)
    }
}

impl MethodDeclarationData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        if self.body.is_none() {
            return SubtreeFacts::TypeScript;
        }
        let is_async = self
            .modifiers
            .as_ref()
            .is_some_and(|m| m.modifier_flags.intersects(ModifierFlags::Async));
        let is_generator = self.asterisk_token.is_some();
        propagate_modifier_list_subtree_facts(self.modifiers.as_deref())
            .union(propagate_subtree_facts(self.asterisk_token.as_ref()))
            .union(propagate_subtree_facts(Some(&self.name)))
            .union(propagate_eraseable_syntax_subtree_facts(self.postfix_token.as_ref()))
            .union(propagate_eraseable_syntax_list_subtree_facts(self.type_parameters.as_deref()))
            .union(propagate_node_list_subtree_facts(Some(&self.parameters), |n: &Arc<Node>| propagate_subtree_facts(Some(n))))
            .union(propagate_subtree_facts(self.body.as_ref()))
            .union(propagate_eraseable_syntax_subtree_facts(self.type_node.as_ref()))
            .union(propagate_eraseable_syntax_subtree_facts(self.full_signature.as_ref()))
            .union(if is_async && is_generator {
                SubtreeFacts::ForAwaitOrAsyncGenerator
            } else {
                SUBTREE_FACTS_NONE
            })
            .union(if is_async && !is_generator {
                SubtreeFacts::AnyAwait
            } else {
                SUBTREE_FACTS_NONE
            })
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts
            .difference(SUBTREE_EXCLUSIONS_METHOD)
            .union(propagate_subtree_facts(Some(&self.name)))
    }
}

impl ModuleDeclarationData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        if self
            .modifiers
            .as_ref()
            .is_some_and(|m| m.modifier_flags.intersects(ModifierFlags::Ambient))
        {
            return SubtreeFacts::TypeScript;
        }
        propagate_modifier_list_subtree_facts(self.modifiers.as_deref())
            .union(propagate_subtree_facts(Some(&self.name)))
            .union(propagate_subtree_facts(self.body.as_ref()))
            .union(SubtreeFacts::TypeScript)
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_MODULE)
    }
}

impl ArrowFunctionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_modifier_list_subtree_facts(self.modifiers.as_deref())
            .union(propagate_eraseable_syntax_list_subtree_facts(self.type_parameters.as_deref()))
            .union(propagate_node_list_subtree_facts(Some(&self.parameters), |n: &Arc<Node>| propagate_subtree_facts(Some(n))))
            .union(propagate_eraseable_syntax_subtree_facts(self.type_node.as_ref()))
            .union(propagate_eraseable_syntax_subtree_facts(self.full_signature.as_ref()))
            .union(propagate_subtree_facts(Some(&self.body)))
            .union(
                if self
                    .modifiers
                    .as_ref()
                    .is_some_and(|m| m.modifier_flags.intersects(ModifierFlags::Async))
                {
                    SubtreeFacts::AnyAwait
                } else {
                    SUBTREE_FACTS_NONE
                },
            )
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_ARROW_FUNCTION)
    }
}

impl AsExpressionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts(Some(&self.expression)).union(SubtreeFacts::TypeScript)
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_OUTER_EXPRESSION)
    }
}

impl SatisfiesExpressionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts(Some(&self.expression)).union(SubtreeFacts::TypeScript)
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_OUTER_EXPRESSION)
    }
}

impl PropertyAccessExpressionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        let private_name = if is_identifier(&self.name) {
            SUBTREE_FACTS_NONE
        } else {
            SubtreeFacts::PrivateIdentifierInExpression
        };
        propagate_subtree_facts(Some(&self.expression))
            .union(propagate_subtree_facts(self.question_dot_token.as_ref()))
            .union(propagate_subtree_facts(Some(&self.name)))
            .union(private_name)
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_PROPERTY_ACCESS)
    }
}

impl ElementAccessExpressionData {
    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_ELEMENT_ACCESS)
    }
}

impl PropertyDeclarationData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_modifier_list_subtree_facts(self.modifiers.as_deref())
            .union(propagate_subtree_facts(Some(&self.name)))
            .union(propagate_eraseable_syntax_subtree_facts(self.postfix_token.as_ref()))
            .union(propagate_eraseable_syntax_subtree_facts(self.type_node.as_ref()))
            .union(propagate_subtree_facts(self.initializer.as_ref()))
            .union(SubtreeFacts::ClassFields)
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts
            .difference(SUBTREE_EXCLUSIONS_PROPERTY)
            .union(propagate_subtree_facts(Some(&self.name)))
    }
}

impl CallExpressionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts(Some(&self.expression))
            .union(propagate_subtree_facts(self.question_dot_token.as_ref()))
            .union(propagate_eraseable_syntax_list_subtree_facts(self.type_arguments.as_deref()))
            .union(propagate_node_list_subtree_facts(Some(&self.arguments), |n: &Arc<Node>| propagate_subtree_facts(Some(n))))
            .union(if self.expression.kind == SyntaxKind::ImportKeyword {
                SubtreeFacts::DynamicImport
            } else {
                SUBTREE_FACTS_NONE
            })
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_CALL)
    }
}

impl NewExpressionData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        propagate_subtree_facts(Some(&self.expression))
            .union(propagate_eraseable_syntax_list_subtree_facts(self.type_arguments.as_deref()))
            .union(propagate_node_list_subtree_facts(self.arguments.as_deref(), |n: &Arc<Node>| propagate_subtree_facts(Some(n))))
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_NEW)
    }
}

impl BindingPatternData {
    pub fn compute_subtree_facts(&self, kind: SyntaxKind) -> SubtreeFacts {
        match kind {
            SyntaxKind::ObjectBindingPattern => propagate_node_list_subtree_facts(
                Some(&self.elements),
                propagate_object_binding_element_subtree_facts,
            ),
            SyntaxKind::ArrayBindingPattern => propagate_node_list_subtree_facts(
                Some(&self.elements),
                propagate_binding_element_subtree_facts,
            ),
            _ => SUBTREE_FACTS_NONE,
        }
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_BINDING_PATTERN)
    }
}

impl ParameterDeclarationData {
    pub fn compute_subtree_facts(&self) -> SubtreeFacts {
        if is_this_identifier(Some(&self.name)) {
            return SubtreeFacts::TypeScript;
        }
        propagate_modifier_list_subtree_facts(self.modifiers.as_deref())
            .union(propagate_subtree_facts(Some(&self.name)))
            .union(propagate_eraseable_syntax_subtree_facts(self.question_token.as_ref()))
            .union(propagate_eraseable_syntax_subtree_facts(self.type_node.as_ref()))
            .union(propagate_subtree_facts(self.initializer.as_ref()))
    }

    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_PARAMETER)
    }
}

impl ArrayLiteralExpressionData {
    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_ARRAY_LITERAL)
    }
}

impl ObjectLiteralExpressionData {
    pub fn propagate_subtree_facts(&self, facts: SubtreeFacts) -> SubtreeFacts {
        facts.difference(SUBTREE_EXCLUSIONS_OBJECT_LITERAL)
    }
}

impl BinaryExpressionData {
    pub fn set_modifiers(&mut self, modifiers: Option<Arc<ModifierList>>) {
        self.modifiers = modifiers;
    }
}

