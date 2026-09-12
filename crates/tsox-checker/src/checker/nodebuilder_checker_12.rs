#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;

fn clean_jsdoc_text(raw: &str) -> String {
    let body = raw
        .trim_start()
        .strip_prefix("/**")
        .map(|b| b.strip_suffix("*/").unwrap_or(b))
        .unwrap_or(raw);
    let mut lines: Vec<String> = Vec::new();
    for line in body.lines() {
        let t = line.trim_start();
        let t = t.strip_prefix('*').map(str::trim_start).unwrap_or(t);
        if !(t.is_empty() && lines.iter().all(|l| l.trim().is_empty())) {
            lines.push(t.to_string());
        }
    }
    while lines.first().is_some_and(|l| l.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

fn node_line_char(
    sf: &tsox_frontend::ast::SourceFile,
    start: usize,
    end: usize,
) -> (usize, usize, usize, usize) {
    let text = &sf.text;
    let lc = |off: usize| {
        let prefix = &text[..off.min(text.len())];
        let line = prefix.matches('\n').count();
        let col = prefix
            .rfind('\n')
            .map(|i| prefix.len() - i - 1)
            .unwrap_or(prefix.len());
        (line + 1, col)
    };
    let (l1, c1) = lc(start);
    let (l2, c2) = lc(end);
    (l1, c1 + 1, l2, c2 + 1)
}

pub(crate) fn is_declaration_name(parent: &Arc<Node>, name: &Arc<Node>) -> bool {
    let hit = |n: Option<&Arc<Node>>| n.is_some_and(|x| Arc::ptr_eq(x, name));
    if let tsox_frontend::ast::NodeData::BindingElement(d) = &parent.data {
        return hit(d.name.as_ref()) || hit(d.property_name.as_ref());
    }
    if let Some(n) = tsox_frontend::ast::node_data_generated::node_name(parent) {
        return hit(Some(n));
    }
    false
}

impl Checker {
    pub(crate) fn resolve_property_access_symbol(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let parent = node.parent()?;
        if parent.kind == SyntaxKind::ElementAccessExpression {
            let tsox_frontend::ast::NodeData::ElementAccessExpression(d) = &parent.data else {
                return None;
            };
            if !Arc::ptr_eq(&d.argument_expression, node) {
                return None;
            }
            let tsox_frontend::ast::NodeData::StringLiteral(sl) = &d.argument_expression.data
            else {
                return None;
            };
            let name = sl.text.clone();
            let obj_type = self.get_type_of_node(&d.expression);
            return self
                .get_property_of_type(&obj_type, &name)
                .or_else(|| self.property_from_union(&obj_type, &name));
        }
        if parent.kind != SyntaxKind::PropertyAccessExpression {
            return None;
        }
        let tsox_frontend::ast::NodeData::PropertyAccessExpression(d) = &parent.data else {
            return None;
        };
        if !Arc::ptr_eq(&d.name, node) {
            return None;
        }
        let name = d.name.text();
        let obj_type = self.get_type_of_node(&d.expression);
        self.get_property_of_type(&obj_type, &name)
            .or_else(|| self.property_from_union(&obj_type, &name))
    }

    fn property_from_union(&mut self, t: &Arc<Type>, name: &str) -> Option<Arc<Symbol>> {
        if !t.is_union() {
            return None;
        }
        self.constituent_types(t)
            .into_iter()
            .find_map(|c| self.get_property_of_type(&c, name))
    }

    // x: C<number> 的成员访问显示：qualified 名带实例实参 + 方法/属性形态
    pub(crate) fn instantiated_member_access_parts(&mut self, node: &Arc<Node>) -> Option<Vec<SymbolDisplayPart>> {
        let p = node.parent()?;
                if p.kind != SyntaxKind::PropertyAccessExpression {
            return None;
        }
        let tsox_frontend::ast::NodeData::PropertyAccessExpression(pae) = &p.data else {
            return None;
        };
        if !Arc::ptr_eq(&pae.name, node) {
            return None;
        }
        let raw_type = self.get_type_of_node(&pae.expression);
        // expando 挂载产生「函数 & {属性}」交集：取承载具名成员的成分
        let obj_type = if let TypeData::Intersection(i) = &raw_type.data {
            i.union_or_intersection
                .types
                .iter()
                .find(|c| {
                    c.as_structured()
                        .is_some_and(|st| !st.properties.is_empty())
                })
                .cloned()
                .unwrap_or_else(|| Arc::clone(&raw_type))
        } else {
            raw_type
        };
        let obj_data = obj_type.as_object()?;
        let call = p.parent().filter(|c| c.kind == SyntaxKind::CallExpression);
                let mut class_sym = obj_type.symbol.clone();
        let mut from_call_return = false;
        if !class_sym
            .as_ref()
            .is_some_and(|s| s.flags.intersects(SymbolFlags::Class | SymbolFlags::Interface))
        {
            // 泛型调用返回的局部函数型：容器用 expando face 携带的函数符号
            // （getProps<{}>.x；实参由调用位推断补足）。仅限接收者是调用表达式，
            // 模块/命名空间成员不在此列
            if pae.expression.kind != SyntaxKind::CallExpression {
                return None;
            }
            from_call_return = true;
        }
        let Some(class_sym) = class_sym else {
            return None;
        };
        let mut display_class_sym = Arc::clone(&class_sym);
        // 方法成员（泛型与非泛型 owner 均可）：调用位推断实参 + 限定名 + 方法形态
        let prop_sym = self.get_property_of_type(&obj_type, &node.text())?;
        let prop_type = self.substituted_member_type_of(&obj_type, &prop_sym);
        let is_method = prop_sym.flags.intersects(SymbolFlags::Method)
            || prop_sym.declarations.iter().any(|d| {
                matches!(d.kind, SyntaxKind::MethodSignature | SyntaxKind::MethodDeclaration)
            });
        let sig = prop_type.as_structured().and_then(|s| s.call_signatures().first().cloned());
        if !is_method && obj_data.type_arguments.is_empty() && !from_call_return {
            return None;
        }
        // 调用位：从实参推断类型实参（f<a>(...) 形态）
        let inferred: Vec<Arc<Type>> = match (&call, &sig) {
            (Some(call_node), Some(sig)) => {
                let args: Vec<Arc<Node>> = match &call_node.data {
                    tsox_frontend::ast::NodeData::CallExpression(d) => {
                        d.arguments.iter().cloned().collect()
                    }
                    _ => Vec::new(),
                };
                if !sig.type_parameters.is_empty() {
                    self.infer_call_type_arguments(call_node, sig, &args)
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        };
        // 容器实参：owner 的类型实参；局部函数返回容器用调用位推断实参（getProps<{}>）；
        // 继承自泛型基类的成员按声明容器 + 成员 mapper 写 `A<string>.m`（Go symbol 链语义）
        let declaring_container = prop_sym
            .parent()
            .clone()
            .or_else(|| self.container_symbol_from_declarations(&prop_sym))
            .filter(|p| {
                p.flags.intersects(SymbolFlags::Class | SymbolFlags::Interface)
                    && !Arc::ptr_eq(&class_sym, p)
            });
        if let Some(declaring) = &declaring_container {
            let args = self.instantiated_member_container_args(&prop_sym, declaring);
            if args.is_some() {
                display_class_sym = Arc::clone(declaring);
            }
        }
        let container_args: Vec<Arc<Type>> = if let Some(declaring) = &declaring_container {
            self.instantiated_member_container_args(&prop_sym, declaring)
                .unwrap_or_default()
        } else if !obj_data.type_arguments.is_empty() {
            obj_data.type_arguments.clone()
        } else if from_call_return && pae.expression.kind == SyntaxKind::CallExpression {
            self.call_inferred_type_arguments(&pae.expression)
        } else {
            Vec::new()
        };
        let mut parts = Vec::new();
        if is_method {
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "method", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
        } else {
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "property", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
        }
        // 限定名：命名空间限定 + 接口/类名 + 成员名
        let ns_prefix = self
            .namespace_only_qualifier_of(&display_class_sym)
            .map(|q| format!("{q}."))
            .unwrap_or_default();
        push_part(
            &mut parts,
            &format!("{ns_prefix}{}", display_class_sym.name),
            DisplayPartKind::ClassName,
        );
        if !container_args.is_empty() {
            let args: Vec<String> = container_args
                .iter()
                .map(|a| self.type_to_string(a))
                .collect();
            push_punctuation(&mut parts, "<");
            push_part(&mut parts, &args.join(", "), DisplayPartKind::Text);
            push_punctuation(&mut parts, ">");
        }
        push_punctuation(&mut parts, ".");
        push_part(&mut parts, &node.text(), DisplayPartKind::PropertyName);
        if is_method {
            if let Some(sig) = &sig {
                if !inferred.is_empty() {
                    let names: Vec<String> =
                        inferred.iter().map(|t| self.type_to_string(t)).collect();
                    push_punctuation(&mut parts, "<");
                    push_part(&mut parts, &names.join(", "), DisplayPartKind::Text);
                    push_punctuation(&mut parts, ">");
                }
                // 参数与返回都按推断实参实例化显示（Go writeSignature display）
                let inst = if inferred.is_empty() {
                    Arc::clone(sig)
                } else {
                    self.get_signature_instantiation(sig, &inferred)
                };
                push_punctuation(&mut parts, "(");
                self.append_signature_parameter_parts(&mut parts, &inst);
                push_punctuation(&mut parts, ")");
                self.append_signature_return_parts(&mut parts, &inst);
            }
        } else {
            push_space(&mut parts, ": ");
            parts.extend(self.type_to_display_parts(&prop_type));
        }
        Some(parts)
    }

    /// 泛型调用返回的局部函数符号：解析调用签名声明体内 return 的标识符
    pub(crate) fn call_return_function_symbol(
        &mut self,
        call: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let sig = self.resolved_call_signature(call);
                let sig = sig?;
        let decl = sig.declaration.clone()?;
                let body = match &decl.data {
            crate::checker::nodebuilder::NodeData::FunctionDeclaration(d) => d.body.clone(),
            _ => return None,
        };
        let mut found: Option<Arc<Node>> = None;
        if let Some(body) = &body {
                        self.find_first_return_expression(body, &mut found);
        }
                let ret_expr = found?;
        if ret_expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(&ret_expr);
                let sym = sym?;
        sym.flags
            .contains(SymbolFlags::Function)
            .then(|| sym)
    }

    fn find_first_return_expression(
        &self,
        node: &Arc<Node>,
        found: &mut Option<Arc<Node>>,
    ) {
        if found.is_some() {
            return;
        }
        use tsox_frontend::ast::node_data_generated::for_each_child;
        match node.kind {
            SyntaxKind::ReturnStatement => {
                if let crate::checker::nodebuilder::NodeData::ReturnStatement(d) = &node.data {
                    *found = d.expression.clone();
                }
                return;
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration => return,
            _ => {}
        }
        for_each_child(node, |child| {
                        self.find_first_return_expression(child, found);
            found.is_some()
        });
    }

    /// 调用位的被调签名推断实参（容器显示 getProps<{}>）
    pub(crate) fn call_inferred_type_arguments(&mut self, call: &Arc<Node>) -> Vec<Arc<Type>> {
        let Some(sig) = self.resolved_call_signature(call) else {
            return Vec::new();
        };
        if sig.type_parameters.is_empty() {
            return Vec::new();
        }
        let args: Vec<Arc<Node>> = match &call.data {
            crate::checker::nodebuilder::NodeData::CallExpression(d) => {
                d.arguments.iter().cloned().collect()
            }
            _ => Vec::new(),
        };
        let inferred = self.infer_call_type_arguments(call, &sig, &args);
                inferred
    }

    // JSX 属性 hover：<Opt propx={2}/> 的 propx → (property) propx: <元素类型属性>
    pub(crate) fn jsx_attribute_parts(&mut self, node: &Arc<Node>) -> Option<Vec<SymbolDisplayPart>> {
        let p = node.parent()?;
        if p.kind != SyntaxKind::JsxAttribute {
            return None;
        }
        let tsox_frontend::ast::NodeData::JsxAttribute(attr_data) = &p.data else {
            return None;
        };
        if !Arc::ptr_eq(&attr_data.name, node) {
            return None;
        }
        // 属性列表（JsxAttributes）与元素之间可能隔一层
        let mut elem = p.parent()?;
        while elem.kind == SyntaxKind::JsxAttributes {
            elem = elem.parent()?;
        }
        let tag_name = match &elem.data {
            tsox_frontend::ast::NodeData::JsxSelfClosingElement(d) => Arc::clone(&d.tag_name),
            tsox_frontend::ast::NodeData::JsxOpeningElement(d) => Arc::clone(&d.tag_name),
            _ => return None,
        };
        let tag_text = tag_name
            .jsx_namespaced_name_text()
            .unwrap_or_else(|| tag_name.text().to_string());
        let name_text = node
            .jsx_namespaced_name_text()
            .unwrap_or_else(|| node.text().to_string());
        if !tag_text.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            // intrinsic 元素属性：props 类型上同名属性（Go getContextualJsxElementAttributesType
            // + GetPropertySymbolsFromContextualType）
            let elem = self.jsx_intrinsic_element_symbol(&tag_text)?;
            let elem_type = self.get_type_of_symbol(&elem);
            let prop = self.get_property_of_type(&elem_type, &name_text)?;
            let prop_type = self.get_type_of_symbol(&prop);
            let display_name = self.symbol_name_as_written(&prop).unwrap_or_else(|| name_text.clone());
            let mut parts = Vec::new();
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "property", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
            push_part(&mut parts, &display_name, DisplayPartKind::PropertyName);
            push_space(&mut parts, ": ");
            parts.extend(self.type_to_display_parts(&prop_type));
            if let Some(decl) = prop.value_declaration.as_ref().or(prop.declarations.first()) {
                let doc = self.declaration_jsdoc_text(decl);
                if !doc.is_empty() {
                    push_space(&mut parts, "\n\n");
                    push_part(&mut parts, &doc, DisplayPartKind::Text);
                }
            }
            return Some(parts);
        }
        let class_sym = self.resolve_identifier(&tag_name)?;
        // Go getContextualJsxElementAttributesType：首个构造签名参数类型经属性值
        // 推断实例化（Component<T> 的 props），成员符号带容器实参显示
        if let Some(prop) =
            self.jsx_attribute_symbol_from_construct_signature(&class_sym, &p, &name_text)
        {
            let prop_type = self.get_type_of_symbol(&prop);
            let mut parts = Vec::new();
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "property", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
            let display_name = self.qualified_symbol_name(&prop);
            push_part(&mut parts, &display_name, DisplayPartKind::PropertyName);
            if prop.flags.contains(SymbolFlags::Optional) {
                push_punctuation(&mut parts, "?");
            }
            push_punctuation(&mut parts, ": ");
            parts.extend(self.type_to_display_parts(&prop_type));
            if let Some(decl) = prop.value_declaration.as_ref().or(prop.declarations.first()) {
                let doc = self.declaration_jsdoc_text(decl);
                if !doc.is_empty() {
                    push_space(&mut parts, "\n\n");
                    push_part(&mut parts, &doc, DisplayPartKind::Text);
                }
            }
            return Some(parts);
        }
        // 类值符号 → 实例类型（成员属性在实例上，构造类型只有静态成员）
        let instance = match class_sym.declarations.first() {
            Some(d) if matches!(d.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) => {
                let inst = self.build_class_instance_type_with_base(d);
                inst
            }
            _ => self.get_type_of_symbol(&class_sym),
        };
        let prop = match self.get_property_of_type(&instance, &name_text) {
            Some(p) => p,
            None => {
                return None;
            }
        };
        let prop_type = self.get_type_of_symbol(&prop);
        let mut parts = Vec::new();
        push_punctuation(&mut parts, "(");
        push_part(&mut parts, "property", DisplayPartKind::Text);
        push_punctuation(&mut parts, ") ");
        push_part(&mut parts, &name_text, DisplayPartKind::PropertyName);
        push_space(&mut parts, ": ");
        parts.extend(self.type_to_display_parts(&prop_type));
        Some(parts)
    }

    /// JSX 元素属性上下文类型（Go getContextualJsxElementAttributesType）：
    /// 类标签首个构造签名参数类型，泛型按属性值（跳过函数值）推断实例化
    pub fn jsx_element_attributes_contextual_type(
        &mut self,
        elem: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let (tag_name, attrs, elem_type_arguments) = match &elem.data {
            tsox_frontend::ast::NodeData::JsxSelfClosingElement(d) => (
                Arc::clone(&d.tag_name),
                Arc::clone(&d.attributes),
                d.type_arguments.clone(),
            ),
            tsox_frontend::ast::NodeData::JsxOpeningElement(d) => (
                Arc::clone(&d.tag_name),
                Arc::clone(&d.attributes),
                d.type_arguments.clone(),
            ),
            _ => return None,
        };
        if !tag_name
            .text()
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_uppercase())
        {
            return None;
        }
        let class_sym = self.resolve_identifier(&tag_name)?;
        let ctor_type = self.get_type_of_symbol(&class_sym);
        let sig = self
            .get_signatures_of_type(&ctor_type, SignatureKind::Construct)
            .first()
            .cloned()?;
        if sig.parameters.is_empty() {
            return None;
        }
        let mut param_type = self
            .signature_instantiated_param_type(&sig, 0)
            .unwrap_or_else(|| self.get_type_of_symbol(&sig.parameters[0]));
        if !sig.type_parameters.is_empty() {
            // 显式类型实参（<Table<Props> .../>）优先于属性值推断
            let explicit: Vec<Arc<Type>> = elem_type_arguments
                .as_ref()
                .map(|args| {
                    args.iter()
                        .map(|arg| self.get_type_from_type_node(arg))
                        .collect()
                })
                .unwrap_or_default();
            if !explicit.is_empty() {
                param_type = self.substitute_infer_type_parameters(
                    &param_type,
                    &sig.type_parameters,
                    &explicit,
                );
            } else {
                let mut attr_args: Vec<Arc<Node>> = Vec::new();
            {
                let properties: Vec<Arc<Node>> = match &attrs.data {
                    tsox_frontend::ast::NodeData::JsxAttributes(data) => {
                        data.properties.iter().cloned().collect()
                    }
                    _ => Vec::new(),
                };
                for attr in properties {
                    let value = match &attr.data {
                        tsox_frontend::ast::NodeData::JsxAttribute(d) => d.initializer.clone(),
                        _ => None,
                    };
                    let Some(value) = value else { continue };
                    let expr = match &value.data {
                        tsox_frontend::ast::NodeData::JsxExpression(je) => je.expression.clone(),
                        _ => Some(Arc::clone(&value)),
                    };
                    if let Some(expr) = expr
                        && !matches!(
                            expr.kind,
                            SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression
                        )
                    {
                        attr_args.push(expr);
                    }
                }
            }
            let inferred = self.infer_call_type_arguments(elem, &sig, &attr_args);
            if !inferred.is_empty() {
                param_type = self.substitute_infer_type_parameters(
                    &param_type,
                    &sig.type_parameters,
                    &inferred,
                );
            }
            }
        }
        Some(param_type)
    }

    /// JSX 属性符号：构造签名参数类型（泛型按属性值推断实例化）上的成员，
    /// union 按 name 判别式收窄（Go GetPropertySymbolsFromContextualType）
    fn jsx_attribute_symbol_from_construct_signature(
        &mut self,
        class_sym: &Arc<Symbol>,
        attr_node: &Arc<Node>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let attrs = attr_node.parent()?;
        let elem = attrs.parent()?;
        let param_type = self.jsx_element_attributes_contextual_type(&elem)?;
        let syms = self.get_property_symbols_from_contextual_type(attr_node, &param_type, false);
        if syms.len() == 1 {
            return Some(Arc::clone(&syms[0]));
        }
        None
    }

    pub(crate) fn is_jsx_tag_name(&self, node: &Arc<Node>) -> bool {
        let is_jsx_container = |k: SyntaxKind| {
            matches!(
                k,
                SyntaxKind::JsxOpeningElement
                    | SyntaxKind::JsxClosingElement
                    | SyntaxKind::JsxSelfClosingElement
            )
        };
        // Go IsJsxTagName：不限 kind，只看父节点的 tag_name 槽位
        if node.kind == SyntaxKind::JsxNamespacedName {
            return node
                .parent()
                .as_ref()
                .is_some_and(|p| is_jsx_container(p.kind));
        }
        if node.kind != SyntaxKind::Identifier {
            return false;
        }
        let Some(p) = node.parent() else {
            return false;
        };
        if is_jsx_container(p.kind) {
            return true;
        }
        // 前端可能把 tag name 包成 TypeReference
        p.kind == SyntaxKind::TypeReference
            && p.parent()
                .as_ref()
                .is_some_and(|g| is_jsx_container(g.kind))
    }

    /// IntrinsicElements 上的元素属性符号（供 tag hover 与属性上下文共用）
    pub fn jsx_intrinsic_element_symbol(&mut self, name: &str) -> Option<Arc<Symbol>> {
        let find_jsx = |checker: &Checker| -> Option<Arc<Symbol>> {
            let symbol_map = checker.program.symbol_map();
            let file = checker.display_enclosing_file.clone()?;
            if let Some(locals) = symbol_map.locals.get(&file.node.id())
                && let Some(sym) = locals.get("JSX")
            {
                return Some(Arc::clone(sym));
            }
            if let Some(sym) = symbol_map.symbol_of(&file.node)
                && let Some(sym) = sym.members.get("JSX")
            {
                return Some(Arc::clone(sym));
            }
            None
        };
        let jsx_sym = find_jsx(self).or_else(|| self.globals.get("JSX").cloned())?;
        let jsx_type = self.get_type_of_symbol(&jsx_sym);
        let find_member = |checker: &Checker, sym: &Arc<Symbol>, member: &str| -> Option<Arc<Symbol>> {
            for d in &sym.declarations {
                if let Some(locals) = checker.program.symbol_map().locals.get(&d.id())
                    && let Some(m) = locals.get(member)
                {
                    return Some(Arc::clone(m));
                }
            }
            sym.members.get(member).cloned().or_else(|| sym.exports.get(member).cloned())
        };
        let intrinsics = find_member(self, &jsx_sym, "IntrinsicElements");
        let intrinsics = intrinsics
            .or_else(|| self.get_property_of_type(&jsx_type, "IntrinsicElements"))?;
        let intrinsics_type = self.get_type_of_symbol(&intrinsics);
        // IntrinsicElements 是接口声明：成员由 checker 按声明构建（resolve_interface_type_ex）
        self.resolve_interface_type_ex(&intrinsics, None)
            .as_structured()
            .and_then(|s| s.members.get(name).cloned())
            .or_else(|| self.get_property_of_type(&intrinsics_type, name))
    }

    /// 符号名按声明原样（保留引号风格，Go getNameOfSymbolAsWritten）
    pub(crate) fn symbol_name_as_written(&mut self, symbol: &Arc<Symbol>) -> Option<String> {
        let decl = symbol
            .value_declaration
            .as_ref()
            .or(symbol.declarations.first())?;
        let name_node = tsox_frontend::ast::node_data_generated::node_name(decl)?;
        self.node_source_text(name_node)
    }

    // 查全局 JSX 命名空间的 IntrinsicElements 属性，产出 (property) JSX.IntrinsicElements.div: any
    pub(crate) fn jsx_intrinsic_element_parts(&mut self, name: &str) -> Option<Vec<SymbolDisplayPart>> {
        // JSX 命名空间可来自文件内 declare namespace 或全局 lib：
        // 沿 enclosing 文件容器 locals/members 找名字为 JSX 的命名空间符号
        let find_jsx = |checker: &Checker| -> Option<Arc<Symbol>> {
            let symbol_map = checker.program.symbol_map();
            let file = checker.display_enclosing_file.clone()?;
            if let Some(locals) = symbol_map.locals.get(&file.node.id())
                && let Some(sym) = locals.get("JSX")
            {
                return Some(Arc::clone(sym));
            }
            if let Some(sym) = symbol_map.symbol_of(&file.node)
                && let Some(sym) = sym.members.get("JSX")
            {
                return Some(Arc::clone(sym));
            }
            None
        };
        let jsx_sym = find_jsx(self).or_else(|| self.globals.get("JSX").cloned())?;
        let jsx_type = self.get_type_of_symbol(&jsx_sym);
        // declare namespace 的成员在 locals（ambient 不进 exports），沿声明手工下钻
        let find_member = |checker: &Checker, sym: &Arc<Symbol>, member: &str| -> Option<Arc<Symbol>> {
            for d in &sym.declarations {
                if let Some(locals) = checker.program.symbol_map().locals.get(&d.id())
                    && let Some(m) = locals.get(member)
                {
                    return Some(Arc::clone(m));
                }
            }
            sym.members.get(member).cloned().or_else(|| sym.exports.get(member).cloned())
        };
        let elem = self.jsx_intrinsic_element_symbol(name)?;
        let elem_type = self.get_type_of_symbol(&elem);
        let mut parts = Vec::new();
        push_punctuation(&mut parts, "(");
        push_part(&mut parts, "property", DisplayPartKind::Text);
        push_punctuation(&mut parts, ") ");
        push_part(&mut parts, "JSX", DisplayPartKind::Text);
        push_punctuation(&mut parts, ".");
        push_part(&mut parts, "IntrinsicElements", DisplayPartKind::InterfaceName);
        // 非标识符成员（foo:bar 等）按声明引号风格渲染元素访问
        if name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
            && !name.chars().next().is_some_and(|c| c.is_ascii_digit())
        {
            push_punctuation(&mut parts, ".");
            push_part(&mut parts, name, DisplayPartKind::PropertyName);
        } else {
            push_punctuation(&mut parts, "[");
            push_part(&mut parts, &format!("\"{name}\""), DisplayPartKind::StringLiteral);
            push_punctuation(&mut parts, "]");
        }
        push_space(&mut parts, ": ");
        parts.extend(self.type_to_display_parts(&elem_type));
        // 属性声明上的 JSDoc
        if let Some(decl) = elem
            .value_declaration
            .as_ref()
            .or(elem.declarations.first())
        {
            let doc = self.declaration_jsdoc_text(decl);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
        }
        Some(parts)
    }

    pub(crate) fn resolve_symbol_for_hover(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        // 1) 引用：沿祖先容器查 binder locals（hover 无作用域栈时的等价物）
        let by_locals = (|| {
            let name = match &node.data {
                tsox_frontend::ast::NodeData::Identifier(i) => i.text.clone(),
                _ => tsox_frontend::ast::node_data_generated::node_name(node)
                    .and_then(|n| match &n.data {
                        tsox_frontend::ast::NodeData::Identifier(i) => Some(i.text.clone()),
                        _ => None,
                    })
                    .unwrap_or_default(),
            };
            if name.is_empty() {
                return None;
            }
            let symbol_map = self.program.symbol_map();
            let mut cur = node.parent();
            while let Some(n) = cur {
                if let Some(locals) = symbol_map.locals.get(&n.id()) {
                    if let Some(sym) = locals.get(&name)
                        && !sym.flags.intersects(
                            SymbolFlags::ValueModule
                                | SymbolFlags::NamespaceModule
                                | SymbolFlags::ModuleExports,
                        )
                    {
                        // 赋值目标位置的对象字面量（解构赋值）shorthand 成员：
                        // 身份让位于外层同名变量
                        let is_destructuring_member = sym.declarations.iter().any(|d| {
                            d.kind == SyntaxKind::ShorthandPropertyAssignment
                                && d.parent().as_ref().is_some_and(|o| {
                                    o.kind == SyntaxKind::ObjectLiteralExpression
                                        && o.pos() >= assignment_target_expr(o).pos()
                                            && o.end() <= assignment_target_expr(o).end()
                                })
                        });
                        if !is_destructuring_member {
                            return Some(Arc::clone(sym));
                        }
                    }
                }
                cur = n.parent();
            }
            None
        })();
        // 1.5) 声明处名字优先于容器查找：const Unit 与 export type Unit 同名时，
        // by_locals 命中的是后声明覆盖的符号；声明名字节点的身份由其自身声明决定
        // （shorthand 属性名排除：其身份由专用分支处理）
        if let Some(parent) = node.parent().as_ref()
            && parent.kind != SyntaxKind::ShorthandPropertyAssignment
            && is_declaration_name(parent, node)
            && let Some(sym) = self.program.symbol_map().symbol_of(parent)
        {
            return Some(Arc::clone(sym));
        }
        if let Some(s) = self.resolve_identifier(node) {
            return Some(s);
        }
        if let Some(s) = by_locals {
            return Some(s);
        }
        // 3) 仅当节点自身或其所属声明（节点为该声明的名字）带符号时采纳，
        //    不做无边界上溯（否则悬停未解析名会误命中外层函数/类符号）
        let symbol_map = self.program.symbol_map();
        if let Some(sym) = symbol_map.symbol_of(node) {
            if sym.flags.intersects(
                SymbolFlags::ValueModule
                    | SymbolFlags::NamespaceModule
                    | SymbolFlags::ModuleExports,
            ) {
                return None;
            }
            return Some(Arc::clone(sym));
        }
        let mut current = node.parent();
        while let Some(n) = current {
            if is_declaration_name(&n, node) {
                if let Some(sym) = symbol_map.symbol_of(&n) {
                    if sym.flags.intersects(
                        SymbolFlags::ValueModule
                            | SymbolFlags::NamespaceModule
                            | SymbolFlags::ModuleExports,
                    ) {
                        return None;
                    }
                    return Some(Arc::clone(sym));
                }
                break;
            }
            current = n.parent();
        }
        None
    }


    /// 对象字面量属性名按上下文类型解析（对齐 Go getSymbolAtLocationForQuickInfo）
    pub(crate) fn resolve_contextual_property_symbol(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let parent = node.parent()?;
        match parent.kind {
            SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodDeclaration => {}
            _ => return None,
        }
        let obj = parent.parent()?;
        if obj.kind != SyntaxKind::ObjectLiteralExpression {
            return None;
        }
        let ct = self.get_contextual_type(&obj, ContextFlags::None)?;
        let syms = self.get_property_symbols_from_contextual_type(&parent, &ct, false);
        if syms.len() == 1 {
            return Some(Arc::clone(&syms[0]));
        }
        None
    }

    /// `new Cat()` / `new C<any>()` 的构造函数悬停：`constructor C<any>(): C<any>`
    pub(crate) fn constructor_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
    ) -> Option<Vec<SymbolDisplayPart>> {
        // 节点须处于 new 表达式中：new 关键字、表达式名或类型实参内
        let mut cur = node.parent()?;
        loop {
            match cur.kind {
                SyntaxKind::NewExpression => break,
                SyntaxKind::PropertyAccessExpression
                | SyntaxKind::TypeReference
                | SyntaxKind::ExpressionWithTypeArguments => cur = cur.parent()?,
                _ => return None,
            }
        }
        // 实参文本：显式 <...>；无实参显示类自身的类型参数
        let type_args_text = self
            .new_expression_type_args_text(&cur, symbol)
            .unwrap_or_default();
        let mut parts = Vec::new();
        push_keyword(&mut parts, "constructor ");
        push_part(
            &mut parts,
            &format!("{}{}", symbol.name, type_args_text),
            DisplayPartKind::ClassName,
        );
        // 首个构造签名
        let declared = self.get_type_of_symbol(symbol);
        if let Some(structured) = declared.as_structured()
            && let Some(sig) = structured.construct_signatures().first()
        {
            let ret = self
                .get_return_type_of_signature(sig)
                .unwrap_or_else(|| self.get_any_type());
            let ret = self.instantiate_new_expression_return(&cur, symbol, ret);
            push_punctuation(&mut parts, "(");
            self.append_signature_parameter_parts(&mut parts, sig);
            push_punctuation(&mut parts, ")");
            push_space(&mut parts, ": ");
            parts.extend(self.type_to_display_parts(&ret));
        }
        Some(parts)
    }

    fn instantiate_new_expression_return(
        &mut self,
        new_expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
        ret: Arc<Type>,
    ) -> Arc<Type> {
        let tsox_frontend::ast::NodeData::NewExpression(d) = &new_expr.data else {
            return ret;
        };
        let Some(args) = &d.type_arguments else {
            return ret;
        };
        let Some(decl) = symbol
            .declarations
            .iter()
            .find(|dn| matches!(dn.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression))
        else {
            return ret;
        };
        let tps = match &decl.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(cd) => cd.type_parameters.clone(),
            tsox_frontend::ast::NodeData::ClassExpression(ce) => ce.type_parameters.clone(),
            _ => None,
        };
        let Some(tps) = tps else {
            return ret;
        };
        let tp_symbols: Vec<Arc<Symbol>> = tps
            .iter()
            .filter_map(|tp| self.program.symbol_map().symbol_of(tp).cloned())
            .collect();
        let tp_types: Vec<Arc<Type>> = tp_symbols
            .iter()
            .map(|s| self.get_type_parameter_from_symbol(s))
            .collect();
        let arg_types: Vec<Arc<Type>> = args.iter().map(|t| self.get_type_from_type_node(t)).collect();
        if tp_types.is_empty() || arg_types.is_empty() {
            return ret;
        }
        let substituted =
            self.substitute_infer_type_parameters(&ret, &tp_types, &arg_types);
        self.rebuild_with_type_arguments(&substituted, arg_types)
    }

    fn new_expression_type_args_text(
        &mut self,
        new_expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Option<String> {
        let tsox_frontend::ast::NodeData::NewExpression(d) = &new_expr.data else {
            return None;
        };
        let Some(args) = &d.type_arguments else {
            return None;
        };
        let texts: Vec<String> = args
            .iter()
            .map(|t| {
                let arg_type = self.get_type_from_type_node(t);
                self.type_to_string(&arg_type)
            })
            .collect();
        let _ = symbol;
        Some(format!("<{}>", texts.join(", ")))
    }

    pub fn get_quick_info_display_parts(&mut self, node: &Arc<Node>) -> Vec<SymbolDisplayPart> {
                self.display_enclosing_file = self.get_source_file_of_node(node);
        self.display_enclosing_node = Some(Arc::clone(node));
        // tsc getTypeOfNode：with 块内无法回答语义问题，类型查询返回 error（显示 any）
        if node_in_with_block(node) {
            let mut parts = Vec::new();
            push_part(&mut parts, "any", DisplayPartKind::Keyword);
            return parts;
        }
        // JSX 属性名：从 tag 元素类型取属性（class 取实例类型属性，对齐 tsc checkJsxAttribute）
        if let Some(parts) = self.jsx_attribute_parts(node) {
            return parts;
        }
        // JSX 元素名：大写=值符号（class MyElement）；小写=JSX.IntrinsicElements 属性
        // （对齐 tsc getJsxElementAttributesType / checkJsxOpeningElement）
        if self.is_jsx_tag_name(node) {
            let name_text = node.text();
            if !name_text.is_empty()
                && name_text.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            {
                if let Some(sym) = self.resolve_identifier(node) {
                    if sym.flags.intersects(SymbolFlags::Class | SymbolFlags::Function) {
                        let mut parts = self.symbol_to_display_parts(&sym, SymbolFlags::all(), &[]);
                        let doc = self.symbol_documentation(&sym);
                        if !doc.is_empty() {
                            push_space(&mut parts, "\n\n");
                            push_part(&mut parts, &doc, DisplayPartKind::Text);
                        }
                        return parts;
                    }
                }
            }
            // 小写或未解析：IntrinsicElements 属性，无 JSX 命名空间时 any
            if let Some(parts) = self.jsx_intrinsic_element_parts(&name_text) {
                return parts;
            }
            let mut parts = Vec::new();
            push_part(&mut parts, "any", DisplayPartKind::Keyword);
            return parts;
        }
        // 合并符号（值+类型同名）按位置意义选身份：声明名字节点属于值声明时走值显示
        // （tsc getSymbolAtLocationForQuickInfo + checkIsDeclarationName 意义判定）
        // 类实例属性访问的成员显示：x: C<number> 的 m → (method) C<number>.m(): void
        // （owner 的 type_arguments 实例化 + 方法签名形态，对齐 tsc writeSymbolClassified）
        if let Some(parts) = self.instantiated_member_access_parts(node) {
            return parts;
        }
        // 合并符号（值+类型同名）按位置意义选身份：声明名字节点属于值声明时走值显示
        // （tsc getSymbolAtLocationForQuickInfo 意义判定）；纯值符号走常规路径
        let merged_with_type = self
            .resolve_symbol_for_hover(node)
            .is_some_and(|s| s.flags.intersects(SymbolFlags::TypeAlias | SymbolFlags::Interface));
        if merged_with_type
            && node.parent().as_ref().is_some_and(|p| {
                matches!(
                    p.kind,
                    SyntaxKind::VariableDeclaration
                        | SyntaxKind::FunctionDeclaration
                        | SyntaxKind::ClassDeclaration
                        | SyntaxKind::Parameter
                ) && is_declaration_name(p, node)
            }) && let Some(value_sym) = self
            .program
            .symbol_map()
            .symbol_of(node.parent().as_ref().expect("checked above"))
            .cloned()
        {
            // 合并符号 flags 含类型意义：值身份强制走变量显示（tsc 按位置意义选前缀）
            let mut parts = self.variable_symbol_display_parts(&value_sym);
            let doc = self.symbol_documentation(&value_sym);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
            return parts;
        }
        // shorthand 属性名：hover 命中的是外层同名变量符号，tsc 显示为字面量属性身份，
        // 类型 = 引用变量类型的 widen（tsc getTypeOfShorthandPropertyAssignment）
        let shorthand_info = node.parent().as_ref().and_then(|p| {
            if p.kind != SyntaxKind::ShorthandPropertyAssignment {
                return None;
            }
            p.parent()
                .as_ref()
                .filter(|o| o.kind == SyntaxKind::ObjectLiteralExpression)?;
            let tsox_frontend::ast::NodeData::ShorthandPropertyAssignment(sa) = &p.data else {
                return None;
            };
            // 仅「纯」shorthand（{name1}）：带赋值初始化器（{b = a}，解构赋值目标形态）
            // 的名字引用保持变量身份
            if sa.object_assignment_initializer.is_some() || sa.equals_token.is_some() {
                return None;
            }
            // 字面量处于赋值目标位置（解构赋值）：成员身份让位于外层变量
            let is_assignment_target = {
                let mut is_target = false;
                let mut cur = p.parent();
                while let Some(n) = cur {
                    match n.kind {
                        SyntaxKind::ObjectLiteralExpression
                        | SyntaxKind::ParenthesizedExpression => cur = n.parent(),
                        SyntaxKind::BinaryExpression => {
                            if let tsox_frontend::ast::NodeData::BinaryExpression(be) = &n.data {
                                let op_is_eq = be.operator_token.kind == SyntaxKind::EqualsToken;
                                // 位置包含判定（parser 可能生成不同实例的同一文本节点）
                                let left_has = be.left.pos() <= p.pos() && p.end() <= be.left.end();
                                is_target = op_is_eq && left_has;
                            }
                            break;
                        }
                        _ => break,
                    }
                }
                is_target
            };
            if is_assignment_target {
                return None;
            }
            let var_sym = self.resolve_symbol_for_hover(&sa.name)?;
            let t = self.get_type_of_symbol(&var_sym);
            Some(self.get_widened_literal_type(&t))
        });
        if let Some(t) = shorthand_info {
            let name = node.text();
            let mut parts = Vec::new();
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "property", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
            push_part(&mut parts, &name, DisplayPartKind::PropertyName);
            push_space(&mut parts, ": ");
            parts.extend(self.type_to_display_parts(&t));
            return parts;
        }
        let symbol = match self.resolve_contextual_property_symbol(node) {
            Some(s) => s,
            None => match self.resolve_property_access_symbol(node) {
                Some(s) => {
                                        s
                }
                None => {
                    // 属性名无具名成员但接收者带字符串索引签名（如反向映射 {[x: string]: ...}）：
                    // 显示索引值类型（无前缀，tsc 经索引访问定型）
                    if let Some(parts) = self.index_signature_property_parts(node) {
                        return parts;
                    }
                    let Some(symbol) = self.resolve_symbol_for_hover(node) else {
                        return Vec::new();
                    };
                    symbol
                }
            },
        };

        // new 表达式中的类名/new 关键字：构造函数签名显示（对齐 Go writeSignatures("constructor ")）
        if symbol.flags.intersects(SymbolFlags::Class)
            && let Some(parts) = self.constructor_display_parts(&symbol, node)
        {
            let mut parts = parts;
            let doc = self.constructor_jsdoc_documentation(&symbol);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
            return parts;
        }
        // 悬停命中改名绑定的 property_name：按源属性身份渲染
        if let Some(elem) = node.parent().as_ref()
            && elem.kind == SyntaxKind::BindingElement
            && let Some(parts) = self.binding_element_property_name_parts(node, &symbol)
        {
            return parts;
        }
        // 调用位的泛型函数属性/变量：属性前缀 + 实例化箭头签名 <number>(x: number) => number
        // （Go getCallOrNewExpression 分支 WriteTypeArgumentsOfSignature|WriteArrowStyleSignature）
        if symbol.flags.intersects(SymbolFlags::VARIABLE | SymbolFlags::Property)
            && let Some(call) = Self::enclosing_call_or_new(node)
            && let Some(sig) = self.resolved_call_signature(&call)
            && !sig.type_parameters.is_empty()
        {
            let args: Vec<Arc<Node>> = match &call.data {
                crate::checker::nodebuilder::NodeData::CallExpression(d) => {
                    d.arguments.iter().cloned().collect()
                }
                crate::checker::nodebuilder::NodeData::NewExpression(d) => d
                    .arguments
                    .as_ref()
                    .map(|a| a.iter().cloned().collect())
                    .unwrap_or_default(),
                _ => Vec::new(),
            };
            let inferred = self.infer_call_type_arguments(&call, &sig, &args);
            let inst = self.get_signature_instantiation(&sig, &inferred);
            let mut parts = self.variable_prefix_and_name_parts(&symbol);
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
            let doc = self.hover_documentation(&symbol, node);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
            return parts;
        }
        // 调用位的泛型函数：显示推断实例化签名 function f<number>(...)（Go getCallOrNewExpression
        // 分支 + writeSignatures WriteTypeArgumentsOfSignature）
        if symbol.flags.intersects(SymbolFlags::Function | SymbolFlags::Method)
            && let Some(call) = Self::enclosing_call_or_new(node)
            && call.kind == SyntaxKind::CallExpression
            && let Some(sig) = self.resolved_call_signature(&call)
            && !sig.type_parameters.is_empty()
        {
            let args: Vec<Arc<Node>> = match &call.data {
                crate::checker::nodebuilder::NodeData::CallExpression(d) => {
                    d.arguments.iter().cloned().collect()
                }
                _ => Vec::new(),
            };
            let inferred = self.infer_call_type_arguments(&call, &sig, &args);
            let inst = self.get_signature_instantiation(&sig, &inferred);
            let mut parts = Vec::new();
            if symbol.flags.intersects(SymbolFlags::Method) {
                push_punctuation(&mut parts, "(");
                push_part(&mut parts, "method", DisplayPartKind::Text);
                push_punctuation(&mut parts, ") ");
            } else {
                push_keyword(&mut parts, "function");
                push_space(&mut parts, " ");
            }
            push_part(
                &mut parts,
                &self.qualified_symbol_name(&symbol),
                DisplayPartKind::FunctionName,
            );
            if !inferred.is_empty() {
                let names: Vec<String> = inferred
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect();
                push_punctuation(&mut parts, "<");
                push_part(&mut parts, &names.join(", "), DisplayPartKind::Text);
                push_punctuation(&mut parts, ">");
            }
            push_punctuation(&mut parts, "(");
            self.append_signature_parameter_parts(&mut parts, &inst);
            push_punctuation(&mut parts, ")");
            push_space(&mut parts, ": ");
            let ret = self
                .get_return_type_of_signature(&inst)
                .unwrap_or_else(|| self.any_type());
            parts.extend(self.type_to_display_parts(&ret));
            let doc = self.hover_documentation(&symbol, node);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
            return parts;
        }
        // new 表达式的 callee：显示构造签名形态 new () => T（Go getCallOrNewExpression 分支）
        if symbol.flags.intersects(SymbolFlags::VARIABLE | SymbolFlags::Property)
            && let Some(call) = Self::enclosing_call_or_new(node)
            && call.kind == SyntaxKind::NewExpression
            && let Some(sig) = self.resolved_call_signature(&call)
            && sig.declaration.as_ref().is_some_and(|d| {
                matches!(d.kind, SyntaxKind::ConstructSignature | SyntaxKind::NewExpression)
                    || matches!(d.kind, SyntaxKind::CallSignature)
            })
        {
            let mut parts = self.variable_prefix_and_name_parts(&symbol);
            push_keyword(&mut parts, "new ");
            parts.extend(self.arrow_signature_parts(&sig));
            let doc = self.hover_documentation(&symbol, node);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
            return parts;
        }
        // 悬停在调用表达式的 callee 上且类型为签名 union：显示合成签名（对齐 Go getCallOrNewExpression 分支）
        if symbol.flags.intersects(SymbolFlags::VARIABLE | SymbolFlags::Property)
            && let Some(sig) = self.call_site_union_signature(&symbol, node)
        {
            let mut parts = self.variable_prefix_and_name_parts(&symbol);
            parts.extend(self.arrow_signature_parts(&sig));
            let doc = self.hover_documentation(&symbol, node);
            if !doc.is_empty() {
                push_space(&mut parts, "\n\n");
                push_part(&mut parts, &doc, DisplayPartKind::Text);
            }
            return parts;
        }
        let mut parts = self.symbol_to_display_parts(&symbol, SymbolFlags::all(), &[]);
        let doc = self.hover_documentation(&symbol, node);
        if !doc.is_empty() {
            push_space(&mut parts, "\n\n");
            push_part(&mut parts, &doc, DisplayPartKind::Text);
        }
        parts
    }

    pub fn get_quick_info_text(&mut self, node: &Arc<Node>) -> String {
        self.display_enclosing_file = self.get_source_file_of_node(node);
        self.display_enclosing_node = Some(Arc::clone(node));
        if node.kind == SyntaxKind::ThisKeyword {
            let t = self.get_type_of_node(node);
            return format!("this: {}", self.type_to_string(&t));
        }
        if node.kind == SyntaxKind::ThisType {
            return "this".to_string();
        }
        let Some(symbol) = self.resolve_symbol_for_hover(node) else {
            if self.node_has_type(node) {
                let t = self.get_type_of_node(node);
                return self.type_to_string(&t);
            }
            // 属性访问名未解析到符号时按表达式类型显示（对齐 Go shouldGetType）
            if let Some(parent) = node.parent().as_ref()
                && parent.kind == SyntaxKind::PropertyAccessExpression
            {
                let t = self.get_type_of_node(parent);
                return self.type_to_string(&t);
            }
            return String::new();
        };
        self.format_quick_info_for_symbol(&symbol, node)
    }

    pub fn symbol_to_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        type_arguments: &[String],
    ) -> Vec<SymbolDisplayPart> {
        let _ = meaning;
        let _ = type_arguments;

        let flags = symbol.flags;
        if flags.intersects(SymbolFlags::Function) {
            return self.function_symbol_display_parts(symbol, false);
        }
        if flags.intersects(SymbolFlags::Method) {
            return self.function_symbol_display_parts(symbol, true);
        }
        if flags.intersects(SymbolFlags::Class) {
            return self.named_type_symbol_display_parts(
                symbol,
                "class",
                DisplayPartKind::ClassName,
            );
        }
        if flags.intersects(SymbolFlags::Interface) {
            return self.named_type_symbol_display_parts(
                symbol,
                "interface",
                DisplayPartKind::InterfaceName,
            );
        }
        if flags.intersects(SymbolFlags::ENUM) {
            let mut parts = Vec::new();
            push_keyword(&mut parts, "enum");
            push_space(&mut parts, " ");
            push_part(&mut parts, &symbol.name, DisplayPartKind::EnumName);
            return parts;
        }
        // 合并符号（值+类型）在值上下文显示值身份（tsc 按访问意义选择）
        if flags.intersects(SymbolFlags::TypeAlias) && !flags.intersects(SymbolFlags::VALUE) {
            return self.type_alias_symbol_display_parts(symbol);
        }
        if flags.intersects(SymbolFlags::TypeParameter) {
            let mut parts = Vec::new();
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "type parameter", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
            parts.extend(self.type_parameter_symbol_display_parts(symbol));
            return parts;
        }
        if flags.intersects(SymbolFlags::EnumMember) {
            let mut parts = Vec::new();
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "enum member", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
            push_part(
                &mut parts,
                &self.qualified_symbol_name(symbol),
                DisplayPartKind::EnumName,
            );
            let t = self.get_type_of_symbol(symbol);
            let literal = match &t.data {
                crate::checker::types::TypeData::Literal(lit) => Some(lit.value.clone()),
                _ => None,
            };
            if let Some(value) = literal {
                push_space(&mut parts, " = ");
                push_part(&mut parts, &value.to_string(), DisplayPartKind::StringLiteral);
            } else {
                push_space(&mut parts, ": ");
                parts.extend(self.type_to_display_parts(&t));
            }
            return parts;
        }
        if flags.intersects(SymbolFlags::VARIABLE)
            || flags.intersects(SymbolFlags::Property)
            || flags.intersects(SymbolFlags::ACCESSOR)
        {
            return self.variable_symbol_display_parts(symbol);
        }
        if flags.intersects(SymbolFlags::MODULE) || flags.intersects(SymbolFlags::NamespaceModule) {
            let mut parts = Vec::new();
            push_keyword(&mut parts, "module");
            push_space(&mut parts, " ");
            push_part(&mut parts, &symbol.name, DisplayPartKind::Text);
            return parts;
        }
        if flags.intersects(SymbolFlags::Alias) {
            let mut parts = Vec::new();
            push_keyword(&mut parts, "import");
            push_space(&mut parts, " ");
            push_part(&mut parts, &symbol.name, DisplayPartKind::Text);
            return parts;
        }

        let mut parts = Vec::new();
        push_part(&mut parts, &symbol.name, DisplayPartKind::VariableName);
        push_space(&mut parts, ": ");
        let t = self.get_type_of_symbol(symbol);
        parts.extend(self.type_to_display_parts(&t));
        parts
    }

    fn doc_lookup_symbols(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        let mut targets = vec![Arc::clone(symbol)];
        if let Some(container) = self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.container_symbol.clone())
        {
            if let Some(st) = self
                .resolve_interface_type_ex(&container, None)
                .as_structured()
                && let Some(member) = st.members.get(&symbol.name)
            {
                targets.push(Arc::clone(member));
            } else if let Some(member) = container.members.entries.get(&symbol.name) {
                targets.push(Arc::clone(member));
            }
        }
        targets
    }

    /// {@link name} -> [name](file:///{file}#{l},{c}-{l2},{c2})（tsc hover 文档链接形态）
    fn render_jsdoc_links(
        &mut self,
        sf: &tsox_frontend::ast::SourceFile,
        decl: &Arc<Node>,
        text: &str,
    ) -> String {
        let mut out = String::new();
        let mut rest = text;
        while let Some(i) = rest.find("{@link ") {
            out.push_str(&rest[..i]);
            let after = &rest[i + 7..];
            let Some(end) = after.find('}') else { break };
            let inner = &after[..end];
            let name = inner.split_whitespace().next().unwrap_or(inner);
            let target = self
                .resolve_name_from(sf, decl, name)
                .and_then(|s| s.declarations.first().cloned());
            match target {
                Some(t) => {
                    // Go writeNameLink：链接指向目标声明的名字节点，URI 取目标
                    // 所在文件（非悬停文件）
                    let target_sf = match self.get_source_file_of_node(&t) {
                        Some(f) => f,
                        None => {
                            out.push_str(name);
                            rest = &after[end + 1..];
                            continue;
                        }
                    };
                    let name_start = t.name().map(|n| n.pos()).unwrap_or_else(|| t.pos());
                    let name_end = t.name().map(|n| n.end()).unwrap_or_else(|| t.end());
                    let (l1, c1, l2, c2) =
                        node_line_char(&target_sf, name_start, name_end);
                    out.push_str(&format!(
                        "[{name}](file:///{}#{l1},{c1}-{l2},{c2})",
                        target_sf.file_name.trim_start_matches('/'),
                    ));
                }
                None => out.push_str(name),
            }
            rest = &after[end + 1..];
        }
        out.push_str(rest);
        out
    }

    fn resolve_name_from(
        &self,
        _sf: &tsox_frontend::ast::SourceFile,
        decl: &Arc<Node>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        let mut cur = decl.parent();
        while let Some(n) = cur {
            if let Some(locals) = symbol_map.locals.get(&n.id())
                && let Some(sym) = locals.get(name)
            {
                return Some(Arc::clone(sym));
            }
            if let Some(container_sym) = symbol_map.symbols.get(&n.id())
                && let Some(sym) = container_sym.members.get(name)
            {
                return Some(Arc::clone(sym));
            }
            cur = n.parent();
        }
        self.globals.get(name).cloned()
    }

    // Go getDocumentationForSymbol：先取调用位解析签名的声明文档（call/construct 签名），
    // 再回退符号声明文档
    pub(crate) fn hover_documentation(&mut self, symbol: &Arc<Symbol>, node: &Arc<Node>) -> String {
        if let Some(call) = Self::enclosing_call_or_new(node)
            && let Some(sig) = self.resolved_call_signature(&call)
            && let Some(decl) = sig.declaration.clone()
            && matches!(decl.kind, SyntaxKind::CallSignature | SyntaxKind::ConstructSignature)
        {
            let doc = self.declaration_jsdoc_text(&decl);
            if !doc.is_empty() {
                return doc;
            }
        }
        self.symbol_documentation(symbol)
    }

    pub(crate) fn enclosing_call_or_new(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut cur = Arc::clone(node);
        // 仅当节点是属性访问的「名字」段（被调函数）时穿透到访问表达式；
        // object 段（如 p1.then 里的 p1）不是被调函数
        if cur.parent().as_ref().map(|p| p.kind) == Some(SyntaxKind::PropertyAccessExpression) {
            let pae = cur.parent().expect("checked Some above");
            if let crate::checker::nodebuilder::NodeData::PropertyAccessExpression(d) = &pae.data
                && Arc::ptr_eq(&d.name, &cur)
            {
                cur = pae;
            }
        }
        let parent = cur.parent()?;
        match parent.kind {
            SyntaxKind::CallExpression => {
                let is_callee = match &parent.data {
                    crate::checker::nodebuilder::NodeData::CallExpression(d) => {
                        Arc::ptr_eq(&d.expression, &cur)
                    }
                    _ => false,
                };
                if is_callee { Some(parent) } else { None }
            }
            SyntaxKind::NewExpression => Some(parent),
            _ => None,
        }
    }

    pub(crate) fn resolved_call_signature(&mut self, call: &Arc<Node>) -> Option<Arc<Signature>> {
        let (callee, args) = match &call.data {
            crate::checker::nodebuilder::NodeData::CallExpression(d) => {
                (&d.expression, d.arguments.clone())
            }
            crate::checker::nodebuilder::NodeData::NewExpression(d) => {
                (&d.expression, d.arguments.clone().unwrap_or_default())
            }
            _ => return None,
        };
        let callee_type = self.get_type_of_node(callee);
        let structured = callee_type.as_structured()?;
        let sigs = if call.kind == SyntaxKind::NewExpression {
            structured.construct_signatures()
        } else {
            structured.call_signatures()
        };
        if sigs.is_empty() {
            return None;
        }
        let idx = if sigs.len() == 1 {
            0
        } else {
            self.find_matching_signature(call, sigs, &args)
        };
        Some(Arc::clone(&sigs[idx]))
    }

    pub(crate) fn declaration_jsdoc_text(&mut self, decl: &Arc<Node>) -> String {
        let Some(sf) = self.get_source_file_of_node(decl) else {
            return String::new();
        };
        // Reparsed 的 JSDoc 合成成员（@property → PropertySignature）：
        // 注释挂在原 jsdoc 标签上，按位置回查标签注释
        if decl.flags.contains(tsox_frontend::ast::NodeFlags::Reparsed) {
            if decl.kind == SyntaxKind::TypeAliasDeclaration {
                // @typedef 重解析的别名：文档为原 JSDoc 块整体（块首注释 +
                // 各标签 "*@name* — comments"，跳过 type/typedef/callback）
                if let Some(doc) = jsdoc_block_doc(&sf, decl) {
                    return doc;
                }
            }
            let doc = self.jsdoc_tag_comment_at(&sf, decl);
            if !doc.is_empty() {
                return doc;
            }
        }
        let jds = tsox_frontend::parser::parse_jsdoc_for_node(&sf, decl);
        for jd in &jds {
            let (p0, p1) = (jd.pos().min(sf.text.len()), jd.end().min(sf.text.len()));
            let raw = if p0 < p1 { sf.text[p0..p1].to_string() } else { String::new() };
            let cleaned = clean_jsdoc_text(&raw);
            if !cleaned.is_empty() {
                return self.render_jsdoc_links(&sf, decl, &cleaned);
            }
        }
        String::new()
    }

    pub(crate) fn symbol_documentation(&mut self, symbol: &Arc<Symbol>) -> String {
        let direct = self.symbol_documentation_direct(symbol);
        if !direct.is_empty() {
            return direct;
        }
        // Go ls/jsdoc.go getJSDocOrTag：声明无 JSDoc 且父节点为类/接口时，
        // 沿基类型找同名成员的 JSDoc（mixin 覆盖场景）
        self.base_member_documentation(symbol)
    }

    fn symbol_documentation_direct(&mut self, symbol: &Arc<Symbol>) -> String {
        // Reparsed 的 JSDoc 合成成员（@property）：注释在原 jsdoc 标签上
        for target in self.doc_lookup_symbols(symbol) {
            for decl in &target.declarations {
                if decl.flags.contains(tsox_frontend::ast::NodeFlags::Reparsed) {
                    if let Some(sf) = self.get_source_file_of_node(decl) {
                        if decl.kind == SyntaxKind::TypeAliasDeclaration {
                            // @typedef 重解析的别名：文档为原 JSDoc 块整体
                            if let Some(doc) = jsdoc_block_doc(&sf, decl) {
                                return doc;
                            }
                        }
                        let doc = self.jsdoc_tag_comment_at(&sf, decl);
                        if !doc.is_empty() {
                            return doc;
                        }
                    }
                }
            }
        }
        for target in self.doc_lookup_symbols(symbol) {
            for decl in &target.declarations {
                let Some(sf) = self.get_source_file_of_node(decl) else {
                    continue;
                };
                // 声明层无 jsdoc 时上探到语句层（var 声明的文档挂在 VariableStatement）
                let mut jds = tsox_frontend::parser::parse_jsdoc_for_node(&sf, decl);
                if jds.is_empty() {
                    let mut p = decl.parent();
                    while let Some(n) = p.as_ref()
                        && n.kind != SyntaxKind::VariableStatement
                    {
                        p = n.parent();
                    }
                    if let Some(stmt) = p {
                        jds = tsox_frontend::parser::parse_jsdoc_for_node(&sf, &stmt);
                    }
                }
                for jd in &jds {
                    let (p0, p1) = (jd.pos().min(sf.text.len()), jd.end().min(sf.text.len()));
                    let raw: String = if p0 < p1 {
                        sf.text[p0..p1].to_string()
                    } else {
                        String::new()
                    };
                    let cleaned = clean_jsdoc_text(&raw);
                    if !cleaned.is_empty() {
                        return self.render_jsdoc_links(&sf, decl, &cleaned);
                    }
                    if let crate::checker::nodebuilder::NodeData::JSDoc(data) = &jd.data {
                        let parsed: String = data
                            .comment
                            .iter()
                            .map(|n| match &n.data {
                                crate::checker::nodebuilder::NodeData::JSDocText(td) => {
                                    td.text.join("")
                                }
                                _ => String::new(),
                            })
                            .collect::<Vec<_>>()
                            .join("");
                        let parsed = parsed.trim();
                        if !parsed.is_empty() {
                            return self.render_jsdoc_links(&sf, decl, parsed);
                        }
                    }
                }
            }
        }
        String::new()
    }

    pub(crate) fn constructor_jsdoc_documentation(&mut self, class_symbol: &Arc<Symbol>) -> String {
        let ctor_decl = class_symbol.declarations.iter().find_map(|decl| {
            let members = match &decl.data {
                tsox_frontend::ast::NodeData::ClassDeclaration(d) => Some(&d.members),
                tsox_frontend::ast::NodeData::ClassExpression(d) => Some(&d.members),
                _ => None,
            }?;
            members
                .iter()
                .find(|m| m.kind == SyntaxKind::Constructor)
                .cloned()
        });
        let Some(ctor_decl) = ctor_decl else {
            return String::new();
        };
        let Some(sf) = self.get_source_file_of_node(&ctor_decl) else {
            return String::new();
        };
        let jds = tsox_frontend::parser::parse_jsdoc_for_node(&sf, &ctor_decl);
        for jd in &jds {
            let (p0, p1) = (jd.pos().min(sf.text.len()), jd.end().min(sf.text.len()));
            let raw: String = if p0 < p1 {
                sf.text[p0..p1].to_string()
            } else {
                String::new()
            };
            let cleaned = clean_jsdoc_text(&raw);
            if !cleaned.is_empty() {
                return self.render_jsdoc_links(&sf, &ctor_decl, &cleaned);
            }
        }
        String::new()
    }

    /// 成员符号无 parent 链接时，沿声明树上溯类/接口声明节点取容器符号
    fn container_symbol_from_declarations(
        &self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        symbol.declarations.first().and_then(|decl| {
            let mut cur = decl.parent();
            while let Some(n) = cur {
                match n.kind {
                    // 类型字面量成员无可限定容器（Go getContainersOfSymbol：
                    // type literal 无符号，匿名成员裸名显示）
                    SyntaxKind::TypeLiteral => return None,
                    SyntaxKind::ClassDeclaration
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::ModuleDeclaration
                    | SyntaxKind::SourceFile => {
                        return symbol_map.symbol_of(&n).map(Arc::clone);
                    }
                    _ => cur = n.parent(),
                }
            }
            None
        })
    }

    /// 合并 namespace+class 容器段：Class&lt;T extends C = D&gt; 形态（约束/默认值经解析渲染）
    fn merged_class_container_segment(&mut self, parent: &Arc<Symbol>) -> Option<String> {
        let tps = parent.declarations.iter().find_map(|d| match &d.data {
            crate::checker::nodebuilder::NodeData::ClassDeclaration(data) => {
                data.type_parameters.as_ref()
            }
            _ => None,
        })?;
        let parts: Vec<String> = tps
            .iter()
            .filter_map(|tp| {
                let crate::checker::nodebuilder::NodeData::TypeParameterDeclaration(tpd) = &tp.data
                else {
                    return None;
                };
                let mut s = tpd.name.text().to_string();
                if let Some(c) = &tpd.constraint {
                    let t = self.get_type_from_type_node(c);
                    let text = self.type_to_string_ex(
                        &t,
                        crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE,
                    );
                    s.push_str(&format!(" extends {text}"));
                }
                if let Some(dflt) = &tpd.default_type {
                    let t = self.get_type_from_type_node(dflt);
                    let text = self.type_to_string_ex(
                        &t,
                        crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE,
                    );
                    s.push_str(&format!(" = {text}"));
                }
                Some(s)
            })
            .collect();
        if parts.is_empty() {
            return None;
        }
        Some(format!("{}<{}>", parent.name, parts.join(", ")))
    }

    pub(crate) fn qualified_symbol_name(&mut self, symbol: &Arc<Symbol>) -> String {
        // Go getNameOfSymbolAsWritten：内部名符号按声明形态显示匿名占位
        if symbol.name.starts_with('\u{FE}') {
            if let Some(display) = anonymous_symbol_display_name(symbol) {
                return display;
            }
        }
        let parent = symbol
            .parent()
            .clone()
            .or_else(|| {
                self.value_symbol_links
                    .get(symbol)
                    .and_then(|l| l.container_symbol.clone())
            })
            .or_else(|| self.container_symbol_from_declarations(symbol));
        let Some(parent) = parent else {
            return symbol.name.clone();
        };
        if parent.flags.contains(SymbolFlags::ValueModule) {
            // 合并 namespace+class 符号：类声明的类型参数（含约束/默认值）
            // 挂在最内层段（AMap.MassMarks<D extends ... = ...>.Data）
            if parent.flags.contains(SymbolFlags::Class)
                && let Some(seg) = self.merged_class_container_segment(&parent)
                && let Some(q) = self.namespace_qualifier_of(symbol)
                && let Some(pos) = q.rfind('.')
            {
                return format!("{}.{seg}.{}", &q[..pos], symbol.name);
            }
            return self
                .namespace_qualifier_of(symbol)
                .map(|q| format!("{q}.{}", symbol.name))
                .unwrap_or_else(|| symbol.name.clone());
        }
        if !(parent.flags.intersects(SymbolFlags::Interface)
            || parent.flags.intersects(SymbolFlags::Class)
            || parent.flags.intersects(SymbolFlags::ENUM))
        {
            return symbol.name.clone();
        }
        let mut q = parent.name.clone();
        if let Some(args) = self.instantiated_member_container_args(symbol, &parent) {
            let rendered: Vec<String> = args.iter().map(|a| self.type_to_string(a)).collect();
            if !rendered.is_empty() {
                q.push('<');
                q.push_str(&rendered.join(", "));
                q.push('>');
            }
            q.push('.');
            q.push_str(&symbol.name);
            return q;
        }
        if let Some(tps) = parent.declarations.iter().find_map(|d| match &d.data {
            crate::checker::nodebuilder::NodeData::InterfaceDeclaration(data) => {
                data.type_parameters.as_ref()
            }
            crate::checker::nodebuilder::NodeData::ClassDeclaration(data) => {
                data.type_parameters.as_ref()
            }
            _ => None,
        }) {
            let names: Vec<String> = tps
                .iter()
                .filter_map(|tp| match &tp.data {
                    crate::checker::nodebuilder::NodeData::TypeParameterDeclaration(tpd) => {
                        Some(tpd.name.text().to_string())
                    }
                    _ => None,
                })
                .collect();
            if !names.is_empty() {
                q.push('<');
                q.push_str(&names.join(", "));
                q.push('>');
            }
        }
        q.push('.');
        q.push_str(&symbol.name);
        q
    }

    pub fn type_to_display_parts(&mut self, t: &Arc<Type>) -> Vec<SymbolDisplayPart> {
        use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
        let s = self.type_to_string_ex(
            t,
            TypeFormatFlags::MULTILINE_OBJECT_LITERALS
                .union(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE),
        );

        if let Some(name) = t.intrinsic_name() {
            if is_keyword_type_name(name) {
                return vec![SymbolDisplayPart::new(s, DisplayPartKind::Keyword)];
            }
        }

        if let Some(sym) = &t.symbol {
            return vec![SymbolDisplayPart::new(s, display_kind_for_symbol(sym))];
        }

        vec![SymbolDisplayPart::new(s, DisplayPartKind::Text)]
    }

    pub(crate) fn function_symbol_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
        is_method: bool,
    ) -> Vec<SymbolDisplayPart> {
        let mut parts: Vec<SymbolDisplayPart> = Vec::new();
        if is_method {
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "method", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
        } else {
            push_keyword(&mut parts, "function");
            push_space(&mut parts, " ");
        }
        push_part(
            &mut parts,
            &self.qualified_symbol_name(symbol),
            DisplayPartKind::FunctionName,
        );
        self.append_type_parameter_parts(&mut parts, symbol);

        let t = self.get_type_of_symbol(symbol);
        if let Some(structured) = t.as_structured() {
            if let Some(sig) = structured.call_signatures().first() {
                push_punctuation(&mut parts, "(");
                self.append_signature_parameter_parts(&mut parts, sig);
                push_punctuation(&mut parts, ")");
                self.append_signature_return_parts(&mut parts, sig);
                return parts;
            }
        }

        push_space(&mut parts, ": ");
        parts.extend(self.type_to_display_parts(&t));
        parts
    }

    pub(crate) fn named_type_symbol_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
        keyword: &'static str,
        name_kind: DisplayPartKind,
    ) -> Vec<SymbolDisplayPart> {
        let mut parts = Vec::new();
        push_keyword(&mut parts, keyword);
        push_space(&mut parts, " ");
        push_part(&mut parts, &self.qualified_symbol_name(symbol), name_kind);
        self.append_type_parameter_parts(&mut parts, symbol);
        parts
    }

    pub(crate) fn type_alias_symbol_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Vec<SymbolDisplayPart> {
        let mut parts = Vec::new();
        push_keyword(&mut parts, "type");
        push_space(&mut parts, " ");
        push_part(&mut parts, &symbol.name, DisplayPartKind::Text);
        self.append_type_parameter_parts(&mut parts, symbol);
        push_space(&mut parts, " = ");
        if let Some(t) = self.try_get_type_alias_declared_type(symbol) {
            parts.extend(self.type_to_display_parts(&t));
        }
        parts
    }

    pub(crate) fn type_parameter_symbol_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Vec<SymbolDisplayPart> {
        let mut parts = Vec::new();
        push_part(&mut parts, &symbol.name, DisplayPartKind::TypeParameterName);
        if let Some(c) = self.get_constraint_of_type_parameter_symbol(symbol) {
            push_keyword(&mut parts, " extends ");
            parts.extend(self.type_to_display_parts(&c));
        }
        // 上下文：`(type parameter) T in type X<T = string>`（对齐 Go hover writeSymbol）
        if let Some(owner) = symbol
            .declarations
            .first()
            .and_then(|tp| tp.parent())
        {
            match owner.kind {
                SyntaxKind::TypeAliasDeclaration => {
                    let alias_sym = self.program.symbol_map().symbol_of(&owner).cloned();
                    push_keyword(&mut parts, " in ");
                    push_keyword(&mut parts, "type ");
                    if let Some(alias_sym) = alias_sym {
                        push_part(&mut parts, &alias_sym.name, DisplayPartKind::InterfaceName);
                        self.append_type_parameter_parts(&mut parts, &alias_sym);
                    }
                }
                SyntaxKind::ClassDeclaration | SyntaxKind::InterfaceDeclaration => {
                    let owner_sym = self.program.symbol_map().symbol_of(&owner).cloned();
                    push_keyword(&mut parts, " in ");
                    if let Some(owner_sym) = owner_sym {
                        push_part(&mut parts, &owner_sym.name, DisplayPartKind::ClassName);
                        self.append_type_parameter_parts(&mut parts, &owner_sym);
                    }
                }
                _ => {}
            }
        }
        parts
    }

    pub(crate) fn variable_prefix_and_name_parts(&mut self, symbol: &Arc<Symbol>) -> Vec<SymbolDisplayPart> {
        let mut parts = Vec::new();
        let is_parameter = symbol.declarations.iter().any(|d| {
            let mut cur = Some(d.clone());
            while let Some(n) = cur {
                match n.kind {
                    SyntaxKind::Parameter => return true,
                    SyntaxKind::BindingElement | SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern => {
                        cur = n.parent();
                    }
                    _ => break,
                }
            }
            false
        });
        if symbol.flags.intersects(SymbolFlags::ACCESSOR) {
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "accessor", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
        } else if symbol.flags.intersects(SymbolFlags::Property) {
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "property", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
        } else if symbol.declarations.iter().any(|d| {
            // 对象字面量纯 shorthand 成员：属性身份（tsc getSymbolAtLocationForQuickInfo）；
            // 解构赋值目标形态（{b = a}）不算
            d.kind == SyntaxKind::ShorthandPropertyAssignment
                && d.parent()
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
                && !matches!(&d.data, NodeData::ShorthandPropertyAssignment(sd) if sd.object_assignment_initializer.is_some() || sd.equals_token.is_some())
        }) {
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "property", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
        } else if is_parameter {
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "parameter", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
        } else {
            push_keyword(&mut parts, self.variable_decl_prefix(symbol).trim());
            push_space(&mut parts, " ");
        }

        let name_kind = if symbol
            .flags
            .intersects(SymbolFlags::Property | SymbolFlags::ACCESSOR)
        {
            DisplayPartKind::PropertyName
        } else {
            DisplayPartKind::VariableName
        };
        // 反向映射属性为合成符号：显示裸名（tsc 不经声明容器限定）
        let display_name = if symbol
            .check_flags
            .contains(tsox_frontend::ast::CheckFlags::ReverseMapped)
        {
            symbol.name.clone()
        } else {
            self.qualified_symbol_name(symbol)
        };
        push_part(&mut parts, &display_name, name_kind);
        if symbol.flags.contains(SymbolFlags::Optional) {
            push_punctuation(&mut parts, "?");
        }
        push_space(&mut parts, ": ");
        parts
    }

    pub(crate) fn variable_symbol_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Vec<SymbolDisplayPart> {
        // 先触发类型解析（binding_element_type 在此挂 container 链接）
        let _ = self.get_type_of_symbol(symbol);
        let mut parts = self.variable_prefix_and_name_parts(symbol);
        let t = self.get_type_of_symbol(symbol);
        parts.extend(self.type_to_display_parts(&t));
        parts
    }

    pub(crate) fn append_signature_parameter_parts(
        &mut self,
        parts: &mut Vec<SymbolDisplayPart>,
        sig: &Signature,
    ) {
        for (i, param) in sig.parameters.iter().enumerate() {
            if i > 0 {
                push_space(parts, ", ");
            }
            if i + 1 == sig.parameters.len() && sig.has_rest_parameter() {
                push_punctuation(parts, "...");
            }
            push_part(parts, &param.name, DisplayPartKind::ParameterName);
            // binder 不给参数符号设 Optional：从声明 question token / initializer 判定
            let declared_optional = param.declarations.iter().any(|d| {
                matches!(&d.data, tsox_frontend::ast::NodeData::ParameterDeclaration(pd) if pd.question_token.is_some() || pd.initializer.is_some())
            });
            if param.flags.contains(SymbolFlags::Optional) || declared_optional {
                push_punctuation(parts, "?");
            }
            push_space(parts, ": ");
            let pt = self
                .signature_instantiated_param_type(sig, i)
                .unwrap_or_else(|| self.get_type_of_symbol(param));
            // Go serializeTypeForDeclaration：解析类型与注解等价时复用注解原文（保留引号风格），
            // kind 分类仍按解析类型（number 等 intrinsic 为 Keyword）
            if let Some(text) = self.annotated_param_type_text(param, &pt) {
                let mut dp = self.type_to_display_parts(&pt);
                if let Some(first) = dp.first_mut() {
                    first.text = text;
                }
                parts.extend(dp);
            } else {
                parts.extend(self.type_to_display_parts(&pt));
            }
        }
    }

    /// 参数注解与解析类型等价（按类型 id）时返回注解源文本
    pub(crate) fn annotated_param_type_text(
        &mut self,
        param: &Arc<Symbol>,
        resolved: &Arc<Type>,
    ) -> Option<String> {
        let decl = param
            .value_declaration
            .as_ref()
            .or(param.declarations.first())?;
        let tn = match &decl.data {
            crate::checker::nodebuilder::NodeData::ParameterDeclaration(
                tsox_frontend::ast::ParameterDeclarationData {
                    type_node: Some(tn),
                    ..
                },
            ) => Arc::clone(tn),
            _ => return None,
        };
        self.equivalent_annotation_text(&tn, resolved)
    }

    /// 声明注解与解析类型等价（按类型 id）时返回注解源文本（保留引号风格）
    pub(crate) fn equivalent_annotation_text(
        &mut self,
        tn: &Arc<Node>,
        resolved: &Arc<Type>,
    ) -> Option<String> {
        let annotated = self.get_type_from_type_node(tn);
        if annotated.id != resolved.id {
            return None;
        }
        self.node_source_text(tn)
    }

    /// 变量/属性声明注解与解析类型等价时返回注解源文本
    pub(crate) fn annotated_decl_type_text(
        &mut self,
        symbol: &Arc<Symbol>,
        resolved: &Arc<Type>,
    ) -> Option<String> {
        let decl = symbol
            .value_declaration
            .as_ref()
            .or(symbol.declarations.first())?;
        let tn = match &decl.data {
            crate::checker::nodebuilder::NodeData::ParameterDeclaration(d) => {
                d.type_node.clone()?
            }
            crate::checker::nodebuilder::NodeData::VariableDeclaration(d) => {
                d.type_node.clone()?
            }
            crate::checker::nodebuilder::NodeData::PropertyDeclaration(d) => {
                d.type_node.clone()?
            }
            _ => return None,
        };
        self.equivalent_annotation_text(&tn, resolved)
    }

    pub(crate) fn call_site_union_signature(
        &mut self,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
    ) -> Option<Arc<Signature>> {
        let mut cur = Arc::clone(node);
        if cur.parent().as_ref().map(|p| p.kind) == Some(SyntaxKind::PropertyAccessExpression) {
            cur = cur.parent().expect("checked Some above");
        }
        let call = cur.parent()?;
        let callee_is_cur = match &call.data {
            tsox_frontend::ast::NodeData::CallExpression(d) => Arc::ptr_eq(&d.expression, &cur),
            _ => false,
        };
        if !callee_is_cur || call.kind != SyntaxKind::CallExpression {
            return None;
        }
        let t = self.get_type_of_symbol(symbol);
        if !t.is_union() {
            return None;
        }
        let TypeData::Union(u) = &t.data else {
            return None;
        };
        let lists: Vec<Vec<Arc<Signature>>> = u
            .union_or_intersection
            .types
            .iter()
            .map(|m| self.get_signatures_of_type(m, crate::checker::SignatureKind::Call))
            .collect();
        let sigs = self.get_union_signatures(&lists);
        if sigs.is_empty() {
            return None;
        }
        Some(Arc::clone(&sigs[0]))
    }

    pub(crate) fn arrow_signature_parts(&mut self, sig: &Arc<Signature>) -> Vec<SymbolDisplayPart> {
        let mut parts = Vec::new();
        if !sig.type_parameters.is_empty() {
            let names: Vec<String> = sig
                .type_parameters
                .iter()
                .filter_map(|tp| tp.symbol.as_ref().map(|s| s.name.clone()))
                .collect();
            if !names.is_empty() {
                push_punctuation(&mut parts, "<");
                push_part(&mut parts, &names.join(", "), DisplayPartKind::TypeParameterName);
                push_punctuation(&mut parts, ">");
            }
        }
        push_punctuation(&mut parts, "(");
        self.append_signature_parameter_parts(&mut parts, sig);
        push_punctuation(&mut parts, ")");
        push_space(&mut parts, " => ");
        let ret = self
            .get_return_type_of_signature(sig)
            .unwrap_or_else(|| self.any_type());
        parts.extend(self.type_to_display_parts(&ret));
        parts
    }

    pub(crate) fn append_type_parameter_parts(
        &self,
        parts: &mut Vec<SymbolDisplayPart>,
        symbol: &Arc<Symbol>,
    ) {
        if let Some(tps) = self.collect_type_parameter_displays(symbol) {
            if !tps.is_empty() {
                push_punctuation(parts, "<");
                for (i, tp) in tps.iter().enumerate() {
                    if i > 0 {
                        push_space(parts, ", ");
                    }
                    push_part(parts, tp, DisplayPartKind::TypeParameterName);
                }
                push_punctuation(parts, ">");
            }
        }
    }
}

