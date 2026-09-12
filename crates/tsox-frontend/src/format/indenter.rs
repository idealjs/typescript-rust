//! Go format/indent.go 中缩进谓词（ShouldIndentChildNode /
//! NodeWillIndentChild 等）与 dynamicIndenter 的移植。

use std::cell::RefCell;
use std::sync::Arc;

use tsox_core::core::text::TextRange;

use crate::ast::node::Node;
use crate::ast::SyntaxKind;
use crate::ast::SourceFile;
use crate::format::FormatCodeSettings;

use super::util;

pub(crate) fn range_is_on_one_line(range: TextRange, file: &SourceFile) -> bool {
    util::line_of_position(file, range.pos()) == util::line_of_position(file, range.end())
}

pub(crate) fn is_control_flow_ending_statement(kind: SyntaxKind, parent_kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::ReturnStatement
        | SyntaxKind::ThrowStatement
        | SyntaxKind::ContinueStatement
        | SyntaxKind::BreakStatement => parent_kind != SyntaxKind::Block,
        _ => false,
    }
}

/// Go ShouldIndentChildNode
pub(crate) fn should_indent_child_node(
    settings: &FormatCodeSettings,
    parent: &Arc<Node>,
    child: Option<&Arc<Node>>,
    file: Option<&SourceFile>,
    is_next_child: bool,
) -> bool {
    node_will_indent_child(settings, parent, child, file, false)
        && !(is_next_child
            && child.is_some_and(|c| is_control_flow_ending_statement(c.kind, parent.kind)))
}

/// Go NodeWillIndentChild：parent 是否按显式规则缩进 child。
pub(crate) fn node_will_indent_child(
    settings: &FormatCodeSettings,
    parent: &Arc<Node>,
    child: Option<&Arc<Node>>,
    file: Option<&SourceFile>,
    indent_by_default: bool,
) -> bool {
    use SyntaxKind::*;
    let child_kind = child.map(|c| c.kind).unwrap_or(Unknown);
    match parent.kind {
        ExpressionStatement
        | ClassDeclaration
        | ClassExpression
        | InterfaceDeclaration
        | EnumDeclaration
        | TypeAliasDeclaration
        | ArrayLiteralExpression
        | Block
        | ModuleBlock
        | ObjectLiteralExpression
        | TypeLiteral
        | MappedType
        | TupleType
        | ParenthesizedExpression
        | PropertyAccessExpression
        | CallExpression
        | NewExpression
        | VariableStatement
        | ExportAssignment
        | ReturnStatement
        | ConditionalExpression
        | ArrayBindingPattern
        | ObjectBindingPattern
        | JsxOpeningElement
        | JsxOpeningFragment
        | JsxSelfClosingElement
        | JsxExpression
        | MethodSignature
        | CallSignature
        | ConstructSignature
        | Parameter
        | FunctionType
        | ConstructorType
        | ParenthesizedType
        | TaggedTemplateExpression
        | AwaitExpression
        | NamedExports
        | NamedImports
        | ExportSpecifier
        | ImportSpecifier
        | PropertyDeclaration
        | CaseClause
        | DefaultClause => true,
        CaseBlock => settings.indent_switch_case.is_true_or_unknown(),
        VariableDeclaration | PropertyAssignment | BinaryExpression => {
            if settings
                .indent_multi_line_object_literal_beginning_on_blank_line
                .is_false_or_unknown()
                && file.is_some()
                && child_kind == ObjectLiteralExpression
            {
                let Some(child) = child else {
                    return range_is_on_one_line(parent.loc, file.unwrap());
                };
                return range_is_on_one_line(child.loc, file.unwrap());
            }
            if parent.kind == BinaryExpression && child_kind == JsxElement && file.is_some() {
                let file = file.unwrap();
                let parent_start_line = util::line_of_position(
                    file,
                    crate::scanner::skip_trivia(&file.text, parent.pos()),
                );
                let child_start_line = child
                    .map(|c| {
                        util::line_of_position(
                            file,
                            crate::scanner::skip_trivia(&file.text, c.pos()),
                        )
                    })
                    .unwrap_or_default();
                return parent_start_line != child_start_line;
            }
            if parent.kind != BinaryExpression {
                return true;
            }
            indent_by_default
        }
        DoStatement | WhileStatement | ForInStatement | ForOfStatement | ForStatement
        | IfStatement | FunctionDeclaration | FunctionExpression | MethodDeclaration
        | Constructor | GetAccessor | SetAccessor => child_kind != Block,
        ArrowFunction => {
            if child_kind == ParenthesizedExpression {
                if let (Some(file), Some(c)) = (file, child) {
                    return range_is_on_one_line(c.loc, file);
                }
            }
            child_kind != Block
        }
        ExportDeclaration => child_kind != NamedExports,
        ImportDeclaration => {
            child_kind != ImportClause
                || child.is_some_and(|c| {
                    matches!(&c.data, crate::ast::node_data_generated::NodeData::ImportClause(d)
                        if d.named_bindings.as_ref().is_some_and(|b| b.kind != SyntaxKind::NamedImports))
                })
        }
        JsxElement => child_kind != JsxClosingElement,
        JsxFragment => child_kind != JsxClosingFragment,
        IntersectionType | UnionType | SatisfiesExpression => {
            !matches!(child_kind, TypeLiteral | TupleType | MappedType) && indent_by_default
        }
        TryStatement => {
            if child_kind == Block {
                return false;
            }
            indent_by_default
        }
        _ => indent_by_default,
    }
}

