#![allow(unused_imports)]

use crate::checker::checker_classes::*;
use tsox_frontend::ast::{Node, NodeData, NodeList, SyntaxKind, for_each_child, is_function_like, is_super_call};
use tsox_frontend::scanner::skip_trivia;

impl Checker {
    // Go checkConstructorDeclaration 尾段：派生类构造器 super 调用
    // 检查（缺失/extends null/根层级要求）
    pub(crate) fn check_constructor_super_calls(&mut self, ctor: &Arc<Node>, body: &Arc<Node>) {
        let Some(class_decl) = ctor.parent() else {
            return;
        };
        let Some(heritage_element) = class_extends_heritage_element(&class_decl) else {
            return;
        };
        let extends_null = expression_with_type_arguments_expression(&heritage_element).kind
            == SyntaxKind::NullKeyword;
        match find_first_super_call(body) {
            Some(super_call) => {
                if extends_null {
                    let file = self.current_file.clone();
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        super_call.loc,
                        A_CONSTRUCTOR_CANNOT_CONTAIN_A_SUPER_CALL_WHEN_ITS_CLASS_EXTENDS_NULL,
                        vec![],
                    ));
                }
                self.check_super_call_root_level(ctor, &class_decl, &super_call, body);
            }
            None => {
                if !extends_null {
                    let file = self.current_file.clone();
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        self.constructor_keyword_loc(ctor),
                        CONSTRUCTORS_FOR_DERIVED_CLASSES_MUST_CONTAIN_A_SUPER_CALL,
                        vec![],
                    ));
                }
            }
        }
    }

    // Go checkConstructorDeclaration：super 调用必须为根级语句且先于
    // this/super 引用（仅当类含实例属性初始化或参数属性时）
    fn check_super_call_root_level(
        &mut self,
        ctor: &Arc<Node>,
        class_decl: &Arc<Node>,
        super_call: &Arc<Node>,
        body: &Arc<Node>,
    ) {
        if self.emit_standard_class_fields {
            return;
        }
        let has_initialized_instance_member = class_members(class_decl).iter().any(|m| {
            is_private_identifier_class_element_declaration(m)
                || (matches!(&m.data, NodeData::PropertyDeclaration(d) if d.initializer.is_some())
                    && !m.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Static))
        });
        let has_parameter_property = ctor_parameters(ctor).iter().any(|p| {
            p.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::ParameterPropertyModifier)
        });
        if !has_initialized_instance_member && !has_parameter_property {
            return;
        }
        let super_call_parent = walk_up_parenthesized_expressions(
            &super_call.parent().expect("super call has parent"),
        );
        if !(super_call_parent.kind == SyntaxKind::ExpressionStatement
            && super_call_parent
                .parent()
                .is_some_and(|p| Arc::ptr_eq(&p, body)))
        {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                super_call.loc,
                A_SUPER_CALL_MUST_BE_A_ROOT_LEVEL_STATEMENT_WITHIN_A_CONSTRUCTOR_OF_A_DERIVED_CLASS_THAT_CONTAINS_INITIALIZED_PROPERTIES_PARAMETER_PROPERTIES_OR_PRIVATE_IDENTIFIERS,
                vec![],
            ));
            return;
        }
        let mut super_call_statement_found = false;
        for statement in block_statements(body).iter() {
            if statement.kind == SyntaxKind::ExpressionStatement
                && let Some(expr) = statement.expression()
                && is_super_call(skip_outer_expressions(expr).as_ref())
            {
                super_call_statement_found = true;
                break;
            }
            if node_immediately_references_super_or_this(statement) {
                break;
            }
        }
        if !super_call_statement_found {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                self.constructor_keyword_loc(ctor),
                A_SUPER_CALL_MUST_BE_THE_FIRST_STATEMENT_IN_THE_CONSTRUCTOR_TO_REFER_TO_SUPER_OR_THIS_WHEN_A_DERIVED_CLASS_CONTAINS_INITIALIZED_PROPERTIES_PARAMETER_PROPERTIES_OR_PRIVATE_IDENTIFIERS,
                vec![],
            ));
        }
    }

    // Go scanner.GetErrorRangeForNode KindConstructor 分支：报错定位到
    // constructor 关键字 token
    fn constructor_keyword_loc(&self, ctor: &Arc<Node>) -> tsox_core::core::text::TextRange {
        let Some(file) = &self.current_file else {
            return ctor.loc;
        };
        let mut start = ctor.loc.pos();
        if let NodeData::ConstructorDeclaration(d) = &ctor.data
            && let Some(modifiers) = &d.modifiers
            && let Some(last) = modifiers.list.iter().last()
        {
            start = start.max(last.loc.end());
        }
        let kw_start = skip_trivia(&file.text, start);
        if file.text[kw_start..].starts_with("constructor") {
            tsox_core::core::text::TextRange::new(kw_start, kw_start + "constructor".len())
        } else {
            ctor.loc
        }
    }
}