fn node_is_descendant_of_expr(node: &Arc<Node>, ancestor: &Arc<Node>) -> bool {
    let mut cur = node.parent();
    while let Some(n) = cur {
        if Arc::ptr_eq(&n, ancestor) {
            return true;
        }
        cur = n.parent();
    }
    false
}

fn assignment_target_expr(obj: &Arc<Node>) -> Arc<Node> {
    // 向上找最近的 BinaryExpression=，返回其左操作数；找不到返回 obj 自身
    let mut cur = obj.parent();
    while let Some(n) = cur {
        match n.kind {
            SyntaxKind::ParenthesizedExpression => cur = n.parent(),
            SyntaxKind::BinaryExpression => {
                if let tsox_frontend::ast::NodeData::BinaryExpression(be) = &n.data {
                    if be.operator_token.kind == SyntaxKind::EqualsToken {
                        return Arc::clone(&be.left);
                    }
                }
                return Arc::clone(obj);
            }
            _ => return Arc::clone(obj),
        }
    }
    Arc::clone(obj)
}

pub(crate) fn node_in_with_block(node: &Arc<Node>) -> bool {
    // 遍历兄弟扫描祖先 WithStatement 的 span 包含（parser 可能不把 with body 挂进祖先链）
    let mut cur = node.parent();
    while let Some(n) = cur {
        if n.kind == SyntaxKind::WithStatement {
            return true;
        }
        if let Some(parent) = n.parent().as_ref() {
            let mut hit = false;
            tsox_frontend::ast::node_data_generated::for_each_child(parent, |sib| {
                if sib.kind == SyntaxKind::WithStatement
                    && sib.pos() <= node.pos()
                    && node.end() <= sib.end()
                {
                    hit = true;
                    return true;
                }
                false
            });
            if hit {
                return true;
            }
        }
        cur = n.parent();
    }
    false
}

