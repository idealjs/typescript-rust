//! Go hover.go writeSymbol 的类别分支移植：
//! 变量/属性/枚举成员/函数/方法/类/接口/枚举/命名空间/类型参数/类型别名/签名成员。

use std::sync::Arc;

use super::hover_display_parts::HoverPartsBuilder;
use crate::checker::nodebuilder::*;
use crate::checker::types::Type;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SyntaxKind};

impl Checker {
    pub(crate) fn hover_write_symbol_kind(
        &mut self,
        b: &mut HoverPartsBuilder,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
        flags: &SymbolFlags,
        container: &Option<Arc<Node>>,
    ) {
        if flags.intersects(SymbolFlags::VARIABLE | SymbolFlags::Property | SymbolFlags::ACCESSOR) {
            // 调用位/new 位/签名 union 的特殊形态（迁移自旧管线）
            if let Some(parts) = self.hover_variable_call_site_parts(symbol, node) {
                b.extend(parts);
                return;
            }
            // Go hover.go：变量/属性在调用位显示解析签名的箭头形态
            if let Some(call) = self.hover_call_or_new_expression(node)
                && call.kind == SyntaxKind::CallExpression
                && let Some(sig) = self
                    .resolved_call_signature(&call)
                    .or_else(|| self.first_call_signature_of_symbol(symbol))
            {
                b.write_new_line();
                b.extend(self.variable_prefix_and_name_parts(symbol));
                let arrow = self.arrow_signature_parts(&sig);
                b.extend(arrow);
                return;
            }
            // Go GetTypeOfSymbolAtLocation：可选链最外层访问把 optionalType
            // marker 转成真 undefined（foo?.bar.baz 的 baz 显示 string | undefined）
            if flags.intersects(SymbolFlags::Property)
                && self.is_outermost_optional_chain_access(node)
            {
                b.write_new_line();
                b.extend(self.variable_prefix_and_name_parts(symbol));
                let t = self.get_type_of_symbol(symbol);
                let with_undef = self.add_optional_chain_undefined(&t);
                b.extend(self.type_to_display_parts(&with_undef));
                return;
            }
            self.hover_write_variable(b, symbol, container);
            return;
        }
        if flags.intersects(SymbolFlags::EnumMember) {
            self.hover_write_enum_member(b, symbol);
            return;
        }
        if flags.intersects(SymbolFlags::Function | SymbolFlags::Method) {
            self.hover_write_function(b, symbol, node);
            return;
        }
        if flags.intersects(SymbolFlags::Class | SymbolFlags::Interface) {
            self.hover_write_class_or_interface(b, symbol, node);
            return;
        }
        if flags.intersects(SymbolFlags::RegularEnum | SymbolFlags::ConstEnum) {
            self.hover_write_enum(b, symbol);
            return;
        }
        if flags.intersects(SymbolFlags::NAMESPACE) {
            self.hover_write_module(b, symbol);
            return;
        }
        if flags.intersects(SymbolFlags::TypeParameter) {
            self.hover_write_type_parameter(b, symbol);
            return;
        }
        if flags.intersects(SymbolFlags::TypeAlias) {
            self.hover_write_type_alias(b, symbol);
            return;
        }
        if flags.intersects(SymbolFlags::Signature) {
            b.write_new_line();
            let t = self.get_type_of_symbol(symbol);
            b.extend(self.type_to_display_parts(&t));
        }
    }

    /// 容器感知的限定名：声明在 hover 容器内时只写裸名（Go symbolToString enclosing 语义）
    pub(crate) fn hover_symbol_name(
        &mut self,
        symbol: &Arc<Symbol>,
        container: Option<&Arc<Node>>,
    ) -> String {
        if let Some(c) = container
            && symbol.declarations.iter().any(|d| {
                let mut cur = Some(Arc::clone(d));
                while let Some(n) = cur {
                    if Arc::ptr_eq(&n, c) {
                        return true;
                    }
                    cur = n.parent.clone();
                }
                false
            })
        {
            return symbol.name.clone();
        }
        self.qualified_symbol_name(symbol)
    }