// Go ast.GetClassExtendsHeritageElement
pub(crate) fn class_extends_heritage_element(class_node: &Arc<Node>) -> Option<Arc<Node>> {
    let heritage = match &class_node.data {
        NodeData::ClassDeclaration(d) => d.heritage_clauses.clone(),
        NodeData::ClassExpression(d) => d.heritage_clauses.clone(),
        _ => return None,
    }?;
    heritage.iter().find_map(|clause| match &clause.data {
        NodeData::HeritageClause(hc) if hc.token == SyntaxKind::ExtendsKeyword => {
            hc.types.iter().next().cloned()
        }
        _ => None,
    })
}

pub(crate) fn expression_with_type_arguments_expression(node: &Arc<Node>) -> Arc<Node> {
    match &node.data {
        NodeData::ExpressionWithTypeArguments(d) => Arc::clone(&d.expression),
        _ => Arc::clone(node),
    }
}

// Go classDeclarationExtendsNull 的语法位近似：extends 表达式为 null 字面量
pub(crate) fn class_decl_extends_null(class_node: &Arc<Node>) -> bool {
    class_extends_heritage_element(class_node)
        .map(|e| expression_with_type_arguments_expression(&e).kind == SyntaxKind::NullKeyword)
        .unwrap_or(false)
}

// Go findFirstSuperCall：不进入嵌套函数体
fn find_first_super_call(node: &Arc<Node>) -> Option<Arc<Node>> {
    if is_super_call(node) {
        return Some(Arc::clone(node));
    }
    if is_function_like(node) || node.kind == SyntaxKind::ClassStaticBlockDeclaration {
        return None;
    }
    let mut found: Option<Arc<Node>> = None;
    for_each_child(node, |child| {
        if found.is_none() {
            found = find_first_super_call(child);
        }
        found.is_some()
    });
    found
}

// Go isPrivateIdentifierClassElementDeclaration
fn is_private_identifier_class_element_declaration(member: &Arc<Node>) -> bool {
    member
        .name()
        .is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier)
        && matches!(
            member.kind,
            SyntaxKind::PropertyDeclaration
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
        )
}

fn class_members(class_decl: &Arc<Node>) -> Arc<NodeList> {
    match &class_decl.data {
        NodeData::ClassDeclaration(d) => Arc::clone(&d.members),
        NodeData::ClassExpression(d) => Arc::clone(&d.members),
        _ => Arc::new(NodeList::new(vec![])),
    }
}

fn ctor_parameters(ctor: &Arc<Node>) -> Arc<NodeList> {
    match &ctor.data {
        NodeData::ConstructorDeclaration(d) => Arc::clone(&d.parameters),
        _ => Arc::new(NodeList::new(vec![])),
    }
}

fn block_statements(body: &Arc<Node>) -> Arc<NodeList> {
    match &body.data {
        NodeData::Block(b) => Arc::clone(&b.statements),
        _ => Arc::new(NodeList::new(vec![])),
    }
}

// Go superCallIsRootLevelInConstructor：super 调用经括号上溯后须为
// 构造器 body 的顶层表达式语句
fn walk_up_parenthesized_expressions(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while current.kind == SyntaxKind::ParenthesizedExpression {
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }
    current
}

// Go nodeImmediatelyReferencesSuperOrThis
fn node_immediately_references_super_or_this(node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::SuperKeyword | SyntaxKind::ThisKeyword => true,
        SyntaxKind::ArrowFunction
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::PropertyDeclaration => false,
        SyntaxKind::Block => {
            if let Some(parent) = node.parent()
                && matches!(
                    parent.kind,
                    SyntaxKind::Constructor
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                )
            {
                return false;
            }
            node_immediately_references_children(node)
        }
        _ => node_immediately_references_children(node),
    }
}

fn node_immediately_references_children(node: &Arc<Node>) -> bool {
    let mut found = false;
    for_each_child(node, |child| {
        if !found {
            found = node_immediately_references_super_or_this(child);
        }
        found
    });
    found
}

// Go ast.SkipOuterExpressions（OEKAll 等价集合）
fn skip_outer_expressions(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    loop {
        let next = match &current.data {
            NodeData::ParenthesizedExpression(d) => Arc::clone(&d.expression),
            NodeData::AsExpression(d) => Arc::clone(&d.expression),
            NodeData::TypeAssertion(d) => Arc::clone(&d.expression),
            NodeData::NonNullExpression(d) => Arc::clone(&d.expression),
            NodeData::SatisfiesExpression(d) => Arc::clone(&d.expression),
            _ => return current,
        };
        current = next;
    }
}