impl Checker {
    /// 悬停命中改名绑定的 property_name：按源属性身份渲染（对齐 tsc checkBindingElement）
    pub(crate) fn binding_element_property_name_parts(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Option<Vec<SymbolDisplayPart>> {
        if let Some(elem) = node.parent().as_ref()
            && elem.kind == SyntaxKind::BindingElement
            && let tsox_frontend::ast::NodeData::BindingElement(d) = &elem.data
            && d.property_name
                .as_ref()
                .is_some_and(|pn| Arc::ptr_eq(pn, node))
        {
            let _ = self.get_type_of_symbol(&symbol);
            if let Some(member) = self
                .value_symbol_links
                .get(&symbol)
                .and_then(|l| l.container_symbol.clone())
            {
                return Some(self.variable_symbol_display_parts(&member));
            }
            // 无链接时按属性名在绑定类型中查源属性（如 property1: {} 嵌套解构）
            let declared_type = self.get_type_of_symbol(&symbol);
            if let Some(st) = declared_type.as_structured()
                && let Some(m) = st.members.get(&node.text())
            {
                return Some(self.variable_symbol_display_parts(&m));
            }
            // 名字是嵌套模式：属性类型即模式自身的形状
            let nested_pattern_type = match d.name.as_ref() {
                Some(name_node) if name_node.kind == SyntaxKind::ObjectBindingPattern => {
                    self.get_type_of_assignment_pattern(name_node)
                }
                _ => None,
            };
            let has_nested_pattern = nested_pattern_type.is_some();
            let be_type = nested_pattern_type.unwrap_or_else(|| Arc::clone(&declared_type));
            if !has_nested_pattern
                && let Some(st) = be_type.as_structured()
                && let Some(m) = st.members.get(&node.text())
            {
                return Some(self.variable_symbol_display_parts(&m));
            }
            let mut parts = Vec::new();
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "property", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
            push_part(&mut parts, &node.text(), DisplayPartKind::PropertyName);
            push_space(&mut parts, ": ");
            parts.extend(self.type_to_display_parts(&be_type));
            return Some(parts);
        }
        None
    }
}

fn anonymous_symbol_display_name(symbol: &Arc<Symbol>) -> Option<String> {
    let decl = symbol
        .value_declaration
        .as_ref()
        .or(symbol.declarations.first())?;
    let unnamed = match &decl.data {
        crate::checker::nodebuilder::NodeData::ClassExpression(d) => d.name.is_none(),
        crate::checker::nodebuilder::NodeData::FunctionExpression(d) => d.name.is_none(),
        crate::checker::nodebuilder::NodeData::ArrowFunction(_) => true,
        _ => false,
    };
    if !unnamed {
        return None;
    }
    match decl.kind {
        SyntaxKind::ClassExpression => Some("(Anonymous class)".to_string()),
        SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
            Some("(Anonymous function)".to_string())
        }
        _ => None,
    }
}

