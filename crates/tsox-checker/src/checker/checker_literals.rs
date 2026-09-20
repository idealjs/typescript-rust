use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::SymbolFlags;
use tsox_frontend::ast::SymbolTable;
use tsox_frontend::ast::SyntaxKind;

use crate::checker::checker::*;

impl Checker {
    pub fn get_widened_type_of_literal(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(crate::checker::TypeFlags::StringLiteral)
            || t.flags.contains(crate::checker::TypeFlags::NumberLiteral)
            || t.flags.contains(crate::checker::TypeFlags::BigIntLiteral)
            || t.flags.contains(crate::checker::TypeFlags::BooleanLiteral)
        {
            return self.get_base_type_of_literal_type(t);
        }
        Arc::clone(t)
    }

    pub(crate) fn types_are_equal(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        if Arc::ptr_eq(a, b) {
            return true;
        }
        if a.flags != b.flags {
            return false;
        }

        match (&a.data, &b.data) {
            (crate::checker::TypeData::Intrinsic(a), crate::checker::TypeData::Intrinsic(b)) => {
                a.intrinsic_name == b.intrinsic_name
            }
            _ => false,
        }
    }

    pub(crate) fn infer_number_literal_type(&mut self, text: &str) -> Arc<Type> {
        let num = tsox_core::jsnum::Number::from_string(text);
        if num.is_nan() {
            return self.number_type();
        }
        self.get_number_literal_type(num)
    }

    pub(crate) fn infer_string_literal_type(&mut self, text: &str) -> Arc<Type> {
        self.get_string_literal_type(text)
    }
    pub(crate) fn find_object_literal_property_name_node(
        &self,
        init: &Arc<Node>,
        prop_name: &str,
    ) -> Option<TextRange> {
        let tsox_frontend::ast::NodeData::ObjectLiteralExpression(data) = &init.data else {
            return None;
        };
        for prop in data.properties.iter() {
            let name = match &prop.data {
                NodeData::PropertyAssignment(p) => &p.name,
                NodeData::ShorthandPropertyAssignment(p) => &p.name,
                _ => continue,
            };
            if self.get_property_name_from_node(name) == prop_name {
                return Some(name.loc);
            }
        }
        None
    }
    pub(crate) fn get_const_assertion_type(&mut self, expr: &Arc<Node>) -> Arc<Type> {
        match expr.kind {
            SyntaxKind::ArrayLiteralExpression => {
                let elements = match &expr.data {
                    tsox_frontend::ast::NodeData::ArrayLiteralExpression(data) => &data.elements,
                    _ => return self.get_any_type(),
                };
                let mut element_types: Vec<Arc<Type>> = Vec::new();
                for elem in elements.iter() {
                    if elem.kind == SyntaxKind::SpreadElement {
                        let t = self.get_type_of_node(elem);
                        element_types.push(t);
                    } else {
                        element_types.push(self.get_type_of_node(elem));
                    }
                }
                self.create_tuple_type(element_types)
            }
            _ => self.get_type_of_node(expr),
        }
    }

    /// Go checkPropertyAssignment：属性赋值类型 = 初始化式按可变位置检查，
    /// 字面量视上下文属性类型保留 regular 字面量或拓宽
    pub(crate) fn property_assignment_type(
        &mut self,
        prop: &Arc<Node>,
        initializer: &Arc<Node>,
        literal: &Arc<Node>,
        name: &str,
    ) -> Arc<Type> {
        let mut t = self.get_type_of_node(initializer);
        let contextual = self.get_contextual_type(literal, ContextFlags::empty());
        let prop_ctx = contextual
            .as_ref()
            .and_then(|c| self.get_type_of_property_of_contextual_type(c, name));
        let literal_of_ctx = prop_ctx
            .as_ref()
            .is_some_and(|pc| self.is_literal_of_contextual_type(&t, pc));
        if !literal_of_ctx {
            t = self.get_widened_literal_type(&t);
        }
        t = self.get_regular_type_of_literal_type(&t);
        if let Some(sym) = self.program.symbol_map().symbol_of(prop) {
            let container = contextual.as_ref().and_then(|c| c.symbol.clone());
            let links = self.value_symbol_links.get_or_default(&sym);
            links.resolved_type = Some(Arc::clone(&t));
            links.container_symbol = container;
        }
        t
    }