    fn hover_write_variable(&mut self, b: &mut HoverPartsBuilder, symbol: &Arc<Symbol>, container: &Option<Arc<Node>>) {
        b.write_new_line();
        if symbol
            .check_flags
            .intersects(tsox_frontend::ast::CheckFlags::IndexSymbol)
        {
            b.write_text(&self.qualified_symbol_name(symbol), DisplayPartKind::Text);
            b.write_punctuation(": ");
            return;
        }
        // variable_prefix_and_name_parts：前缀 + 名字 + ? + ": "
        let prefix = self.variable_prefix_and_name_parts(symbol);
        b.extend(prefix);
        // shorthand 成员类型 = 引用变量类型的 widen
        let mut t = shorthand_widen_type(self, symbol).unwrap_or_else(|| self.get_type_of_symbol(symbol));
        // Go hover.go:741：类型是带约束的类型参数 → 显示 "T extends 约束"
        //（TypeParameterToDeclaration 渲染）
        if let TypeData::TypeParameter(tp) = &t.data
            && !tp.is_this_type
            && let Some(constraint) = self.get_constraint_of_type_parameter(&t)
        {
            let name = t
                .symbol
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "T".to_string());
            let rendered = format!("{name} extends {}", self.type_to_string(&constraint));
            b.write_text(&rendered, DisplayPartKind::Text);
            return;
        }
        // 对象字面量属性在拓宽位（无注解函数返回等）显示拓宽类型
        // （tsc getTypeOfSymbolAtLocation 经字面量拓宽上下文取型）
        if symbol.flags.contains(SymbolFlags::Property)
            && symbol
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::PropertyAssignment)
            && t.flags.intersects(crate::checker::types::TypeFlags::StringLiteral | crate::checker::types::TypeFlags::NumberLiteral | crate::checker::types::TypeFlags::BooleanLiteral)
        {
            t = self.get_widened_type(&t);
        }
                b.extend(self.type_to_display_parts(&t));
    }

    fn hover_write_enum_member(&mut self, b: &mut HoverPartsBuilder, symbol: &Arc<Symbol>) {
        b.write_new_line();
        b.write_punctuation("(");
        b.write_text("enum member", DisplayPartKind::Text);
        b.write_punctuation(") ");
        // 成员显示限定名（Demo.Emoji），字面量成员随后 " = value"
        b.write_text(&self.qualified_symbol_name(symbol), DisplayPartKind::EnumName);
        let t = self.get_type_of_symbol(symbol);
        if let Some(v) = t.literal_value() {
            b.write_space(" = ");
            b.write_text(&self.literal_value_to_string(&v), DisplayPartKind::Text);
        } else {
            b.write_punctuation(": ");
            b.extend(self.type_to_display_parts(&t));
        }
    }

    /// new 表达式变量与签名 union 调用位的箭头形态（旧管线行为平移）
    fn hover_variable_call_site_parts(
        &mut self,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
    ) -> Option<Vec<SymbolDisplayPart>> {
        if let Some(call) = Checker::enclosing_call_or_new(node)
            && call.kind == SyntaxKind::NewExpression
            && let Some(sig) = self.resolved_call_signature(&call)
            && sig.declaration.as_ref().is_some_and(|d| {
                matches!(
                    d.kind,
                    SyntaxKind::ConstructSignature
                        | SyntaxKind::NewExpression
                        | SyntaxKind::CallSignature
                )
            })
        {
            let mut parts = self.variable_prefix_and_name_parts(symbol);
            push_keyword(&mut parts, "new ");
            parts.extend(self.arrow_signature_parts(&sig));
            let doc = self.hover_documentation(symbol, node);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
            return Some(parts);
        }
        // 调用位的泛型函数属性/变量：前缀 + 实例化箭头签名（旧管线分支平移）
        if symbol.flags.intersects(SymbolFlags::VARIABLE | SymbolFlags::Property)
            && let Some(call) = Checker::enclosing_call_or_new(node)
            && let Some(sig) = self.resolved_call_signature(&call)
            && !sig.type_parameters.is_empty()
        {
            let args: Vec<Arc<Node>> = match &call.data {
                tsox_frontend::ast::NodeData::CallExpression(d) => {
                    d.arguments.iter().cloned().collect()
                }
                tsox_frontend::ast::NodeData::NewExpression(d) => d
                    .arguments
                    .as_ref()
                    .map(|a| a.iter().cloned().collect())
                    .unwrap_or_default(),
                _ => Vec::new(),
            };
            let inferred = self.infer_call_type_arguments(&call, &sig, &args);
            let inst = self.get_signature_instantiation(&sig, &inferred);
            let mut parts = self.variable_prefix_and_name_parts(symbol);
            if !inferred.is_empty() {
                let names: Vec<String> = inferred.iter().map(|t| self.type_to_string(t)).collect();
                push_punctuation(&mut parts, "<");
                push_part(&mut parts, &names.join(", "), DisplayPartKind::Text);
                push_punctuation(&mut parts, ">");
            }
            if call.kind == SyntaxKind::NewExpression {
                push_keyword(&mut parts, "new ");
            }
            push_punctuation(&mut parts, "(");
            self.append_signature_parameter_parts(&mut parts, &inst);
            push_punctuation(&mut parts, ")");
            push_space(&mut parts, " => ");
            let ret = self
                .get_return_type_of_signature(&inst)
                .unwrap_or_else(|| self.any_type());
            parts.extend(self.type_to_display_parts(&ret));
            let doc = self.hover_documentation(symbol, node);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
            return Some(parts);
        }
        if let Some(sig) = self.call_site_union_signature(symbol, node) {
            let mut parts = self.variable_prefix_and_name_parts(symbol);
            parts.extend(self.arrow_signature_parts(&sig));
            let doc = self.hover_documentation(symbol, node);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
            return Some(parts);
        }
        None
    }

    fn hover_write_function(
        &mut self,
        b: &mut HoverPartsBuilder,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
    ) {
        let is_method = symbol.flags.intersects(SymbolFlags::Method)
            || symbol.declarations.iter().any(|d| {
                matches!(d.kind, SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature)
            });
        let prefix = if is_method { "method" } else { "function " };
        if node.kind == SyntaxKind::Identifier
            && let Some(parent) = node.parent.as_ref()
            && tsox_frontend::ast::is_function_like_kind(parent.kind)
            && tsox_frontend::ast::node_data_generated::node_name(parent)
                .is_some_and(|n| Arc::ptr_eq(&n, node))
            && symbol.declarations.iter().any(|d| Arc::ptr_eq(d, parent))
        {
            let t = self.get_type_of_function_like(parent);
            if let Some(sig) = t.as_structured().and_then(|s| s.call_signatures().first().cloned())
            {
                self.hover_write_signatures(b, &[sig], prefix, is_method, symbol, None);
                return;
            }
        }
        let (signatures, call) = self.hover_get_signatures_at_location(symbol, node);
        self.hover_write_signatures(b, &signatures, prefix, is_method, symbol, call.as_ref());
    }

    /// 交集类型变量调用位：取首个带调用签名的成分（Go 结构化成员合并的近似）
    fn first_call_signature_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Signature>> {
        let t = self.get_type_of_symbol(symbol);
        if !t.is_intersection() {
            return None;
        }
        for c in t.types()?.to_vec() {
            if let Some(sig) = self
                .get_signatures_of_type(&c, crate::checker::SignatureKind::Call)
                .into_iter()
                .next()
            {
                return Some(sig);
            }
        }
        None
    }

    fn hover_write_type_parameter(&mut self, b: &mut HoverPartsBuilder, symbol: &Arc<Symbol>) {
        b.write_new_line();
        b.write_punctuation("(");
        b.write_text("type parameter", DisplayPartKind::Text);
        b.write_punctuation(") ");
        let t = self.get_declared_type_of_symbol(symbol);
        b.write_text(&symbol.name, DisplayPartKind::TypeParameterName);
        if let Some(c) = self.get_constraint_of_type_parameter(&t) {
            b.write_keyword(" extends ");
            b.extend(self.type_to_display_parts(&c));
        }
        if let Some(parent) = &symbol.parent {
            if parent.flags.intersects(SymbolFlags::TypeAlias) {
                // 别名的类型参数：`in type X<T = string>`
                b.write_keyword(" in type ");
                b.write_text(&parent.name, DisplayPartKind::InterfaceName);
                if let Some(tps) = self.type_alias_type_param_decls(parent) {
                    self.hover_write_type_params_from_decls(b, &tps);
                }
                return;
            }
            b.write_keyword(" in ");
            b.write_text(&parent.name, DisplayPartKind::Text);
            let pt = self.get_declared_type_of_symbol(parent);
            let params: Vec<Arc<Type>> = pt
                .as_interface()
                .map(|it| it.local_type_parameters().to_vec())
                .unwrap_or_default();
            self.hover_write_type_params(b, &params);
            return;
        }
        let decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::TypeParameter)
            .cloned();
        if let Some(decl) = decl
            && let Some(host) = decl.parent.as_ref()
        {
            if tsox_frontend::ast::is_function_like(host) {
                b.write_keyword(" in ");
                if host.kind == SyntaxKind::ConstructSignature {
                    b.write_keyword("new ");
                } else if host.kind != SyntaxKind::CallSignature
                    && let Some(name) = tsox_frontend::ast::node_data_generated::node_name(host)
                {
                    let name_text = name.text().to_string();
                    b.write_text(&name_text, DisplayPartKind::Text);
                }
                let t = self.get_type_of_function_like(host);
                if let Some(sig) = t.as_structured().and_then(|s| s.call_signatures().first().cloned())
                {
                    b.write_space(" ");
                    self.hover_write_signature(b, &sig, None);
                }
            } else if host.kind == SyntaxKind::TypeAliasDeclaration
                && let Some(alias_sym) = self.program.symbol_map().symbol_of(host).cloned()
            {
                b.write_keyword(" in type ");
                let alias_name = alias_sym.name.clone();
                b.write_text(&alias_name, DisplayPartKind::InterfaceName);
                if let Some(tps) = self.type_alias_type_param_decls(&alias_sym) {
                    self.hover_write_type_params_from_decls(b, &tps);
                }
            }
        }
    }

    fn hover_write_type_alias(&mut self, b: &mut HoverPartsBuilder, symbol: &Arc<Symbol>) {
        b.write_new_line();
        b.write_keyword("type ");
        b.write_text(&symbol.name, DisplayPartKind::InterfaceName);
        if let Some(tps) = self.type_alias_type_param_decls(symbol) {
            self.hover_write_type_params_from_decls(b, &tps);
        }
        b.write_space(" = ");
        let t = self.try_get_type_alias_declared_type(symbol).unwrap_or_else(|| self.get_declared_type_of_symbol(symbol));
        b.extend(self.type_to_display_parts(&t));
    }

    pub(crate) fn hover_write_declared_type_params(&mut self, b: &mut HoverPartsBuilder, symbol: &Arc<Symbol>) {
        // 声明节点的类型参数直渲染（名字/extends/default），不依赖已构建的类型
        for decl in &symbol.declarations {
            let tps = match &decl.data {
                tsox_frontend::ast::NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
                tsox_frontend::ast::NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
                _ => continue,
            };
            if let Some(tps) = tps {
                self.hover_write_type_params_from_decls(b, tps);
                return;
            }
        }
    }

    /// 类型别名的类型参数声明
    fn type_alias_type_param_decls(&self, symbol: &Arc<Symbol>) -> Option<Arc<tsox_frontend::ast::NodeList>> {
        for decl in &symbol.declarations {
            if let tsox_frontend::ast::NodeData::TypeAliasDeclaration(d) = &decl.data {
                return d.type_parameters.clone();
            }
        }
        None
    }
}