impl Checker {
    fn base_member_documentation(&mut self, symbol: &Arc<Symbol>) -> String {
        let decl = symbol
            .value_declaration
            .as_ref()
            .or(symbol.declarations.first())
            .cloned();
        let Some(decl) = decl else {
            return String::new();
        };
        let Some(class_node) = decl.parent().filter(|p| {
            matches!(p.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression)
        }) else {
            return String::new();
        };
        let Some(class_sym) = self
            .program
            .symbol_map()
            .symbol_of(&class_node)
            .map(Arc::clone)
        else {
            return String::new();
        };
        let is_static = decl
            .has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Static);
        let base_type: Option<Arc<Type>> = if is_static {
            let t = self.get_type_of_symbol(&class_sym);
            self.get_base_types(&t).into_iter().next()
        } else {
            match class_node.data {
                crate::checker::nodebuilder::NodeData::ClassDeclaration(_)
                | crate::checker::nodebuilder::NodeData::ClassExpression(_) => {
                    let inst = self.build_class_instance_type_with_base(&class_node);
                    self.get_base_types(&inst).into_iter().next()
                }
                _ => None,
            }
        };
        let Some(base) = base_type else {
            return String::new();
        };
        let Some(prop) = self.get_property_of_type(&base, &symbol.name) else {
            return String::new();
        };
        let Some(prop_decl) = prop
            .value_declaration
            .as_ref()
            .or(prop.declarations.first())
        else {
            return String::new();
        };
        let Some(sf) = self.get_source_file_of_node(prop_decl) else {
            return String::new();
        };
        let jds = tsox_frontend::parser::parse_jsdoc_for_node(&sf, prop_decl);
        for jd in &jds {
            let (p0, p1) = (jd.pos().min(sf.text.len()), jd.end().min(sf.text.len()));
            if p0 >= p1 {
                continue;
            }
            let raw = sf.text[p0..p1].to_string();
            let cleaned = clean_jsdoc_text(&raw);
            if !cleaned.is_empty() {
                return self.render_jsdoc_links(&sf, prop_decl, &cleaned);
            }
        }
        String::new()
    }
}

