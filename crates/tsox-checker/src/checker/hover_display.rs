//! Go ls/hover.go getQuickInfoAndDeclarationAtLocation 的结构化移植：
//! 符号驱动的 display parts 构建（非 VS 纯文本模式），替代 nodebuilder_checker_12 的补丁式路径。

use std::sync::Arc;

use super::hover_display_context::{is_in_expression_context, is_this_in_type_query};
use super::hover_display_target::is_assignment_target_literal;
use super::hover_display_parts::HoverPartsBuilder;
use crate::checker::nodebuilder::*;
use tsox_frontend::ast::{
    get_module_instance_state, is_ambient_module, ModuleInstanceState, Node, Symbol, SymbolFlags,
    SyntaxKind,
};

/// Go SemanticMeaning：Value=1 Type=2 Namespace=4
pub(crate) const MEANING_VALUE: u8 = 1;
pub(crate) const MEANING_TYPE: u8 = 2;
pub(crate) const MEANING_NAMESPACE: u8 = 4;
pub(crate) const MEANING_ALL: u8 = 7;

impl Checker {
    /// hover 入口：等价 Go ProvideHover 中段（节点定位由 tsox-lsp 完成）
    pub fn quick_info_display_for_node(&mut self, node: &Arc<Node>) -> Vec<SymbolDisplayPart> {
                // 限定名链（namespace_qualifier_of 等）依赖显示文件/节点上下文
        self.display_enclosing_file = self.get_source_file_of_node(node);
        self.display_enclosing_node = Some(Arc::clone(node));
        let container = get_container_node(node);
        let mut b = HoverPartsBuilder::new();

        // with 块内无法回答语义问题（tsc getTypeOfNode 语义）
        if crate::checker::nodebuilder_checker_12::node_in_with_block(node) {
            b.write_keyword("any");
            return b.parts;
        }

        // this 分支：表达式位的 this / 类型查询中的 ThisType（Go 首个分支）
        if (node.kind == SyntaxKind::ThisKeyword && is_in_expression_context(node))
            || is_this_in_type_query(node)
        {
            b.write_keyword("this");
            b.write_punctuation(": ");
            let t = self.get_type_of_node(node);
            b.extend(self.type_to_display_parts(&t));
            return b.parts;
        }

        // `as const` 的 const：我们的 AST 是 ConstKeyword 直挂 AsExpression
        if let Some(parts) = self.const_assertion_parts(node) {
            return parts;
        }

        // JSX tag 名/属性名：旧管线分支（JSX.IntrinsicElements 等）
        if self.is_jsx_tag_name(node) {
            let name_text = node
                .jsx_namespaced_name_text()
                .unwrap_or_else(|| node.text().to_string());
            let is_upper = !name_text.is_empty()
                && name_text.chars().next().is_some_and(|c| c.is_ascii_uppercase());
            if is_upper
                && let Some(sym) = self.resolve_symbol_for_hover(node)
                && sym.flags.intersects(SymbolFlags::Class | SymbolFlags::Function)
            {
                let mut parts = self.symbol_to_display_parts(&sym, SymbolFlags::all(), &[]);
                let doc = self.symbol_documentation(&sym);
                if !doc.is_empty() {
                    push_space(&mut parts, "\n\n");
                    push_part(&mut parts, &doc, DisplayPartKind::Text);
                }
                return parts;
            }
            if let Some(parts) = self.jsx_intrinsic_element_parts(&name_text) {
                return parts;
            }
            let mut parts = Vec::new();
            push_part(&mut parts, "any", DisplayPartKind::Keyword);
            return parts;
        }
        if let Some(parts) = self.jsx_attribute_parts(node) {
            return parts;
        }

        // 泛型 owner 成员访问（C<number>.m 等）：与旧管线一致，先于符号解析
        let member_parts = self.instantiated_member_access_parts(node);

        let symbol = self.get_symbol_at_location_for_quick_info(node);
        let Some(symbol) = symbol else {
            // 属性访问无具名成员但接收者带字符串索引签名：显示索引值类型（无前缀）
            if let Some(parts) = self.index_signature_property_parts(node) {
                return parts;
            }
            if should_get_type(node) {
                let t = self.get_type_of_node(node);
                b.extend(self.type_to_display_parts(&t));
            }
            return b.parts;
        };

        // 改名绑定的 property_name：源属性身份
        if node.parent().as_ref().is_some_and(|p| p.kind == SyntaxKind::BindingElement)
            && let Some(parts) =
                self.binding_element_property_name_parts(node, &symbol)
        {
            return parts;
        }
        if let Some(mut parts) = member_parts {
            let doc = self.hover_documentation_for_symbol(&symbol, node, None);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
            b.extend(parts);
            return b.parts;
        }

        let meaning = get_meaning_from_location(node);
        self.hover_write_symbol(&mut b, &symbol, node, &container, meaning);
        // 文档链（Go getDocumentationForSymbol），附加在 quickinfo 之后
        let doc = self.hover_documentation_for_symbol(&symbol, node, b.declaration.as_ref());
        if !doc.is_empty() && !b.parts.is_empty() {
            b.write_space("\n\n");
            b.write_text(&doc, DisplayPartKind::Text);
        }
        b.parts
    }

