#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;

fn clean_jsdoc_text(raw: &str) -> String { ::tsox_core::fntrace::enter("clean_jsdoc_text"); 
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
) -> (usize, usize, usize, usize) { ::tsox_core::fntrace::enter("node_line_char"); 
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

pub(crate) fn is_declaration_name(parent: &Arc<Node>, name: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_declaration_name"); 
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
    pub(crate) fn resolve_property_access_symbol(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_property_access_symbol"); 
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

    fn property_from_union(&mut self, t: &Arc<Type>, name: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("property_from_union"); 
        if !t.is_union() {
            return None;
        }
        self.constituent_types(t)
            .into_iter()
            .find_map(|c| self.get_property_of_type(&c, name))
    }

    // x: C<number> 的成员访问显示：qualified 名带实例实参 + 方法/属性形态
    pub(crate) fn instantiated_member_access_parts(&mut self, node: &Arc<Node>) -> Option<Vec<SymbolDisplayPart>> { ::tsox_core::fntrace::enter("instantiated_member_access_parts"); 
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
                let class_sym = obj_type.symbol.clone();
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
        // 限定名：命名空间限定（含别名/纯类型命名空间段/export= 退化）+ 成员名
        let container_display = self.namespace_qualified_display_name(&display_class_sym);
        push_part(&mut parts, &container_display, DisplayPartKind::ClassName);
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
        // Go writeSymbol 用符号声明形态名：字符串字面量名带引号（"__proto__"）
        let member_display =
            self.symbol_name_as_written(&prop_sym)
                .unwrap_or_else(|| node.text().to_string());
        push_part(&mut parts, &member_display, DisplayPartKind::PropertyName);
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
                    self.get_signature_instantiation(sig, &inferred, &[])
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
    #[allow(dead_code)]
    pub(crate) fn call_return_function_symbol(
        &mut self,
        call: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("call_return_function_symbol"); 
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

    #[allow(dead_code)]
    fn find_first_return_expression(
        &self,
        node: &Arc<Node>,
        found: &mut Option<Arc<Node>>,
    ) { ::tsox_core::fntrace::enter("find_first_return_expression"); 
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
    pub(crate) fn call_inferred_type_arguments(&mut self, call: &Arc<Node>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("call_inferred_type_arguments"); 
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
    pub(crate) fn jsx_attribute_parts(&mut self, node: &Arc<Node>) -> Option<Vec<SymbolDisplayPart>> { ::tsox_core::fntrace::enter("jsx_attribute_parts"); 
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
        // Go writeSymbol 属性链限定（Foo.#privateProperty）+ 流收窄后的型
        let display_name = self.qualified_symbol_name(&prop);
        let declared = self.get_type_of_symbol(&prop);
        let prop_type = if p.kind == SyntaxKind::PropertyAccessExpression {
            self.flow_type_of_access_expression(&p, Some(&prop), declared)
        } else {
            declared
        };
        let mut parts = Vec::new();
        push_punctuation(&mut parts, "(");
        push_part(&mut parts, "property", DisplayPartKind::Text);
        push_punctuation(&mut parts, ") ");
        push_part(&mut parts, &display_name, DisplayPartKind::PropertyName);
        push_space(&mut parts, ": ");
        parts.extend(self.type_to_display_parts(&prop_type));
        Some(parts)
    }

    /// JSX 元素属性上下文类型（Go getContextualJsxElementAttributesType +
    /// inferJsxTypeArguments + chooseOverload 两阶段）：组件标签首个签名参数
    /// 类型，泛型按属性值推断实例化。阶段一跳过上下文敏感属性值固定已有
    /// 候选，阶段二以部分实例化参数为上下文重查（函数值属性参数定型后，
    /// 其返回型对剩余类型参数的推断可达）
    pub fn jsx_element_attributes_contextual_type(
        &mut self,
        elem: &Arc<Node>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("jsx_element_attributes_contextual_type"); 
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
        drop(attrs);
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
        // 函数组件（declare function Opt(attrs: Bag)）属性上下文 = 首个调用
        // 签名第 0 参（Go getContextualTypeForArgumentAtIndex）；类组件为构造
        // 签名第 0 参
        let sig = self
            .get_signatures_of_type(&ctor_type, SignatureKind::Call)
            .first()
            .cloned()
            .or_else(|| {
                self.get_signatures_of_type(&ctor_type, SignatureKind::Construct)
                    .first()
                    .cloned()
            })?;
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
                return Some(self.substitute_infer_type_parameters(
                    &param_type,
                    &sig.type_parameters,
                    &explicit,
                ));
            }
            if let Some((id, t)) = self.jsx_attr_ctx_guard.clone() {
                if id == elem.id() {
                    return Some(t);
                }
            }
            let inferences: Vec<crate::checker::inference::InferenceInfo> = sig
                .type_parameters
                .iter()
                .map(|p| crate::checker::inference::InferenceInfo::new(Arc::clone(p)))
                .collect();
            let mut context = crate::checker::inference::InferenceContext::new(inferences);
            context.signature = Some(Arc::clone(&sig));
            let saved_guard = self.jsx_attr_ctx_guard.take();
            // 阶段一（Go CheckModeSkipContextSensitive）：上下文敏感属性值
            // 折叠为 anyFunctionType，由非敏感属性值先固定候选
            self.jsx_attr_ctx_guard = Some((elem.id(), Arc::clone(&param_type)));
            let skip = self.create_jsx_attributes_type_phased(
                elem,
                Some(&param_type),
                crate::checker::jsx_props_attributes::JsxAttrsBuildMode::SkipContextSensitive,
            );
            self.infer_types(
                &mut context.inferences,
                Some(skip),
                Some(Arc::clone(&param_type)),
                crate::checker::inference::InferencePriority::None,
                false,
            );
            // Go fixTypeParameters：已有候选的类型参数先固定（isFixed），
            // 阶段二新候选不再并入（如函数值属性返回位的低优先候选）
            let fixed: Vec<Arc<Type>> = context
                .inferences
                .iter()
                .enumerate()
                .map(|(i, info)| {
                    if info.candidates.is_empty() && info.contra_candidates.is_empty() {
                        Arc::clone(&info.type_parameter)
                    } else {
                        self.get_inferred_type(&context, i)
                    }
                })
                .collect();
            for (info, t) in context.inferences.iter_mut().zip(fixed.iter()) {
                if !info.candidates.is_empty() || !info.contra_candidates.is_empty() {
                    info.is_fixed = true;
                    info.inferred_type = Some(Arc::clone(t));
                }
            }
            let inst_param =
                self.substitute_infer_type_parameters(&param_type, &sig.type_parameters, &fixed);
            // 阶段二（Go CheckModeNormal 二次推断）：参数型已定型的函数值
            // 属性此时定型，其返回型对剩余类型参数的推断自此可达
            self.jsx_attr_ctx_guard = Some((elem.id(), Arc::clone(&inst_param)));
            let full = self.create_jsx_attributes_type_phased(
                elem,
                Some(&inst_param),
                crate::checker::jsx_props_attributes::JsxAttrsBuildMode::GuardedPartial,
            );
            self.infer_types(
                &mut context.inferences,
                Some(full),
                Some(Arc::clone(&param_type)),
                crate::checker::inference::InferencePriority::None,
                false,
            );
            self.jsx_attr_ctx_guard = saved_guard;
            let inferred = self.get_inferred_types(&context);
            param_type =
                self.substitute_infer_type_parameters(&param_type, &sig.type_parameters, &inferred);
        }
        Some(param_type)
    }

    /// JSX 属性符号：构造签名参数类型（泛型按属性值推断实例化）上的成员，
    /// union 按 name 判别式收窄（Go GetPropertySymbolsFromContextualType）
    fn jsx_attribute_symbol_from_construct_signature(
        &mut self,
        _class_sym: &Arc<Symbol>,
        attr_node: &Arc<Node>,
        _name: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("jsx_attribute_symbol_from_construct_signature"); 
        let attrs = attr_node.parent()?;
        let elem = attrs.parent()?;
        let param_type = self.jsx_element_attributes_contextual_type(&elem)?;
        let syms = self.get_property_symbols_from_contextual_type(attr_node, &param_type, false);
        if syms.len() == 1 {
            return Some(Arc::clone(&syms[0]));
        }
        None
    }

    pub(crate) fn is_jsx_tag_name(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_jsx_tag_name"); 
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
    pub fn jsx_intrinsic_element_symbol(&mut self, name: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("jsx_intrinsic_element_symbol"); 
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
    pub(crate) fn symbol_name_as_written(&mut self, symbol: &Arc<Symbol>) -> Option<String> { ::tsox_core::fntrace::enter("symbol_name_as_written"); 
        let decl = symbol
            .value_declaration
            .as_ref()
            .or(symbol.declarations.first())?;
        let name_node = tsox_frontend::ast::node_data_generated::node_name(decl)?;
        self.node_source_text(name_node)
    }

    // 查全局 JSX 命名空间的 IntrinsicElements 属性，产出 (property) JSX.IntrinsicElements.div: any
    pub(crate) fn jsx_intrinsic_element_parts(&mut self, name: &str) -> Option<Vec<SymbolDisplayPart>> { ::tsox_core::fntrace::enter("jsx_intrinsic_element_parts"); 
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
        let _jsx_type = self.get_type_of_symbol(&jsx_sym);
        // declare namespace 的成员在 locals（ambient 不进 exports），沿声明手工下钻
        let _find_member = |checker: &Checker, sym: &Arc<Symbol>, member: &str| -> Option<Arc<Symbol>> {
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

    pub(crate) fn resolve_symbol_for_hover(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_symbol_for_hover"); 
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
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_contextual_property_symbol"); 
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
    ) -> Option<Vec<SymbolDisplayPart>> { ::tsox_core::fntrace::enter("constructor_display_parts"); 
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
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("instantiate_new_expression_return"); 
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
    ) -> Option<String> { ::tsox_core::fntrace::enter("new_expression_type_args_text"); 
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

    pub fn get_quick_info_display_parts(&mut self, node: &Arc<Node>) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("get_quick_info_display_parts"); 
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
            let inst = self.get_signature_instantiation(&sig, &inferred, &[]);
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
            let inst = self.get_signature_instantiation(&sig, &inferred, &[]);
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

    pub fn get_quick_info_text(&mut self, node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("get_quick_info_text"); 
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
    ) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("symbol_to_display_parts"); 
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

    fn doc_lookup_symbols(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("doc_lookup_symbols"); 
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
    ) -> String { ::tsox_core::fntrace::enter("render_jsdoc_links"); 
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
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_name_from"); 
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
    pub(crate) fn hover_documentation(&mut self, symbol: &Arc<Symbol>, node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("hover_documentation"); 
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

    pub(crate) fn enclosing_call_or_new(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("enclosing_call_or_new"); 
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

    pub(crate) fn resolved_call_signature(&mut self, call: &Arc<Node>) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("resolved_call_signature"); 
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

    pub(crate) fn declaration_jsdoc_text(&mut self, decl: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("declaration_jsdoc_text"); 
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

    pub(crate) fn symbol_documentation(&mut self, symbol: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("symbol_documentation"); 
        let direct = self.symbol_documentation_direct(symbol);
        if !direct.is_empty() {
            return direct;
        }
        // Go ls/jsdoc.go getJSDocOrTag：声明无 JSDoc 且父节点为类/接口时，
        // 沿基类型找同名成员的 JSDoc（mixin 覆盖场景）
        self.base_member_documentation(symbol)
    }

    fn symbol_documentation_direct(&mut self, symbol: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("symbol_documentation_direct"); 
        // JS 函数参数：文档来自所在函数 JSDoc 的同名 @param tag 注释
        if let Some(doc) = self.jsdoc_param_tag_documentation(symbol) {
            return doc;
        }
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

    pub(crate) fn constructor_jsdoc_documentation(&mut self, class_symbol: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("constructor_jsdoc_documentation"); 
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
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("container_symbol_from_declarations"); 
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
    fn merged_class_container_segment(&mut self, parent: &Arc<Symbol>) -> Option<String> { ::tsox_core::fntrace::enter("merged_class_container_segment"); 
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

    pub(crate) fn qualified_symbol_name(&mut self, symbol: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("qualified_symbol_name"); 
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
        // 容器自身嵌在命名空间内时先向上限定（Underscore.Static.all）；
        // 容器是父模块 export=（隔代自身）时链退化为限定名，不追加符号名
        let mut q = self.namespace_qualified_display_name(&parent);
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
            // Go typeParameterToNode：链段容器的类型参数按声明形态显示
            //（含约束与默认值，`D extends AMap.MassMarks.Data = AMap.MassMarks.Data`）
            let mut names: Vec<String> = Vec::new();
            for tp in tps.iter() {
                let crate::checker::nodebuilder::NodeData::TypeParameterDeclaration(tpd) = &tp.data
                else {
                    continue;
                };
                let mut seg = tpd.name.text().to_string();
                // Go symbolToTypeNode：约束/默认值里的类型引用按其符号的
                // 命名空间限定名显示（AMap.MassMarks.Data）
                let render_ref = |checker: &mut Self, node: &Arc<tsox_frontend::ast::Node>| -> String {
                    let t = checker.get_type_from_type_node(node);
                    if let Some(sym) = &t.symbol
                        && sym.flags.intersects(
                            SymbolFlags::Interface
                                | SymbolFlags::Class
                                | SymbolFlags::TypeAlias
                                | SymbolFlags::ENUM,
                        )
                    {
                        checker.namespace_qualified_display_name(sym)
                    } else {
                        checker.type_to_string(&t)
                    }
                };
                if let Some(c) = &tpd.constraint {
                    let s = render_ref(self, c);
                    seg.push_str(" extends ");
                    seg.push_str(&s);
                }
                if let Some(d) = &tpd.default_type {
                    let s = render_ref(self, d);
                    seg.push_str(" = ");
                    seg.push_str(&s);
                }
                names.push(seg);
            }
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

    pub fn type_to_display_parts(&mut self, t: &Arc<Type>) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("type_to_display_parts"); 
        use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
        self.type_to_display_parts_ex(
            t,
            TypeFormatFlags::MULTILINE_OBJECT_LITERALS
                .union(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE),
        )
    }

    pub(crate) fn type_to_display_parts_ex(
        &mut self,
        t: &Arc<Type>,
        flags: crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags,
    ) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("type_to_display_parts_ex"); 
        let s = self.type_to_string_ex(t, flags);

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
    ) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("function_symbol_display_parts"); 
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
    ) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("named_type_symbol_display_parts"); 
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
    ) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("type_alias_symbol_display_parts"); 
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
    ) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("type_parameter_symbol_display_parts"); 
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

    pub(crate) fn variable_prefix_and_name_parts(&mut self, symbol: &Arc<Symbol>) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("variable_prefix_and_name_parts"); 
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
        // 反向映射属性为合成符号：显示裸名（tsc 不经声明容器限定）；
        // 参数符号的父容器是函数（非 class/enum/module），Go SymbolToStringEx
        // 不限定 → 裸名
        let display_name = if is_parameter
            || symbol
                .check_flags
                .contains(tsox_frontend::ast::CheckFlags::ReverseMapped)
        {
            symbol.name.clone()
        } else {
            self.qualified_symbol_name(symbol)
        };
        // Go writeSymbol：字符串字面量声明名按书写形态带引号（"__proto__"）
        let display_name = if name_kind == DisplayPartKind::PropertyName
            && symbol
                .declarations
                .first()
                .and_then(|d| tsox_frontend::ast::node_data_generated::node_name(d))
                .is_some_and(|n| n.kind == SyntaxKind::StringLiteral)
        {
            self.symbol_name_as_written(symbol)
                .unwrap_or(display_name)
        } else {
            display_name
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
    ) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("variable_symbol_display_parts"); 
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
    ) { ::tsox_core::fntrace::enter("append_signature_parameter_parts"); 
        // Go getExpandedParameters：rest 参数类型为元组时按元素展开为具名
        // 参数序列（标签取元素 label，回退 rest 符号名_i）
        if let Some(expanded) = self.tuple_expanded_params(sig) {
            for (i, (name, ty, optional, variadic)) in expanded.iter().enumerate() {
                if i > 0 {
                    push_space(parts, ", ");
                }
                if *variadic {
                    push_punctuation(parts, "...");
                }
                push_part(parts, name, DisplayPartKind::ParameterName);
                if *optional {
                    push_punctuation(parts, "?");
                }
                push_space(parts, ": ");
                parts.extend(self.type_to_display_parts(ty));
            }
            return;
        }
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
            // Go serializeTypeForDeclaration：解析类型与注解等价时复用注解
            // （节点级复用经打印机呈现），kind 分类仍按解析类型（number 等
            // intrinsic 为 Keyword）
            if let Some(text) = self.annotated_param_type_text(
                param,
                &pt,
                Self::signature_params_instantiated(sig),
                TypeFormatFlags::MULTILINE_OBJECT_LITERALS
                    .union(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE),
            ) {
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

    /// 签名参数是否处于实例化形态：instantiateSignature 产物带 target
    ///（m1h.rs instantiate_signature_ex），实例化上下文（type_argument_stack
    /// 活跃）下自声明重建的签名带 instantiated_parameter_types 旁路
    ///（typenode_impl_chunk_checker_2.rs，Go 同位用 cloneSymbol 瞬态符号）
    pub(crate) fn signature_params_instantiated(sig: &Signature) -> bool {
        sig.target.is_some() || sig.instantiated_parameter_types.is_some()
    }

    /// 参数注解与解析类型等价（按类型 id）时返回注解类型的常规呈现
    pub(crate) fn annotated_param_type_text(
        &mut self,
        param: &Arc<Symbol>,
        resolved: &Arc<Type>,
        instantiated_signature: bool,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("annotated_param_type_text");
        // Go 复用门首条件 ctx.enclosingDeclaration != nil
        // （nodebuilderimpl.go:2380）：错误消息显示经 TypeToString(t, nil)
        //（printer.go:51-57）传 nil enclosing，门不成立按解析型渲染（可选
        // 注解带隐式 | undefined）；判定惯例同 reuse_declared_property_type
        if self.type_render_enclosing.is_none() && self.display_enclosing_node.is_none() {
            return None;
        }
        // 实例化签名参数按解析型渲染的门（9e34ecc70c2/0b2b2734aad）收窄为
        // 仅实参帧活跃的呈现窗：Go serializeTypeForDeclaration 的注解复用门
        //（nodebuilderimpl.go:2376-2404）不区分符号是否实例化——
        // pseudoTypeEquivalentToType（pseudotypenodebuilder.go:367）以声明
        // 注解伪型对解析型比较、可选声明剥合成 undefined 后判等价，等价即
        // 复用注解源文本。lib 方法（slice/subarray/initCustomEvent/
        // toLocaleString）参数注解 `number` 与实例化符号型 `number | undefined`
        // 在 strictNullChecks 下剥 undefined 恒等价，Go 恒呈 `number`；全量
        // 禁复用使 .types 呈 `number | undefined`（arrayToLocaleStringES2015
        // 行127、bigint64ArraySubarray 行15、controlFlowOuterVariable 行37、
        // customEventDetail 行10、discriminantsAndPrimitives 行182、
        // genericArrayMethods1 行7、library_ArraySlice 行7、
        // library_RegExpExecArraySlice 行10 等门基线新增 anchor）。帧活跃时
        // 注解节点经 get_type_from_type_node 会被帧解析成实例化态与解析型
        // 伪等价而复用源序注解（Promise.then onrejected 回归源，
        // 0b2b2734aad），故帧活跃维持按解析型渲染；无帧（.types 导出窗）
        // 注解按泛型声明态解析，onrejected（注解含 TResult2 泛型）与解析
        // 型恒失配走解析渲染，等价路只放行 lib 无型变量参数
        if (instantiated_signature
            || param
                .check_flags
                .contains(tsox_frontend::ast::CheckFlags::Instantiated))
            && !self.type_argument_stack.is_empty()
        {
            return None;
        }
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
        // 对象字面量注解同样复用（Go pseudoTypeToNode Direct → reuseTypeNode
        // 不区分注解节点种类，nodecopy.go:60），成员分隔符按打印机规范
        // （printer.go:2113 emitTypeLiteral + writeTrailingSemicolon）由
        // print_annotation_type_node 产出，不保留书写风格；TypeQuery 注解
        // 同走节点级复用（Go canReuseExistingJSTypeNode
        // nodebuilderimpl.go:528 无注解种类门，typeof 实体名经
        // annotation_has_inaccessible_reference 可及性判定）
        // Go 复用访问器（nodecopy getExistingNodeTreeVisitor → visitEntityName）
        // 对注解内每个实体名引用做 isSymbolAccessible 判定
        //（EmitResolver.hasVisibleDeclarations：声明须 Export/可见容器），
        // 任一不可及即整个复用失败，回退 typeToTypeNode 按 checker 类型渲染
        //——函数局部别名（如 callsOnComplexSignatures 的 stringType1）引用
        // 由此展开为底层类型而非保留源文本
        let enclosing = self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone());
        // 全局 Array/ReadonlyArray 引用注解：Go 复用访问器对其实体名的
        // isSymbolAccessible 判定恒过（globals 表直名命中
        // symbolaccessibility.go:588-592 + isDeclarationVisible 全局源文件恒真），
        // 结论不随发射上下文变化，直接进入复用判定；仅该类注解绕过通用门
        let annotated = self.get_type_from_type_node(&tn);
        let global_array_annotation = tn.kind == tsox_frontend::ast::SyntaxKind::TypeReference
            && self.annotation_type_node_is_global_array_reference(&annotated);
        if !global_array_annotation
            && self.annotation_has_inaccessible_reference(&tn, &enclosing)
        {
            return self.partial_reuse_inaccessible_annotation_text(&tn, &enclosing, flags);
        }
        // Go nodebuilder：可选参数注解复用的 ignoreUndefined 判据
        //（nodebuilderimpl.go:2365 requiresAddingUndefined →
        // requiresAddingImplicitUndefined，:2398 传入等价比较）仅在声明需
        // 补合成的 `| undefined`（注解未含 undefined）时成立；联合注解
        //（lib onfinally?: (() => void) | undefined | null）顶层 flags 为
        // Union 不含 Undefined 位，判 undefined 须入联合成员；注解本含
        // undefined 时解析型与注解型同型（getTypeFromTypeNode 节点缓存
        // 同对象）按 id 命中复用，剥解析侧反而必失配、复用恒败回退合成
        // 渲染丢源码序（pseudoTypeEquivalentToType 双侧对称，
        // nodebuilderimpl.go:2402）
        let annotation_contains_undefined = match &annotated.data {
            TypeData::Union(u) => u
                .union_or_intersection
                .types
                .iter()
                .any(|m| m.flags.intersects(TypeFlags::Undefined)),
            _ => annotated.flags.intersects(TypeFlags::Undefined),
        };
        let resolved = if annotation_contains_undefined {
            Arc::clone(resolved)
        } else {
            self.strip_param_optionality_undefined(param, resolved)
        };
        let requires_adding_undefined = matches!(
            decl.kind,
            SyntaxKind::Parameter
                | SyntaxKind::PropertySignature
                | SyntaxKind::PropertyDeclaration
        ) && {
            let resolver = self.get_emit_resolver();
            resolver.requires_adding_implicit_undefined(
                self,
                decl,
                Some(param),
                enclosing.as_ref(),
            )
        };
        let resolved = self.add_optionality_ex(&resolved, false, requires_adding_undefined);
        // Go serializeTypeForDeclaration 伪等价第三参 isOptionalAnnotated
        //（nodebuilderimpl.go:2398）= !requiresAddingUndefined && 三 kind &&
        // isOptionalDeclaration（utilities.go:338-341 纯 question token）：
        // 命中时等价先按剥 resolved 侧 undefined 比较
        //（pseudotypenodebuilder.go:378-393 getTypeWithFacts NEUndefined），
        // 注解自含 undefined 的形态仍走全型恒等臂（同 :395-403）
        let is_optional_annotated = !requires_adding_undefined
            && matches!(
                &decl.data,
                crate::checker::nodebuilder::NodeData::ParameterDeclaration(
                    tsox_frontend::ast::ParameterDeclarationData {
                        question_token: Some(_),
                        ..
                    }
                )
            );
        self.equivalent_annotation_text_ex(&tn, &resolved, flags, is_optional_annotated)
    }

    /// 复用门判不可及后的骨架部分复用（Go nodecopy 访问器
    /// serializeTypeName 对不可及实体返 nil 致整体复用失败，注解型按
    /// 求值形结构渲染；括号形态决定骨架取舍：
    /// (typeof e)['k'] 保骨架——对象段结构渲染、索引段保书写文本
    /// （declarationEmitResolveTypesIfNotReusable >f o2 形如
    /// ({ readonly a: ...; })['a']）；keyof (E['k']) 保 keyof 与括号、
    /// 操作数按索引访问求值型结构渲染（>f o4 形如 keyof ({ b: ...; })）；
    /// 无括号同形（typeof a['a']、A['a']['b']）整体解析不保骨架
    pub(crate) fn partial_reuse_inaccessible_annotation_text(
        &mut self,
        tn: &Arc<Node>,
        enclosing: &Option<Arc<Node>>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("partial_reuse_inaccessible_annotation_text");
        match tn.kind {
            tsox_frontend::ast::SyntaxKind::IndexedAccessType => {
                let (object_type, index_type) = match &tn.data {
                    tsox_frontend::ast::NodeData::IndexedAccessTypeNode(d) => {
                        (&d.object_type, &d.index_type)
                    }
                    _ => return None,
                };
                let inner = match &object_type.data {
                    tsox_frontend::ast::NodeData::ParenthesizedTypeNode(pd) => &pd.type_node,
                    _ => return None,
                };
                if inner.kind != tsox_frontend::ast::SyntaxKind::TypeQuery
                    || !self.annotation_has_inaccessible_reference(inner, enclosing)
                {
                    return None;
                }
                let entity_type = self.get_type_from_type_node(inner);
                let struct_text = self.type_to_string_ex(&entity_type, flags);
                let index_text = self.node_source_text(index_type)?;
                Some(format!("({})[{}]", struct_text, index_text))
            }
            tsox_frontend::ast::SyntaxKind::TypeOperator => {
                let (operator, operand) = match &tn.data {
                    tsox_frontend::ast::NodeData::TypeOperatorNode(d) => {
                        (&d.operator, &d.type_node)
                    }
                    _ => return None,
                };
                if *operator != tsox_frontend::ast::SyntaxKind::KeyOfKeyword {
                    return None;
                }
                let inner = match &operand.data {
                    tsox_frontend::ast::NodeData::ParenthesizedTypeNode(pd) => &pd.type_node,
                    _ => return None,
                };
                if inner.kind != tsox_frontend::ast::SyntaxKind::IndexedAccessType
                    || !self.annotation_has_inaccessible_reference(inner, enclosing)
                {
                    return None;
                }
                let operand_type = self.get_type_from_type_node(inner);
                let struct_text = self.type_to_string_ex(&operand_type, flags);
                Some(format!("keyof ({})", struct_text))
            }
            _ => None,
        }
    }

    /// 签名返回型注解复用（Go serializeReturnTypeForSignature
    /// nodebuilderimpl.go:2196-2226：真实声明的书写返回注解经伪检查器
    /// Direct 通道与签名返回型伪等价时，pseudoTypeToNodeWithCheckerFallback
    /// 复用注解节点呈现）。伪型来源 Go pseudochecker
    /// GetReturnTypeOfSignature（lookup.go:12-32）→ createReturnFromSignature
    ///（lookup.go:214-228）：函数型声明带书写注解即 Direct，无注解为
    /// Inferred/NoResult，伪等价不成立，回退结构渲染
    pub(crate) fn annotated_return_type_text(
        &mut self,
        sig: &Arc<Signature>,
        resolved: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("annotated_return_type_text");
        // Go 复用门首条件 ctx.enclosingDeclaration != nil
        // （nodebuilderimpl.go:2380）：错误消息显示经 TypeToString(t, nil)
        //（printer.go:51-57）传 nil enclosing，门不成立按解析型渲染——
        // 与 annotated_param_type_text 门同源，缺门时错误头函数返回注解
        // 复用书写形（qualify v4: ()=>K1.I3 应结构化短名 I3）
        if self.type_render_enclosing.is_none() && self.display_enclosing_node.is_none() {
            return None;
        }
        let decl = sig.declaration.as_ref()?;
        if tsox_frontend::ast::node_is_synthesized(decl) {
            return None;
        }
        if !tsox_frontend::ast::is_function_like(decl) {
            return None;
        }
        let tn = Arc::clone(decl.type_node()?);
        // 复用前可及性门与参数注解同源：Go 复用访问器对注解内实体名引用做
        // isSymbolAccessible 判定，任一不可及即整体回退结构渲染
        let enclosing = self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone());
        if self.annotation_has_inaccessible_reference(&tn, &enclosing) {
            return None;
        }
        if tn.kind == tsox_frontend::ast::SyntaxKind::ParenthesizedType {
            if self.annotation_contains_optional_type_node(&tn) {
                return self.node_source_text(&tn);
            }
            let annotated = self.get_type_from_type_node(&tn);
            if annotated.id != resolved.id {
                let annotated_str = self.type_to_string_ex(&annotated, flags);
                let resolved_str = self.type_to_string_ex(resolved, flags);
                if annotated_str != resolved_str {
                    return None;
                }
            }
            return self.print_annotation_type_node(&tn, flags);
        }
        self.equivalent_annotation_text(&tn, resolved, flags)
    }

    /// 注解子树内实体名引用是否存在对 enclosingDeclaration 不可及者
    pub(crate) fn annotation_has_inaccessible_reference(
        &mut self,
        node: &Arc<Node>,
        enclosing: &Option<Arc<Node>>,
    ) -> bool { ::tsox_core::fntrace::enter("annotation_has_inaccessible_reference");
        match node.kind {
            SyntaxKind::TypeReference | SyntaxKind::QualifiedName | SyntaxKind::TypeQuery => {
                // TypeQuery 同判：Go trackExistingEntityName（nodecopy.go:372）
                // 经 GetFirstIdentifier 对 typeof 操作数解名判可及
                //（declarationEmitResolveTypesIfNotReusable (typeof a)['a'] 的
                // a 为 decl.ts 模块私有 const，main.ts 上下文不可及须整体回落）；
                // 本侧子树遍历对 Identifier 不解名，须在此按型的别名符号优先判
                let t = self.get_type_from_type_node(node);
                // Go 复用访问器（nodecopy.go trackExistingEntityName:372
                // resolveEntityName(leftmost)）对书写实体名解出【名】符号：别名
                // 引用解为别名声明符号 A，IsSymbolAccessible 按其声明容器链判定。
                // 本侧原取型的 symbol 优先——别名解出型为别名体匿名对象，其
                // symbol 是类型字面量匿名符号（无容器链，恒 NotAccessible），
                // 别名引用注解（A['a']['b']、keyof (A['a'])）据此被误判不可及、
                // 复用失败回落解析渲染（declarationEmitResolveTypesIfNotReusable
                // >o1/>o4）。改为别名符号优先、型自身符号兜底
                let referenced = t
                    .alias
                    .as_ref()
                    .and_then(|a| a.symbol.clone())
                    .or_else(|| t.symbol.clone());
                if let Some(sym) = referenced
                    && !sym.flags.contains(SymbolFlags::TypeParameter)
                    && !self.annotation_symbol_in_global_script(&sym)
                    && !self.is_type_symbol_accessible(&sym, enclosing.as_ref())
                {
                    return true;
                }
            }
            _ => {}
        }
        let Some(sf) = self.get_source_file_of_node(node) else {
            return false;
        };
        let mut found = false;
        tsox_frontend::ast::mig::w3::for_each_child_and_js_doc(node, &sf, &mut |child| {
            if !found && self.annotation_has_inaccessible_reference(child, enclosing) {
                found = true;
            }
        });
        found
    }

    // 全局脚本文件顶层声明（lib/global scripts）程序内处处可见——Go
    // symbolaccessibility 链对全局符号终结于 SourceFile Visible 臂
    //（determineIfDeclarationIsVisible），等价快捷判定；含模块内声明的
    // 符号仍走完整 isTypeSymbolAccessible 链
    fn annotation_symbol_in_global_script(&mut self, sym: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("annotation_symbol_in_global_script");
        !sym.declarations.is_empty()
            && sym.declarations.iter().all(|d| {
                self.get_source_file_of_node(d)
                    .map(|sf| Self::is_global_source_file(&sf.node))
                    .unwrap_or(false)
            })
    }

    // Go nodebuilder 对箭头函数/函数表达式参数复用源注解打印：可选参数
    // 合成的 `| undefined` 不出现在输出里（注解自身含 undefined 除外）
    pub(crate) fn strip_param_optionality_undefined(
        &mut self,
        param: &Arc<Symbol>,
        t: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("strip_param_optionality_undefined"); 
        if !self.param_declared_optional(param) || !t.flags.contains(TypeFlags::Union) {
            return Arc::clone(t);
        }
        let Some(decl) = param.declarations.first() else {
            return Arc::clone(t);
        };
        let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &decl.data else {
            return Arc::clone(t);
        };
        if pd.type_node.is_none() {
            return Arc::clone(t);
        }
        // Go pseudoTypeEquivalentToType 的 ignoreUndefined 判据
        // （nodebuilderimpl.go:2402）覆盖全部可选 Parameter/PropertySignature/
        // PropertyDeclaration，不区分宿主（方法签名/函数形参同待遇）
        self.remove_undefined_from_union(t)
    }

    /// 声明注解与解析类型等价（按类型 id，或退而按常规呈现逐字相同）时
    /// 按节点级复用返回注解子树的打印机呈现；注解含后缀 `?` 元素时按
    /// Go 节点级复用返回注解源文本
    pub(crate) fn equivalent_annotation_text(
        &mut self,
        tn: &Arc<Node>,
        resolved: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("equivalent_annotation_text");
        self.equivalent_annotation_text_ex(tn, resolved, flags, false)
    }

    /// 解析侧型参被替换为注解侧型参的实例化克隆（TypeParameter target 指回
    /// 注解侧型参；instantiate_signature_ex 的 clone_type_parameter 与
    /// build_signature_from_function_like_type_node 的 member frame 克隆同型，
    /// 声明型参 target 恒 None）。克隆与原型参同名，呈现逐字相同，字符串
    /// 通道无法区分；Go pseudoTypeEquivalentToType（pseudotypenodebuilder.go:367-403）
    /// 无字符串通道，联合臂 compareTypesIdentical 对型参按同一对象判
    /// （relater.go isTypeParameterSource identity 臂 target === source 才
    /// True）恒判否，落 typeToTypeNode 解析渲染（formatUnionTypes printer.go:274-318
    /// 置 null 先 undefined 尾序）——lib then/catch onrejected 源序
    /// `| undefined | null` 应呈 `| null | undefined`（esModuleInteropImportCall
    /// 行14、implicitAnyGenericTypeInference 行92、modularizeLibrary_Dom/Worker.asynciterable
    /// 行6、truthinessPromiseCoercion 行78）
    fn annotation_type_params_replaced_by_clones(
        &mut self,
        annotated: &Arc<Type>,
        resolved: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("annotation_type_params_replaced_by_clones");
        let mut annotated_params: Vec<Arc<Type>> = Vec::new();
        self.collect_free_type_parameters_deep(annotated, &mut annotated_params);
        if annotated_params.is_empty() {
            return false;
        }
        let mut resolved_params: Vec<Arc<Type>> = Vec::new();
        self.collect_free_type_parameters_deep(resolved, &mut resolved_params);
        resolved_params.iter().any(|rp| {
            let crate::checker::types::TypeData::TypeParameter(tp) = &rp.data else {
                return false;
            };
            let Some(target) = tp.target.as_ref() else {
                return false;
            };
            if !target.flags.contains(crate::checker::types::TypeFlags::TypeParameter) {
                return false;
            }
            annotated_params.iter().any(|ap| Arc::ptr_eq(ap, target))
                && !annotated_params.iter().any(|ap| Arc::ptr_eq(ap, rp))
        })
    }

    pub(crate) fn equivalent_annotation_text_ex(
        &mut self,
        tn: &Arc<Node>,
        resolved: &Arc<Type>,
        flags: TypeFormatFlags,
        strip_resolved_undefined: bool,
    ) -> Option<String> { ::tsox_core::fntrace::enter("equivalent_annotation_text_ex");
        let annotated = self.get_type_from_type_node(tn);
        if annotated.id != resolved.id {
            if self.annotation_type_params_replaced_by_clones(&annotated, resolved) {
                return None;
            }
            let annotated_str = self.type_to_string_ex(&annotated, flags);
            if annotated_str != self.type_to_string_ex(resolved, flags) {
                // Go pseudoTypeEquivalentToType 的 isOptionalAnnotated 剥臂
                //（pseudotypenodebuilder.go:378-393）：可选注解参数解析型并入的
                // 隐式 undefined 不参与比较（undefinedStripped == typeFromPseudo）
                if !strip_resolved_undefined {
                    return None;
                }
                let stripped = self.remove_undefined_from_union(resolved);
                if annotated.id != stripped.id
                    && annotated_str != self.type_to_string_ex(&stripped, flags)
                {
                    return None;
                }
            }
        }
        // Go serializeTypeForDeclaration（nodebuilderimpl.go:2329）复用注解是
        // 节点级复用：伪等价时 pseudoTypeToNode（pseudotypenodebuilder.go:54）
        // Direct → reuseTypeNode（nodecopy.go:60）产出注解节点子树并经打印机
        // 输出（type_symbol_baseline.go:415）——联合/交集按书写成员序呈现，
        // 不按求值后 getUnionType 的 id 排序序。print_annotation_type_node
        // 按打印机规范结构化打印复合注解节点（联合/交集成员低于
        // TypeOperator 优先级加括号，printer.go:2195-2215 +
        // precedence.go:661），叶节点回退常规渲染——例外是后缀 `?` 元素：
        // 注解求值经 getTypeFromOptionalTypeNode（checker.go:25638）并入
        // undefined 而节点打印保留元注解（[number, string?] vs [number,
        // (string | undefined)?]），此节点类按源文本复用
        if self.annotation_contains_optional_type_node(tn) {
            return self.node_source_text(tn);
        }
        self.print_annotation_type_node(tn, flags)
    }

    /// 注解解为全局 Array/ReadonlyArray 的类型引用
    fn annotation_type_node_is_global_array_reference(&self, annotated: &Arc<Type>) -> bool {
        // Go t.Target() == globalArrayType（nodebuilderimpl.go:3167）按符号恒等
        // 落地：全局 Array/ReadonlyArray 符号全 checker 单实例（globals 表唯一
        // Arc），声明型与符号一一对应；目标类型对象在本仓存在多表示
        //（resolve_interface_type_ex 的 type_alias_links 槽、
        // declared_type_links 槽、subst 缓存异构实例），对象级 ptr_eq 不可靠。
        // 注解类型非引用形态（无 target）回退其自带符号
        let target_symbol = annotated
            .as_type_reference()
            .and_then(|d| d.target.as_ref().and_then(|t| t.symbol.clone()))
            .or_else(|| annotated.symbol.clone());
        let Some(target_symbol) = target_symbol else {
            return false;
        };
        let matches = |g: Option<&Arc<Type>>| {
            g.and_then(|gt| gt.symbol.clone())
                .is_some_and(|gs| gs.id() == target_symbol.id())
        };
        matches(self.global_array_type.get()) || matches(self.global_readonly_array_type.get())
    }

    fn annotation_contains_optional_type_node(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("annotation_contains_optional_type_node");
        if node.kind == SyntaxKind::OptionalType {
            return true;
        }
        let Some(sf) = self.get_source_file_of_node(node) else {
            return false;
        };
        let mut found = false;
        tsox_frontend::ast::mig::w3::for_each_child_and_js_doc(node, &sf, &mut |child| {
            if !found && self.annotation_contains_optional_type_node(child) {
                found = true;
            }
        });
        found
    }

    /// 注解子树节点级打印（Go reuseTypeNode（nodecopy.go:60）产出注解节点
    /// 子树经打印机输出）。结构节点按打印机规范递归：单行对象字面量成员
    /// 各自带尾分号、以空格分隔（printer.go:2113 emitTypeLiteral +
    /// LFSingleLineTypeLiteralMembers + writeTrailingSemicolon
    /// printer.go:1773/1881），空字面量 `{}`（LFNoSpaceIfEmpty）；叶节点按
    /// 常规 type_to_string_ex 呈现。任一子树不可打印则整体回退常规渲染
    pub(crate) fn print_annotation_type_node(
        &mut self,
        node: &Arc<Node>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("print_annotation_type_node");
        match node.kind {
            SyntaxKind::TypeLiteral => {
                let NodeData::TypeLiteralNode(d) = &node.data else {
                    return None;
                };
                if d.members.nodes.is_empty() {
                    return Some("{}".to_string());
                }
                let mut parts = Vec::with_capacity(d.members.nodes.len());
                for m in &d.members.nodes {
                    parts.push(self.print_annotation_type_member(m, flags)?);
                }
                Some(format!("{{ {} }}", parts.join(" ")))
            }
            SyntaxKind::UnionType => {
                let NodeData::UnionTypeNode(d) = &node.data else {
                    return None;
                };
                let mut parts = Vec::with_capacity(d.types.nodes.len());
                for t in &d.types.nodes {
                    parts.push(self.print_annotation_type_constituent(t, flags)?);
                }
                Some(parts.join(" | "))
            }
            SyntaxKind::IntersectionType => {
                let NodeData::IntersectionTypeNode(d) = &node.data else {
                    return None;
                };
                let mut parts = Vec::with_capacity(d.types.nodes.len());
                for t in &d.types.nodes {
                    parts.push(self.print_annotation_type_constituent(t, flags)?);
                }
                Some(parts.join(" & "))
            }
            SyntaxKind::ParenthesizedType => {
                let NodeData::ParenthesizedTypeNode(d) = &node.data else {
                    return None;
                };
                Some(format!(
                    "({})",
                    self.print_annotation_type_node(&d.type_node, flags)?
                ))
            }
            SyntaxKind::TypeQuery => {
                let NodeData::TypeQueryNode(d) = &node.data else {
                    return None;
                };
                if self.track_existing_entity_name_introduces_error(&d.expr_name) {
                    return None;
                }
                let mut s = format!("typeof {}", self.type_query_entity_name_text(&d.expr_name)?);
                if let Some(ta) = &d.type_arguments {
                    let mut parts = Vec::with_capacity(ta.nodes.len());
                    for t in &ta.nodes {
                        parts.push(self.print_annotation_type_node(t, flags)?);
                    }
                    s.push_str(&format!("<{}>", parts.join(", ")));
                }
                Some(s)
            }
            SyntaxKind::MappedType => {
                // Go reuseTypeNode（nodecopy.go:60）产出注解节点子树经
                // emitMappedType（printer.go:2306-2345）按书写形打印：
                // { readonly [K in 约束 as 名型][?]: 模板; }，模板联合保持书写成员序
                let NodeData::MappedTypeNode(d) = &node.data else {
                    return None;
                };
                let NodeData::TypeParameterDeclaration(tpd) = &d.type_parameter.data else {
                    return None;
                };
                let constraint = tpd.constraint.as_ref()?;
                let mut s = String::from("{ ");
                if d.readonly_token.is_some() {
                    s.push_str("readonly ");
                }
                s.push('[');
                s.push_str(tpd.name.text());
                s.push_str(" in ");
                s.push_str(&self.print_annotation_type_node(constraint, flags)?);
                s.push(']');
                if let Some(name_type) = &d.name_type {
                    s.push_str(" as ");
                    s.push_str(&self.print_annotation_type_node(name_type, flags)?);
                }
                if d.question_token.is_some() {
                    s.push('?');
                }
                if let Some(template) = &d.type_node {
                    s.push_str(": ");
                    s.push_str(&self.print_annotation_type_node(template, flags)?);
                }
                s.push_str("; }");
                Some(s)
            }
            SyntaxKind::FunctionType => {
                let NodeData::FunctionTypeNode(d) = &node.data else {
                    return None;
                };
                let mut s = String::new();
                if let Some(tps) = &d.type_parameters {
                    s.push_str(&self.print_annotation_type_parameters(tps, flags)?);
                }
                s.push('(');
                s.push_str(&self.print_annotation_params(&d.parameters, flags)?);
                s.push(')');
                if let Some(ret) = &d.type_node {
                    s.push_str(" => ");
                    s.push_str(&self.print_annotation_type_node(ret, flags)?);
                }
                Some(s)
            }
            SyntaxKind::ConstructorType => {
                let NodeData::ConstructorTypeNode(d) = &node.data else {
                    return None;
                };
                let mut s = Self::annotation_member_modifier_prefix(node);
                s.push_str("new ");
                if let Some(tps) = &d.type_parameters {
                    s.push_str(&self.print_annotation_type_parameters(tps, flags)?);
                }
                s.push('(');
                s.push_str(&self.print_annotation_params(&d.parameters, flags)?);
                s.push(')');
                if let Some(ret) = &d.type_node {
                    s.push_str(" => ");
                    s.push_str(&self.print_annotation_type_node(ret, flags)?);
                }
                Some(s)
            }
            SyntaxKind::InferType => {
                let NodeData::InferTypeNode(d) = &node.data else {
                    return None;
                };
                let NodeData::TypeParameterDeclaration(tp) = &d.type_parameter.data else {
                    return None;
                };
                let mut s = format!("infer {}", tp.name.text());
                if let Some(c) = &tp.constraint {
                    s.push_str(" extends ");
                    s.push_str(&self.print_annotation_type_node(c, flags)?);
                }
                Some(s)
            }
            SyntaxKind::TypeReference => {
                match self.reuse_type_reference_display_text(node, flags) {
                    Some(text) => Some(text),
                    None => self.node_source_text(node),
                }
            }
            SyntaxKind::ArrayType => {
                let NodeData::ArrayTypeNode(d) = &node.data else {
                    return None;
                };
                // Go emitArrayType（printer.go:2126-2134）→ emitPostfixTypeOperand
                //（printer.go:2131-2149）：解析树 TypeQuery 操作数免括号
                //（typeof C[K] 保持书写形），其余按 Postfix 优先级加括号
                let elem = self.print_annotation_type_node(&d.element_type, flags)?;
                let parens = d.element_type.kind != SyntaxKind::TypeQuery
                    && d.element_type.kind != SyntaxKind::MissingDeclaration
                    && tsox_frontend::ast::mig::m3e_3::get_type_node_precedence(&d.element_type)
                        < tsox_frontend::ast::mig::m3e_3::TypePrecedence::Postfix;
                Some(if parens { format!("({elem})[]") } else { format!("{elem}[]") })
            }
            SyntaxKind::IndexedAccessType => {
                // Go emitIndexedAccessType（printer.go:2133-2140）：对象段经
                // emitPostfixTypeOperand（printer.go:2006-2013）——解析树
                // TypeQuery 操作数免括号（typeof C[K] 保书写形），其余低于
                // Postfix 优先级加括号（printer.go:2291 emitTypeNode 判定）；
                // 复用节点子树经打印机重渲染，内层类型字面量/映射类型成员
                // 尾分号由对应臂产出（contextualTypeOfIndexedAccessParameter
                // >g 的 ({ a: string; } & { b: string; })[K]）；索引段经
                // emitTypeNodeOutsideExtends，字面量保书写引号（emitLiteral
                // raw 文本），按源文本复用（同 partial_reuse 索引段先例）
                let NodeData::IndexedAccessTypeNode(d) = &node.data else {
                    return None;
                };
                let obj = if d.object_type.kind == SyntaxKind::TypeQuery {
                    self.print_annotation_type_node(&d.object_type, flags)?
                } else {
                    let s = self.print_annotation_type_node(&d.object_type, flags)?;
                    let parens = tsox_frontend::ast::mig::m3e_3::get_type_node_precedence(
                        &d.object_type,
                    ) < tsox_frontend::ast::mig::m3e_3::TypePrecedence::Postfix;
                    if parens {
                        format!("({s})")
                    } else {
                        s
                    }
                };
                let idx = match self.node_source_text(&d.index_type) {
                    Some(t) => t,
                    None => self.print_annotation_type_node(&d.index_type, flags)?,
                };
                Some(format!("{obj}[{idx}]"))
            }
            SyntaxKind::TypeOperator => {
                // Go emitTypeOperator（printer.go:2124-2131）：关键字 + 操作数
                // 按 TypeOperator 优先级加括号（readonly 为 Postfix），
                // 操作数经打印机重渲染（inferenceShouldFailOnEvolvingArrays
                // >logLength 的 keyof 括号内求值形）
                let NodeData::TypeOperatorNode(d) = &node.data else {
                    return None;
                };
                let op = match d.operator {
                    SyntaxKind::KeyOfKeyword => "keyof",
                    SyntaxKind::UniqueKeyword => "unique",
                    SyntaxKind::ReadonlyKeyword => "readonly",
                    _ => return None,
                };
                let required = if d.operator == SyntaxKind::ReadonlyKeyword {
                    tsox_frontend::ast::mig::m3e_3::TypePrecedence::Postfix
                } else {
                    tsox_frontend::ast::mig::m3e_3::TypePrecedence::TypeOperator
                };
                let s = self.print_annotation_type_node(&d.type_node, flags)?;
                let parens =
                    tsox_frontend::ast::mig::m3e_3::get_type_node_precedence(&d.type_node)
                        < required;
                Some(if parens {
                    format!("{op} ({s})")
                } else {
                    format!("{op} {s}")
                })
            }
            SyntaxKind::LiteralType => {
                // Go 复用注解经打印机输出（type_symbol_baseline.go:415
                // printer.Write）：字面量叶节点按节点自身 TokenFlags 写引号
                //（emitStringLiteral），nodecopy.go:845 仅在
                // UseSingleQuotesForStringLiteralType 置位时改写、.types 通道
                // 不置该位——单引号书写随源呈现（'a' | 'b'），双引号书写与
                // 规范渲染同形；常规 type_to_string_ex 恒双引号
                self.node_source_text(node)
            }
            // Go 复用（pseudoTypeToNode Direct → reuseTypeNode，nodecopy.go:60）
            // 按注解节点原文经打印机输出；求值为 error type 的注解（缺类型
            // 实参的泛型引用等）型重求值打 any，节点复用保留书写名
            SyntaxKind::ImportType => {
                let NodeData::ImportTypeNode(d) = &node.data else {
                    return None;
                };
                let NodeData::LiteralTypeNode(lt) = &d.argument.data else {
                    return None;
                };
                if lt.literal.kind != SyntaxKind::StringLiteral {
                    return None;
                }
                let raw = lt.literal.text();
                let quote = raw.chars().next().filter(|c| *c == '"' || *c == '\'').unwrap_or('"');
                let spec = raw.trim_matches(['"', '\'']);
                let spec = self.import_specifier_text_for_display(spec);
                let mut s = String::new();
                if d.is_type_of {
                    s.push_str("typeof ");
                }
                s.push_str(&format!("import({quote}{spec}{quote})"));
                if let Some(q) = &d.qualifier {
                    s.push('.');
                    s.push_str(&crate::checker::checker_attach_explicit_type_arguments::qualified_name_text(q));
                }
                if let Some(ta) = &d.type_arguments {
                    let mut parts = Vec::with_capacity(ta.nodes.len());
                    for a in &ta.nodes {
                        parts.push(self.print_annotation_type_node(a, flags)?);
                    }
                    s.push('<');
                    s.push_str(&parts.join(", "));
                    s.push('>');
                }
                Some(s)
            }
            _ => {
                let t = self.get_type_from_type_node(node);
                if self.is_error_type(&t)
                    && let Some(text) = self.node_source_text(node)
                {
                    return Some(text);
                }
                // Go 节点级复用（pseudoTypeToNode Direct → reuseTypeNode，
                // nodecopy.go:60 → tryVisitTypeReference nodecopy.go:456-472）
                // 对 TypeReference 保留源节点输出（UpdateTypeReferenceNode
                // 原样携带书写名与实参），实体名不重渲染：缺省实参的泛型引用
                // 注解（Generator<U>）按书写形呈现而非解析型展开
                //（Generator<U, any, any>）；守卫对齐 tryVisitTypeReference：
                // const 类型引用（ast.go IsConstTypeReference 排除）不入复用，
                // 可及性门由复用入口 annotation_has_inaccessible_reference
                // 全子树先行。解为全局 Array/ReadonlyArray 的引用书写形
                // Array<T>/ReadonlyArray<T> 同样原样保留（常规
                // type_to_string_ex 走 T[]/readonly T[] 转换，
                // typeReferenceToTypeNode nodebuilderimpl.go:3160-3185）。
                // 作用域对齐 serializeTypeForDeclaration 复用门
                //（nodebuilderimpl.go:2380 ctx.enclosingDeclaration != nil）：
                // 该门只在 types 基线通道（TypeToStringEx 传 enclosing，即
                // type_render_enclosing 置位）成立；错误主消息通道
                // TypeToString(t, nil)（printer.go:51-57）门不成立，限定名
                // 按结构渲染省略（qualify 47 行 () => K1.I3 应呈 () => I3）
                if node.kind == SyntaxKind::TypeReference
                    && self.type_render_enclosing.is_some()
                    && !crate::checker::utilities_get_assignment_target::is_const_type_reference(
                        node,
                    )
                {
                    // Go 产物经 setTextRange 后由打印机发射（空白规范化、单行）；
                    // 本仓源文本是原始切片，多行书写退回解析渲染避免换行漏出
                    if let Some(text) = self.node_source_text(node)
                        && !text.contains('\n')
                    {
                        return Some(text);
                    }
                }
                if node.kind == SyntaxKind::TypeReference
                    && self.annotation_type_node_is_global_array_reference(&t)
                {
                    return self.node_source_text(node);
                }
                // Go 复用注解子树经打印机输出（type_symbol_baseline.go:415），
                // 源码字面量叶按原文呈现（printer utilities.go getLiteralText
                // canUseOriginalText：源 StringLiteral 保留书写引号风格 'bar'，
                // 不落合成规范形 "bar"）
                if node.kind == SyntaxKind::LiteralType
                    && let tsox_frontend::ast::NodeData::LiteralTypeNode(d) = &node.data
                    && d.literal.kind == SyntaxKind::StringLiteral
                    && let Some(text) = self.node_source_text(node)
                {
                    return Some(text);
                }
                Some(self.type_to_string_ex(&t, flags))
            }
        }
    }

    /// Go emitEntityName（printer.go:1308）：Identifier / QualifiedName /
    /// PropertyAccessExpression（typeof foo.x）三分支，限定名以 . 连接
    fn type_query_entity_name_text(&self, name: &Arc<Node>) -> Option<String> {
        match &name.data {
            NodeData::Identifier(_) => Some(name.text().to_string()),
            NodeData::QualifiedName(q) => {
                let left = self.type_query_entity_name_text(&q.left)?;
                Some(format!("{left}.{}", q.right.text()))
            }
            NodeData::PropertyAccessExpression(pa) => {
                let left = self.type_query_entity_name_text(&pa.expression)?;
                Some(format!("{left}.{}", pa.name.text()))
            }
            _ => None,
        }
    }

    /// 类型引用注解的实体名复用判定（Go nodecopy.go trackExistingEntityName
    /// nodecopy.go:356-419）：实体名在显示上下文重解仍指同一符号且可及时保留
    /// 书写文本（qualified 名原样），否则交由链名重写段处理。类型引用标识符
    /// 不入 binder 符号表，最左符号由引用符号沿 parent 上取（限定段数-1 层），
    /// 可及性按名字种类取 TYPE（裸标识）/NAMESPACE（限定名）语义
    fn reuse_entity_name_verbatim(
        &mut self,
        type_name: &Arc<Node>,
        referenced: &Arc<Symbol>,
    ) -> bool { ::tsox_core::fntrace::enter("reuse_entity_name_verbatim");
        if referenced.flags.contains(SymbolFlags::TypeParameter) {
            return true;
        }
        let mut segments = 1usize;
        let mut cur = Arc::clone(type_name);
        loop {
            let next = match &cur.data {
                NodeData::QualifiedName(q) => Some(Arc::clone(&q.left)),
                _ => None,
            };
            match next {
                Some(l) => {
                    segments += 1;
                    cur = l;
                }
                None => break,
            }
        }
        let leftmost_text = cur.text().to_string();
        let mut leftmost = Arc::clone(referenced);
        for _ in 1..segments {
            match self.get_parent_of_symbol(&leftmost) {
                Some(p) => leftmost = p,
                None => return false,
            }
        }
        if leftmost.name != leftmost_text {
            return false;
        }
        let meaning = if segments > 1 {
            SymbolFlags::NAMESPACE
        } else {
            SymbolFlags::TYPE
        };
        match self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone())
        {
            Some(enclosing) => {
                let chain =
                    self.get_accessible_symbol_chain(&leftmost, Some(&enclosing), meaning, false);
                chain.len() == 1 && chain[0].id() == leftmost.id()
            }
            None => true,
        }
    }

    /// 注解复用的类型引用呈现（Go nodecopy.go tryVisitTypeReference
    /// nodecopy.go:423-464）：实体名在显示上下文重解同一符号且可及 → 保留
    /// 书写文本；否则 serializeTypeName（nodebuilderimpl.go:457）按符号链
    /// 重写名段，类型实参逐段递归同规则。None = 该节点类不复用（const 引用
    /// / import 型名/链不可达），调用方按源文本保留
    fn reuse_type_reference_display_text(
        &mut self,
        node: &Arc<Node>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("reuse_type_reference_display_text");
        let NodeData::TypeReferenceNode(d) = &node.data else {
            return None;
        };
        if !matches!(
            d.type_name.kind,
            SyntaxKind::Identifier | SyntaxKind::QualifiedName
        ) {
            return None;
        }
        if crate::checker::utilities_get_assignment_target::is_const_type_reference(node) {
            return None;
        }
        let annotated = self.get_type_from_type_node(node);
        let referenced = annotated
            .alias
            .as_ref()
            .and_then(|a| a.symbol.clone())
            .or_else(|| annotated.symbol.clone())?;
        let args_text = match &d.type_arguments {
            Some(args) => {
                let mut parts = Vec::with_capacity(args.nodes.len());
                for a in &args.nodes {
                    parts.push(self.print_annotation_type_node(a, flags)?);
                }
                format!("<{}>", parts.join(", "))
            }
            None => String::new(),
        };
        if self.reuse_entity_name_verbatim(&d.type_name, &referenced) {
            return Some(format!(
                "{}{}",
                Self::entity_name_written_text(&d.type_name),
                args_text
            ));
        }
        let chain = self.symbol_chain_display_name(&referenced, tsox_frontend::ast::SymbolFlags::TYPE)?;
        Some(format!("{}{}", chain, args_text))
    }

    /// 实体名书写文本（QualifiedName 逐段拼接；Node::text 仅标识符承载文本）
    fn entity_name_written_text(node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("entity_name_written_text");
        match &node.data {
            NodeData::QualifiedName(q) => {
                format!(
                    "{}.{}",
                    Self::entity_name_written_text(&q.left),
                    q.right.text()
                )
            }
            _ => node.text().to_string(),
        }
    }

    // Go printer.Write(typeNode, currentSourceFile)（type_symbol_baseline.go:415）
    // 对复用注解节点的 import 说明符按显示上下文输出：rewriteRelativeImportExtensions
    // 开启时 .ts 上下文的相对 .ts 说明符呈无扩展名（rewrite 语义，emit 侧补回），
    // .d.ts 上下文保留源文本
    fn import_specifier_text_for_display(&self, spec: &str) -> String { ::tsox_core::fntrace::enter("import_specifier_text_for_display");
        if self
            .compiler_options
            .rewrite_relative_import_extensions
            .is_true()
            && (spec.starts_with("./") || spec.starts_with("../"))
            && spec.ends_with(".ts")
            && !spec.ends_with(".d.ts")
            && let Some(ctx_file) = self.display_context_file()
            && !ctx_file.file_name.ends_with(".d.ts")
        {
            return spec[..spec.len() - 3].to_string();
        }
        spec.to_string()
    }

    /// 联合/交集成员按 TypePrecedenceTypeOperator 发射（printer.go:2195-2215
    /// emitUnionTypeConstituent / emitIntersectionTypeConstituent），优先级
    /// 更低的成员加括号（precedence.go:661 GetTypeNodePrecedence：条件/函数/
    /// 构造/联合/交集及带约束 infer 低于 TypeOperator）
    fn print_annotation_type_constituent(
        &mut self,
        node: &Arc<Node>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("print_annotation_type_constituent");
        let s = self.print_annotation_type_node(node, flags)?;
        let parens = tsox_frontend::ast::mig::m3e_3::get_type_node_precedence(node)
            < tsox_frontend::ast::mig::m3e_3::TypePrecedence::TypeOperator;
        Some(if parens { format!("({s})") } else { s })
    }

    /// 字面量成员打印，尾分号随成员输出（printer.go writeTrailingSemicolon）
    fn print_annotation_type_member(
        &mut self,
        member: &Arc<Node>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("print_annotation_type_member");
        match member.kind {
            SyntaxKind::CallSignature => {
                let NodeData::CallSignatureDeclaration(d) = &member.data else {
                    return None;
                };
                let body = self.print_annotation_signature(
                    d.type_parameters.as_ref(),
                    &d.parameters,
                    d.type_node.as_ref(),
                    flags,
                )?;
                Some(format!("{body};"))
            }
            SyntaxKind::ConstructSignature => {
                let NodeData::ConstructSignatureDeclaration(d) = &member.data else {
                    return None;
                };
                let body = self.print_annotation_signature(
                    d.type_parameters.as_ref(),
                    &d.parameters,
                    d.type_node.as_ref(),
                    flags,
                )?;
                Some(format!("new {body};"))
            }
            SyntaxKind::MethodSignature => {
                let NodeData::MethodSignatureDeclaration(d) = &member.data else {
                    return None;
                };
                let mut s = Self::annotation_member_modifier_prefix(member);
                s.push_str(&self.annotation_name_text(&d.name)?);
                if d.postfix_token.is_some() {
                    s.push('?');
                }
                s.push_str(&self.print_annotation_signature(
                    d.type_parameters.as_ref(),
                    &d.parameters,
                    d.type_node.as_ref(),
                    flags,
                )?);
                s.push(';');
                Some(s)
            }
            SyntaxKind::PropertySignature => {
                let NodeData::PropertySignatureDeclaration(d) = &member.data else {
                    return None;
                };
                let mut s = Self::annotation_member_modifier_prefix(member);
                s.push_str(&self.annotation_name_text(&d.name)?);
                if d.postfix_token.is_some() {
                    s.push('?');
                }
                if d.type_node.kind != SyntaxKind::MissingDeclaration {
                    s.push_str(": ");
                    s.push_str(&self.print_annotation_type_node(&d.type_node, flags)?);
                } else if d.initializer.kind == SyntaxKind::MissingDeclaration {
                    // Go 复用访问对无类型无初始化式的属性签名插 any
                    //（nodecopy.go:689、nodecopy.go:699-706）
                    s.push_str(": any");
                }
                s.push(';');
                Some(s)
            }
            SyntaxKind::IndexSignature => {
                let NodeData::IndexSignatureDeclaration(d) = &member.data else {
                    return None;
                };
                let mut s = Self::annotation_member_modifier_prefix(member);
                s.push('[');
                s.push_str(&self.print_annotation_params(&d.parameters, flags)?);
                s.push_str("]: ");
                s.push_str(&self.print_annotation_type_node(&d.type_node, flags)?);
                s.push(';');
                Some(s)
            }
            _ => None,
        }
    }

    /// 签名主体：`<T>(params): ret`（printer.go emitSignature）
    fn print_annotation_signature(
        &mut self,
        type_parameters: Option<&Arc<NodeList>>,
        parameters: &Arc<NodeList>,
        type_node: Option<&Arc<Node>>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("print_annotation_signature");
        let mut s = String::new();
        if let Some(tps) = type_parameters {
            s.push_str(&self.print_annotation_type_parameters(tps, flags)?);
        }
        s.push('(');
        s.push_str(&self.print_annotation_params(parameters, flags)?);
        s.push(')');
        if let Some(ret) = type_node {
            s.push_str(": ");
            s.push_str(&self.print_annotation_type_node(ret, flags)?);
        } else {
            // Go 复用访问对无返回注解的函数型节点插 any（nodecopy.go:689、
            // nodecopy.go:725-758 CallSignature/MethodSignature/
            // ConstructSignature 臂）
            s.push_str(": any");
        }
        Some(s)
    }

    /// 参数列表 `, ` 分隔：`...name?: type`（printer.go emitParameter）
    fn print_annotation_params(
        &mut self,
        parameters: &Arc<NodeList>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("print_annotation_params");
        if parameters.nodes.is_empty() {
            return Some(String::new());
        }
        let mut parts = Vec::with_capacity(parameters.nodes.len());
        for p in &parameters.nodes {
            let NodeData::ParameterDeclaration(pd) = &p.data else {
                return None;
            };
            let mut s = String::new();
            if pd.dot_dot_dot_token.is_some() {
                s.push_str("...");
            }
            s.push_str(&self.annotation_name_text(&pd.name)?);
            if pd.question_token.is_some() {
                s.push('?');
            }
            if let Some(tn) = pd.type_node.as_ref() {
                s.push_str(": ");
                s.push_str(&self.print_annotation_type_node(tn, flags)?);
            } else if pd.initializer.is_none() {
                // Go existingNodeTreeVisitor（nodecopy.go:689-724）：复用注解
                // 子树中无类型注解且无初始化的参数声明合成 any 关键字呈现
                s.push_str(": any");
            }
            parts.push(s);
        }
        Some(parts.join(", "))
    }

    fn print_annotation_type_parameters(
        &mut self,
        type_parameters: &Arc<NodeList>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("print_annotation_type_parameters");
        if type_parameters.nodes.is_empty() {
            return None;
        }
        let mut parts = Vec::with_capacity(type_parameters.nodes.len());
        for tp in &type_parameters.nodes {
            let NodeData::TypeParameterDeclaration(d) = &tp.data else {
                return None;
            };
            let mut s = d.name.text().to_string();
            if let Some(c) = &d.constraint {
                s.push_str(" extends ");
                s.push_str(&self.print_annotation_type_node(c, flags)?);
            }
            if let Some(def) = &d.default_type {
                s.push_str(" = ");
                s.push_str(&self.print_annotation_type_node(def, flags)?);
            }
            parts.push(s);
        }
        Some(format!("<{}>", parts.join(", ")))
    }

    /// 成员修饰符前缀（printer.go emitPropertySignature emitModifierList）
    fn annotation_member_modifier_prefix(member: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("annotation_member_modifier_prefix");
        let has_readonly = member
            .modifier_nodes()
            .iter()
            .any(|m| m.kind == SyntaxKind::ReadonlyKeyword);
        if has_readonly {
            "readonly ".to_string()
        } else {
            String::new()
        }
    }

    /// 成员/参数名：Identifier 取节点文本，字面量名按源文本保留书写引号
    fn annotation_name_text(&self, name: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("annotation_name_text");
        match name.kind {
            SyntaxKind::Identifier => Some(name.text().to_string()),
            _ => Some(
                self.node_source_text(name).unwrap_or_else(|| name.text().to_string()),
            ),
        }
    }

    /// 变量/属性声明注解与解析类型等价时返回注解源文本
    #[allow(dead_code)]
    pub(crate) fn annotated_decl_type_text(
        &mut self,
        symbol: &Arc<Symbol>,
        resolved: &Arc<Type>,
    ) -> Option<String> { ::tsox_core::fntrace::enter("annotated_decl_type_text"); 
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
        self.equivalent_annotation_text(&tn, resolved, TypeFormatFlags::NONE)
    }

    pub(crate) fn call_site_union_signature(
        &mut self,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
    ) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("call_site_union_signature"); 
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

    pub(crate) fn arrow_signature_parts(&mut self, sig: &Arc<Signature>) -> Vec<SymbolDisplayPart> { ::tsox_core::fntrace::enter("arrow_signature_parts"); 
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
    ) { ::tsox_core::fntrace::enter("append_type_parameter_parts"); 
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


fn assignment_target_expr(obj: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("assignment_target_expr"); 
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

pub(crate) fn node_in_with_block(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("node_in_with_block"); 
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
    ) -> Option<Vec<SymbolDisplayPart>> { ::tsox_core::fntrace::enter("binding_element_property_name_parts"); 
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

fn anonymous_symbol_display_name(symbol: &Arc<Symbol>) -> Option<String> { ::tsox_core::fntrace::enter("anonymous_symbol_display_name"); 
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
    fn base_member_documentation(&mut self, symbol: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("base_member_documentation"); 
        let mut seen = std::collections::HashSet::from([Arc::as_ptr(symbol) as usize]);
        self.base_member_documentation_seen(symbol, &mut seen)
    }

    fn base_member_documentation_seen(
        &mut self,
        symbol: &Arc<Symbol>,
        seen: &mut std::collections::HashSet<usize>,
    ) -> String { ::tsox_core::fntrace::enter("base_member_documentation_seen"); 
        let decl = symbol
            .value_declaration
            .as_ref()
            .or(symbol.declarations.first())
            .cloned();
        let Some(decl) = decl else {
            return String::new();
        };
        let Some(class_node) = decl.parent().filter(|p| {
            matches!(
                p.kind,
                SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::InterfaceDeclaration
            )
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
        let class_type = self.get_declared_type_of_symbol(&class_sym);
        if is_static {
            let Some(base_ctor) = self.get_base_constructor_type_of_class(&class_type) else {
                return String::new();
            };
            let static_base = self.get_apparent_type(&base_ctor);
            if let Some(doc) = self.jsdoc_from_base_member(&static_base, symbol, seen) {
                return doc;
            }
        } else {
            for base_type in self.get_base_types(&class_type) {
                if let Some(doc) = self.jsdoc_from_base_member(&base_type, symbol, seen) {
                    return doc;
                }
            }
        }
        String::new()
    }

    fn jsdoc_from_base_member(
        &mut self,
        base_type: &Arc<Type>,
        symbol: &Arc<Symbol>,
        seen: &mut std::collections::HashSet<usize>,
    ) -> Option<String> { ::tsox_core::fntrace::enter("jsdoc_from_base_member"); 
        let prop = self.get_property_of_type(base_type, &symbol.name)?;
        if !seen.insert(Arc::as_ptr(&prop) as usize) {
            return None;
        }
        let prop_decl = prop.value_declaration.as_ref()?;
        let doc = self.declaration_jsdoc_text(prop_decl);
        if !doc.is_empty() {
            return Some(doc);
        }
        let inherited = self.base_member_documentation_seen(&prop, seen);
        if inherited.is_empty() {
            None
        } else {
            Some(inherited)
        }
    }
}

impl Checker {
    /// 位置落在 jsdoc 标签（@property 等）内的合成声明：取该标签的注释文本
    fn jsdoc_tag_comment_at(
        &mut self,
        sf: &Arc<tsox_frontend::ast::SourceFile>,
        decl: &Arc<Node>,
    ) -> String { ::tsox_core::fntrace::enter("jsdoc_tag_comment_at"); 
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
fn jsdoc_tag_trailing_comment(text: &str, tag: &crate::checker::nodebuilder::Node) -> String { ::tsox_core::fntrace::enter("jsdoc_tag_trailing_comment"); 
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

fn jsdoc_comment_text(comment: Option<&Arc<tsox_frontend::ast::NodeList>>) -> String { ::tsox_core::fntrace::enter("jsdoc_comment_text"); 
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
fn jsdoc_block_doc(sf: &Arc<tsox_frontend::ast::SourceFile>, decl: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("jsdoc_block_doc"); 
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

fn jsdoc_tag_name(tag: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("jsdoc_tag_name"); 
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

fn jsdoc_tag_comment(tag: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("jsdoc_tag_comment"); 
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