    pub(crate) fn get_type_of_object_literal(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let properties = match &node.data {
            tsox_frontend::ast::NodeData::ObjectLiteralExpression(data) => &data.properties,
            _ => return self.get_any_type(),
        };

        let mut prop_pairs: Vec<(String, Arc<Type>, Vec<Arc<Node>>)> = Vec::new();
        let mut fell_back_to_any = false;
        for prop in properties.iter() {
            let is_accessor = matches!(
                &prop.data,
                NodeData::GetAccessorDeclaration(_) | NodeData::SetAccessorDeclaration(_)
            );
            if is_accessor {
                let name = self.get_property_name_from_node(
                    &prop.name().expect("accessor member has name"),
                );
                if name.is_empty() {
                    fell_back_to_any = true;
                    break;
                }
                match prop_pairs.iter_mut().find(|(n, _, decls)| {
                    n == &name
                        && decls.iter().any(|d| {
                            matches!(
                                d.data,
                                NodeData::GetAccessorDeclaration(_)
                                    | NodeData::SetAccessorDeclaration(_)
                            )
                        })
                }) {
                    Some((_, _, decls)) => decls.push(Arc::clone(prop)),
                    None => {
                        prop_pairs.push((name, self.get_any_type(), vec![Arc::clone(prop)]));
                    }
                }
                continue;
            }
            match &prop.data {
                NodeData::PropertyAssignment(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        fell_back_to_any = true;
                        break;
                    }

                    let t = self.property_assignment_type(prop, &data.initializer, node, &name);
                    prop_pairs.push((name, t, vec![Arc::clone(prop)]));
                }
                NodeData::ShorthandPropertyAssignment(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        fell_back_to_any = true;
                        break;
                    }

                    let raw = self.get_type_of_node(&data.name);
                    let t = self.get_widened_literal_type(&raw);
                    // 回写变量符号类型（shorthand 引用的外层变量在字面量上下文中显示属性类型）
                    if let Some(sym) = self.program.symbol_map().symbol_of(&data.name) {
                        self.value_symbol_links
                            .get_or_default(&sym)
                            .resolved_type = Some(t.clone());
                    }
                    prop_pairs.push((name, t, vec![Arc::clone(prop)]));
                }
                NodeData::MethodDeclaration(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        fell_back_to_any = true;
                        break;
                    }
                    // 方法成员：符号类型（声明签名/体推断），类型容器挂字面量
                    let sym = self
                        .program
                        .symbol_map()
                        .symbol_of(prop)
                        .map(Arc::clone);
                    let t = match &sym {
                        Some(sym) => {
                            self.get_type_of_symbol(sym)
                        }
                        None => self.get_type_of_function_like(prop),
                    };
                    prop_pairs.push((name, t, vec![Arc::clone(prop)]));
                }
                NodeData::SpreadAssignment(_) => {
                    fell_back_to_any = true;
                    break;
                }
                _ => {
                    fell_back_to_any = true;
                    break;
                }
            }
        }
        if fell_back_to_any {
            return self.get_any_type();
        }

        // Go getTypeOfAccessors 解析序：getter 注解 → setter 参数注解 →
        // getter 体返回推断（加宽）
        for idx in 0..prop_pairs.len() {
            if !prop_pairs[idx].2.iter().any(|d| {
                matches!(
                    d.data,
                    NodeData::GetAccessorDeclaration(_) | NodeData::SetAccessorDeclaration(_)
                )
            }) {
                continue;
            }
            let name = prop_pairs[idx].0.clone();
            let t = self.object_literal_accessor_type(node, &name);
            prop_pairs[idx].1 = t;
        }

        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(prop_pairs.len());
        for (name, t, decls) in prop_pairs {
            let mut sym = Symbol::new(SymbolFlags::Property, name.clone());
            sym.declarations.extend(decls);
            let symbol = Arc::new(sym);
            members.insert(name, Arc::clone(&symbol));
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(t),
                    ..Default::default()
                },
            );
            props.push(symbol);
        }
        // Go checkObjectLiteral createObjectLiteralType：newAnonymousType(node.Symbol())
        // 挂 binder 对象字面量符号（SymbolFlagsObjectLiteral），索引签名推断依赖它
        let literal_symbol = self.program.symbol_map().symbol_of(node).map(Arc::clone);
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous | ObjectFlags::ObjectLiteral,
            id: crate::checker::types::next_type_id(),
            symbol: literal_symbol,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    fn object_literal_accessor_type(&mut self, node: &Arc<Node>, name: &str) -> Arc<Type> {
        let properties = match &node.data {
            NodeData::ObjectLiteralExpression(data) => &data.properties,
            _ => return self.get_any_type(),
        };
        let getter = properties.iter().find(|p| {
            p.kind == SyntaxKind::GetAccessor
                && self.get_property_name_from_node(&p.name().expect("accessor has name")) == name
        });
        let setter = properties.iter().find(|p| {
            p.kind == SyntaxKind::SetAccessor
                && self.get_property_name_from_node(&p.name().expect("accessor has name")) == name
        });
        if let Some(g) = getter
            && let NodeData::GetAccessorDeclaration(gd) = &g.data
            && let Some(tn) = &gd.type_node
        {
            return self.get_type_from_type_node(tn);
        }
        if let Some(s) = setter
            && let NodeData::SetAccessorDeclaration(sd) = &s.data
            && let Some(param) = sd.parameters.iter().next()
            && let NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(tn) = &pd.type_node
        {
            return self.get_type_from_type_node(tn);
        }
        if let Some(g) = getter
            && let NodeData::GetAccessorDeclaration(gd) = &g.data
            && let Some(body) = &gd.body
        {
            let inferred = self.infer_method_return_type(&Some(Arc::clone(body)));
            return self.get_widened_type(&inferred);
        }
        self.get_any_type()
    }

    pub(crate) fn get_excess_property_name(
        &self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<String> {
        if !crate::checker::is_object_literal_type(source) {
            return None;
        }
        let source_struct = source.as_structured()?;
        let target_struct = target.as_structured()?;

        if !target_struct.index_infos.is_empty() {
            return None;
        }
        // Go hasExcessProperties：目标为 globalObjectType 的超集（含 Object 或
        // object 非原始型成分）时跳过多余属性检查
        if self.target_admits_any_properties(target) {
            return None;
        }
        for prop in &source_struct.properties {
            if !self.target_has_property(target, &prop.name) {
                return Some(prop.name.clone());
            }
        }
        None
    }

    // Go isTypeSubsetOf(globalObjectType, target)：Object/object 成分出现即
    // 视为全局 Object 型的容器
    fn target_admits_any_properties(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::NonPrimitive) {
            return true;
        }
        if let Some(sym) = &t.symbol
            && sym.name == "Object"
            && sym.flags.contains(SymbolFlags::Interface)
        {
            return true;
        }
        if t.flags.contains(TypeFlags::Union)
            && let Some(members) = t.types()
        {
            return members.iter().any(|m| self.target_admits_any_properties(m));
        }
        false
    }



    fn target_has_property(&self, t: &Arc<Type>, name: &str) -> bool {
        if matches!(&t.data, TypeData::Mapped(m) if m.type_parameter.is_some()) {
            return true;
        }
        if let Some(structured) = t.as_structured() {
            if structured.members.get(name).is_some() {
                return true;
            }

            if !structured.index_infos.is_empty() {
                return true;
            }
        }

        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|ct| self.target_has_property(ct, name));
            }
        }

        if t.flags.contains(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = &t.data {
                return i
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|ct| self.target_has_property(ct, name));
            }
        }
        false
    }
    pub(crate) fn get_constant_numeric_value(&self, node: &Arc<Node>) -> Option<f64> {
        match &node.data {
            tsox_frontend::ast::NodeData::NumericLiteral(data) => data.text.parse::<f64>().ok(),
            _ => None,
        }
    }
}