impl Checker {
    /// 位置落在 jsdoc 标签（@property 等）内的合成声明：取该标签的注释文本
    fn jsdoc_tag_comment_at(
        &mut self,
        sf: &Arc<tsox_frontend::ast::SourceFile>,
        decl: &Arc<Node>,
    ) -> String {
        use crate::checker::nodebuilder::NodeData;
        let comments = tsox_frontend::scanner::get_leading_comment_ranges(&sf.text, 0);
        for range in comments {
            let mut parser = tsox_frontend::parser::Parser::new(sf.text.clone());
            let jsdoc = parser.parse_jsdoc_comment(range.pos, range.end, range.pos);
            let Some(jsdoc) = jsdoc else {
                continue;
            };
            let NodeData::JSDoc(d) = &jsdoc.data else {
                continue;
            };
            let Some(tags) = &d.tags else {
                continue;
            };
            for tag in tags.nodes.iter() {
                if tag.loc.pos <= decl.loc.pos && decl.loc.end <= tag.loc.end {
                    if let crate::checker::nodebuilder::NodeData::JSDocParameterOrPropertyTag(
                        ptag,
                    ) = &tag.data
                    {
                        return jsdoc_comment_text(ptag.comment.as_ref());
                    }
                    return jsdoc_tag_trailing_comment(&sf.text, tag);
                }
            }
        }
        String::new()
    }
}

