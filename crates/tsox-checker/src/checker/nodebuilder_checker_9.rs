use std::collections::HashSet;

use crate::checker::nodebuilder::*;
use tsox_frontend::ast::is_global_scope_augmentation;
use crate::checker::checker_heritage_retry_limit::Program;
use crate::checker::mig::m2g::{count_path_components, SortedSymbolNamePair};
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
use crate::checker::symbolaccessibility_symbol_table_id::{
    get_qualified_left_meaning, has_non_global_augmentation_external_module_symbol,
};
use tsox_tsoptions::modulespecifiers::mig::m3m_2::{
    get_all_module_paths, get_info, try_get_module_name_as_node_module,
};
use tsox_tsoptions::modulespecifiers::{
    ModuleSpecifierGenerationHost, ModuleSpecifierOptions, UserPreferences,
    IMPORT_MODULE_SPECIFIER_PREFERENCE_PROJECT_RELATIVE,
};

enum PropertyMemberDisplay {
    Type(Arc<Type>),
    Annotation(Arc<Node>),
}

impl Checker {
    pub(crate) fn object_literal_to_string(
        &mut self,
        _t: &Arc<Type>,
        structured: &StructuredTypeData,
        flags: TypeFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("object_literal_to_string");
        let parts = self.structured_members_to_strings(_t, structured, flags, false);
        if parts.is_empty() {
            "{}".to_string()
        } else if structured.properties.is_empty()
            && structured.call_signatures().is_empty()
            && structured.construct_signatures().len() == 1
        {
            parts.join("")
        } else if flags.contains(TypeFormatFlags::MULTILINE_OBJECT_LITERALS) {
            // 嵌套字面量的续行同步缩进（成员串内含换行时逐行加进）
            let inner: String = parts
                .iter()
                .map(|p| {
                    let indented = p
                        .split('\n')
                        .map(|l| format!("    {l}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{indented};")
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("{{\n{inner}\n}}")
        } else {
            format!("{{ {} }}", format!("{};", parts.join("; ")))
        }
    }

    pub(crate) fn structured_members_to_strings(
        &mut self,
        t: &Arc<Type>,
        structured: &StructuredTypeData,
        flags: TypeFormatFlags,
        _member_form: bool,
    ) -> Vec<String> { ::tsox_core::fntrace::enter("structured_members_to_strings");
        let mut parts: Vec<String> = Vec::new();

        for sig in structured.call_signatures() {
            // Go signatureToSignatureDeclarationHelper（nodebuilderimpl.go:1932）：
            // 每签名经 enterSignatureScope 隔离命名（cleanup 丢弃新增名），顺序
            // typeParameters（1941-1949）→ parameters（1963）→ this（1971-1979）
            // → returnType（1982）
            let scope = self.push_display_naming_scope();
            let tp = self.signature_type_param_prefix(sig);
            // Go signatureToTypeNode 参数表含 this 参数（signature.thisParameter
            // 置于实参表首位）：CallableFunction.apply 的 this: (this: T) => R
            // 由此呈现，缺位会连带丢失 apply/call 的接收者推断显示
            let this_prefix = sig
                .this_parameter
                .as_ref()
                .filter(|p| !p.name.is_empty())
                .map(|p| {
                    let t = self.get_type_of_symbol(p);
                    let type_str = self.type_to_string_ex(&t, flags);
                    format!("this: {type_str}")
                });
            let mut params: Vec<String> = Vec::new();
            if let Some(this) = this_prefix {
                params.push(this);
            }
            params.extend(
                sig.parameters
                    .iter()
                    .enumerate()
                    .map(|(i, param)| {
                        let name = param.name.clone();
                        let param_type = self
                            .signature_instantiated_param_type(sig, i)
                            .unwrap_or_else(|| self.get_type_of_symbol(param));
                        // Go symbolToParameterDeclaration（nodebuilderimpl.go:1788）
                        // →serializeTypeForDeclaration（:2380-2441）：注解与解析型
                        // 伪等价时复用注解节点呈现（源序）；约束复用语境中无类型
                        // 注解且无初始化的参数声明经 Go existingNodeTreeVisitor
                        // （nodecopy.go:689-724）合成 any 呈现
                        let type_str = if self.display_constraint_reuse_params
                            && crate::checker::mig::m2g_2::get_effective_parameter_declaration(
                                param,
                            )
                            .is_some_and(|d| {
                                matches!(
                                    &d.data,
                                    NodeData::ParameterDeclaration(pd)
                                        if pd.type_node.is_none() && pd.initializer.is_none()
                                )
                            })
                        {
                            "any".to_string()
                        } else {
                            self.annotated_param_type_text(
                                param,
                                &param_type,
                                Self::signature_params_instantiated(sig),
                                flags,
                            )
                            .unwrap_or_else(|| self.type_to_string_ex(&param_type, flags))
                        };
                        let prefix = if i + 1 == sig.parameters.len() && sig.has_rest_parameter() {
                            "..."
                        } else {
                            ""
                        };
                        if self.param_declared_optional(param) {
                            format!("{prefix}{name}?: {type_str}")
                        } else {
                            format!("{prefix}{name}: {type_str}")
                        }
                    }),
            );
            let ret_str = match self.signature_predicate_return_string(sig, flags) {
                Some(predicate_str) => predicate_str,
                None => {
                    let ret_type = self.get_return_type_of_signature_lazy(sig);
                    // Go serializeReturnTypeForSignature（nodebuilderimpl.go:2196，
                    // signatureToSignatureDeclarationHelper :1982 tryReuse=true）：
                    // 声明返回注解与解析型伪等价时复用注解节点呈现（对象字面量
                    // 内调用签名的 `(): m2.connectExport` 呈书写形而非链渲染形）
                    self.annotated_return_type_text(sig, &ret_type, flags)
                        .unwrap_or_else(|| self.type_to_string_ex(&ret_type, flags))
                }
            };
            self.pop_display_naming_scope(scope);
            parts.push(format!("{tp}({}): {}", params.join(", "), ret_str));
        }

        for sig in structured.construct_signatures() {
            let scope = self.push_display_naming_scope();
            let tp = self.signature_type_param_prefix(sig);
            let params: Vec<String> = sig
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    let param_type = self
                        .signature_instantiated_param_type(sig, i)
                        .unwrap_or_else(|| self.get_type_of_symbol(param));
                    let type_str = if self.display_constraint_reuse_params
                        && crate::checker::mig::m2g_2::get_effective_parameter_declaration(
                            param,
                        )
                        .is_some_and(|d| {
                            matches!(
                                &d.data,
                                NodeData::ParameterDeclaration(pd)
                                    if pd.type_node.is_none() && pd.initializer.is_none()
                            )
                        })
                    {
                        "any".to_string()
                    } else {
                        self.annotated_param_type_text(
                            param,
                            &param_type,
                            Self::signature_params_instantiated(sig),
                            flags,
                        )
                        .unwrap_or_else(|| self.type_to_string_ex(&param_type, flags))
                    };
                    let prefix = if i + 1 == sig.parameters.len() && sig.has_rest_parameter() {
                        "..."
                    } else {
                        ""
                    };
                    format!("{prefix}{}: {type_str}", param.name)
                })
                .collect();
            let ret_type = self.get_return_type_of_signature_lazy(sig);
            // 同调用签名臂：构造签名返回注解伪等价时复用书写形
            //（Go serializeReturnTypeForSignature，nodebuilderimpl.go:2196）；
            // 无注解时模板态匿名类实例型呈裸书写名，其余链式渲染
            let ret_str = self
                .annotated_return_type_text(sig, &ret_type, flags)
                .unwrap_or_else(|| self.construct_sig_return_string(&ret_type, flags));
            self.pop_display_naming_scope(scope);
            parts.push(format!("new {tp}({}): {}", params.join(", "), ret_str));
        }

        for info in &structured.index_infos {
            // Go indexInfoToObjectComputedNamesOrSignatureDeclaration
            //（nodebuilderimpl.go:2826-2828 调用，:2232-2279）：对象/类计算名
            // 贡献的索引信息以 components 成员节点呈现，签名形态仅作回退
            if let Some(members) = self.object_literal_computed_name_members_text(info, flags) {
                parts.extend(members);
                continue;
            }
            let key_str = info
                .key_type
                .as_ref()
                .map(|k| self.type_to_string_ex(k, flags))
                .unwrap_or_else(|| "string".to_string());
            // Go：反向映射型的索引签名值打印省略号
            let val_str = if t
                .object_flags
                .contains(crate::checker::ObjectFlags::ReverseMapped)
            {
                "...".to_string()
            } else {
                info.value_type
                    .as_ref()
                    .map(|v| self.type_to_string_ex(v, flags))
                    .unwrap_or_else(|| "any".to_string())
            };

            let key_name = info
                .declaration
                .as_ref()
                .and_then(|d| {
                    let NodeData::IndexSignatureDeclaration(sd) = &d.data else {
                        return None;
                    };
                    sd.parameters.iter().next().and_then(|p| match &p.data {
                        NodeData::ParameterDeclaration(pd) => Some(pd.name.text().to_string()),
                        _ => None,
                    })
                })
                .unwrap_or_else(|| "x".to_string());
            let readonly = if info.is_readonly { "readonly " } else { "" };
            parts.push(format!("{readonly}[{key_name}: {key_str}]: {val_str}"));
        }

        // Go setStructuredTypeMembers → getNamedMembers：类/接口容器声明成员先于继承成员，
        // 两段各自按 compareSymbols（声明位置优先、名字回退）排序后参与显示
        let container = t.symbol.clone();
        let class_like = container.as_ref().is_some_and(|s| {
            s.flags.intersects(
                tsox_frontend::ast::SymbolFlags::Class
                    | tsox_frontend::ast::SymbolFlags::Interface,
            )
        });
        let mut contained: Vec<&Arc<Symbol>> = Vec::new();
        let mut rest: Vec<&Arc<Symbol>> = Vec::new();
        for prop in &structured.properties {
            let owned = class_like
                && prop
                    .value_declaration
                    .as_ref()
                    .zip(container.as_ref())
                    .is_some_and(|(d, c)| {
                        c.declarations.iter().any(|cd| {
                            cd.loc.pos() <= d.loc.pos() && d.loc.end() <= cd.loc.end()
                        })
                    });
            if owned {
                contained.push(prop);
            } else {
                rest.push(prop);
            }
        }
        contained.sort_by(|a, b| self.compare_symbols_for_display(a, b));
        rest.sort_by(|a, b| self.compare_symbols_for_display(a, b));
        let sorted_props: Vec<&Arc<Symbol>> = contained.into_iter().chain(rest).collect();
        for prop in sorted_props {
            // Go getPropertyNameNodeForSymbolFromNameType（nodebuilderimpl.go:2611-2659）：
            // nameType 为枚举字面量型的 late-bound 计算名属性，属性名重建为
            // [枚举.成员] 计算名，先于符号名字面量呈现
            let enum_computed = self.get_property_name_node_for_symbol_from_name_type(prop);
            // Go symbolToString：晚绑定名（__@ 前缀）渲染为计算属性名；
            // well-known 符号成员为 [Symbol.x]，其余 unique symbol 建名为 [x]
            let late_bound = prop.name.strip_prefix("__@");
            let well_known = late_bound
                .filter(|s| crate::binder::symbols_binder_4::is_well_known_symbol_member_name(s))
                .map(|s| format!("[Symbol.{s}]"));
            let computed_name = enum_computed
                .or(well_known)
                .or_else(|| late_bound.map(|s| format!("[{s}]")));
            let name = computed_name
                .clone()
                .unwrap_or_else(|| prop.name.clone());

            // Go nodebuilder classifyPropertyName（nodebuilderimpl.go:2535-2545）：
            // 方法名 new 恒按字符串字面量呈引号（isMethod 取
            // symbol.Flags&SymbolFlagsMethod，getPropertyNameNodeForSymbol
            // nodebuilderimpl.go:2592，经 addPropertyToElementList
            // nodebuilderimpl.go:2696 施于每个成员）；identifier 原样/数值名非
            // stringNamed 不加引号；引号风格按全部声明为单引号字符串名
            //（isSingleQuotedStringNamed nodebuilderimpl.go:2575-2581，经
            // getPropertyNameNodeForSymbol nodebuilderimpl.go:2594 →
            // NewStringLiteral TokenFlagsSingleQuote）。无真声明的急切展开
            // 映射成员（Go 侧 prop.Declarations = modifiersProp.Declarations
            // checker.go:22199-22208）引号风格取 value_symbol_links 的
            // display_mapped_* 提示
            let mapped_quoting = self
                .value_symbol_links
                .get(&**prop)
                .map(|l| (l.display_mapped_string_named, l.display_mapped_single_quoted));
            let string_named = mapped_quoting.is_some_and(|(s, _)| s)
                || (!prop.declarations.is_empty()
                    && prop
                        .declarations
                        .iter()
                        .all(|d| d.name().is_some_and(|n| n.kind == SyntaxKind::StringLiteral)));
            let is_method = prop.flags.intersects(SymbolFlags::Method);
            let single_quoted = mapped_quoting.is_some_and(|(s, q)| s && q)
                || (!prop.declarations.is_empty()
                    && prop.declarations.iter().all(|d| {
                        d.name().is_some_and(|n| {
                            matches!(
                                &n.data,
                                NodeData::StringLiteral(sd)
                                    if sd.token_flags & tsox_frontend::scanner::TOKEN_FLAGS_SINGLE_QUOTE != 0
                            )
                        })
                    }));
            let name = if computed_name.is_some()
                || (!(is_method && name == "new")
                    && crate::checker::checker_get_excluded_symbol_flags::is_valid_identifier_text(&name))
                || (!string_named && crate::checker::nodecopy_property_name::is_numeric_literal_name(&name))
            {
                name
            } else if single_quoted {
                format!("'{name}'")
            } else {
                format!("\"{name}\"")
            };
            let prop_type = self.get_type_of_symbol(prop);
            // Go serializeTypeForDeclaration（nodebuilderimpl.go:2331-2437，经
            // addPropertyToElementList nodebuilderimpl.go:2781）：属性成员序列化
            // 优先复用声明注解（pseudoTypeDirect），注解型与解析型一致（可选注解
            // 按 treatUndefinedAsOptional 容许差缺省 undefined，
            // pseudoTypeEquivalentToType pseudotypenodebuilder.go:367-404）时按
            // 注解渲染，否则回落解析型；error 型注解经
            // pseudoTypeToNodeWithCheckerFallback（pseudotypenodebuilder.go:33-46）
            // Direct 臂复用注解节点原文本
            let display = self.reuse_declared_property_type(prop, &prop_type);
            // Go shouldUsePlaceholderForProperty：反向映射属性的三条件省略 +
            // 打印栈追踪（嵌套时对非匿名源立即截断为 ...）
            let use_placeholder = self.should_use_placeholder_for_property(prop);
            let pushed = prop
                .check_flags
                .contains(tsox_frontend::ast::CheckFlags::ReverseMapped);
            if pushed {
                self.reverse_mapped_print_stack.push(Arc::clone(prop));
            }
            let type_str = if use_placeholder {
                // Go createElidedInformationPlaceholder：NoTruncation 打印 any，
                // 否则省略号
                if flags.contains(
                    crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags::NO_TRUNCATION,
                ) {
                    "any".to_string()
                } else {
                    "...".to_string()
                }
            } else if let Some(text) = self.annotated_property_type_text(prop, &prop_type, flags) {
                text
            } else {
                match &display {
                    PropertyMemberDisplay::Type(t) => self.type_to_string_ex(t, flags),
                    PropertyMemberDisplay::Annotation(node) => self
                        .print_annotation_type_node(node, flags)
                        .unwrap_or_else(|| {
                            let declared = self.get_type_from_type_node(node);
                            self.type_to_string_ex(&declared, flags)
                        }),
                }
            };
            if pushed {
                self.reverse_mapped_print_stack.pop();
            }
            // Go addPropertyToElementList（nodebuilderimpl.go:2700-2746）accessor
            // 臂：writeType := getWriteTypeOfSymbol(propertySymbol)，与
            // propertyType（getNonMissingTypeOfSymbol，上文 266 行）指针分异且
            // 双侧非 error 时按 get/set 声明渲染后即返回。克隆符号（加宽等
            // createSymbolWithType 系）flags 丢 Accessor 位时 getWriteTypeOfSymbol
            // 落 Property 臂返回读型恒等，双侧注解俱在且指针分异即同判据的
            // 声明级等价形态，保留为兜底
            let getter_decl =
                tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(prop, SyntaxKind::GetAccessor);
            let setter_decl =
                tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(prop, SyntaxKind::SetAccessor);
            let mut divergent = false;
            if prop.flags.intersects(SymbolFlags::ACCESSOR) {
                let write_type = self.get_write_type_of_symbol(prop);
                divergent = !self.is_error_type(&prop_type)
                    && !self.is_error_type(&write_type)
                    && !Arc::ptr_eq(&prop_type, &write_type);
            }
            if !divergent {
                if let (Some(g), Some(s)) = (&getter_decl, &setter_decl) {
                    divergent = match (
                        self.get_annotated_accessor_type(g),
                        self.get_annotated_accessor_type(s),
                    ) {
                        (Some(rt), Some(wt)) => {
                            !self.is_error_type(&rt)
                                && !self.is_error_type(&wt)
                                && !Arc::ptr_eq(&rt, &wt)
                        }
                        _ => false,
                    };
                }
            }
            if divergent {
                if let (Some(g), Some(s)) = (&getter_decl, &setter_decl) {
                    if let Some(form) = self.accessor_member_form(g, &name, flags) {
                        parts.push(form);
                    }
                    if let Some(form) = self.accessor_member_form(s, &name, flags) {
                        parts.push(form);
                    }
                    continue;
                }
            }
            // Go isReadonlySymbol（checker.go:14610）accessor 臂：无 SetAccessor
            // 位的 accessor 恒只读。以声明对复算——加宽克隆（createSymbolWithType
            // 系）声明侧存活实证于本轮 obj2 行 26 分异臂触发，flags 侧存在丢失
            // 通道（obj3 行 45 新鲜型 readonly 而加宽型丢失）
            let readonly = self.is_readonly_symbol(prop)
                || (getter_decl.is_some() && setter_decl.is_none());
            let method_suffix = if use_placeholder {
                None
            } else {
                self.method_form_suffix(prop, &prop_type, flags)
            };
            if let Some(suffix) = method_suffix {
                let ro = if readonly { "readonly " } else { "" };
                if prop.flags.contains(SymbolFlags::Optional) {
                    parts.push(format!("{ro}{}?{suffix}", name));
                } else {
                    parts.push(format!("{ro}{}{suffix}", name));
                }
            } else if prop.flags.contains(SymbolFlags::Optional) {
                let ro = if readonly { "readonly " } else { "" };
                parts.push(format!("{ro}{}?: {}", name, type_str));
            } else if readonly {
                parts.push(format!("readonly {}: {}", name, type_str));
            } else {
                parts.push(format!("{}: {}", name, type_str));
            }
        }


        parts
    }

    // Go serializeTypeForDeclaration 注解复用门的显示侧镜像：带注解的
    // PropertySignature/PropertyDeclaration 成员，注解型与解析型伪等价
    //（pseudoTypeEquivalentToType pseudotypenodebuilder.go:367-404 的 Direct
    // 恒等判据立于 getTypeFromTypeNode 节点级缓存，呈现相同的不同实例按
    // 常规呈现逐字相同补判）时返回注解节点复用呈现，其余形态回落解析型
    fn reuse_declared_property_type(
        &mut self,
        prop: &Arc<Symbol>,
        resolved: &Arc<Type>,
    ) -> PropertyMemberDisplay {
        // Go 复用门首条件 ctx.enclosingDeclaration != nil
        // （nodebuilderimpl.go:2380；addPropertyToElementList 于 :2697 已把
        // ctx 恢复为 TypeToTypeNode 入口的值）：错误消息显示经
        // TypeToString(t, nil)（printer.go:51-57，relater.go:1342-1355）传
        // nil enclosing，门不成立，成员按解析型渲染（可选注解带隐式
        // | undefined）；.types 基线通道 TypeToStringEx 传 node.Parent
        // 非 nil，门成立。Rust 以 type_render_enclosing（仅
        // type_to_string_ex_with_enclosing 设，nodebuilder_checker_7.rs:28）
        // 与 display_enclosing_node（错误显示 context-sensitive 分支设，
        // relater_compare_checker_4.rs:197）承载，判定同
        // symbol_chain_display_name 惯例
        if self.type_render_enclosing.is_none() && self.display_enclosing_node.is_none() {
            return PropertyMemberDisplay::Type(Arc::clone(resolved));
        }
        let declaration = prop
            .value_declaration
            .as_ref()
            .or_else(|| prop.declarations.first());
        let type_node = match declaration.map(|d| &d.data) {
            Some(tsox_frontend::ast::NodeData::PropertySignatureDeclaration(d))
                if d.type_node.kind != SyntaxKind::MissingDeclaration =>
            {
                Some(Arc::clone(&d.type_node))
            }
            Some(tsox_frontend::ast::NodeData::PropertyDeclaration(d)) => d.type_node.clone(),
            _ => return PropertyMemberDisplay::Type(Arc::clone(resolved)),
        };
        let Some(type_node) = type_node else {
            return PropertyMemberDisplay::Type(Arc::clone(resolved));
        };
        // Go pseudoTypeEquivalentToType 的 Direct 恒等判据
        //（pseudotypenodebuilder.go:374-378）立于 getTypeFromTypeNode 节点级
        // 缓存永久之上：符号类型与注解求值同源同一对象
        let declared = self.get_type_from_type_node(&type_node);
        if crate::checker::utilities::is_type_error(resolved) {
            // Go pseudoTypeEquivalentToType error 短路
            //（pseudotypenodebuilder.go:370-372）后经
            // pseudoTypeToNodeWithCheckerFallback Direct 臂复用注解节点
            //（pseudotypenodebuilder.go:33-46 → pseudoTypeToNode :54-55
            // reuseTypeNode），显示取注解子树原文本而非解析型字符串；
            // JSDoc 意图型注解（canReuseExistingJSTypeNode :35）不可复用，
            // 回落解析型（typeToTypeNode(errorType) → any）
            if self
                .get_intended_type_from_jsdoc_type_reference(&type_node)
                .is_none()
            {
                return PropertyMemberDisplay::Annotation(type_node);
            }
            return PropertyMemberDisplay::Type(declared);
        }
        if same_type_identity(&declared, resolved) {
            return PropertyMemberDisplay::Annotation(type_node);
        }
        if prop.flags.contains(SymbolFlags::Optional) {
            let stripped = self
                .get_type_with_facts(resolved, crate::checker::types::TypeFacts::NE_UNDEFINED);
            if same_type_identity(&stripped, &declared) {
                return PropertyMemberDisplay::Annotation(type_node);
            }
        }
        PropertyMemberDisplay::Type(Arc::clone(resolved))
    }

    // Go addPropertyToElementList（nodebuilderimpl.go:2773-2788）属性型经
    // serializeTypeForDeclaration（:2380-2441）tryReuse 臂：pseudochecker
    // GetTypeOfDeclaration（lookup.go:45/105-110）对带注解的
    // PropertySignature/PropertyDeclaration 产 Direct(annotation)，
    // pseudoTypeEquivalentToType（pseudotypenodebuilder.go:367-404）判定等价后
    // pseudoTypeToNodeWithCheckerFallback（:2405）复用注解节点——联合等复合
    // 注解按书写成员序呈现，不按 getUnionType 求值后的 CompareTypes 序
    pub(crate) fn annotated_property_type_text(
        &mut self,
        prop: &Arc<Symbol>,
        resolved: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("annotated_property_type_text");
        // 仅 .types 基线通道（TypeToStringEx 传 node.Parent 设
        // type_render_enclosing）启用；错误显示通道维持解析型渲染不变
        let enclosing = self.type_render_enclosing.clone()?;
        let declaration = prop
            .value_declaration
            .as_ref()
            .or_else(|| prop.declarations.first())?;
        let tn = match &declaration.data {
            NodeData::PropertySignatureDeclaration(d)
                if d.type_node.kind != SyntaxKind::MissingDeclaration =>
            {
                Arc::clone(&d.type_node)
            }
            NodeData::PropertyDeclaration(d) => d.type_node.clone()?,
            NodeData::PropertyAssignment(d) => {
                // Go pseudochecker typeFromPropertyAssignment（lookup.go:59-78）：
                // 无书写注解时按初始化式求伪型；typeFromExpression
                // （lookup.go:277-316）对括号穿透后的 as/尖括号断言经
                // typeFromTypeAssertion（lookup.go:534-540，非 const 型引用）
                // 产 Direct(断言型节点)，其余 Inferred/NoResult 伪等价不成立
                Self::pseudo_direct_annotation_node_of_initializer(
                    &d.type_node,
                    &d.initializer,
                )?
            }
            _ => return None,
        };
        // 复用前可及性门与参数注解同源（annotated_param_type_text）：Go 复用
        // 访客对注解内实体名引用做 isSymbolAccessible 判定，任一不可及即整体
        // 回退结构渲染
        if self.annotation_has_inaccessible_reference(&tn, &Some(enclosing)) {
            return None;
        }
        // Go pseudoTypeEquivalentToType 的 isOptionalAnnotated 臂
        //（pseudotypenodebuilder.go:378-393）：可选注解按剥 undefined 后恒等
        let resolved_cmp = if prop.flags.contains(SymbolFlags::Optional) {
            self.get_type_with_facts(resolved, crate::checker::types::TypeFacts::NE_UNDEFINED)
        } else {
            Arc::clone(resolved)
        };
        self.equivalent_annotation_text(&tn, &resolved_cmp, flags)
    }

    // Go indexInfoToObjectComputedNamesOrSignatureDeclaration 的组件呈现
    //（nodebuilderimpl.go:2232-2279）：components 非空且全部计算名可序列化
    //（isTriviallySerializableComputedName nodebuilderimpl.go:2222-2230：
    // 计算名表达式为实体名表达式且对 enclosingDeclaration 可及）时逐组件
    // 产出 [计算名]: 成员符号型；晚绑定名组件跳过（hasLateBindableName，
    // 由属性成员通道呈现），全为晚绑定时产出空成员表（Go 返回空元素集，
    // 不落索引签名形态）。None = 回退索引签名形态（无组件/enclosing 缺失/
    // 计算名不可序列化）
    fn object_literal_computed_name_members_text(
        &mut self,
        info: &crate::checker::types_impl_chunk_3::IndexInfo,
        flags: TypeFormatFlags,
    ) -> Option<Vec<String>> {
        if info.components.is_empty() {
            return None;
        }
        let enclosing = self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone())?;
        let all_serializable = info.components.iter().all(|c| {
            c.name().is_some_and(|name| {
                tsox_frontend::ast::is_computed_property_name(&name)
                    && name.expression().is_some_and(|expr| {
                        tsox_frontend::ast::is_entity_name_expression(&expr)
                            && self.is_entity_name_visible(&expr, &enclosing).accessibility
                                == crate::checker::types_alias_symbol_links::SymbolAccessibility::
                                    Accessible
                    })
            })
        });
        if !all_serializable {
            return None;
        }
        let mut members: Vec<String> = Vec::new();
        for comp in &info.components {
            if self.has_late_bindable_name(comp) {
                continue;
            }
            let name = comp.name()?;
            let expr = name.expression()?;
            let name_text = self
                .node_source_text(&expr)
                .unwrap_or_else(|| expr.text().to_string());
            let sym = self.program.symbol_map().symbol_of(comp).map(Arc::clone)?;
            let value_type = self.get_type_of_symbol(&sym);
            let readonly = if info.is_readonly { "readonly " } else { "" };
            members.push(format!(
                "{readonly}[{name_text}]: {}",
                self.type_to_string_ex(&value_type, flags)
            ));
        }
        Some(members)
    }