    /// 符号定位（Go getSymbolAtLocationForQuickInfo + 本地解析回退）
    fn get_symbol_at_location_for_quick_info(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        // 类型位限定名段（AMap.MassMarks.Data 的 Data）：解析整条限定链
        // （限定链可能是 QualifiedName 或 heritage 位的 PropertyAccessExpression）
        let in_type_chain = node.parent().as_ref().is_some_and(|p| {
            if p.kind == SyntaxKind::QualifiedName {
                return true;
            }
            if p.kind == SyntaxKind::PropertyAccessExpression {
                let mut cur = Some(Arc::clone(p));
                while let Some(n) = cur {
                    if matches!(
                        n.kind,
                        SyntaxKind::HeritageClause | SyntaxKind::ExpressionWithTypeArguments
                    ) {
                        return true;
                    }
                    cur = n.parent();
                }
            }
            false
        });
        if in_type_chain {
            let mut root = Arc::clone(node);
            while let Some(parent) = root.parent().as_ref() {
                if matches!(
                    parent.kind,
                    SyntaxKind::QualifiedName | SyntaxKind::PropertyAccessExpression
                ) {
                    root = Arc::clone(parent);
                } else {
                    break;
                }
            }
                        if root.parent().as_ref().is_some_and(|p| {
                matches!(
                    p.kind,
                    SyntaxKind::TypeReference
                        | SyntaxKind::HeritageClause
                        | SyntaxKind::ExpressionWithTypeArguments
                        | SyntaxKind::TypeAliasDeclaration
                )
            }) {
                if let Ok(sym) = self.resolve_qualified_symbol_traced(&root) {
                    return Some(sym);
                }
            }
        }
        // Go getSymbolAtLocation：模块说明符字符串 → 目标模块符号
        if node.kind == SyntaxKind::StringLiteral {
            if let Some(sym) = self.module_symbol_of_specifier(node) {
                return Some(sym);
            }
        }
        // 类型位的 this（ThisType 节点）：容器类/接口符号；this 参数名是
        // ThisKeyword，走下方参数符号解析
        if node.kind == SyntaxKind::ThisType {
            let mut cur = node.parent();
            while let Some(n) = cur {
                if matches!(
                    n.kind,
                    SyntaxKind::ClassDeclaration
                        | SyntaxKind::ClassExpression
                        | SyntaxKind::InterfaceDeclaration
                ) {
                    if let Some(sym) = self.program.symbol_map().symbol_of(&n).cloned() {
                        return Some(sym);
                    }
                    break;
                }
                cur = n.parent();
            }
        }
        if let Some(sym) = self.resolve_contextual_property_symbol(node) {
            return Some(sym);
        }
        if let Some(sym) = self.resolve_property_access_symbol(node) {
            return Some(sym);
        }
        // 属性访问无具名成员但接收者带字符串索引签名：放弃标识符回退，
        // 由调用方显示索引值类型
        if self.receiver_string_index_type(node).is_some() {
            return None;
        }
        // 纯 shorthand 成员（{name1}）：属性身份优先于外层同名变量；
        // 解构赋值目标形态（parser 重建丢失字段）需位置判定排除
        if node.parent().as_ref().is_some_and(|p| p.kind == SyntaxKind::ShorthandPropertyAssignment)
            && let Some(parent) = node.parent().as_ref()
            && !matches!(
                &parent.data,
                tsox_frontend::ast::NodeData::ShorthandPropertyAssignment(sd)
                    if sd.object_assignment_initializer.is_some() || sd.equals_token.is_some()
            )
            && !is_assignment_target_literal(parent)
            && let Some(sym) = self.program.symbol_map().symbol_of(parent).cloned()
        {
            return Some(sym);
        }
        self.resolve_symbol_for_hover(node)
    }

    /// Go writeSymbol：按 meaning 选意义位，递归 alias 链，逐类别分支
    fn hover_write_symbol(
        &mut self,
        b: &mut HoverPartsBuilder,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
        container: &Option<Arc<Node>>,
        meaning: u8,
    ) {
        // alias 链：以目标符号整体呈现（替换语义）；目标即自身（合并符号的
        // export 别名回查 members 命中本地声明）时落到主体书写
        if symbol.flags.intersects(SymbolFlags::Alias) {
            let key = symbol.id();
            if b.visit_alias(key)
                && let Some(aliased) = self.follow_alias_resolving(symbol)
                && !Arc::ptr_eq(&aliased, symbol)
            {
                b.alias_level += 1;
                self.hover_write_symbol(b, &aliased, node, container, meaning);
                b.alias_level -= 1;
                return;
            }
        }

        let mut flags = select_flags_by_meaning(&symbol.flags, meaning);
        if flags.is_empty() {
            if b.alias_level != 0 || !b.is_empty() {
                return;
            }
            flags = symbol.flags
                & (SymbolFlags::VALUE
                    | SymbolFlags::Signature
                    | SymbolFlags::TYPE
                    | SymbolFlags::NAMESPACE);
            if flags.is_empty() {
                return;
            }
        }
        // 声明为方法的 property 按 method 显示（Go: flags = SymbolFlagsMethod）
        if flags.intersects(SymbolFlags::Property)
            && (symbol
                .value_declaration
                .as_ref()
                .is_some_and(|d| d.kind == SyntaxKind::MethodDeclaration)
                || symbol.declarations.iter().any(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature
                    )
                }))
        {
            flags = SymbolFlags::Method;
        }

        self.hover_write_symbol_kind(b, symbol, node, &flags, container);
        if b.declaration.is_none() {
            b.declaration = symbol
                .value_declaration
                .clone()
                .or_else(|| symbol.declarations.first().cloned());
        }
    }
}