/// @tag 注释段：标签行尾起的连续注释文本（去除前导 * 与空白）
fn jsdoc_tag_trailing_comment(text: &str, tag: &crate::checker::nodebuilder::Node) -> String {
    let mut lines: Vec<String> = Vec::new();
    let body = &text[tag.loc.pos as usize..tag.loc.end as usize];
    for (i, raw) in body.split('\n').enumerate() {
        let line = raw.trim();
        let line = if i == 0 {
            // 首行：去掉 @tag 与名字/类型（启发式：首个空格后内容）
            match line.find(' ') {
                Some(sp) => line[sp + 1..].trim_start(),
                None => "",
            }
        } else {
            line.strip_prefix('*').map(|l| l.trim_start()).unwrap_or(line)
        };
        if !line.is_empty() {
            lines.push(line.to_string());
        }
    }
    lines.join("\n")
}

fn jsdoc_comment_text(comment: Option<&Arc<tsox_frontend::ast::NodeList>>) -> String {
    let Some(list) = comment else {
        return String::new();
    };
    let mut out = String::new();
    for part in list.nodes.iter() {
        if let crate::checker::nodebuilder::NodeData::JSDocText(t) = &part.data {
            out.push_str(&t.text.join(""));
        } else if part.kind != tsox_frontend::ast::SyntaxKind::JSDocText {
            break;
        }
    }
    out.trim().to_string()
}