    // Go pseudochecker typeFromExpression 的 Direct 产出臂（lookup.go:277-316）：
    // 声明位带书写注解直取；括号逐层穿透后命中 as/尖括号断言且断言型非
    // const 引用（typeFromTypeAssertion lookup.go:534-540）时返回断言型节点
    fn pseudo_direct_annotation_node_of_initializer(
        type_node: &Arc<Node>,
        initializer: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        // 表达式位 PropertyAssignment 无书写注解：解析器置 Unknown 占位节点
        //（expressions_parser_7.rs），非注解
        let has_written_annotation = type_node.kind != SyntaxKind::MissingDeclaration
            && type_node.kind != SyntaxKind::Unknown;
        if has_written_annotation {
            return Some(Arc::clone(type_node));
        }
        let mut expr = initializer;
        while let tsox_frontend::ast::NodeData::ParenthesizedExpression(d) = &expr.data {
            expr = &d.expression;
        }
        let assertion_type_node = match expr.kind {
            SyntaxKind::TypeAssertionExpression => {
                let tsox_frontend::ast::NodeData::TypeAssertion(d) = &expr.data else {
                    return None;
                };
                &d.type_node
            }
            SyntaxKind::AsExpression => {
                let tsox_frontend::ast::NodeData::AsExpression(d) = &expr.data else {
                    return None;
                };
                &d.type_node
            }
            _ => return None,
        };
        (!crate::checker::utilities_has_only_expression_initialization::is_const_type_reference(
            assertion_type_node,
        ))
        .then(|| Arc::clone(assertion_type_node))
    }