/// Go testNode 后的 flag 选择：按位置的语义意义过滤符号意义位
pub(crate) fn select_flags_by_meaning(flags: &SymbolFlags, meaning: u8) -> SymbolFlags {
    match meaning {
        MEANING_VALUE => *flags & (SymbolFlags::VALUE | SymbolFlags::Signature),
        MEANING_TYPE => *flags & SymbolFlags::TYPE,
        MEANING_NAMESPACE => *flags & SymbolFlags::NAMESPACE,
        _ => *flags & (SymbolFlags::VALUE | SymbolFlags::Signature | SymbolFlags::TYPE | SymbolFlags::NAMESPACE),
    }
}

/// Go ls/utilities.go getContainerNode
pub(crate) fn get_container_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut cur = node.parent();
    while let Some(n) = cur {
        if matches!(
            n.kind,
            SyntaxKind::SourceFile
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::MethodSignature
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::ClassDeclaration
                | SyntaxKind::InterfaceDeclaration
                | SyntaxKind::EnumDeclaration
                | SyntaxKind::ModuleDeclaration
        ) {
            return Some(n);
        }
        cur = n.parent();
    }
    None
}

/// Go hover.go shouldGetType：无符号时按类型显示的节点范围
pub(crate) fn should_get_type(node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::Identifier => {
            !node_in_jsdoc(node)
                && !node
                    .parent()
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::TypeReference && p.text() == "const")
        }
        SyntaxKind::ThisKeyword
        | SyntaxKind::ThisType
        | SyntaxKind::SuperKeyword
        | SyntaxKind::NamedTupleMember => true,
        SyntaxKind::MetaProperty => {
            matches!(&node.data, tsox_frontend::ast::NodeData::MetaProperty(mp) if mp.keyword_token == SyntaxKind::ImportKeyword)
        }
        _ => false,
    }
}

fn node_in_jsdoc(node: &Arc<Node>) -> bool {
    let mut cur = node.parent();
    while let Some(n) = cur {
        if matches!(
            n.kind,
            SyntaxKind::JSDoc
                | SyntaxKind::JSDocTypeTag
                | SyntaxKind::JSDocParameterTag
                | SyntaxKind::JSDocPropertyTag
        ) {
            return true;
        }
        cur = n.parent();
    }
    false
}


/// Go ls/utilities.go getMeaningFromLocation（精简：覆盖声明名/类型引用/默认值）
pub(crate) fn get_meaning_from_location(node: &Arc<Node>) -> u8 {
    let Some(parent) = node.parent() else {
        return MEANING_VALUE;
    };
    use SyntaxKind::*;
    if matches!(
        parent.kind,
        ExportAssignment | ExportSpecifier | ExternalModuleReference | ImportSpecifier | ImportClause
    ) {
        return MEANING_ALL;
    }
    // 声明名字：意义由声明类别决定
    if crate::checker::nodebuilder_checker_12::is_declaration_name(&parent, node) {
        return match parent.kind {
            TypeParameter | InterfaceDeclaration | TypeAliasDeclaration | TypeLiteral => MEANING_TYPE,
            EnumMember | ClassDeclaration => MEANING_VALUE | MEANING_TYPE,
            ModuleDeclaration => {
                // Go getMeaningFromDeclaration：ambient/实例化模块带 Value，仅类型 namespace 只 Namespace
                let instantiated = is_ambient_module(&parent)
                    || get_module_instance_state(&parent) == ModuleInstanceState::Instantiated;
                if instantiated {
                    MEANING_NAMESPACE | MEANING_VALUE
                } else {
                    MEANING_NAMESPACE
                }
            }
            EnumDeclaration => MEANING_VALUE | MEANING_TYPE,
            _ => MEANING_VALUE,
        };
    }
    if parent.kind == TypeParameter {
        return MEANING_TYPE;
    }
    if parent.kind == LiteralType {
        return MEANING_TYPE | MEANING_VALUE;
    }
    // 类型引用位（TypeReference 的 typeName / QualifiedName 左右段）
    if matches!(parent.kind, TypeReference | QualifiedName) || is_part_of_type_reference(node) {
        return MEANING_TYPE;
    }
    MEANING_VALUE
}

fn is_part_of_type_reference(node: &Arc<Node>) -> bool {
    let mut cur = node.parent();
    while let Some(n) = cur {
        match n.kind {
            SyntaxKind::TypeReference | SyntaxKind::QualifiedName => cur = n.parent(),
            _ => return false,
        }
    }
    false
}