/// Go getDocumentationFromDeclaration 对 reparse @typedef 别名的整块文档：
/// 块首注释 + 各标签 "*@name* — comments"（跳过 type/typedef/callback）
fn jsdoc_block_doc(sf: &Arc<tsox_frontend::ast::SourceFile>, decl: &Arc<Node>) -> Option<String> {
    let ranges = tsox_frontend::scanner::get_leading_comment_ranges(&sf.text, 0);
    for range in ranges {
        if !(range.pos <= decl.pos() && decl.end() <= range.end) {
            continue;
        }
        let mut parser = tsox_frontend::parser::Parser::new(sf.text.clone());
        let jsdoc = parser.parse_jsdoc_comment(range.pos, range.end, range.pos)?;
        if jsdoc.kind != tsox_frontend::ast::SyntaxKind::JSDoc {
            continue;
        }
        let crate::checker::nodebuilder::NodeData::JSDoc(d) = &jsdoc.data else {
            continue;
        };
        let mut out = jsdoc_comment_text(Some(&d.comment));
        if let Some(tags) = &d.tags {
            for tag in tags.nodes.iter() {
                if matches!(
                    tag.kind,
                    tsox_frontend::ast::SyntaxKind::JSDocTypeTag
                        | tsox_frontend::ast::SyntaxKind::JSDocTypedefTag
                        | tsox_frontend::ast::SyntaxKind::JSDocCallbackTag
                ) {
                    continue;
                }
                let Some(tag_name) = jsdoc_tag_name(tag) else {
                    continue;
                };
                out.push_str("\n\n*@");
                out.push_str(&tag_name);
                out.push('*');
                if matches!(
                    tag.kind,
                    tsox_frontend::ast::SyntaxKind::JSDocParameterTag
                        | tsox_frontend::ast::SyntaxKind::JSDocPropertyTag
                ) {
                    if let crate::checker::nodebuilder::NodeData::JSDocParameterOrPropertyTag(p) =
                        &tag.data
                    {
                        out.push_str(" `");
                        out.push_str(p.name.text());
                        out.push('`');
                    }
                }
                let comments = jsdoc_tag_comment(tag);
                if !comments.is_empty() {
                    out.push(' ');
                    if !comments.starts_with('-') {
                        out.push_str("— ");
                    }
                    out.push_str(&comments);
                }
            }
        }
        let out = out.trim().to_string();
        if !out.is_empty() {
            return Some(out);
        }
    }
    None
}