fn shorthand_widen_type(checker: &mut Checker, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
    // reverse-mapped 合成符号复制了源属性声明（含 shorthand），但类型始终经
    // reverse links 惰性推断（Go getTypeOfReverseMappedSymbol），不做 shorthand 改道
    if symbol
        .check_flags
        .contains(tsox_frontend::ast::CheckFlags::ReverseMapped)
    {
        return None;
    }
    let sa = symbol.declarations.iter().find(|d| d.kind == SyntaxKind::ShorthandPropertyAssignment)?;
    let name = match &sa.data {
        tsox_frontend::ast::NodeData::ShorthandPropertyAssignment(d) => Arc::clone(&d.name),
        _ => return None,
    };
    let var_sym = checker.resolve_symbol_for_hover(&name)?;
    let t = checker.get_type_of_symbol(&var_sym);
    Some(checker.get_widened_literal_type(&t))
}

impl Checker {
    /// node 是 PAE 名字段，且所在访问是可选链最外层、链下方存在 ?.
    fn is_outermost_optional_chain_access(&self, node: &Arc<Node>) -> bool {
        let Some(parent) = node.parent.as_ref() else {
            return false;
        };
        if parent.kind != SyntaxKind::PropertyAccessExpression {
            return false;
        }
        let NodeData::PropertyAccessExpression(pd) = &parent.data else {
            return false;
        };
        if !Arc::ptr_eq(&pd.name, node) {
            return false;
        }
        // 上溯到链最外层（父级名字链）
        let mut outer = Arc::clone(parent);
        while let Some(grand) = outer.parent.as_ref() {
            if grand.kind == SyntaxKind::PropertyAccessExpression {
                let NodeData::PropertyAccessExpression(gd) = &grand.data else {
                    break;
                };
                if Arc::ptr_eq(&gd.name, &outer) {
                    outer = Arc::clone(grand);
                    continue;
                }
            }
            break;
        }
        if !Arc::ptr_eq(&outer, parent) {
            return false;
        }
        // 链下方（接收者子树）存在 ?.；本访问自身的 ?. 只产 marker（显示剥离）
        self.receiver_chain_has_question_dot(&parent)
    }