/// Go childIsUnindentedBranchOfConditionalExpression
pub(crate) fn child_is_unindented_branch_of_conditional_expression(
    parent: &Arc<Node>,
    child: &Arc<Node>,
    child_start_line: usize,
    file: &SourceFile,
) -> bool {
    if parent.kind != SyntaxKind::ConditionalExpression {
        return false;
    }
    let crate::ast::node_data_generated::NodeData::ConditionalExpression(d) = &parent.data else {
        return false;
    };
    let condition_end_line = util::line_of_position(file, d.condition.end());
    let same = |a: &Arc<Node>| Arc::ptr_eq(a, child);
    if same(&d.when_true) {
        return child_start_line == condition_end_line;
    }
    if same(&d.when_false) {
        let true_start_line = util::line_of_position(file, d.when_true.pos());
        let true_end_line = util::line_of_position(file, d.when_true.end());
        return condition_end_line == true_start_line && true_end_line == child_start_line;
    }
    false
}

/// Go argumentStartsOnSameLineAsPreviousArgument
pub(crate) fn argument_starts_on_same_line_as_previous_argument(
    parent: &Arc<Node>,
    child: &Arc<Node>,
    child_start_line: usize,
    file: &SourceFile,
) -> bool {
    if parent.kind != SyntaxKind::CallExpression && parent.kind != SyntaxKind::NewExpression {
        return false;
    }
    let mut arguments: Option<Arc<crate::ast::node::NodeList>> = None;
    match &parent.data {
        crate::ast::node_data_generated::NodeData::CallExpression(d) => {
            arguments = Some(d.arguments.clone())
        }
        crate::ast::node_data_generated::NodeData::NewExpression(d) => {
            arguments = d.arguments.clone()
        }
        _ => {}
    }
    let Some(args) = arguments else { return false };
    let mut index: Option<usize> = None;
    for (i, a) in args.nodes.iter().enumerate() {
        if Arc::ptr_eq(a, child) {
            index = Some(i);
            break;
        }
    }
    let Some(i) = index else { return false };
    if i == 0 {
        return false;
    }
    let previous = &args.nodes[i - 1];
    util::line_of_position(file, previous.end()) == child_start_line
}

/// Go dynamicIndenter：一个缩进作用域（node + 起始行 + indentation/delta）。
/// 作用域在兄弟之间共享（recomputeIndentation 原地生效），用 RefCell 共享。
pub(crate) type IndenterRef = Arc<RefCell<DynamicIndenter>>;

pub(crate) struct DynamicIndenter {
    pub(crate) node: Arc<Node>,
    pub(crate) node_start_line: usize,
    pub(crate) indentation: i64,
    pub(crate) delta: i64,
    pub(crate) options: FormatCodeSettings,
    pub(crate) file: Arc<SourceFile>,
}

impl DynamicIndenter {
    pub(crate) fn new_ref(
        node: Arc<Node>,
        node_start_line: usize,
        indentation: i64,
        delta: i64,
        options: FormatCodeSettings,
        file: Arc<SourceFile>,
    ) -> IndenterRef {
        Arc::new(RefCell::new(DynamicIndenter {
            node,
            node_start_line,
            indentation,
            delta,
            options,
            file,
        }))
    }

    /// Go getIndentationForComment
    pub(crate) fn get_indentation_for_comment(
        &self,
        kind: SyntaxKind,
        token_indentation: i64,
        container: &Arc<Node>,
    ) -> i64 {
        match kind {
            SyntaxKind::CloseBraceToken | SyntaxKind::CloseBracketToken
            | SyntaxKind::CloseParenToken => {
                return self.indentation + self.get_delta(Some(container));
            }
            _ => {}
        }
        if token_indentation != -1 {
            return token_indentation;
        }
        self.indentation
    }