fn jsdoc_tag_name(tag: &Arc<Node>) -> Option<String> {
    let name = match &tag.data {
        crate::checker::nodebuilder::NodeData::JSDocUnknownTag(d) => &d.tag_name,
        crate::checker::nodebuilder::NodeData::JSDocParameterOrPropertyTag(d) => &d.tag_name,
        crate::checker::nodebuilder::NodeData::JSDocTemplateTag(d) => &d.tag_name,
        crate::checker::nodebuilder::NodeData::JSDocReturnTag(d) => &d.tag_name,
        crate::checker::nodebuilder::NodeData::JSDocTypeTag(d) => &d.tag_name,
        crate::checker::nodebuilder::NodeData::JSDocTypedefTag(d) => &d.tag_name,
        crate::checker::nodebuilder::NodeData::JSDocThrowsTag(d) => &d.tag_name,
        crate::checker::nodebuilder::NodeData::JSDocSeeTag(d) => &d.tag_name,
        crate::checker::nodebuilder::NodeData::JSDocDeprecatedTag(d) => &d.tag_name,
        _ => return None,
    };
    Some(name.text().to_string())
}

fn jsdoc_tag_comment(tag: &Arc<Node>) -> String {
    let comment = match &tag.data {
        crate::checker::nodebuilder::NodeData::JSDocUnknownTag(d) => d.comment.as_ref(),
        crate::checker::nodebuilder::NodeData::JSDocParameterOrPropertyTag(d) => d.comment.as_ref(),
        crate::checker::nodebuilder::NodeData::JSDocTemplateTag(d) => d.comment.as_ref(),
        crate::checker::nodebuilder::NodeData::JSDocReturnTag(d) => d.comment.as_ref(),
        crate::checker::nodebuilder::NodeData::JSDocDeprecatedTag(d) => d.comment.as_ref(),
        _ => None,
    };
    jsdoc_comment_text(comment)
}