    fn receiver_chain_has_question_dot(&self, access: &Arc<Node>) -> bool {
        let NodeData::PropertyAccessExpression(d) = &access.data else {
            return false;
        };
        let mut expr = Arc::clone(&d.expression);
        loop {
            match &expr.data {
                NodeData::PropertyAccessExpression(p) => {
                    if p.question_dot_token.is_some() {
                        return true;
                    }
                    expr = Arc::clone(&p.expression);
                }
                NodeData::ElementAccessExpression(_) => return false,
                NodeData::ParenthesizedExpression(p) => {
                    let next = Arc::clone(&p.expression);
                    expr = next;
                }
                NodeData::NonNullExpression(p) => {
                    let next = Arc::clone(&p.expression);
                    expr = next;
                }
                NodeData::CallExpression(c) => {
                    let next = Arc::clone(&c.expression);
                    expr = next;
                }
                _ => return false,
            }
        }
    }

    fn add_optional_chain_undefined(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let undef = self.undefined_type();
        if t.flags.contains(TypeFlags::Undefined) {
            return Arc::clone(t);
        }
        if let TypeData::Union(u) = &t.data
            && u.union_or_intersection.types.iter().any(|c| {
                c.flags.contains(TypeFlags::Undefined)
            })
        {
            return Arc::clone(t);
        }
        let types = vec![Arc::clone(t), undef];
        self.get_union_type(types)
    }
}
