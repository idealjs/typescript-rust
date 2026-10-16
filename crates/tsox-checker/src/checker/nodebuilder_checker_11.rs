#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;

impl Checker {
    pub(crate) fn signature_to_parameter_nodes(&mut self, sig: &Signature) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("signature_to_parameter_nodes");
        sig.parameters
            .iter()
            .map(|param| {
                let name = self.identifier(&param.name);
                let param_type = self.get_type_of_symbol(param);
                // Go symbolToParameterDeclaration（nodebuilderimpl.go:1788）→
                // serializeTypeForDeclaration（nodebuilderimpl.go:2329）→
                // pseudoTypeToNode Direct（pseudotypenodebuilder.go:53-55）：
                // 参数带注解且与符号型等价时复用注解节点本身（源序），否则
                // 序列化符号型；等价判定见 parameter_display_type_equivalent
                let type_node =
                    self.parameter_declaration_display_type_node(param, &param_type, Self::signature_params_instantiated(sig));
                let optional = param.flags.contains(SymbolFlags::Optional);
                self.parameter_node(name, optional, type_node)
            })
            .collect()
    }

    pub(crate) fn parameter_declaration_display_type_node(
        &mut self,
        param: &Arc<tsox_frontend::ast::Symbol>,
        param_type: &Arc<Type>,
        instantiated_signature: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parameter_declaration_display_type_node");
        let decl = param
            .value_declaration
            .clone()
            .or_else(|| param.declarations.first().cloned());
        let Some(decl) = decl else {
            return self.type_to_type_node(param_type);
        };
        // 实例化签名参数按解析型渲染的门（nodebuilder_checker_12
        // annotated_param_type_text 同口径）收窄为仅实参帧活跃窗：Go
        // serializeTypeForDeclaration（nodebuilderimpl.go:2376-2404）注解
        // 复用不区分符号是否实例化，pseudoTypeEquivalentToType（
        // pseudotypenodebuilder.go:367）可选声明剥合成 undefined 后等价即
        // 复用注解节点；lib 无型变量参数（slice/subarray/initCustomEvent/
        // toLocaleString 的 `number`）strictNullChecks 下剥 undefined 恒
        // 等价应复用 `number`（bigint64ArraySubarray 行15、
        // arrayToLocaleStringES2015 行127 等门基线新增 anchor 全量禁复用
        // 所致）。帧活跃时注解经 get_type_from_type_node 被帧解析成实例化
        // 态与解析型伪等价会复用源序注解（Promise.then onrejected 回归源），
        // 维持解析渲染
        if (instantiated_signature
            || param
                .check_flags
                .contains(tsox_frontend::ast::CheckFlags::Instantiated))
            && !self.type_argument_stack.is_empty()
        {
            return self.type_to_type_node(param_type);
        }
        let annotation = match &decl.data {
            tsox_frontend::ast::NodeData::ParameterDeclaration(d) => d.type_node.clone(),
            _ => None,
        };
        let Some(annotation) = annotation else {
            return self.type_to_type_node(param_type);
        };
        let declared = self.get_type_from_type_node(&annotation);
        let treat_optional_as_compatible =
            crate::checker::utilities_has_only_expression_initialization::is_optional_declaration(
                &decl,
            );
        if self.parameter_display_type_equivalent(
            &declared,
            param_type,
            treat_optional_as_compatible,
        ) && self.annotation_type_node_reusable(&annotation, param_type)
        {
            return annotation;
        }
        self.type_to_type_node(param_type)
    }

    pub(crate) fn annotation_type_node_reusable(
        &mut self,
        existing: &Arc<Node>,
        t: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("canReuseExistingJSTypeNode");
        // Go canReuseExistingJSTypeNode（nodebuilderimpl.go:528-530）+
        // existingTypeNodeIsNotReferenceOrIsReferenceWithCompatibleTypeArgumentCount
        //（nodebuilderimpl.go:547-570）
        if self.get_intended_type_from_jsdoc_type_reference(existing).is_some() {
            return false;
        }
        if !t.object_flags.intersects(crate::checker::types::ObjectFlags::Reference) {
            return true;
        }
        if !tsox_frontend::ast::is_type_reference_node(existing) {
            return true;
        }
        self.get_type_from_type_node(existing);
        let Some(symbol) = self.get_resolved_symbol(existing) else {
            return true;
        };
        let Some(target) = t.as_type_reference().and_then(|d| d.target.clone()) else {
            return true;
        };
        let existing_target = self.get_declared_type_of_symbol(&symbol);
        if !Arc::ptr_eq(&existing_target, &target) {
            return true;
        }
        let existing_count = match &existing.data {
            tsox_frontend::ast::NodeData::TypeReferenceNode(tr) => {
                tr.type_arguments.as_ref().map_or(0, |l| l.nodes.len())
            }
            _ => 0,
        };
        let Some(interface_target) = target.as_interface_type() else {
            return true;
        };
        existing_count
            >= self.get_min_type_argument_count(
                &interface_target.all_type_parameters[..interface_target.outer_type_parameter_count],
            )
    }

    pub(crate) fn parameter_display_type_equivalent(
        &mut self,
        declared: &Arc<Type>,
        t: &Arc<Type>,
        is_optional_annotated: bool,
    ) -> bool { ::tsox_core::fntrace::enter("parameter_display_type_equivalent");
        // Go pseudoTypeEquivalentToType（pseudotypenodebuilder.go:367）类型级
        // 快路径：恒等、可选剥 undefined 后恒等、fresh 字面量经
        // getRegularTypeOfLiteralType 归一后恒等、联合型逐一同一
        if crate::checker::utilities::is_type_error(t) {
            return true;
        }
        if Arc::ptr_eq(declared, t) {
            return true;
        }
        if is_optional_annotated {
            let stripped =
                self.get_type_with_facts(t, crate::checker::types::TypeFacts::NE_UNDEFINED);
            if Arc::ptr_eq(&stripped, declared) {
                return true;
            }
            if declared.flags.contains(crate::checker::types::TypeFlags::Union)
                && stripped.flags.contains(crate::checker::types::TypeFlags::Union)
                && self.compare_types_identical(declared, &stripped).is_true()
            {
                return true;
            }
        }
        if Arc::ptr_eq(
            &self.get_regular_type_of_literal_type(declared),
            &self.get_regular_type_of_literal_type(t),
        ) {
            return true;
        }
        if declared.flags.contains(crate::checker::types::TypeFlags::Union)
            && t.flags.contains(crate::checker::types::TypeFlags::Union)
            && self.compare_types_identical(declared, t).is_true()
        {
            return true;
        }
        false
    }

    pub(crate) fn call_signature_to_node(&mut self, sig: &Signature) -> Arc<Node> { ::tsox_core::fntrace::enter("call_signature_to_node"); 
        let params = self.signature_to_parameter_nodes(sig);
        let ret_type = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        let ret_node = self.type_to_type_node(&ret_type);
        self.function_type_node(params, ret_node)
    }

    pub fn symbol_to_type_node(
        &mut self,
        symbol: &Arc<Symbol>,
        mask: SymbolFlags,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("symbol_to_type_node");
        let _ = mask;

        // Go symbolToTypeNode（nodebuilderimpl.go:682-687/747）链计算先行：
        // 类/接口实例位（typeToTypeNode ClassOrInterface 臂
        // nodebuilderimpl.go:3619-3621 symbolToTypeNode(Type)）与别名臂
        //（nodebuilder_checker_7.rs:428-439）同规——链经 getSymbolChain
        // 上爬，链根为外部模块时整链以 ImportTypeNode 开头
        //（mergedDeclarationExports 的 >d 呈
        // import("./mergedDeclarationExports").d：双符号 binder 的 export
        // 符号 needsQualification 命中 local 身份，本仓单符号世界由
        // display_export_shadowing 模拟位承载）。错误消息通道无 enclosing
        // 上下文（lookupSymbolChainWorker nodebuilderimpl.go:1110-1117 无
        // enclosing 链 = [符号] 裸名），维持裸名
        let name_text = if self.type_render_enclosing.is_some() {
            self.symbol_chain_display_name(symbol, SymbolFlags::TYPE)
                .or_else(|| {
                    self.namespace_qualifier_of(symbol)
                        .map(|q| format!("{q}.{}", symbol.name))
                })
                .unwrap_or_else(|| Self::written_symbol_name(symbol))
        } else {
            Self::written_symbol_name(symbol)
        };
        let name = self.identifier(&name_text);

        let type_args = type_arguments.or_else(|| {
            let t = self.get_type_of_symbol(symbol);
            if let Some(obj) = t.as_object() {
                if !obj.type_arguments.is_empty() {
                    let arg_nodes: Vec<Arc<Node>> = obj
                        .type_arguments
                        .iter()
                        .map(|ty| self.type_to_type_node(ty))
                        .collect();
                    return Some(Arc::new(NodeList::new(arg_nodes)));
                }
            }
            None
        });
        self.type_reference_node(name, type_args)
    }

    // Go getNameOfSymbolAsWritten（nodebuilderimpl.go:1008）：default 命名符号
    // 在链首、同绑定上下文、有声明时不早退 "default"，按声明名渲染——名取自
    // GetNonAssignedNameOfDeclaration（ast/utilities.go:1643-1648）：
    // ExportAssignment 取表达式标识符（`export default foo` 的合并 default
    // 符号因此按 foo 渲染），其余声明取声明名；无声明或无名可取才落符号名
    pub(crate) fn written_symbol_name(symbol: &Arc<Symbol>) -> String {
        if symbol.name != tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT {
            return symbol.name.clone();
        }
        if symbol.declarations.is_empty() {
            return symbol.name.clone();
        }
        for decl in &symbol.declarations {
            if let tsox_frontend::ast::NodeData::ExportAssignment(ea) = &decl.data
                && ea.expression.kind == SyntaxKind::Identifier
            {
                return ea.expression.text().to_string();
            }
            if let Some(name_node) = decl.name()
                && name_node.kind == SyntaxKind::Identifier
            {
                return name_node.text().to_string();
            }
        }
        symbol.name.clone()
    }

    pub(crate) fn keyword_node(&self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("keyword_node"); 
        Arc::new(Node::new(kind, NodeData::Token))
    }

    pub(crate) fn identifier(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("identifier"); 
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    pub(crate) fn string_literal_node(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("string_literal_node"); 
        Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text: text.to_string(),
                token_flags: 0,
            }),
        ))
    }

    pub(crate) fn numeric_literal_node(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("numeric_literal_node"); 
        Arc::new(Node::new(
            SyntaxKind::NumericLiteral,
            NodeData::NumericLiteral(NumericLiteralData {
                text: text.to_string(),
                token_flags: 0,
            }),
        ))
    }

    pub(crate) fn bigint_literal_node(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("bigint_literal_node"); 
        Arc::new(Node::new(
            SyntaxKind::BigIntLiteral,
            NodeData::BigIntLiteral(BigIntLiteralData {
                text: format!("{}n", text),
                token_flags: 0,
            }),
        ))
    }

    pub(crate) fn literal_type_node(&self, literal: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("literal_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::LiteralType,
            NodeData::LiteralTypeNode(LiteralTypeNodeData { literal }),
        ))
    }

    pub(crate) fn type_reference_node(
        &self,
        type_name: Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("type_reference_node"); 
        Arc::new(Node::new(
            SyntaxKind::TypeReference,
            NodeData::TypeReferenceNode(TypeReferenceNodeData {
                type_name,
                type_arguments,
            }),
        ))
    }

    pub(crate) fn array_type_node(&self, element_type: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("array_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::ArrayType,
            NodeData::ArrayTypeNode(ArrayTypeNodeData { element_type }),
        ))
    }

    pub(crate) fn tuple_type_node(&self, elements: Vec<Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("tuple_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::TupleType,
            NodeData::TupleTypeNode(TupleTypeNodeData {
                elements: Arc::new(NodeList::new(elements)),
            }),
        ))
    }

    pub(crate) fn union_type_node(&self, types: Vec<Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("union_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::UnionType,
            NodeData::UnionTypeNode(UnionTypeNodeData {
                types: Arc::new(NodeList::new(types)),
            }),
        ))
    }

    pub(crate) fn intersection_type_node(&self, types: Vec<Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("intersection_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::IntersectionType,
            NodeData::IntersectionTypeNode(IntersectionTypeNodeData {
                types: Arc::new(NodeList::new(types)),
            }),
        ))
    }

    pub(crate) fn parenthesized_type_node(&self, type_node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("parenthesized_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::ParenthesizedType,
            NodeData::ParenthesizedTypeNode(ParenthesizedTypeNodeData { type_node }),
        ))
    }

    pub(crate) fn function_type_node(&self, params: Vec<Arc<Node>>, ret: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("function_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::FunctionType,
            NodeData::FunctionTypeNode(FunctionTypeNodeData {
                type_parameters: None,
                parameters: Arc::new(NodeList::new(params)),
                type_node: Some(ret),
            }),
        ))
    }

    pub(crate) fn type_literal_node(&self, members: Vec<Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("type_literal_node"); 
        Arc::new(Node::new(
            SyntaxKind::TypeLiteral,
            NodeData::TypeLiteralNode(TypeLiteralNodeData {
                members: Arc::new(NodeList::new(members)),
            }),
        ))
    }

    pub(crate) fn property_signature_node(
        &self,
        name: Arc<Node>,
        optional: bool,
        type_node: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("property_signature_node"); 
        let postfix_token = if optional {
            Some(self.keyword_node(SyntaxKind::QuestionToken))
        } else {
            None
        };

        let initializer = Arc::new(Node::new(
            SyntaxKind::MissingDeclaration,
            NodeData::MissingDeclaration(MissingDeclarationData { modifiers: None }),
        ));
        Arc::new(Node::new(
            SyntaxKind::PropertySignature,
            NodeData::PropertySignatureDeclaration(PropertySignatureDeclarationData {
                modifiers: None,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
        ))
    }

    pub(crate) fn parameter_node(
        &self,
        name: Arc<Node>,
        optional: bool,
        type_node: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parameter_node"); 
        let question_token = if optional {
            Some(self.keyword_node(SyntaxKind::QuestionToken))
        } else {
            None
        };
        Arc::new(Node::new(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: None,
                name,
                question_token,
                type_node: Some(type_node),
                initializer: None,
            }),
        ))
    }

    pub(crate) fn rest_type_node(&self, type_node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("rest_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::RestType,
            NodeData::RestTypeNode(RestTypeNodeData { type_node }),
        ))
    }

    pub(crate) fn type_operator_node(
        &self,
        operator: SyntaxKind,
        type_node: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("type_operator_node"); 
        Arc::new(Node::new(
            SyntaxKind::TypeOperator,
            NodeData::TypeOperatorNode(TypeOperatorNodeData {
                operator,
                type_node,
            }),
        ))
    }

    pub fn symbol_to_string(&mut self, symbol: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("symbol_to_string"); 
        self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::AllowAnyNodeKind,
            SymbolFlags::all(),
        )
    }

    pub fn symbol_to_string_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        flags: SymbolFormatFlags,
        _meaning: SymbolFlags,
    ) -> String { ::tsox_core::fntrace::enter("symbol_to_string_ex");
        if symbol.name == "__computed"
            && !symbol
                .check_flags
                .contains(tsox_frontend::ast::CheckFlags::Late)
        {
            let has_literal_name_type = self
                .value_symbol_links
                .get(symbol)
                .and_then(|l| l.name_type.as_ref())
                .is_some_and(|nt| {
                    nt.flags.intersects(
                        crate::checker::types_type_id::TYPE_FLAGS_STRING_OR_NUMBER_LITERAL,
                    )
                });
            if !has_literal_name_type {
                let decl = symbol
                    .value_declaration
                    .as_ref()
                    .or_else(|| symbol.declarations.first());
                if let Some(decl) = decl {
                    if let Some(name_node) = decl.name() {
                        if let tsox_frontend::ast::NodeData::ComputedPropertyName(d) = &name_node.data
                        {
                            return format!(
                                "[{}]",
                                crate::checker::checker_attach_explicit_type_arguments::qualified_name_text(&d.expression)
                            );
                        }
                    }
                }
            }
        }
        let name = match &symbol.value_declaration {
            Some(decl)
                if tsox_frontend::ast::mig::m3g_2::is_private_identifier_class_element_declaration(decl) =>
            {
                decl.name().map(|n| n.text().to_string()).unwrap_or_else(|| symbol.name.clone())
            }
            _ => symbol.name.clone(),
        };

        if flags.contains(SymbolFormatFlags::WriteTypeParametersOrArguments) {
            if let Some(tps) = self.collect_type_parameter_names(symbol) {
                if !tps.is_empty() {
                    return format!("{}<{}>", name, tps.join(", "));
                }
            }
        }
        name
    }

    pub(crate) fn collect_type_parameter_names(&self, symbol: &Arc<Symbol>) -> Option<Vec<String>> { ::tsox_core::fntrace::enter("collect_type_parameter_names"); 
        for decl in &symbol.declarations {
            let tps = match &decl.data {
                NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::TypeAliasDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::FunctionDeclaration(d) => d.type_parameters.as_ref(),
                _ => continue,
            };
            if let Some(tps) = tps {
                return Some(
                    tps.iter()
                        .map(|tp| match &tp.data {
                            NodeData::TypeParameterDeclaration(tpd) => tpd.name.text().to_string(),
                            _ => String::new(),
                        })
                        .collect(),
                );
            }
        }
        None
    }

    /// 类型参数显示文本：`T`、`T extends C`、`T = D`
    pub(crate) fn collect_type_parameter_displays(
        &self,
        symbol: &Arc<Symbol>,
    ) -> Option<Vec<String>> { ::tsox_core::fntrace::enter("collect_type_parameter_displays"); 
        for decl in &symbol.declarations {
            let tps = match &decl.data {
                NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::TypeAliasDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::FunctionDeclaration(d) => d.type_parameters.as_ref(),
                _ => continue,
            };
            if let Some(tps) = tps {
                if tps.is_empty() {
                    continue;
                }
                let sf = self.get_source_file_of_node(decl)?;
                let text = &sf.text;
                let node_text = |n: &Arc<Node>| {
                    let (s, e) = (n.pos().min(text.len()), n.end().min(text.len()));
                    if s < e {
                        text[s..e].to_string()
                    } else {
                        String::new()
                    }
                };
                return Some(
                    tps.iter()
                        .map(|tp| {
                            let NodeData::TypeParameterDeclaration(tpd) = &tp.data else {
                                return String::new();
                            };
                            let mut s = tpd.name.text().to_string();
                            if let Some(c) = &tpd.constraint {
                                s.push_str(" extends ");
                                s.push_str(&node_text(c));
                            }
                            if let Some(def) = &tpd.default_type {
                                s.push_str(" = ");
                                s.push_str(&node_text(def));
                            }
                            s
                        })
                        .collect(),
                );
            }
        }
        None
    }
}