    // Go nodebuilder ctx.enclosingDeclaration 的宿主文件
    //（nodebuilderimpl.go:701 GetSourceFileOfNode(MostOriginal(enclosing))）：
    // hover/错误消息路径设 display_enclosing_file，.types 基线 TypeToStringEx
    // 传 node.Parent（type_render_enclosing），跨文件别名查找从两态取上下文文件
    pub(crate) fn display_context_file(
        &self,
    ) -> Option<Arc<tsox_frontend::ast::SourceFile>> { ::tsox_core::fntrace::enter("display_context_file");
        self.display_enclosing_file.clone().or_else(|| {
            self.type_render_enclosing
                .as_ref()
                .and_then(|enc| self.get_source_file_of_node(enc))
        })
    }

    // Go computeModuleSpecifiers 的 existing-import 快路径
    //（modulespecifiers/specifiers.go:389-412）：导入文件已有解析到目标
    // SourceFile 的 import 说明符时，生成 specifier 即该 import 文本
    //（GetResolvedModuleFromModuleSpecifier 路径匹配；relativePreference
    // Auto 保留相对文本，非 Node16 体系 mode 检查双 None 不短路）
    pub(crate) fn module_specifier_from_existing_import(
        &mut self,
        sym: &Arc<Symbol>,
        context_file: &Arc<tsox_frontend::ast::SourceFile>,
    ) -> Option<String> { ::tsox_core::fntrace::enter("module_specifier_from_existing_import");
        let module_file = sym
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
            .and_then(|d| self.get_source_file_of_node(d))
            .map(|sf| sf.file_name.clone())?;
        let statements: &[Arc<Node>] = match &context_file.node.data {
            NodeData::SourceFile(sf) => &sf.statements.nodes,
            _ => return None,
        };
        for stmt in statements {
            let spec: &Arc<Node> = match &stmt.data {
                // Go ast SourceFile.Imports()：import 说明符、
                // import = require 的 external reference 表达式、
                // export ... from 的模块说明符均在收集范围内
                NodeData::ImportDeclaration(d) => &d.module_specifier,
                NodeData::ExportDeclaration(d) => match &d.module_specifier {
                    Some(m) => m,
                    None => continue,
                },
                NodeData::ImportEqualsDeclaration(d) => {
                    match &d.module_reference.data {
                        NodeData::ExternalModuleReference(emr) => &emr.expression,
                        _ => continue,
                    }
                }
                _ => continue,
            };
            if spec.kind != SyntaxKind::StringLiteral {
                continue;
            }
            let text = spec.text().trim_matches(['"', '\'']).to_string();
            let Some(resolved) = self.program.resolve_external_module_path(
                &text,
                &context_file.file_name,
                tsox_core::core::compiler_options::ModuleKind::None,
            ) else {
                continue;
            };
            // Go computeModuleSpecifiers（specifiers.go:388-396）的比较集合
            // modulePaths 由 getAllModulePathsWorker 展开 GetRedirectTargets
            //（specifiers.go:290-296）：PackageId 去重（compiler/filesparser.go:
            // 460-486，默认启用）把重定向路径映射到首个包实例的 SourceFile。
            // 本仓等价：两侧路径各经 GetSourceFileForResolvedModule
            //（compiler/program.go:2116-2126，filesByPath 去重语义）归一后比
            // 文件名——duplicatePackage_peerDependencies 的 b/index.d.ts
            // "foo" 解析到 b/node_modules/foo（去重重定向到 a 侧包实例的同一
            // SourceFile），与链根 moduleFile 归一后同一文件，命中现有 import
            // 文本 "foo"
            let resolved_file = self
                .program
                .get_source_file_for_resolved_module(&resolved)
                .map(|sf| sf.file_name.clone())
                .unwrap_or_else(|| resolved.clone());
            let module_file_name = self
                .program
                .get_source_file_for_resolved_module(&module_file)
                .map(|sf| sf.file_name.clone())
                .unwrap_or_else(|| module_file.clone());
            if resolved_file == module_file_name
                || self.program.canonicalize_path(&resolved_file)
                    == self.program.canonicalize_path(&module_file_name)
            {
                return Some(text);
            }
        }
        None
    }

    // Go getLocalModuleSpecifier（modulespecifiers/specifiers.go:504-545）
    // paths=nil 默认臂：以显示上下文文件目录为基 GetRelativePathFromDirectory，
    // processEnding Minimal（specifiers.go:677-683）去扩展名后剥尾部 /index，
    // 仅当剥后路径无同名文件（目录名不被文件占用）
    pub(crate) fn module_specifier_relative_to_context(
        &mut self,
        sym: &Arc<Symbol>,
        context_file: &Arc<tsox_frontend::ast::SourceFile>,
    ) -> Option<String> {
        let module_file = sym
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
            .and_then(|d| self.get_source_file_of_node(d))
            .map(|sf| sf.file_name.clone())?;
        // Go modulespecifiers/specifiers.go:178 sourceDirectory =
        // GetDirectoryPath(importingSourceFileName)（path.go:266）：导入目录取
        // GetDirectoryPath（"/main.ts" → "/"），非末斜杠前缀截断——根路径文件
        // 截出空串后经 cwd 绝对化会多出相对层（../other）
        let importing_dir = tsox_core::tspath::get_directory_path(&context_file.file_name);
        let options = tsox_core::tspath::ComparePathsOptions {
            use_case_sensitive_file_names: self.program.use_case_sensitive_file_names(),
            current_directory: self.program.current_directory().to_string(),
        };
        // GetRelativePathFromDirectory 对一侧绝对一侧相对的输入 panic——
        // 两侧先按 cwd 绝对化（Go 模块说明符推导内部路径恒绝对），
        // bundled:// 等异卷形态回退 basename 兜底
        let cwd = self.program.current_directory();
        let from_abs =
            tsox_core::tspath::get_normalized_absolute_path(&importing_dir, cwd);
        let to_abs = tsox_core::tspath::get_normalized_absolute_path(&module_file, cwd);
        if (tsox_core::tspath::get_root_length(&from_abs) > 0)
            != (tsox_core::tspath::get_root_length(&to_abs) > 0)
        {
            return None;
        }
        let relative = tsox_core::tspath::mig::m3i::get_relative_path_from_directory(
            &from_abs,
            &to_abs,
            &options,
        );
        let prefixed =
            tsox_core::tspath::ensure_path_is_non_module_name(&relative);
        let no_extension = tsox_core::tspath::remove_file_extension(&prefixed);
        // Go tryGetAnyFileFromPath（modulespecifiers/util.go:204-218）：剥 /index
        // 前查剥后路径加 TS/JS/node/json 各扩展是否存在同名文件，存在则保留 index
        match no_extension.strip_suffix("/index").map(|s| s.to_string()) {
            Some(candidate)
                if ![".ts", ".tsx", ".d.ts", ".js", ".jsx", ".json", ".node"]
                    .iter()
                    .any(|e| self.program.file_exists(&format!("{candidate}{e}"))) =>
            {
                Some(candidate)
            }
            _ => Some(no_extension),
        }
    }

    // Go computeModuleSpecifiers 的 node_modules 臂（modulespecifiers/
    // specifiers.go:428-447）：目标文件任一路径命中 node_modules 时先取
    // tryGetModuleNameAsNodeModule 包名 specifier（@types 代理目录还原实际
    // 包名，包根 index 文件呈裸包名），选取序 nodeModulesSpecifiers 先于
    // relativeSpecifiers（specifiers.go:484-489）；无法生成（兄弟
    // node_modules 等）回退相对路径臂
    pub(crate) fn module_specifier_node_modules_name(
        &mut self,
        sym: &Arc<Symbol>,
    ) -> Option<String> { ::tsox_core::fntrace::enter("module_specifier_node_modules_name");
        let module_file = sym
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
            .and_then(|d| self.get_source_file_of_node(d))
            .map(|sf| sf.file_name.clone())?;
        let importing_file = self.display_context_file()?;
        let host = DisplaySpecifierHost {
            program: self.program.as_ref(),
        };
        let info = get_info(&importing_file.file_name, &host);
        let preferences = UserPreferences {
            import_module_specifier_preference: IMPORT_MODULE_SPECIFIER_PREFERENCE_PROJECT_RELATIVE
                .to_string(),
            ..UserPreferences::default()
        };
        let options = ModuleSpecifierOptions::default();
        let compiler_options = self.program.options();
        let module_paths = get_all_module_paths(
            &info,
            &module_file,
            &host,
            compiler_options,
            &preferences,
            &options,
        );
        for module_path in &module_paths {
            if !module_path.is_in_node_modules {
                continue;
            }
            let name = try_get_module_name_as_node_module(
                module_path,
                &info,
                importing_file.as_ref(),
                &host,
                compiler_options,
                &preferences,
                false,
                tsox_core::core::compiler_options::ResolutionMode::None,
            );
            if !name.is_empty() {
                return Some(name);
            }
        }
        None
    }