    /// Go getIndentationForToken
    pub(crate) fn get_indentation_for_token(
        &self,
        line: usize,
        kind: SyntaxKind,
        container: &Arc<Node>,
        suppress_delta: bool,
    ) -> i64 {
        if !suppress_delta && self.should_add_delta(line, kind, container) {
            return self.indentation + self.get_delta(Some(container));
        }
        self.indentation
    }

    pub(crate) fn get_indentation(&self) -> i64 {
        self.indentation
    }

    /// Go getDelta
    pub(crate) fn get_delta(&self, child: Option<&Arc<Node>>) -> i64 {
        if node_will_indent_child(&self.options, &self.node, child, Some(&self.file), true) {
            return self.delta;
        }
        0
    }

    /// Go recomputeIndentation
    pub(crate) fn recompute_indentation(&mut self, line_added: bool, parent: &Arc<Node>) {
        if should_indent_child_node(&self.options, parent, Some(&self.node), Some(&self.file), false) {
            if line_added {
                self.indentation += self.options.editor_settings.indent_size as i64;
            } else {
                self.indentation -= self.options.editor_settings.indent_size as i64;
            }
            if should_indent_child_node(&self.options, &self.node, None, Some(&self.file), false) {
                self.delta = self.options.editor_settings.indent_size as i64;
            } else {
                self.delta = 0;
            }
        }
    }

    /// Go shouldAddDelta
    fn should_add_delta(&self, line: usize, kind: SyntaxKind, container: &Arc<Node>) -> bool {
        use SyntaxKind::*;
        match kind {
            OpenBraceToken | CloseBraceToken | CloseParenToken | ElseKeyword | WhileKeyword
            | AtToken => return false,
            SlashToken | GreaterThanToken => match container.kind {
                JsxOpeningElement | JsxClosingElement | JsxSelfClosingElement => return false,
                _ => {}
            },
            OpenBracketToken | CloseBracketToken => {
                if container.kind != MappedType {
                    return false;
                }
            }
            _ => {}
        }
        self.node_start_line != line
            && !(has_decorators(&self.node)
                && Some(kind) == first_non_decorator_token_of_node(&self.node))
    }
}

pub(crate) fn has_decorators(node: &Node) -> bool {
    node.modifiers()
        .is_some_and(|m| m.flags().contains(crate::ast::ModifierFlags::Decorator))
}

/// Go getFirstNonDecoratorTokenOfNode
pub(crate) fn first_non_decorator_token_of_node(node: &Arc<Node>) -> Option<SyntaxKind> {
    use SyntaxKind::*;
    if let Some(mods) = node.modifiers() {
        let nodes = &mods.list.nodes;
        let first_decorator = nodes.iter().position(|n| n.kind == Decorator);
        if let Some(idx) = first_decorator {
            for n in nodes.iter().skip(idx) {
                if is_modifier_kind(n.kind) {
                    return Some(n.kind);
                }
            }
        }
    }
    match node.kind {
        ClassDeclaration => Some(ClassKeyword),
        InterfaceDeclaration => Some(InterfaceKeyword),
        FunctionDeclaration => Some(FunctionKeyword),
        // Go 原样返回 KindEnumDeclaration（与 KindEnumKeyword 不同），保持一致
        EnumDeclaration => Some(EnumDeclaration),
        GetAccessor => Some(GetKeyword),
        SetAccessor => Some(SetKeyword),
        MethodDeclaration => {
            let crate::ast::node_data_generated::NodeData::MethodDeclaration(d) = &node.data
            else {
                return None;
            };
            if let Some(ast) = &d.asterisk_token {
                return Some(ast.kind);
            }
            node.name().map(|n| n.kind)
        }
        PropertyDeclaration | Parameter => node.name().map(|n| n.kind),
        _ => None,
    }
}

fn is_modifier_kind(kind: SyntaxKind) -> bool {
    crate::ast::node_data_generated::is_modifier_kind(kind)
}

/// Go childStartsOnTheSameLineWithElseInIfStatement
pub(crate) fn child_starts_on_same_line_with_else_in_if_statement(
    parent: &Arc<Node>,
    child: &Arc<Node>,
    child_start_line: usize,
    file: &SourceFile,
) -> bool {
    if parent.kind != SyntaxKind::IfStatement {
        return false;
    }
    let crate::ast::node_data_generated::NodeData::IfStatement(d) = &parent.data else {
        return false;
    };
    let Some(else_statement) = &d.else_statement else {
        return false;
    };
    if !Arc::ptr_eq(else_statement, child) {
        return false;
    }
    let Some(else_keyword) = crate::astnav::find_preceding_token(&file.node, child.pos()) else {
        return false;
    };
    let else_start_line =
        util::line_of_position(file, util::token_pos_of_node(file, &else_keyword));
    else_start_line == child_start_line
}