    // Go getSymbolChain 的别名感知限定：命名空间父级链显示时，
    // 遇显示上下文文件内解析到该命名空间的别名用别名名；到 enclosing 文件自身模块不加前缀
    pub(crate) fn namespace_qualifier_of(&mut self, symbol: &Arc<Symbol>) -> Option<String> { ::tsox_core::fntrace::enter("namespace_qualifier_of"); 
        let mut parts: Vec<String> = Vec::new();
        let mut cur = symbol
            .parent()
            .clone()
            .or_else(|| self.namespace_container_from_declarations(symbol));
        // Go isSymbolAccessible：符号与显示上下文同文件时可无限定访问，
        // 错误消息等场景（display_enclosing_file 缺失）以 current_file 兜底
        let enclosing_file = self
            .display_enclosing_file
            .clone()
            .or_else(|| self.current_file.clone());
        if let Some(enc_file) = enclosing_file
            && symbol.declarations.iter().any(|d| {
                tsox_frontend::ast::utilities::get_source_file_of_node(d)
                    .is_some_and(|sf| Arc::ptr_eq(&sf, &enc_file.node))
            })
        {
            return None;
        }
        // Go getSymbolChain（nodebuilderimpl.go:1127-1135）先取可及别名链：
        // 显示上下文文件内的别名解析目标经自身 exports 暴露该符号时
        //（getCandidateListForSymbol，symbolaccessibility.go:617-641，export=
        // 符号带 bindCommonJSTypeExports 提升的导出），链 = [别名, 符号]，
        // 限定前缀即别名名（a.Foo）
        if let Some(enc_file) = self.display_context_file()
            && let Some(alias_name) = self.alias_of_symbol_via_exports_in_file(symbol, &enc_file)
        {
            return Some(alias_name);
        }
        while let Some(ns) = cur {
            if !ns.flags.intersects(SymbolFlags::MODULE) {
                // Go getSymbolChain：非模块父（脚本文件符号/函数）终止上爬，
                // 已收集的命名空间段保留；纯类型命名空间（NamespaceModule）
                // 同样参与限定（Underscore.Static.all 的 Underscore 段）
                break;
            }
            // declare global 增强容器不参与限定名
            if ns
                .declarations
                .iter()
                .any(|d| tsox_frontend::ast::is_global_scope_augmentation(d))
            {
                break;
            }
            // hover 在该命名空间声明内：符号就地可访问，无需限定；
            // 位置查询（get_type_at_location）同样记录可达性位置
            if let Some(enclosing) = self
                .display_enclosing_node
                .clone()
                .or_else(|| self.access_location.clone())
                && ns
                    .declarations
                    .iter()
                    .any(|d| Self::node_within(&enclosing, d))
            {
                return if parts.is_empty() { None } else { Some(parts.join(".")) };
            }
            // 文件模块（含 lib 全局）：路径名不参与限定；跨文件引用经别名（import）访问
            let is_file_module = ns
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::SourceFile);
            if let Some(file) = self.display_context_file()
                && let Some(alias_name) = self.alias_name_of_namespace_in_file(&ns, &file)
            {
                parts.insert(0, alias_name);
                return Some(parts.join("."));
            }
            if is_file_module
                || ns.name
                    .chars()
                    .next()
                    .is_some_and(|c| c == '"' || c == '\'' || c == '`')
            {
                // 外部模块名段（含 declare module 增强建的符号，声明表无
                // SourceFile）不参与限定：跨文件经 import 别名显示
                // （tsc getSymbolChain 跳过外部模块 root 段）
                return if parts.is_empty() { None } else { Some(parts.join(".")) };
            }
            parts.insert(0, ns.name.clone());
            cur = ns.parent();
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("."))
        }
    }

    // 类型成员限定名：只收集命名空间段（ValueModule 且非文件模块），不使用 alias 前缀
    // （tsc getSymbolChain：外部模块 root 的链段被跳过；值成员才经 alias 显示）
    #[allow(dead_code)]
    pub(crate) fn namespace_only_qualifier_of(&mut self, symbol: &Arc<Symbol>) -> Option<String> { ::tsox_core::fntrace::enter("namespace_only_qualifier_of"); 
        let mut parts: Vec<String> = Vec::new();
        let mut cur = symbol.parent().clone();
        while let Some(ns) = cur {
            if !ns.flags.contains(SymbolFlags::ValueModule) {
                return None;
            }
            // declare global 增强容器不参与限定名（tsc IsGlobalScopeAugmentation 跳过）
            if ns.declarations.iter().any(|d| tsox_frontend::ast::is_global_scope_augmentation(d)) {
                break;
            }
            if ns.declarations.iter().any(|d| d.kind == SyntaxKind::SourceFile) {
                break;
            }
            parts.insert(0, ns.name.clone());
            cur = ns.parent();
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("."))
        }
    }

    // 符号的命名空间限定显示名：export= 符号退化为限定名（隔代自身），
    // 否则 限定名.符号名（Go getSymbolChain + getAliasForSymbolInContainer）
    pub(crate) fn namespace_qualified_display_name(&mut self, sym: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("namespace_qualified_display_name"); 
        let export_eq_container = sym
            .parent()
            .or_else(|| self.namespace_container_from_declarations(sym));
        let is_export_equals = export_eq_container
            .as_ref()
            .and_then(|p| p.exports.get("export="))
            .is_some_and(|exp| {
                Arc::ptr_eq(exp, sym)
                    || exp
                        .export_symbol
                        .as_ref()
                        .is_some_and(|t| Arc::ptr_eq(t, sym))
                    || self
                        .follow_alias_resolving(exp)
                        .is_some_and(|target| Arc::ptr_eq(&target, sym))
            });
        if is_export_equals {
            return self
                .namespace_qualifier_of(sym)
                .unwrap_or_else(|| sym.name.clone());
        }
        self.namespace_qualifier_of(sym)
            .map(|q| format!("{q}.{}", sym.name))
            .unwrap_or_else(|| sym.name.clone())
    }

    // import X = A.B.C 形式别名：解析实体名链
    fn resolve_import_equals_entity(&mut self, alias: &Arc<Symbol>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_import_equals_entity"); 
        let decl = alias
            .declarations
            .iter()
            .find(|d| matches!(d.data, tsox_frontend::ast::NodeData::ImportEqualsDeclaration(_)))?
            .clone();
        let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(data) = &decl.data else {
            return None;
        };
        // require('./x') 形式：解析模块说明符到模块符号
        if let Some(spec) = self.module_specifier_of_require(&data.module_reference) {
            let file = self.display_enclosing_file.clone()?;
            let dir = match file.file_name.rfind('/') {
                Some(i) => file.file_name[..i].to_string(),
                None => String::new(),
            };
            if let Some(sym) = self.resolve_module_file_symbol_in(&dir, &spec) {
                return Some(sym);
            }
            let path = self.program.resolve_external_module_path(
                &spec,
                &file.file_name,
                tsox_core::core::compiler_options::ModuleKind::None,
            )?;
            let sf = self.program.get_source_file(&path)?;
            return self.program.symbol_map().symbol_of(&sf.node).cloned();
        }
        let mut segments: Vec<String> = Vec::new();
        let mut cur = Arc::clone(&data.module_reference);
        loop {
            match &cur.data {
                tsox_frontend::ast::NodeData::Identifier(id) => {
                    segments.push(id.text.clone());
                    break;
                }
                tsox_frontend::ast::NodeData::QualifiedName(q) => {
                    if let tsox_frontend::ast::NodeData::Identifier(id) = &q.right.data {
                        segments.push(id.text.clone());
                    }
                    cur = Arc::clone(&q.left);
                }
                _ => return None,
            }
        }
        segments.reverse();
        let first = segments.first()?.clone();
        let file = self.display_enclosing_file.clone()?;
        let mut resolved = self.symbol_by_name_in_file_scope(&first, &file)?;
        for seg in segments.iter().skip(1) {
            resolved = resolved.exports.get(seg).cloned().or_else(|| resolved.members.get(seg).cloned())?;
        }
        Some(resolved)
    }

    fn module_specifier_of_require(&self, module_reference: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("module_specifier_of_require"); 
        let tsox_frontend::ast::NodeData::ExternalModuleReference(emr) = &module_reference.data
        else {
            return None;
        };
        let expr = &emr.expression;
        let tsox_frontend::ast::NodeData::StringLiteral(s) = &expr.data else {
            return None;
        };
        Some(s.text.trim_matches(['"', '\'', '`']).to_string())
    }

    fn symbol_by_name_in_file_scope(
        &self,
        name: &str,
        file: &Arc<tsox_frontend::ast::SourceFile>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("symbol_by_name_in_file_scope"); 
        let symbol_map = self.program.symbol_map();
        if let Some(locals) = symbol_map.locals_of(&file.node)
            && let Some(sym) = locals.get(name)
        {
            return Some(Arc::clone(sym));
        }
        if let Some(module_sym) = symbol_map.symbol_of(&file.node) {
            if let Some(sym) = module_sym.exports.get(name) {
                return Some(Arc::clone(sym));
            }
            if let Some(sym) = module_sym.members.get(name) {
                return Some(Arc::clone(sym));
            }
        }
        None
    }

    fn node_within(node: &Arc<Node>, ancestor: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("node_within"); 
        let mut cur = node.parent();
        while let Some(n) = cur {
            if Arc::ptr_eq(&n, ancestor) {
                return true;
            }
            cur = n.parent();
        }
        false
    }

    pub(crate) fn namespace_container_from_declarations(
        &self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("namespace_container_from_declarations"); 
        let symbol_map = self.program.symbol_map();
        symbol.declarations.first().and_then(|decl| {
            let mut cur = decl.parent();
            while let Some(n) = cur {
                match n.kind {
                    SyntaxKind::ModuleDeclaration | SyntaxKind::SourceFile => {
                        return symbol_map.symbol_of(&n).map(Arc::clone);
                    }
                    _ => cur = n.parent(),
                }
            }
            None
        })
    }

    // Go getAccessibleSymbolChain 的 getCandidateListForSymbol 臂
    //（symbolaccessibility.go:617-641）：别名解析目标经自身 exports 暴露
    // 目标符号（export= 符号带提升导出）时，可及链 = [别名, 目标符号]
    fn alias_of_symbol_via_exports_in_file(
        &mut self,
        symbol: &Arc<Symbol>,
        file: &Arc<tsox_frontend::ast::SourceFile>,
    ) -> Option<String> { ::tsox_core::fntrace::enter("alias_of_symbol_via_exports_in_file");
        let symbol_map = self.program.symbol_map();
        let mut tables: Vec<tsox_frontend::ast::SymbolTable> = Vec::new();
        if let Some(locals) = symbol_map.locals_of(&file.node) {
            tables.push(locals.clone());
        }
        if let Some(module_sym) = symbol_map.symbol_of(&file.node) {
            tables.push(module_sym.members.clone());
            tables.push(module_sym.exports.clone());
        }
        for table in tables {
            for (_name, sym) in table.iter() {
                if !sym.flags.contains(SymbolFlags::Alias) {
                    continue;
                }
                let target = self
                    .resolve_import_equals_entity(sym)
                    .or_else(|| self.resolve_import_alias_target_symbol(sym));
                if let Some(target) = target
                    && target
                        .exports
                        .get(&symbol.name)
                        .is_some_and(|s| Arc::ptr_eq(s, symbol))
                {
                    return Some(sym.name.clone());
                }
            }
        }
        None
    }

    fn alias_name_of_namespace_in_file(
        &mut self,
        ns: &Arc<Symbol>,
        file: &Arc<tsox_frontend::ast::SourceFile>,
    ) -> Option<String> { ::tsox_core::fntrace::enter("alias_name_of_namespace_in_file");
        let symbol_map = self.program.symbol_map();
        let mut tables: Vec<tsox_frontend::ast::SymbolTable> = Vec::new();
        if let Some(locals) = symbol_map.locals_of(&file.node) {
            tables.push(locals.clone());
        }
        if let Some(module_sym) = symbol_map.symbol_of(&file.node) {
            tables.push(module_sym.members.clone());
            tables.push(module_sym.exports.clone());
        }
        for table in tables {
            for (_name, sym) in table.iter() {
                if !sym.flags.contains(SymbolFlags::Alias) {
                    continue;
                }
                // Go trySymbolTable（symbolaccessibility.go:592）：UMD 导出别名
                // （首声明 NamespaceExportDeclaration）在外部模块上下文不入
                // 可及链——本助手复刻该别名迭代，同门适用，export as namespace
                // 宿主模块自身的类型才落 import 说明符呈现而非别名名
                if file.external_module_indicator.is_some()
                    && crate::checker::symbolaccessibility_symbol_table_id::is_umd_export_symbol(
                        sym,
                    )
                {
                    continue;
                }
                let target = self
                    .resolve_import_equals_entity(sym)
                    .or_else(|| self.resolve_import_alias_target_symbol(sym));
                if let Some(target) = target {
                    if Arc::ptr_eq(&target, ns) {
                        return Some(sym.name.clone());
                    }
                    // Go getCandidateListForSymbol（symbolaccessibility.go:617-641）：
                    // 别名解析目标的 exports 暴露容器符号自身时（export= 符号经
                    // bindCommonJSTypeExports 提升的命名空间段），链为
                    // [别名, 容器]（getSymbolChain 的 parentChain 拼接），
                    // 限定 = 别名名 + 容器名（a.Bar）
                    if target
                        .exports
                        .get(&ns.name)
                        .is_some_and(|s| Arc::ptr_eq(s, ns))
                    {
                        return Some(format!("{}.{}", sym.name, ns.name));
                    }
                }
            }
        }
        None
    }

    pub(crate) fn symbol_type_to_string(
        &mut self,
        t: &Arc<Type>,
        sym: &Arc<Symbol>,
        flags: TypeFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("symbol_type_to_string"); 
        if sym.flags.intersects(SymbolFlags::ENUM) {
            // Go createAnonymousTypeNodeEx（nodebuilderimpl.go:3033-3047）：枚举
            // 静态侧 shouldEmitTypeOfSymbol 恒真 → symbolToTypeNode(symbol, Value)
            //（nodebuilderimpl.go:680-766），与类静态侧/ValueModule 同径——链 =
            // getAccessibleSymbolChain + needsQualification 容器上爬
            //（getSymbolChain，nodebuilderimpl.go:1127-1174）：遮蔽时链根停在
            // 就地可见的最近容器（X.Y.base.Z 内 var E 遮蔽枚举 E 时呈
            // typeof base.E，非裸名 E 亦非全限定 X.Y.base.E）
            let name = self
                .symbol_chain_display_name(sym, SymbolFlags::VALUE)
                .or_else(|| self.accessible_chain_full_name(sym, SymbolFlags::VALUE))
                .or_else(|| self.alias_chain_qualified_type_name(sym))
                .unwrap_or_else(|| self.symbol_display_name_for_print(sym));
            if matches!(&t.data, TypeData::Object(_) | TypeData::Interface(_)) {
                return format!("typeof {name}");
            }
            return name;
        }

        let obj_data = match &t.data {
            TypeData::Object(o) => Some(o),
            TypeData::Interface(i) => Some(&i.object),
            _ => None,
        };

        if let Some(obj) = obj_data {
            if obj.type_arguments.len() == 1
                && (sym.name == "Array" || sym.name == "ReadonlyArray")
            {
                return self.reference_to_string(t, flags);
            }
            if !obj.type_arguments.is_empty() {
                // Go nodebuilder：Iterable/IterableIterator/AsyncIterable/
                // AsyncIterableIterator 的尾随默认实参在显示中省略
                let mut arg_count = obj.type_arguments.len();
                if matches!(
                    sym.name.as_str(),
                    "Iterable" | "IterableIterator" | "AsyncIterable" | "AsyncIterableIterator"
                ) {
                    let defaults = self.interface_default_type_arguments(&Arc::clone(sym));
                    if defaults.len() == arg_count {
                        while arg_count > 0 {
                            let arg_str =
                                self.type_to_string_ex(&obj.type_arguments[arg_count - 1], flags);
                            let default_str =
                                self.type_to_string_ex(&defaults[arg_count - 1], flags);
                            if arg_str != default_str {
                                break;
                            }
                            arg_count -= 1;
                        }
                    }
                }
                let args: Vec<String> = obj.type_arguments[..arg_count]
                    .iter()
                    .map(|ty| self.type_to_string_ex(ty, flags))
                    .collect();
                // Go getSymbolChain：符号是父模块 export=（隔代自身）时，
                // 链退化为模块限定名（容器解析与限定名同源）
                let container = sym
                    .parent()
                    .clone()
                    .or_else(|| self.namespace_container_from_declarations(sym));
                let is_export_equals = container
                    .as_ref()
                    .and_then(|p| p.exports.get("export="))
                    .is_some_and(|exp| {
                        Arc::ptr_eq(exp, sym)
                            || exp
                                .export_symbol
                                .as_ref()
                                .is_some_and(|t| Arc::ptr_eq(t, sym))
                            || self
                                .follow_alias_resolving(exp)
                                .is_some_and(|target| Arc::ptr_eq(&target, sym))
                    });
                let mut qualified = if is_export_equals {
                    self.namespace_qualifier_of(sym)
                        .unwrap_or_else(|| sym.name.clone())
                } else {
                    self.namespace_qualifier_of(sym)
                        .map(|q| format!("{q}.{}", sym.name))
                        .unwrap_or_else(|| sym.name.clone())
                };
                if let Some(chain_name) = self.alias_chain_qualified_type_name(sym) {
                    qualified = chain_name;
                }
                return format!("{}<{}>", qualified, args.join(", "));
            }
        }

        if sym.flags.contains(SymbolFlags::Class) {
            // Go createAnonymousTypeNodeEx：静态/实例侧按 isClassInstanceSide
            // 判定（nodebuilderimpl.go:3031-3036、349-352），实例侧裸名、静态侧
            // typeof；不以构造签名有无判定（ambient class 无构造签名但仍是
            // 静态侧）。命名空间内类同尾部分支给限定名（d.D）
            if !crate::checker::mig::m2f_3::is_class_instance_side(self, t) {
                // Go shouldEmitTypeOfSymbol→symbolToTypeNode(symbol, Value)
                //（nodebuilderimpl.go:3047）：静态侧 typeof 名同样从渲染上下文
                // 取可及链（isTypeOf 走 getSymbolChain，nodebuilderimpl.go:686），
                // import 别名就地可见时 C → E（import E = A.C 的 typeof E），
                // 同名遮蔽时按容器上爬限定（getSymbolChain）
                let name = self
                    .symbol_chain_display_name(sym, tsox_frontend::ast::SymbolFlags::VALUE)
                    .or_else(|| {
                        self.accessible_chain_full_name(sym, tsox_frontend::ast::SymbolFlags::VALUE)
                    })
                    .or_else(|| self.alias_chain_qualified_type_name(sym))
                    .unwrap_or_else(|| self.symbol_display_name_for_print(sym));
                return format!("typeof {name}");
            }
            // Go symbolToTypeNode（nodebuilderimpl.go:680/1110-1119）：实例侧
            // mask = SymbolFlagsType（createAnonymousTypeNodeEx
            // nodebuilderimpl.go:3030-3047 isClassInstanceSide 臂），引用名从
            // 渲染上下文取可及链（getAccessibleSymbolChain），import
            // 别名就地可见时按别名呈现（class B extends E 中基类 C → E），
            // 单段别名链同样消费；链不可用时回退限定名；同名遮蔽时按容器
            // 上爬限定（getSymbolChain）。mask 取 Type 位：export class d 与
            // 同名局部 interface d 合并的局部符号 flags=Interface|ExportValue
            // 在 needsQualification（symbolaccessibility.go:720-750）按
            // flags&meaning 判遮蔽——Type 位命中 → 导出符号需经模块容器限定
            // （getContainersOfSymbol → 链根模块符号），.types 呈
            // import("./mergedDeclarationExports").d；Value 位不命中 → 裸名 d
            let display_name = self
                .symbol_chain_display_name(sym, tsox_frontend::ast::SymbolFlags::TYPE)
                .or_else(|| {
                    self.accessible_chain_full_name(sym, tsox_frontend::ast::SymbolFlags::TYPE)
                })
                .or_else(|| self.alias_chain_qualified_type_name(sym))
                .unwrap_or_else(|| self.symbol_display_name_for_print(sym));
            return format!("{}{}", display_name, self.declared_type_param_suffix(t, sym));
        }

        if sym.flags.contains(SymbolFlags::ValueModule)
            && !t.object_flags.contains(ObjectFlags::Interface)
        {
            // Go symbolToTypeNode（nodebuilderimpl.go:680-682）链头判定先于
            // 声明形态：getSymbolChain→getAccessibleSymbolChain
            //（symbolaccessibility.go:388/588-612）在显示上下文作用域表
            // 别名迭代中命中解析到该模块的 import 别名（trySymbolTable 直名
            // 对模块符号恒败于 hasNonGlobalAugmentationExternalModuleSymbol
            // 门，别名迭代是唯一入链路径），链 [别名] 经
            // createEntityNameFromSymbolChain 呈 typeof 别名
            //（nodebuilderimpl.go:765）——可及链从 enclosing 声明沿容器
            // 作用域链上爬（collectSymbolTablesInScope 含 ModuleDeclaration
            // locals∪members 合并表），任意深度容器局部作用域的别名（ambient
            // 模块体内 import x = require(...)）同径；链头为模块符号自身时
            // 落说明符/声明名形态
            let enclosing = self
                .type_render_enclosing
                .clone()
                .or_else(|| self.display_enclosing_node.clone());
            let accessible = self.get_accessible_symbol_chain(
                sym,
                enclosing.as_ref(),
                tsox_frontend::ast::SymbolFlags::VALUE,
                false,
            );
            if let Some(head) = accessible.first()
                && !head
                    .declarations
                    .iter()
                    .any(|d| has_non_global_augmentation_external_module_symbol(d))
                && let Some(name) =
                    self.symbol_chain_display_name(sym, tsox_frontend::ast::SymbolFlags::VALUE)
            {
                return format!("typeof {name}");
            }
            if let Some(file) = self.display_context_file()
                && let Some(alias_name) = self.alias_name_of_namespace_in_file(sym, &file)
            {
                return format!("typeof {alias_name}");
            }
            if sym
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::SourceFile)
            {
                // Go getSpecifierForModuleSymbol（nodebuilderimpl.go:1294）→
                // GetModuleSpecifiers（modulespecifiers/specifiers.go:39-56）→
                // computeModuleSpecifiers existing-import 快路径
                //（specifiers.go:389-412）：导入文件已有解析到目标模块的
                // import 时 specifier 即 import 文本（relativePreference Auto、
                // 非 Node16 模式下 mode 检查双 None 不短路），先于全部候选排序
                if let Some(file) = self.display_context_file()
                    && let Some(spec) = self.module_specifier_from_existing_import(sym, &file)
                {
                    return format!("typeof import(\"{spec}\")");
                }
                let recorded = self.module_display_specifiers.get(&sym.id()).cloned();
                if let Some(spec) = recorded {
                    return format!("typeof import(\"{spec}\")");
                }
                if let Some(file) = self.display_context_file()
                    && let Some(spec) = self.module_specifier_relative_to_context(sym, &file)
                {
                    return format!("typeof import(\"{spec}\")");
                }
                return format!("typeof import(\"{}\")", module_specifier_of_name(&sym.name));
            }
            for d in &sym.declarations {
                if let NodeData::ModuleDeclaration(md) = &d.data
                    && md.name.kind == SyntaxKind::StringLiteral
                {
                    let attributes = self.import_attributes_suffix_for_module(sym);
                    return format!(
                        "typeof import(\"{}\"{})",
                        md.name.text().trim_matches(['"', '\'']),
                        attributes
                    );
                }
            }
            // Go symbolToTypeNode：模块名从渲染上下文取可及链（import O = Outer
            // 就地可见时 typeof Outer → typeof O，nodebuilderimpl.go:1127-1135）；
            // 静态侧 mask 为 Value（createAnonymousTypeNodeEx 的
            // isInstanceType，nodebuilderimpl.go:3033-3047）——namespace 内
            // var 同名遮蔽全局命名空间时链根为就地别名（Y）；
            // 同名遮蔽（needsQualification）时按容器上爬限定（getSymbolChain）
            let name = self
                .symbol_chain_display_name(sym, SymbolFlags::VALUE)
                .or_else(|| self.accessible_chain_full_name(sym, SymbolFlags::VALUE))
                .unwrap_or_else(|| self.symbol_written_name(sym));
            return format!("typeof {name}");
        }

        // Go symbolToTypeNode（nodebuilderimpl.go:682-687/747）：链计算先行，
        // 链根声明含非全局增强外部模块（外部模块 SourceFile / 字符串名模块
        // 声明）时整链以 ImportTypeNode 开头（module is root, must use
        // ImportTypeNode）。Type mask 通道（nodebuilderimpl.go:3619
        // ObjectFlagsClassOrInterface 分支 symbolToTypeNode(symbol, Type)，
        // isTypeOf=false）→ `import("<spec>")`——UMD 全局（export as
        // namespace）别名经 mergeSymbol 克隆宿主模块符号后与脚本顶层同名
        // interface 合并，声明型符号的声明集含模块根，Go 由此呈
        // import("./other") 而非裸名/文件路径
        if sym
            .declarations
            .iter()
            .any(|d| has_non_global_augmentation_external_module_symbol(d))
        {
            if let Some(name) = self.symbol_chain_display_name(sym, SymbolFlags::TYPE) {
                return format!("{}{}", name, self.declared_type_param_suffix(t, sym));
            }
        }

        // Go typeToTypeNodeHelper → symbolToTypeNode（nodebuilderimpl.go:680-682）：
        // 具名符号型一律先取可及链（lookupSymbolChain，nodebuilderimpl.go:1099-1119），
        // 链头 import 别名就地可见时按别名呈现（import a = x.c 的接口 c 在
        // a 就地可见处显示 a），链无别名头/为自身时再落父命名空间限定名
        if let Some(chain_name) = self.accessible_chain_type_name(sym) {
            return format!("{}{}", chain_name, self.declared_type_param_suffix(t, sym));
        }

        // Go symbolToTypeNode → getSymbolChain：渲染作用域内同名同 meaning
        // 遮蔽（needsQualification）时按容器上爬限定（namespace M.P 内的
        // v: M.D 经局部 enum D 遮蔽判定产出限定名）。Go getParentOfSymbol =
        // symbol.Parent（binder 对容器成员恒设 = 容器符号），父容器为
        // ValueModule 或 NamespaceModule（仅类型成员的非实例化 namespace）
        // 都按容器限定；本仓部分 namespace 块内成员 parent 链缺失，缺失时
        // 按声明节点树回溯容器符号（GetContainerNode 等价语义），否则限定名
        // 退化为裸名
        let parent_for_qualification = sym
            .parent()
            .clone()
            .or_else(|| self.namespace_container_from_declarations(sym));
        if parent_for_qualification
            .as_ref()
            .is_some_and(|p| p.flags.intersects(SymbolFlags::ValueModule.union(SymbolFlags::NamespaceModule)))
        {
            let name = self
                .symbol_chain_display_name(sym, SymbolFlags::TYPE)
                .or_else(|| {
                    self.namespace_qualifier_of(sym)
                        .map(|q| format!("{q}.{}", sym.name))
                })
                .unwrap_or_else(|| sym.name.clone());
            return name + &self.declared_type_param_suffix(t, sym);
        }

        format!(
            "{}{}",
            sym.name.clone(),
            self.declared_type_param_suffix(t, sym)
        )
    }

    // Go getSymbolChain（nodebuilderimpl.go:1127-1192）：可及链为空或链头被
    // 同名同 meaning 符号遮蔽（needsQualification，symbolaccessibility.go:720）
    // 时按容器上爬（getContainersOfSymbol）拼接父链；export= 自指容器退化为
    // 父链，无链段时经容器内别名（getAliasForSymbolInContainer）兜底
    pub(crate) fn get_display_symbol_chain(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        end_of_chain: bool,
        yield_module_symbol: bool,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_display_symbol_chain");
        let enclosing = self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone());
        let mut accessible_symbol_chain =
            self.get_accessible_symbol_chain(symbol, enclosing.as_ref(), meaning, false);
        let qualifier_meaning = if accessible_symbol_chain.len() > 1 {
            get_qualified_left_meaning(meaning)
        } else {
            meaning
        };
        if accessible_symbol_chain.is_empty()
            || self.needs_qualification(
                &accessible_symbol_chain[0],
                enclosing.as_ref(),
                qualifier_meaning,
            )
            || self.display_export_shadowing(
                &accessible_symbol_chain[0],
                enclosing.as_ref(),
                qualifier_meaning,
            )
        {
            let root = accessible_symbol_chain
                .first()
                .cloned()
                .unwrap_or_else(|| Arc::clone(symbol));
            let mut parents =
                self.get_containers_of_symbol(&root, enclosing.as_ref(), meaning, true);
            // 脚本文件的路径名文件符号不得入链（tsc 脚本无文件符号），否则环境模块链多出 import("../..x") 段
            parents.retain(|p| !Checker::is_script_file_module_symbol(p));
            // Go 参考（baseExpressionTypeParameters 行22 `>base : <T>() => typeof
            // Base` 对照行21 `>base<T>() : base.Base`）：函数局部符号的宿主函数
            // 限定段仅呈于 Type 意（实例型引用），Value 意（typeof 显示）保持
            // 裸名。宿主函数容器只出自 get_containers_of_symbol 的认领臂，且
            // 仅 parent-of-symbol 为空的符号可能走该臂（有 parent 者走父臂，
            // 如函数+枚举合并符号的成员 y4.One 不受影响）
            if meaning == SymbolFlags::VALUE && self.get_parent_of_symbol(&root).is_none() {
                parents.retain(|p| {
                    !p.declarations
                        .iter()
                        .any(|d| tsox_frontend::ast::is_function_like_kind(d.kind))
                });
            }
            // Go getSymbolChain：模块容器取 getSpecifierForModuleSymbol 显示
            // 说明符参序（nodebuilderimpl.go:1143-1149），sortByBestName
            //（1194-1212）决胜——非相对说明符优先（宿主文件现有 import 的
            // @scope/pkg 胜相对路径），同相对性按路径段数，其余按符号序
            //（稳定排序）
            let mut parent_specifiers: Vec<SortedSymbolNamePair> = parents
                .into_iter()
                .map(|p| {
                    let name = if p
                        .declarations
                        .iter()
                        .any(|d| has_non_global_augmentation_external_module_symbol(d))
                    {
                        self.get_specifier_for_module_symbol(&p, enclosing.as_ref())
                    } else {
                        String::new()
                    };
                    SortedSymbolNamePair { sym: p, name }
                })
                .collect();
            parent_specifiers.sort_by(|a, b| self.sort_by_best_name(a, b));
            for pair in &parent_specifiers {
                let parent = &pair.sym;
                let parent_chain = self.get_display_symbol_chain(
                    parent,
                    get_qualified_left_meaning(meaning),
                    false,
                    yield_module_symbol,
                );
                if parent_chain.is_empty() {
                    continue;
                }
                if let Some(export_equals) = parent.exports.get("export=")
                    && self
                        .get_symbol_if_same_reference(export_equals, symbol)
                        .is_some()
                {
                    accessible_symbol_chain = parent_chain;
                    break;
                }
                let mut combined = parent_chain;
                if accessible_symbol_chain.is_empty() {
                    let fallback = self
                        .get_alias_for_symbol_in_container(parent, symbol)
                        .unwrap_or_else(|| Arc::clone(symbol));
                    combined.push(fallback);
                } else {
                    combined.extend(accessible_symbol_chain);
                }
                accessible_symbol_chain = combined;
                break;
            }
        }
        // Go trySymbolTable（symbolaccessibility.go:614-617）：可及链为空且无容器
        // 时，globals 表回退经 getCandidateListForSymbol（649-675）在 globalThis
        // 的 exports（=globals，checker.go:974）中以 ignoreQualification 直名命中，
        // 链 = [globalThis, 符号]（needsQualification 为真，脚本内同名遮蔽）；
        // 无需限定（就地同名可见）时 Go 链为 [符号]（isAccessible 直名命中）
        if accessible_symbol_chain.is_empty() && end_of_chain {
            let needs = self.needs_qualification(symbol, enclosing.as_ref(), meaning);
            let in_globals = self.globals.get(&symbol.name).is_some_and(|res| {
                self.get_merged_symbol(res).id() == self.get_merged_symbol(symbol).id()
            });
            if needs
                && in_globals
                && let Some(global_this) = self.global_this_symbol.clone()
            {
                return vec![global_this, Arc::clone(symbol)];
            }
        }
        if !accessible_symbol_chain.is_empty() {
            return accessible_symbol_chain;
        }
        if end_of_chain
            || !symbol
                .flags
                .intersects(SymbolFlags::TypeLiteral | SymbolFlags::ObjectLiteral)
        {
            if !end_of_chain
                && !yield_module_symbol
                && symbol
                    .declarations
                    .iter()
                    .any(|d| has_non_global_augmentation_external_module_symbol(d))
            {
                return Vec::new();
            }
            return vec![Arc::clone(symbol)];
        }
        Vec::new()
    }

    // Go getSpecifierForModuleSymbol（nodebuilderimpl.go:1294）的 .types 显示
    // 通道 Checker 侧等价：parentSpecifiers 参序说明符（getSymbolChain
    // nodebuilderimpl.go:1145）与链根显示同源——现有 import 快路径 → 上下文
    // 相对说明符 → 符号名说明符
    pub(crate) fn get_specifier_for_module_symbol(
        &mut self,
        sym: &Arc<Symbol>,
        enclosing: Option<&Arc<Node>>,
    ) -> String { ::tsox_core::fntrace::enter("get_specifier_for_module_symbol");
        let context_file = enclosing.and_then(|n| self.get_source_file_of_node(n));
        if let Some(f) = &context_file {
            if let Some(spec) = self.module_specifier_from_existing_import(sym, f) {
                return spec;
            }
            if let Some(spec) = self.module_specifier_node_modules_name(sym) {
                return spec;
            }
            if let Some(spec) = self.module_specifier_relative_to_context(sym, f) {
                return spec;
            }
        }
        module_specifier_of_name(&sym.name)
    }

    // Go sortByBestName（nodebuilderimpl.go:1194-1212）：双非空说明符先比
    // 相对性（非相对优先），同相对性比路径段数；任一为空落符号序
    fn sort_by_best_name(
        &self,
        a: &SortedSymbolNamePair,
        b: &SortedSymbolNamePair,
    ) -> std::cmp::Ordering { ::tsox_core::fntrace::enter("sort_by_best_name");
        if !a.name.is_empty() && !b.name.is_empty() {
            let b_relative = tsox_core::tspath::path_is_relative(&b.name);
            if tsox_core::tspath::path_is_relative(&a.name) == b_relative {
                return count_path_components(&a.name).cmp(&count_path_components(&b.name));
            }
            return if b_relative {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        self.compare_symbols_for_display(&a.sym, &b.sym)
    }

    pub(crate) fn symbol_chain_display_name(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Option<String> {
        if symbol.name.starts_with('\u{FE}') {
            return None;
        }
        // Go lookupSymbolChainWorker（nodebuilderimpl.go:1110-1117）：无 enclosing
        // 声明（且无 UseFullyQualifiedType）时链 = [符号] 裸名，不走可及链/容器
        // 上爬——错误消息等无 enclosing 的 typeToString 不产生 import(...) 限定
        if self.type_render_enclosing.is_none() && self.display_enclosing_node.is_none() {
            return Some(symbol.name.clone());
        }
        // Go symbolToTypeNode（nodebuilderimpl.go:682）：yieldModuleSymbol =
        // ctx.flags 无 UseAliasDefinedOutsideCurrentScope 时为 true。false 时
        // 链中模块根段被 elide（getSymbolChain 尾段
        // 「If a parent symbol is an external module, don't write it」），链坍缩
        // 为裸符号名——re-export 接口本应呈 import("./common").Row2 却呈 Row2。
        // Checker.TypeToString（printer.go:56）旗标含
        // UseAliasDefinedOutsideCurrentScope，yield 恒 false（错误消息中
        // 同文件模块局部型呈裸名，如 Float 而非 import("f").Float）；
        // display_yield_module_symbol 按显示通道承载该值
        let chain = self.get_display_symbol_chain(symbol, meaning, true, self.display_yield_module_symbol);
        if chain.is_empty() {
            return None;
        }
        // Go symbolToTypeNode（nodebuilderimpl.go:689-757）：链根为外部模块
        // 符号时必须以 ImportTypeNode 开头（「module is root, must use
        // ImportTypeNode」），段名为 getSpecifierForModuleSymbol 的说明符；
        // 非 IDENTITY 段仍按符号名（import("./m").M 的 M）
        let context_file = self.display_context_file();
        let mut parts: Vec<String> = Vec::with_capacity(chain.len());
        for (i, s) in chain.iter().enumerate() {
            // Go symbolToTypeNode（nodebuilderimpl.go:689）：链根判定
            // Some(declarations, hasNonGlobalAugmentationExternalModuleSymbol)
            //——字符串字面量名 ambient module 与外部模块 SourceFile 同为
            // ImportTypeNode 根，不限于 SourceFile 声明
            if i == 0
                && s.declarations
                    .iter()
                    .any(|d| has_non_global_augmentation_external_module_symbol(d))
            {
                // Go getSpecifierForModuleSymbol（nodebuilderimpl.go:1294-1330）：
                // 先取 SourceFile 声明走 GetModuleSpecifiers（existing-import/
                // 相对路径）；无 SourceFile 声明时按 IsModuleWithStringLiteralName
                // 声明取模块名文本为说明符（本仓符号名带源文本引号，去引号）
                let spec = if s.declarations.iter().any(|d| d.kind == SyntaxKind::SourceFile) {
                    context_file
                        .as_ref()
                        .and_then(|f| self.module_specifier_from_existing_import(s, f))
                        .or_else(|| self.module_specifier_node_modules_name(s))
                        .or_else(|| {
                            context_file
                                .as_ref()
                                .and_then(|f| self.module_specifier_relative_to_context(s, f))
                        })
                        .unwrap_or_else(|| module_specifier_of_name(&s.name))
                } else {
                    s.declarations
                        .iter()
                        .find(|d| tsox_frontend::ast::is_module_with_string_literal_name(d))
                        .and_then(|d| d.name())
                        .map(|n| n.text().trim_matches(['"', '\'']).to_string())
                        .unwrap_or_else(|| module_specifier_of_name(&s.name))
                };
                parts.push(format!("import(\"{spec}\")"));
                continue;
            }
            parts.push(if i == 0 {
                // Go createAccessFromSymbolChain（nodebuilderimpl.go:784-800）：
                // 链首段名走 getNameOfSymbolAsWritten——default 命名的合并
                // default 导出符号按首个带名声明渲染
                //（GetNonAssignedNameOfDeclaration 的 ExportAssignment 分支取
                // 表达式标识符，ast/utilities.go:1643-1648）
                Self::written_symbol_name(s)
            } else if s.name.starts_with('\u{FE}') {
                // Go createAccessFromSymbolChain 尾段（nodebuilderimpl.go:839）：
                // 非首段父 exports 未命中（匿名声明不入表）回落
                // getNameOfSymbolAsWritten——内部名段呈 (Anonymous class) 等
                // 书写名而非原始 \u{FE} 名
                self.internal_symbol_written_name(s).unwrap_or_else(|| s.name.clone())
            } else {
                s.name.clone()
            });
        }
        Some(parts.join("."))
    }

    pub(crate) fn accessible_chain_type_name(&mut self, symbol: &Arc<Symbol>) -> Option<String> {
        if symbol.flags.intersects(SymbolFlags::Alias) {
            return None;
        }
        let enclosing = self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone());
        let chain = self.get_accessible_symbol_chain(
            symbol,
            enclosing.as_ref(),
            SymbolFlags::TYPE,
            false,
        );
        // 仅当链头是异于目标符号的别名（就地经 import 别名可见）时接管显示；
        // 直名可见（链=[自身]/空）与限定路径维持既有输出
        if chain.len() != 1 || !chain[0].flags.intersects(SymbolFlags::Alias) {
            return None;
        }
        if Arc::ptr_eq(&chain[0], symbol) {
            return None;
        }
        if self.needs_qualification(&chain[0], enclosing.as_ref(), SymbolFlags::TYPE) {
            return None;
        }
        Some(chain[0].name.clone())
    }

    // Go getSymbolChain（nodebuilderimpl.go:1127-1135）：符号显示名从渲染上下文
    // 取可及链（getAccessibleSymbolChain），链头经 import 别名可见时按链渲染
    //（Outer → O、Outer.A → O.A）；链头须无需再限定（needsQualification）。
    // enclosing 以 .types 基线传入的 node.Parent（type_render_enclosing）优先
    pub(crate) fn accessible_chain_full_name(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Option<String> {
        if symbol.flags.intersects(SymbolFlags::Alias) {
            return None;
        }
        let enclosing = self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone());
        let chain = self.get_accessible_symbol_chain(
            symbol,
            enclosing.as_ref(),
            meaning,
            false,
        );
        if chain.is_empty() || (chain.len() == 1 && Arc::ptr_eq(&chain[0], symbol)) {
            return None;
        }
        if !chain[0].flags.intersects(SymbolFlags::Alias) {
            return None;
        }
        if self.needs_qualification(&chain[0], enclosing.as_ref(), meaning) {
            return None;
        }
        Some(
            chain
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
                .join("."),
        )
    }

    fn declared_type_param_suffix(&self, t: &Arc<Type>, sym: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("declared_type_param_suffix"); 
        let empty_args = match &t.data {
            TypeData::Object(o) => o.type_arguments.is_empty(),
            TypeData::Interface(i) => i.object.type_arguments.is_empty(),
            _ => true,
        };
        if !empty_args {
            return String::new();
        }
        let mut names: Vec<String> = Vec::new();
        for decl in &sym.declarations {
            let tps = match &decl.data {
                NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::ClassExpression(d) => d.type_parameters.as_ref(),
                NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::TypeAliasDeclaration(d) => d.type_parameters.as_ref(),
                _ => continue,
            };
            let Some(tps) = tps else { continue };
            for tp in tps.nodes.iter() {
                if let NodeData::TypeParameterDeclaration(tpd) = &tp.data {
                    let name = tpd.name.text().to_string();
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
            }
        }
        if names.is_empty() {
            String::new()
        } else {
            format!("<{}>", names.join(", "))
        }
    }

    pub(crate) fn internal_symbol_written_name(&self, sym: &Arc<Symbol>) -> Option<String> { ::tsox_core::fntrace::enter("internal_symbol_written_name"); 
        if !sym.name.starts_with('\u{FE}') {
            return None;
        }
        let decl = sym.declarations.first()?;
        match decl.kind {
            SyntaxKind::ClassExpression
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction => {}
            _ => return None,
        }
        for d in &sym.declarations {
            if let Some(name) = tsox_frontend::ast::get_name_of_declaration(d) {
                return Some(
                    tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(&name)),
                );
            }
        }
        let parent = decl.parent();
        if parent.as_ref().is_some_and(|p| p.kind == SyntaxKind::VariableDeclaration)
            && let Some(NodeData::VariableDeclaration(vd)) = parent.as_ref().map(|p| &p.data)
        {
            return Some(
                tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(&vd.name)),
            );
        }
        match decl.kind {
            SyntaxKind::ClassExpression => Some("(Anonymous class)".to_string()),
            _ => Some("(Anonymous function)".to_string()),
        }
    }

    pub(crate) fn symbol_written_name(&self, sym: &Arc<Symbol>) -> String {
        // Go getNameOfSymbolAsWritten（nodebuilderimpl.go:1023-1060）终末回退：
        // 常规名直用符号名；内部名（\u{FE}global）先取声明名——declare global
        // 的符号名是内部名，声明名字标识符为 global（Go 呈 typeof global），
        // 无声明名时落 EscapeInternalSymbolName
        if !sym.name.starts_with('\u{FE}') {
            return sym.name.clone();
        }
        sym.declarations
            .iter()
            .filter_map(tsox_frontend::ast::get_name_of_declaration)
            .next()
            .map(|n| tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(&n)))
            .unwrap_or_else(|| {
                crate::checker::mig::m2f_3::r24k13_defs::escape_internal_symbol_name(&sym.name)
            })
    }

    pub(crate) fn symbol_display_name_for_print(&mut self, sym: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("symbol_display_name_for_print");
        // Go getNameOfSymbolAsWritten（nodebuilderimpl.go:1008）：default 命名
        // 符号（default 导出声明的合并符号）在链首直名场景按首个带名声明渲染
        if sym.name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT {
            let written = Self::written_symbol_name(sym);
            if written != sym.name {
                return written;
            }
        }
        self.internal_symbol_written_name(sym)
            .unwrap_or_else(|| self.namespace_qualified_name(sym, tsox_frontend::ast::SymbolFlags::VALUE))
    }

    pub(crate) fn namespace_qualified_name(&mut self, sym: &Arc<Symbol>, _meaning: tsox_frontend::ast::SymbolFlags) -> String { ::tsox_core::fntrace::enter("namespace_qualified_name");         if sym
            .parent()
            .as_ref()
            .is_some_and(|p| p.flags.contains(SymbolFlags::ValueModule))
        {
            if let Some(q) = self.namespace_qualifier_of(sym) {
                return format!("{q}.{}", sym.name);
            }
        }
        sym.name.clone()
    }

    /// Go 括号判据的对偶门：型经别名门按 TypeReference 呈现时为
    /// NonArray 档（ast/precedence.go:661,707），emitTypeNode 的
    /// GetTypeNodePrecedence(node) < precedence（printer.go:2463）在任何成分位
    /// （数组元素 Postfix printer.go:2148、并/交成分 TypeOperator
    /// printer.go:2197/2209）均不成立，恒不加括号。门条件与渲染别名分支
    /// 同源（nodebuilderimpl.go:3552 ↔ nodebuilder_checker_7.rs:299-303），
    /// 两侧同门保证括号决策与实际渲染形态一致
    pub(crate) fn renders_as_alias_reference(
        &mut self,
        t: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> bool { ::tsox_core::fntrace::enter("renders_as_alias_reference");
        if flags.contains(TypeFormatFlags::IN_TYPE_ALIAS) {
            return false;
        }
        let Some(sym) = t.alias.as_ref().and_then(|a| a.symbol.clone()) else {
            return false;
        };
        flags.contains(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE) || {
            let enclosing = self
                .type_render_enclosing
                .clone()
                .or_else(|| self.display_enclosing_node.clone());
            self.is_type_symbol_accessible(&sym, enclosing.as_ref())
        }
    }

    pub(crate) fn needs_parens_in_union(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("needs_parens_in_union");
        // Go typeToTypeNodeHelper（nodebuilderimpl.go:3447-3450）：TypeFlagsBoolean
        // 且无别名的成分先按 boolean 关键字节点呈现，关键字节点优先级高于
        // TypePrecedenceTypeOperator 档（ast/precedence.go:695），联合内不加括号
        if t.flags.contains(TypeFlags::Boolean) && t.alias.is_none() {
            return false;
        }
        // Go emitUnionTypeConstituent（printer.go:2195-2198）联合成分按
        // TypePrecedenceTypeOperator 发射，GetTypeNodePrecedence
        //（ast/precedence.go:695）交集型 = TypePrecedenceIntersection、
        // 联合型 = TypePrecedenceUnion 均低于该级，交/联合成分在联合内
        // 加括号（如 Common | (Common & A)、X[] | (string | false)）。
        // 加括号前提是成分按并/交表达式渲染：经符号名/别名名呈现的成分
        //（TypeReference 档 nodebuilderimpl.go:682）不高括号；条件型见下方
        // 独立分支（nonNullableReduction 语料佐证）
        if (t.is_union() || t.is_intersection())
            && !self.symbol_type_printed_by_name(t)
            && !t
                .alias
                .as_ref()
                .is_some_and(|a| a.symbol.is_some())
        {
            return true;
        }
        // 条件型档位 TypePrecedenceConditional（ast/precedence.go:664-665，最低）
        // 低于并集成分的 TypePrecedenceTypeOperator（printer.go:2195-2198），
        // GetTypeNodePrecedence < precedence（printer.go:2463）成立 → 加括号，
        // 如 T | (string extends T ? null | undefined : never)；经别名门按
        // TypeReference 呈现（NonArray 档）的别名条件型不加
        if matches!(&t.data, TypeData::Conditional(_))
            && !t
                .alias
                .as_ref()
                .is_some_and(|a| a.symbol.is_some())
        {
            return true;
        }
        if let Some(structured) = t.as_structured()
            && !self.symbol_type_printed_by_name(t)
            && structured.signatures.len() == 1
            && structured.properties.is_empty()
            && structured.index_infos.is_empty()
        {
            return true;
        }

        false
    }

    pub(crate) fn needs_parens_as_array_element(
        &mut self,
        t: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> bool { ::tsox_core::fntrace::enter("needs_parens_as_array_element");
        // Go typeToTypeNodeHelper（nodebuilderimpl.go:3447-3450）：TypeFlagsBoolean
        // 且无别名的元素先按 boolean 关键字节点呈现，关键字节点优先级高于
        // TypePrecedencePostfix 档（printer.go:2148/2463），数组元素位不加括号
        if t.flags.contains(TypeFlags::Boolean) && t.alias.is_none() {
            return false;
        }
        // 别名门先于一切节点形态判定（Go nodebuilderimpl.go:3552 别名分支
        // 先于并集/条件/typeof 分派）：按别名引用呈现的型不加括号
        if self.renders_as_alias_reference(t, flags) {
            return false;
        }
        if t.is_union() || t.is_intersection() {
            // Go typeToTypeNodeHelper（nodebuilderimpl.go:3447-3450）：TypeFlagsBoolean
            // 且无别名的类型先于联合分派渲染为 boolean 关键字节点，关键字优先级
            // 高于 TypePrecedencePostfix，数组元素不加括号（checker.go:1013/
            // 27277-27279 booleanType 为 false|true 置旗联合）
            if t.flags.contains(TypeFlags::Boolean) && t.alias.is_none() {
                return false;
            }
            // Go 数组元素以 TypePrecedencePostfix 发射（printer.go:2148），
            // 仅节点优先级严格低于该级才加括号（printer.go:2463）。枚举联合
            // 走 type_to_string_ex 的枚举联合分支（nodebuilder_checker_7.rs:268
            // ↔ Go nodebuilderimpl.go:3473-3474 EnumLike symbolToTypeNode，
            // .types 通道 maxExpansionDepth=-1 恒不展开）；别名呈现型已由
            // 顶部 renders_as_alias_reference 门承接
            if t.is_union()
                && let Some(sym) = &t.symbol
                && sym.flags.intersects(SymbolFlags::ENUM)
            {
                return false;
            }
            // Go 括号判定在节点层：unionToTypeNode 先经 formatUnionTypes
            //（printer.go:274-318）把枚举成员联合塌缩回基枚举单引用
            //（首成员 EnumLike 且尾部与基联合对齐），塌缩后为 TypeReference
            // 节点优先级高于 Postfix 档不加括号；成员联合（如
            // TestType[Extract<keyof TestType, string>]）无枚举符号，
            // 按塌缩后条目数判定
            if t.is_union() {
                let members = t.types().unwrap_or(&[]);
                if members
                    .first()
                    .is_some_and(|f| f.flags.intersects(TypeFlags::ENUM_LIKE))
                {
                    let collapsed = self.format_union_types(members, false);
                    if collapsed.len() == 1 {
                        return false;
                    }
                }
            }
            return true;
        }
        if matches!(&t.data, TypeData::Conditional(_) | TypeData::Index(_)) {
            return true;
        }
        // typeof X（类构造类型/命名空间值）作为数组元素需括号：(typeof Foo)[]
        if let Some(sym) = &t.symbol
            && (sym.flags.contains(SymbolFlags::ValueModule)
                || (sym.flags.contains(SymbolFlags::Class)
                    && t.as_structured()
                        .is_some_and(|s| !s.construct_signatures().is_empty())))
        {
            return true;
        }
        self.needs_parens_in_union(t)
    }

    pub(crate) fn maybe_parenthesize_array_element_ex(
        &mut self,
        elem: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("maybe_parenthesize_array_element_ex"); 
        let s = self.type_to_string_ex(elem, flags);
        if self.needs_parens_as_array_element(elem, flags) {
            format!("({})", s)
        } else {
            s
        }
    }

    pub fn type_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> { ::tsox_core::fntrace::enter("type_to_type_node"); 
        self.type_to_type_node_worker(t)
    }

    // 方法形态：声明为 MethodSignature/MethodDeclaration 且类型为裸函数时，
    // 按方法签名渲染（Go addPropertyToElement 的 FunctionTypeNode 分支）
    pub(crate) fn method_form_suffix(
        &mut self,
        prop: &Arc<Symbol>,
        prop_type: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("method_form_suffix"); 
        if !prop.declarations.iter().any(|d| {
            matches!(
                d.kind,
                SyntaxKind::MethodSignature | SyntaxKind::MethodDeclaration
            )
        }) {
            return None;
        }
        // Go addPropertyToTypeElement（nodebuilderimpl.go:2757-2761）：方法形态
        // 取签名前先 filterType 滤并集中的 undefined 位成员——可选方法的符号型
        // 为（签名型 | undefined），滤后取 Call 签名渲染 foo?(): ret；非并集
        // 原样（filterType 对非并集恒返回 t 本身）
        let prop_type = if prop_type.flags.intersects(TypeFlags::Union) {
            let types = prop_type.types().unwrap_or(&[]);
            if types.iter().any(|m| m.flags.intersects(TypeFlags::Undefined)) {
                let kept: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|m| !m.flags.intersects(TypeFlags::Undefined))
                    .cloned()
                    .collect();
                match kept.len() {
                    0 => self.never_type(),
                    1 => kept.into_iter().next().unwrap(),
                    _ => self.get_union_type(kept),
                }
            } else {
                Arc::clone(prop_type)
            }
        } else {
            Arc::clone(prop_type)
        };
        let st = prop_type.as_structured()?;
        if st.signatures.len() != 1 || !st.properties.is_empty() || !st.index_infos.is_empty() {
            return None;
        }
        let sig = &st.signatures[0];
        let params: Vec<String> = sig
            .parameters
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let name = self.signature_parameter_display_name(param);
                let param_type = self
                    .signature_instantiated_param_type(sig, i)
                    .unwrap_or_else(|| self.get_type_of_symbol(param));
                let param_type = self.strip_param_optionality_undefined(param, &param_type);
                let type_str = self.type_to_string_ex(&param_type, flags);
                let prefix = if i + 1 == sig.parameters.len() && sig.has_rest_parameter() {
                    "..."
                } else {
                    ""
                };
                if self.param_declared_optional(param) {
                    format!("{prefix}{name}?: {type_str}")
                } else {
                    format!("{prefix}{name}: {type_str}")
                }
            })
            .collect();
        let ret_type = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        // Go addPropertyToTypeElement 方法形经 signatureToSignatureDeclarationHelper
        // → serializeReturnTypeForSignature（nodebuilderimpl.go:2196）tryReuse=true：
        // 方法声明返回注解伪等价时复用书写形（test2(): m2.connectModule 呈源注解）
        let ret_str = self
            .annotated_return_type_text(sig, &ret_type, flags)
            .unwrap_or_else(|| self.type_to_string_ex(&ret_type, flags));
        let tp = self.signature_type_param_prefix(sig);
        Some(format!("{tp}({}): {}", params.join(", "), ret_str))
    }

    /// Go addPropertyToElementList accessor 臂的元素渲染
    ///（nodebuilderimpl.go:2711-2745）：getter `get name(): ret`、setter
    /// `set name(param)`（Go setter 无返回型）。形态取声明注解
    ///（signatureToSignatureDeclarationHelper 的注解复用通道），setter 参数取
    /// 值参数（getSetAccessorValueParameter）
    fn accessor_member_form(
        &mut self,
        decl: &Arc<Node>,
        name: &str,
        flags: TypeFormatFlags,
    ) -> Option<String> {
        match decl.kind {
            SyntaxKind::GetAccessor => {
                let tn = decl.type_node()?;
                let ret_type = self.get_type_from_type_node(tn);
                let ret_str = self.type_to_string_ex(&ret_type, flags);
                Some(format!("get {name}(): {ret_str}"))
            }
            SyntaxKind::SetAccessor => {
                let param =
                    crate::checker::utilities_is_private_within_ambient::get_set_accessor_value_parameter(decl)?;
                let pname = param.name().map(|n| n.text().to_string())?;
                let tn = param.type_node()?;
                let param_type = self.get_type_from_type_node(tn);
                let tstr = self.type_to_string_ex(&param_type, flags);
                Some(format!("set {name}({pname}: {tstr})"))
            }
            _ => None,
        }
    }

    /// Go utilities.go compareSymbols：声明位置优先（跨文件按程序内文件序），
    /// 无声明回退名字
    fn compare_symbols_for_display(&self, a: &Arc<Symbol>, b: &Arc<Symbol>) -> std::cmp::Ordering { ::tsox_core::fntrace::enter("compare_symbols_for_display");
        match (a.declarations.first(), b.declarations.first()) {
            (Some(da), Some(db)) => self
                .compare_nodes_for_display(da, db)
                .then_with(|| a.name.cmp(&b.name)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        }
    }

    fn compare_nodes_for_display(&self, a: &Arc<Node>, b: &Arc<Node>) -> std::cmp::Ordering { ::tsox_core::fntrace::enter("compare_nodes_for_display"); 
        let fa = self.get_source_file_of_node(a);
        let fb = self.get_source_file_of_node(b);
        if let (Some(fa), Some(fb)) = (&fa, &fb) {
            if !Arc::ptr_eq(fa, fb) {
                let ia = self.file_index_map.get(&fa.id()).copied().unwrap_or(usize::MAX);
                let ib = self.file_index_map.get(&fb.id()).copied().unwrap_or(usize::MAX);
                return ia.cmp(&ib);
            }
        }
        a.loc.pos().cmp(&b.loc.pos())
    }
    pub(crate) fn container_chain_qualified_name(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Option<String> {
        ::tsox_core::fntrace::enter("container_chain_qualified_name");
        let enclosing = self
            .display_enclosing_node
            .clone()
            .or_else(|| self.type_render_enclosing.clone())
            .or_else(|| self.access_location.clone());
        if !self.needs_qualification(symbol, enclosing.as_ref(), meaning) {
            return None;
        }
        let mut parts: Vec<String> = Vec::new();
        let merged_id = self.get_merged_symbol(symbol).id();
        // Go 合并符号单实体单父（容器即所在命名空间，getContainersOfSymbol →
        // getParentOfSymbol，symbolaccessibility.go:293-296）。本仓合并体父链
        // 可能串入同实体 namespace 侧符号（如接口 D 的父为 namespace D），容器
        // 链从声明词法容器起算（getContainersOfSymbol 的 declarations 兜底，
        // symbolaccessibility.go:301-334），parent 链仅兜底
        let mut cur = self
            .namespace_container_from_declarations(symbol)
            .or_else(|| symbol.parent().clone());
        let mut visited: HashSet<u64> = HashSet::new();
        while let Some(ns) = cur {
            if !visited.insert(ns.id()) {
                break;
            }
            if !ns.flags.intersects(SymbolFlags::MODULE) {
                break;
            }
            // 链中段串入同实体符号（合并孪生）非容器段，跳过
            if self.get_merged_symbol(&ns).id() == merged_id {
                cur = ns.parent();
                continue;
            }
            if ns
                .declarations
                .iter()
                .any(|d| tsox_frontend::ast::is_global_scope_augmentation(d))
            {
                break;
            }
            let is_file_module = ns
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::SourceFile);
            if is_file_module
                || ns
                    .name
                    .chars()
                    .next()
                    .is_some_and(|c| c == '"' || c == '\'' || c == '`')
            {
                break;
            }
            parts.insert(0, ns.name.clone());
            cur = ns.parent();
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("."))
        }
    }

    /// Go createImportAttributesForModuleSpecifier（nodebuilderimpl.go:1408-1441）：
    /// 字符串名模块声明的 with 属性型非空时，import 型节点附
    /// `, { with: { name: "value", ... } }` 尾段——属性按名排序，仅取字符
    /// 串字面量值型成员，名按标识符合法性取裸名或引号名。渲染上下文说明符
    /// 的属性复核（getSpecifierForModuleSpecifier 的 originalImportAttributes
    /// 臂，nodebuilderimpl.go:1318-1321）在 .types 通道与模块自身属性型同
    /// 内容，取后者
    fn import_attributes_suffix_for_module(&mut self, sym: &Arc<Symbol>) -> String {
        let attributes_type = self.get_type_of_module_import_attributes(sym);
        if self.is_empty_object_type(&attributes_type) {
            return String::new();
        }
        let mut properties = self.get_properties_of_type(&attributes_type);
        properties.sort_by(|a, b| a.name.cmp(&b.name));
        let mut attributes: Vec<String> = Vec::new();
        for property in properties {
            let property_type = self.get_type_of_symbol(&property);
            if !property_type.flags.intersects(TypeFlags::StringLiteral) {
                continue;
            }
            let value = crate::checker::mig::wc2::get_string_literal_value(&property_type);
            let name = if crate::checker::checker_get_excluded_symbol_flags::
                is_valid_identifier_text(&property.name)
            {
                property.name.clone()
            } else {
                format!("\"{}\"", property.name)
            };
            attributes.push(format!("{name}: \"{value}\""));
        }
        if attributes.is_empty() {
            return String::new();
        }
        format!(", {{ with: {{ {} }} }}", attributes.join(", "))
    }

}

// compareTypesIdentical 的显示侧恒等替代（双 union 同序逐成员 ptr，非 union ptr）
fn same_type_identity(a: &Arc<Type>, b: &Arc<Type>) -> bool {
    if Arc::ptr_eq(a, b) {
        return true;
    }
    if !a.is_union() || !b.is_union() {
        return false;
    }
    let (Some(au), Some(bu)) = (a.types(), b.types()) else {
        return false;
    };
    au.len() == bu.len() && au.iter().zip(bu.iter()).all(|(x, y)| Arc::ptr_eq(x, y))
}

struct DisplaySpecifierHost<'a> {
    program: &'a dyn Program,
}

impl ModuleSpecifierGenerationHost for DisplaySpecifierHost<'_> {
    fn get_current_directory(&self) -> String { ::tsox_core::fntrace::enter("get_current_directory");
        self.program.current_directory().to_string()
    }

    fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names");
        self.program.use_case_sensitive_file_names()
    }

    fn common_source_directory(&self) -> String { ::tsox_core::fntrace::enter("common_source_directory");
        self.program.common_source_directory()
    }

    fn file_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("file_exists");
        self.program.file_exists(path)
    }
}
