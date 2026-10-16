#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn resolve_type_reference(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_type_reference"); 
        let (type_name, type_arguments) = match &node.data {
            NodeData::TypeReferenceNode(data) => (&data.type_name, data.type_arguments.clone()),
            NodeData::ExpressionWithTypeArguments(data) => {
                (&data.expression, data.type_arguments.clone())
            }
            _ => return self.error_type(),
        };

        // Go resolveEntityName：missing（零宽）实体名直接返回 nil，
        // 不进 2304/2749 补救链
        if type_name.loc.pos() >= type_name.loc.end() {
            return self.error_type();
        }

        if type_name.kind == SyntaxKind::Identifier && type_name.text() == "intrinsic" {
            return self.error_type();
        }
        // Go checkGrammarForAtLeastOneTypeArgument：`C<>` 空实参列表报 1099，
        // 跨度覆盖 <>，文件带解析错误时不报（文法检查短路）
        if let Some(args) = &type_arguments
            && args.nodes.is_empty()
            && !self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 1099 && d.loc.pos() == args.loc.pos())
            && !self
                .get_source_file_of_node(node)
                .or_else(|| self.current_file.clone())
                .is_some_and(|f| f.has_parse_diagnostics)
        {
            let file = self
                .get_source_file_of_node(node)
                .or_else(|| self.current_file.clone());
            let loc = tsox_core::core::text::TextRange::new(args.loc.pos(), args.loc.end() + 1);
            let mut diag = tsox_frontend::ast::Diagnostic::new(
                file,
                loc,
                tsox_core::diagnostics::messages_generated::TYPE_ARGUMENT_LIST_CANNOT_BE_EMPTY,
                vec![],
            );
            self.diagnostics.add(diag);
        }
        // Go nameresolver.go KindExpressionWithTypeArguments 分支：基类表达式内
        // 名字落在所在类类型参数上时报 TS2562，且早退跳过 2304 补救链
        if type_name.kind == SyntaxKind::Identifier
            && self.base_expression_type_parameters_hit(
                type_name,
                type_name.text(),
                SymbolFlags::TYPE,
            )
        {
            let file = self
                .get_source_file_of_node(type_name)
                .or_else(|| self.current_file.clone());
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                type_name.loc,
                tsox_core::diagnostics::messages_generated::BASE_CLASS_EXPRESSIONS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS,
                vec![],
            ));
            // Go resolveTypeReferenceName（checker.go:24449-24452）：解析失败接
            // getUnresolvedSymbolForEntityName——TypeAlias+Unresolved 符号，
            // getTypeReferenceType（checker.go:24507-24509）落
            // getTypeFromTypeAliasReference 未解析分支产 errorType+alias，
            // 渲染按实体名而非 unresolved 内在名
            return match self.get_unresolved_symbol_for_entity_name(type_name) {
                Some(sym) => self.get_type_reference_type(node, &sym),
                None => self.error_type(),
            };
        }
        let mut symbol = if type_name.kind == SyntaxKind::Identifier {
            match self.resolve_identifier_scope_symbol(type_name, SymbolFlags::TYPE) {
                Some(s) => {
                    // Go resolveEntityName（checker.go:16758-16768）：meaning 满
                    // 足的作用域符号就地返回，不随别名链落到值符号（
                    // getSymbol 的 getSymbolFlagsEx 别名回退
                    // checker.go:2272-2288 以链上含义过滤）；引用登记对齐
                    // resolve_identifier_use（noUnusedLocals 类型位使用计数）
                    self.record_symbol_reference(&s, SymbolFlags::TYPE | SymbolFlags::NAMESPACE);
                    s
                }
                None => {
                    // Go onFailedToResolveSymbol 失败钩子阶梯（checker.go:1643-1650）：
                    // checkAndReportErrorForUsingNamespaceAsTypeOrValue
                    //（checker.go:1724-1741，Type&^Value 含义时以 Module 再解析、
                    // 命中命名空间报 2709）先于 checkAndReportErrorForUsingValueAsType
                    //（checker.go:1801-1811，以 Value&^Type 再解析、命中符号带
                    // Namespace 位时静默），全未命中才走 2304 阶梯
                    let report_allowed = self.ts2304_reporting_allowed_for(type_name)
                        && !self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.file_name.starts_with("bundled://"))
                        // Go 表达式位的 instantiation expression（`f<T>`）由
                        // checkExpressionWithTypeArguments（checker.go:11222-11246）
                        // 按表达式求值，从不进 resolveTypeReferenceName 的失败阶梯
                        // （2709/2749/2304 均不适用）；heritage 位仍是类型位保留阶梯
                        && !(node.kind == SyntaxKind::ExpressionWithTypeArguments
                            && !node
                                .parent()
                                .is_some_and(|p| p.kind == SyntaxKind::HeritageClause));
                    let namespace_hit = self
                        .resolve_identifier_scope_symbol(type_name, SymbolFlags::MODULE)
                        .and_then(|s| self.follow_alias(&s));
                    let value_hit_sym = self
                        .resolve_identifier_scope_symbol(
                            type_name,
                            SymbolFlags::VALUE.difference(SymbolFlags::TYPE),
                        )
                        .and_then(|s| self.follow_alias(&s));
                    let value_hit = namespace_hit.is_none()
                        && value_hit_sym
                            .is_some_and(|s| !s.flags.intersects(SymbolFlags::NAMESPACE));
                    if namespace_hit.is_some() {
                        if report_allowed {
                            let name_text = type_name.text();
                            let file = self
                                .get_source_file_of_node(type_name)
                                .or_else(|| self.current_file.clone());
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                type_name.loc,
                                tsox_core::diagnostics::messages_generated::
                                    CANNOT_USE_NAMESPACE_0_AS_A_TYPE,
                                vec![name_text.to_string()],
                            ));
                        }
                    } else if value_hit {
                        if report_allowed {
                            let name_text = type_name.text();
                            let file = self
                                .get_source_file_of_node(type_name)
                                .or_else(|| self.current_file.clone());
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                type_name.loc,
                                tsox_core::diagnostics::messages_generated::
                                    X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                                vec![name_text.to_string()],
                            ));
                        }
                    } else if report_allowed {
                        use tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
                        let name_text = type_name.text();

                        let file = self
                            .get_source_file_of_node(type_name)
                            .or_else(|| self.current_file.clone());
                        // Go checkAndReportErrorForUsingTypeAsValue：原生类型名
                        // 出现在 heritage 子句时报 2840/2863/2862 专用消息
                        let heritage_message = Self::primitive_heritage_message(node, name_text);
                        if let Some(msg) = heritage_message {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                type_name.loc,
                                msg,
                                vec![name_text.to_string()],
                            ));
                        } else if let Some(lib) =
                            crate::checker::checker_lib_feature_map::suggested_lib_for_name(
                                name_text,
                            )
                        {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                type_name.loc,
                                tsox_core::diagnostics::messages_generated::
                                    CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_1_OR_LATER,
                                vec![name_text.to_string(), lib.to_string()],
                            ));
                        } else if let Some(suggestion) =
                            self.find_name_suggestion(name_text, SymbolFlags::TYPE)
                        {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                type_name.loc,
                                tsox_core::diagnostics::messages_generated::
                                    CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                                vec![name_text.to_string(), suggestion],
                            ));
                        } else {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                type_name.loc,
                                CANNOT_FIND_NAME_0,
                                vec![name_text.to_string()],
                            ));
                        }
                    }
                    // Go resolveTypeReferenceName（checker.go:24438-24452）：解析失败
                    // 经 getUnresolvedSymbolForEntityName 建 TypeAlias+Unresolved 符号，
                    // getTypeReferenceType（checker.go:24511-24513）落
                    // getTypeFromTypeAliasReference 未解析臂产 errorType+alias，
                    // 渲染按别名符号实体名（nodebuilderimpl.go:3431-3434）呈 _element
                    // 而非 intrinsic 名 unresolved
                    return match self.get_unresolved_symbol_for_entity_name(type_name) {
                        Some(sym) => self.get_type_reference_type(node, &sym),
                        None => self.error_type(),
                    };
                }
            }
        } else if matches!(
            type_name.kind,
            SyntaxKind::Identifier
                | SyntaxKind::QualifiedName
                | SyntaxKind::PropertyAccessExpression
        ) {
            // Go resolveEntityName（checker.go:16747）按调用方 meaning（类型位
            // 为 Type，checker.go:24432）解析：getSymbol locals 门以链上含义
            // 过滤（checker.go:2272-2288 getSymbolFlagsEx 别名回退），与
            // default 导出 local 合并出的别名符号（链上仅值意义）在类型位不
            // 命中，walk 上溯由模块 exports["default"] 名臂供接口符号
            //（nameresolver.go:122-129）。全含义解析会让该合并符号就地命中
            // 并沿别名落到值符号（TS2749 误报）。Type 解析失败时回退全含义，
            // 纯值名的既有诊断路径零变化
            let traced = if type_name.kind == SyntaxKind::Identifier {
                match self.resolve_identifier_with_meaning(type_name, SymbolFlags::TYPE) {
                    Some(s) => Ok(s),
                    None => self
                        .resolve_identifier_with_meaning(type_name, SymbolFlags::all())
                        .ok_or((Arc::clone(type_name), String::new(), String::new())),
                }
            } else {
                self.resolve_qualified_symbol_traced(type_name)
            };
            match traced {
                Ok(s) => {
                    // Go resolveQualifiedName：成员查找按 Type 含义过滤，命中纯值
                    // 符号（函数/变量/实例化命名空间）时经 tryGetQualifiedNameAsValue
                    // 给出 TS2749 typeof 建议；类 extends 位按 Value 解析除外
                    let in_class_extends_position = node.kind
                        == SyntaxKind::ExpressionWithTypeArguments
                        && node.parent().is_some_and(|cl| {
                            cl.kind == SyntaxKind::HeritageClause
                                && matches!(&cl.data, NodeData::HeritageClause(h) if h.token == SyntaxKind::ExtendsKeyword)
                                && cl.parent().is_some_and(|gp| {
                                    matches!(
                                        gp.kind,
                                        SyntaxKind::ClassDeclaration
                                            | SyntaxKind::ClassExpression
                                    )
                                })
                        });
                    if type_name.kind == SyntaxKind::QualifiedName
                        && !in_class_extends_position
                        && s.flags != SymbolFlags::Alias
                        && !s.flags.intersects(SymbolFlags::TYPE)
                        && self.try_get_qualified_name_as_value(type_name).is_some()
                        && self.ts2304_reporting_allowed_for(type_name)
                        && self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let name_text =
                            crate::checker::checker::qualified_name_text(type_name);
                        let file = self
                            .get_source_file_of_node(type_name)
                            .or_else(|| self.current_file.clone());
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            type_name.loc,
                            tsox_core::diagnostics::messages_generated::
                                X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                            vec![name_text.clone(), name_text],
                        ));
                        // Go resolveTypeReferenceName（checker.go:24438-24452）：
                        // resolveQualifiedName 报 TS2749 返 nil 后接
                        // getUnresolvedSymbolForEntityName——TypeAlias+Unresolved
                        // 符号带父链（A），getTypeReferenceType
                        // （checker.go:24511-24513）落 getTypeFromTypeAliasReference
                        // 未解析分支产 errorType+alias，渲染按别名符号限定名（A.B）
                        return match self.get_unresolved_symbol_for_entity_name(type_name) {
                            Some(sym) => self.get_type_reference_type(node, &sym),
                            None => self.error_type(),
                        };
                    }
                    s
                }
                Err((segment, ns_path, member)) => {
                    self.report_qualified_name_resolution_failure(
                        type_name, &segment, ns_path, member,
                    );
                    return match self.get_unresolved_symbol_for_entity_name(type_name) {
                        Some(sym) => self.get_type_reference_type(node, &sym),
                        None => self.error_type(),
                    };
                }
            }
        } else {
            return self.error_type();
        };

        // Go checkAndReportErrorForUsingNamespaceAsTypeOrValue：类型位解析到
        // 模块符号（不含类型含义）报 TS2709，优先于 TS2749/TS2304 兜底
        {
            let alias_decl_is_namespace_form = !symbol.flags.contains(SymbolFlags::Alias)
                || symbol.declarations.iter().any(|d| match &d.data {
                    NodeData::ImportEqualsDeclaration(_) => true,
                    NodeData::NamespaceImport(_) => true,
                    NodeData::ImportDeclaration(id) => id
                        .import_clause
                        .as_ref()
                        .and_then(|c| match &c.data {
                            NodeData::ImportClause(ic) => ic.named_bindings.as_ref(),
                            _ => None,
                        })
                        .is_some_and(|nb| nb.kind == SyntaxKind::NamespaceImport),
                    _ => false,
                });
            // Go getSymbolFlags：别名链上标志逐级 OR（合并符号如
            // `import * as B` + `interface B` 的 Interface 位保留在链中）
            let chain_flags = self.symbol_flags_with_alias_chain(&symbol);
            // Go 类 extends 子句按 Value 含义解析（ValueModule 命中不失败），
            // 2507 由基类构造检查裁决；仅 implements/类型位走 2709
            let in_class_extends_position = node.kind == SyntaxKind::ExpressionWithTypeArguments
                && node.parent().is_some_and(|cl| {
                    cl.kind == SyntaxKind::HeritageClause
                        && matches!(&cl.data, NodeData::HeritageClause(h) if h.token == SyntaxKind::ExtendsKeyword)
                        && cl.parent().is_some_and(|gp| {
                            matches!(
                                gp.kind,
                                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                            )
                        })
                });
            if alias_decl_is_namespace_form
                && !in_class_extends_position
                && chain_flags.intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule)
                && !chain_flags.intersects(SymbolFlags::TYPE)
                && type_name.kind == SyntaxKind::Identifier
                && self.ts2304_reporting_allowed_for(type_name)
                && self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
            {
                let file = self
                    .get_source_file_of_node(type_name)
                    .or_else(|| self.current_file.clone());
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    tsox_core::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_TYPE,
                    vec![type_name.text().to_string()],
                ));
                // Go resolveTypeReferenceName（checker.go:24449-24452）：解析失败
                // 后接 getUnresolvedSymbolForEntityName——TypeAlias+Unresolved
                // 符号走 getTypeReferenceType（checker.go:24511-24513）TypeAlias
                // 臂，getTypeFromTypeAliasReference（checker.go:24972-24985）
                // Unresolved 分支产 intrinsic("error")+alias 型，渲染按实体名
                // （var a: A 呈 A 而非 any），赋值表达式型随 RHS 呈对侧命名空间名
                return match self.get_unresolved_symbol_for_entity_name(type_name) {
                    Some(sym) => self.get_type_reference_type(node, &sym),
                    None => self.error_type(),
                };
            }
        }

        if symbol.flags == SymbolFlags::Alias {
            let alias_name = type_name.text().to_string();
            // Go getTypeReferenceType（checker.go:24507-24514）：别名符号落
            // tryGetDeclaredTypeOfSymbol → getDeclaredTypeOfAlias → resolveAlias，
            // 内部 import a = X.Y 实体名与 require 模块同达。
            // Go 具名导入与 re-export 导出的目标语义不同：
            // getTargetOfImportSpecifier（checker.go:15463-15478）以
            // dontResolveAlias=TRUE 取模块自有导出表原始条目（getExportOfModule，
            // checker.go:15614-15622）——增广合并后条目为 Alias|TypeAlias 合并
            // 符号，getTypeReferenceType（checker.go:24505-24507）按 TypeAlias
            // 臂经 getDeclaredTypeOfSymbol 取合并声明中 TypeAliasDeclaration
            //（增广声明）的类型；getTargetOfExportSpecifier
            //（checker.go:16693-16694）则 dontRecursivelyResolve=TRUE 解析目标
            // 模块成员到最终符号。按别名声明形态（ImportSpecifier vs
            // ExportSpecifier）区分两条链——越过导出表条目深解会把增广
            // 声明换成深解目标（mergeSymbolReexportInterface 的 index.d.ts
            // Row2 呈 Row2 而非 import("./common").Row2）
            let is_import_specifier_alias = symbol
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::ImportSpecifier);
            let is_export_specifier_alias = symbol
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::ExportSpecifier);
            let mut target = if is_import_specifier_alias {
                // 具名导入条目须带 TypeAlias 位方走 TypeAlias 臂；纯 re-export
                // 别名条目（无 TypeAlias 位）回落既有深解链
                self.import_specifier_raw_export_entry(&symbol)
                    .filter(|t| t.flags.intersects(SymbolFlags::TypeAlias))
            } else if is_export_specifier_alias {
                // Go getTargetOfExportSpecifier（checker.go:15783-15797）：
                // 解析 ExportDeclaration 模块说明符指向模块的成员
                //（dontRecursivelyResolve=TRUE 不再深解成员别名）
                self.export_specifier_module_member(&symbol)
            } else {
                None
            };
            if target.is_none() {
                // 条目缺位：既有深解链
                target = self.resolve_import_alias_target_symbol(&symbol).or_else(|| {
                    let resolved = self.resolve_alias(&symbol);
                    let unknown = self.unknown_symbol();
                    if Arc::ptr_eq(&resolved, &unknown) {
                        None
                    } else {
                        Some(resolved)
                    }
                });
            }
            if let Some(target) = target {
                let target_has_type_meaning = target.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::ENUM
                        | SymbolFlags::TypeParameter,
                );
                // Go resolveEntityName（checker.go:16764-16769）：meaning 位未
                // 满足且符号仍带别名位时沿 resolveAlias 继续穿透——import
                // default 的目标常是模块 default 导出别名（export default X），
                // 须穿到 X 实体符号；链断（unknownSymbol）不穿透
                let target = if !target_has_type_meaning
                    && target.flags.intersects(SymbolFlags::Alias)
                {
                    let resolved = self.resolve_alias(&target);
                    let unknown = self.unknown_symbol();
                    if Arc::ptr_eq(&resolved, &unknown) {
                        None
                    } else {
                        Some(resolved)
                    }
                } else {
                    Some(target)
                };
                match target {
                    Some(target) => {
                        let target_has_type_meaning = target.flags.intersects(
                            SymbolFlags::Interface
                                | SymbolFlags::Class
                                | SymbolFlags::TypeAlias
                                | SymbolFlags::ENUM
                                | SymbolFlags::TypeParameter,
                        );
                        if target_has_type_meaning {
                            symbol = target;
                        } else {
                            // Go 值作类型仅在值含义解析成功而类型含义失败时报；
                            // 目标仍是纯别名（链断，对应 unknownSymbol 全含义）不报
                            if type_name.kind == SyntaxKind::Identifier
                                && target.flags.intersects(SymbolFlags::VALUE)
                                && self.ts2304_reporting_allowed_for(type_name)
                                && !self.has_same_named_type_symbol(&alias_name)
                                && self
                                    .current_file
                                    .as_ref()
                                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                            {
                                let file = self.current_file.clone();
                                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    file,
                                    type_name.loc,
                                    tsox_core::diagnostics::messages_generated::
                                        X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                                    vec![alias_name.clone(), alias_name],
                                ));
                            }
                            // Go resolveTypeReferenceName（checker.go:24443-24451）：
                            // resolveEntityName 值作类型报 2749 后 resolveName 返
                            // nil，接 getUnresolvedSymbolForEntityName 建
                            // TypeAlias+Unresolved 符号，getTypeReferenceType
                            //（checker.go:24507-24509）TypeAlias 臂经
                            // getTypeFromTypeAliasReference（checker.go:24971-24985）
                            // Unresolved 分支产 errorType+alias，.types 发射通道
                            // 按别名符号实体名呈 originalZZZ 而非 intrinsic any
                            return match self.get_unresolved_symbol_for_entity_name(type_name)
                            {
                                Some(sym) => self.get_type_reference_type(node, &sym),
                                None => self.error_type(),
                            };
                        }
                    }
                    None => return self.error_type(),
                }
            }
        }

        if !self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"))
            && symbol
                .flags
                .intersects(SymbolFlags::Interface | SymbolFlags::Class | SymbolFlags::TypeAlias)
            && !type_name_inside_conditional_branch(type_name)
            && !type_name_shadowed_by_type_parameter(type_name)
            && !self.check_type_reference_arguments(node, type_name, &symbol)
        {
            return self.error_type();
        }

        if symbol.flags.contains(SymbolFlags::TypeParameter) {
            return self.resolve_type_parameter_reference(&symbol);
        }
        // Go getTypeFromClassOrInterfaceReference（checker.go:24525）取声明型前先过
        // getMergedSymbol：模块增广的同名 interface 符号经 recordMergedSymbol 映射到
        // 合并后的类符号（Class|Interface）。缺此步时增广 interface 的纯符号视图把
        // 引用带进 interface 分支另产接口型，类合并语义（isClassInstanceSide 对
        // getDeclaredTypeOfClassOrInterface 单例的同一性）失效，引用处渲染落静态侧
        let symbol = self.get_merged_symbol(&symbol);
        // tsc getTypeReferenceType：class+interface 同名合并的符号走 class 引用路径，
        // interface 声明的成员与基类并入同一声明类型（getDeclaredTypeOfClassOrInterface）。
        // Go getTypeFromClassOrInterfaceReference（checker.go:24525-24530）对任意
        // class 符号取 getDeclaredTypeOfSymbol，不限声明形态——命名类表达式
        //（bindAnonymousDeclaration，声明为 ClassExpression）同走此臂
        if symbol.flags.contains(SymbolFlags::Class)
            && symbol
                .declarations
                .iter()
                .any(|d| {
                    d.kind == SyntaxKind::ClassDeclaration
                        || d.kind == SyntaxKind::ClassExpression
                })
        {
            let key = Arc::as_ptr(&symbol) as *const tsox_frontend::ast::Symbol;
            let merged_with_ns = symbol.flags.contains(SymbolFlags::ValueModule);
            // 声明缓存仅在无类型实参时可直接复用；带实参引用须实例化（attach），
            // 否则 C<number> 会被裸声明形态污染。
            // Class|Interface 合并符号不吃该槽：resolve_interface_type_ex 对同一
            // 槽写接口视图型（typenode_references_checker_4.rs:144），早缓存会把
            // 接口型当声明型返回，绕过 getDeclaredTypeOfClassOrInterface 单例，
            // isClassInstanceSide 同一性失效渲染 typeof——Go 合并符号的
            // declaredType 槽恒持类实例声明型（接口成员原地并入），无接口视图
            let class_interface_merged = symbol.flags.intersects(SymbolFlags::Interface);
            if !merged_with_ns && !class_interface_merged && type_arguments.is_none() {
                // Go getTypeFromClassOrInterfaceReference（checker.go:24525）无实参
                // 类引用恒经 getDeclaredTypeOfClassOrInterface(getMergedSymbol)
                // 单例（resolvedSymbolLinks.declaredType，静态/实例侧判定
                // isClassInstanceSide nodebuilderimpl.go:349-352 凭该单例同一性）。
                // 本仓等价单例是 build_class_instance_type_with_base 的按节点缓存，
                // 经 get_declared_type_of_class_or_interface 取；不读
                // type_alias_links.declared_type 槽——同一槽被
                // resolve_interface_type_ex（typenode_references_checker_4.rs:
                // 143-145）按接口视图型复用，且其调用方直传 t.symbol 不限
                // Interface 位（checker_symbol_types_checker_2.rs:625 等），类符号
                // 被写入接口视图后类引用读到非单例对象，ptr_eq 失效渲染静态侧
                // typeof（instanceAndStaticDeclarations1 的 (p: typeof Point) vs
                // (p: Point)）。Class|Interface 合并符号本就不吃该槽读，此处收紧
                // 到普通类同口径
                return self.get_declared_type_of_class_or_interface(&symbol);
            }
            if !self.resolving_type_aliases.insert(key) {
                return self.error_type();
            }
            let class_node = symbol
                .declarations
                .iter()
                .find(|d| {
                    d.kind == SyntaxKind::ClassDeclaration || d.kind == SyntaxKind::ClassExpression
                })
                .cloned();
            let has_type_arguments = type_arguments.is_some();
            let arg_types: Option<Vec<Arc<Type>>> = type_arguments.map(|nodes| {
                nodes
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect()
            });
            let class_tps: Vec<Arc<tsox_frontend::ast::Symbol>> = match &class_node {
                Some(node) => match &node.data {
                    tsox_frontend::ast::NodeData::ClassDeclaration(cd) => {
                        match &cd.type_parameters {
                            Some(tps) => tps
                                .iter()
                                .filter_map(|tp| {
                                    self.program
                                        .symbol_map()
                                        .symbol_of(tp)
                                        .map(Arc::clone)
                                })
                                .collect(),
                            None => Vec::new(),
                        }
                    }
                    tsox_frontend::ast::NodeData::ClassExpression(ce) => {
                        match &ce.type_parameters {
                            Some(tps) => tps
                                .iter()
                                .filter_map(|tp| {
                                    self.program
                                        .symbol_map()
                                        .symbol_of(tp)
                                        .map(Arc::clone)
                                })
                                .collect(),
                            None => Vec::new(),
                        }
                    }
                    _ => Vec::new(),
                },
                None => Vec::new(),
            };
            if has_type_arguments && class_tps.is_empty() {
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_GENERIC,
                    vec![symbol.name.clone()],
                ));
                self.resolving_type_aliases.remove(&key);
                return self.error_type();
            }
            // Go getTypeFromClassOrInterfaceReference（checker.go:24551-24553）：
            // 类型别名声明体内的类引用走延迟引用——isDeferredTypeReferenceNode
            //（checker.go:24595-24597）首判 getAliasSymbolForTypeNode 命中即
            // createDeferredTypeReference（checker.go:26614-26627，声明型为
            // target，getAliasForTypeNode 挂宿主别名），显示按别名符号先行渲染
            //（nodebuilderimpl.go:3552），实参留待实例化按节点补齐。arity 窗口
            // 对齐 Go：numTypeArguments 须落在 [minTypeArgumentCount, 参数数]
            // 内（checker.go:24535-24568 先 arity 校验后延迟）；越界维持既有
            // 急切路径（Go 该位报 arity 错误，本侧无对应错误通道）。窗口从
            // 类声明节点类型参数取——本侧类声明型是 TypeData::Object
            //（build_class_instance_type_with_base，checker_class_instance_type.rs:24），
            // 无 InterfaceData，声明型 as_interface_type 恒 None
            if node.kind == SyntaxKind::TypeReference
                && self.alias_symbol_for_type_node(node).is_some()
            {
                let (tps_len, min_count) = class_node
                    .as_ref()
                    .and_then(|cn| match &cn.data {
                        NodeData::ClassDeclaration(cd) => cd.type_parameters.as_ref().map(|tps| {
                            let min_count = tps
                                .iter()
                                .position(|tp| {
                                    matches!(
                                        &tp.data,
                                        NodeData::TypeParameterDeclaration(td)
                                            if td.default_type.is_some()
                                    )
                                })
                                .unwrap_or(tps.len());
                            (tps.len(), min_count)
                        }),
                        _ => None,
                    })
                    .unwrap_or((0, 0));
                let num_type_arguments = arg_types.as_ref().map_or(0, |a| a.len());
                if tps_len > 0
                    && num_type_arguments >= min_count
                    && num_type_arguments <= tps_len
                {
                    let declared = self.get_declared_type_of_class_or_interface(&symbol);
                    let deferred =
                        self.create_deferred_type_reference(&declared, Some(node), None, None);
                    self.resolving_type_aliases.remove(&key);
                    return deferred;
                }
            }
            let args_match = arg_types
                .as_ref()
                .is_some_and(|a| !a.is_empty() && a.len() == class_tps.len());
            let instance_type = match class_node {
                Some(node) => {
                    let class_type = if args_match {
                        self.instantiate_class_instance_type(
                            &node,
                            &symbol,
                            &class_tps,
                            arg_types.as_ref().unwrap(),
                        )
                    } else {
                        self.build_class_instance_type_with_base(&node)
                    };
                    if symbol.flags.intersects(SymbolFlags::Interface) {
                        if args_match {
                            let iface_type =
                                self.resolve_interface_type_ex(&symbol, arg_types.clone());
                            self.merge_instance_types(&class_type, &iface_type)
                        } else {
                            // Go getTypeFromClassOrInterfaceReference（checker.go:24525）
                            // 无实参引用取 getDeclaredTypeOfClassOrInterface 单例：
                            // class+interface 合并符号（含模块增广）的声明型经机制
                            // 收敛同一对象（get_declared_type_of_symbol Class 分支
                            // 原地并 interface 成员），isClassInstanceSide
                            //（nodebuilderimpl.go:349-352）凭同一性渲染实例侧裸名
                            self.get_declared_type_of_class_or_interface(&symbol)
                        }
                    } else {
                        class_type
                    }
                }
                None => self.error_type(),
            };
            self.resolving_type_aliases.remove(&key);
            if !merged_with_ns && arg_types.is_none() {
                self.type_alias_links.get_or_default(&symbol).declared_type =
                    Some(Arc::clone(&instance_type));
            }

            if let Some(arg_types) = arg_types {
                if args_match {
                    return instance_type;
                }
                let tps = self.declared_type_parameter_types(&symbol);
                if !tps.is_empty() && tps.len() == arg_types.len() {
                    return self.attach_explicit_type_arguments_cached(&instance_type, arg_types);
                }
            }
            return instance_type;
        }
        if symbol.flags.contains(SymbolFlags::Interface) {
            let declared_tp_count = self.declared_type_parameter_types(&symbol).len();
            if type_arguments.is_some() && declared_tp_count == 0 {
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_GENERIC,
                    vec![symbol.name.clone()],
                ));
                return self.error_type();
            }
            // tsc getTypeFromClassOrInterfaceReference：无实参引用且类型参数带默认值时按默认值实例化；
            // 部分实参（如 Iterator<T> 少于 TReturn/TNext）补声明默认值
            //（Go getTypeArguments 的默认填充语义）
            match &type_arguments {
                None => {
                    let defaults = self.interface_default_type_arguments(&symbol);
                    if !defaults.is_empty() {
                        return self.resolve_interface_type_ex(&symbol, Some(defaults));
                    }
                }
                Some(nodes) if nodes.len() < declared_tp_count => {
                    // Go getTypeFromClassOrInterfaceReference（checker.go:24525-24563）
                    // 以 getDeclaredTypeOfClassOrInterface(...).LocalTypeParameters()
                    // ——合并收敛后的全声明并集表——为 fill 基准；本仓符号克隆链
                    // 不收敛同名类型参数（binder.go:464-466 + mergeSymbol
                    // checker.go:14963-14965 的收敛本仓缺失），臂内重建收敛表：
                    // get_merged_symbol 前置 + AppendIfUnique 身份去重 + 同名折叠
                    //（TS2428 identical 约束保证同位同名）。收敛表仅限本臂使用
                    // ——该表其他消费者依赖全声明原始表逐符号映射，折叠出臂会使
                    // 未收敛同名参数成孤儿（未实例化泄漏）；收敛后仍满实参的引用
                    // （Array/ReadonlyArray 等跨 lib 合并单参接口）不走 fill，
                    // 保持波起点解析路径
                    let raw_table =
                        self.get_local_type_parameters_of_class_or_interface_or_type_alias(&symbol);
                    let mut merged: Vec<Arc<Type>> = Vec::with_capacity(raw_table.len());
                    for tp in raw_table {
                        let tp = match tp.symbol.as_ref() {
                            Some(s) => {
                                let ms = self.get_merged_symbol(s);
                                self.get_declared_type_of_type_parameter(&ms)
                            }
                            None => tp,
                        };
                        if !merged.iter().any(|t| Arc::ptr_eq(t, &tp)) {
                            merged.push(tp);
                        }
                    }
                    let mut type_parameters: Vec<Arc<Type>> =
                        Vec::with_capacity(merged.len());
                    for tp in merged {
                        let duplicated = tp.symbol.as_ref().is_some_and(|s| {
                            type_parameters.iter().any(|t| {
                                t.symbol.as_ref().is_some_and(|o| o.name == s.name)
                            })
                        });
                        if !duplicated {
                            type_parameters.push(tp);
                        }
                    }
                    if nodes.len() < type_parameters.len() {
                        let min_type_argument_count =
                            self.get_min_type_argument_count(&type_parameters);
                        if nodes.len() >= min_type_argument_count {
                            let mut args: Vec<Arc<Type>> = nodes
                                .iter()
                                .map(|a| self.get_type_from_type_node(a))
                                .collect();
                            let filled = self.fill_missing_type_arguments(
                                &args,
                                &type_parameters,
                                min_type_argument_count,
                                tsox_frontend::ast::is_in_js_file(node),
                            );
                            return self.resolve_interface_type_ex(&symbol, Some(filled));
                        }
                    }
                    let defaults = self.interface_default_type_arguments(&symbol);
                    if !defaults.is_empty() && nodes.len() < defaults.len() {
                        let mut args: Vec<Arc<Type>> = nodes
                            .iter()
                            .map(|a| self.get_type_from_type_node(a))
                            .collect();
                        args.extend(defaults[nodes.len()..].iter().cloned());
                        return self.resolve_interface_type_ex(&symbol, Some(args));
                    }
                }
                _ => {}
            }
            return self.resolve_interface_type(&symbol, type_arguments);
        }
        if symbol.flags.intersects(SymbolFlags::ENUM) {
            return self.resolve_enum_type(&symbol);
        }
        if symbol.flags.intersects(SymbolFlags::EnumMember) {
            // Go getTypeReferenceType（checker.go:24514-24519）：EnumMember 的
            // 声明型是 fresh 字面量，须经 getRegularTypeOfLiteralType 常规化后
            // 返回——类型位引用不携带 fresh 性（let 初始化式 widening 只对
            // isFreshLiteralType 为真的入参触发）
            let t = self.get_type_of_symbol(&symbol);
            return self.get_regular_type_of_literal_type(&t);
        }
        if !symbol.flags.contains(SymbolFlags::TypeAlias) {
            if matches!(&node.data, NodeData::ExpressionWithTypeArguments(_))
                && symbol.flags.intersects(
                    SymbolFlags::BlockScopedVariable
                        | SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::Function,
                )
                && !symbol.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::TypeParameter,
                )
            {
                let value_type = self.get_type_of_symbol(&symbol);
                if let Some(structured) = value_type.as_structured() {
                    for sig in structured.construct_signatures() {
                        if let Some(ret) = self.get_return_type_of_signature(sig) {
                            return ret;
                        }
                    }
                }
                return self.get_any_type();
            }

            if type_name.kind == SyntaxKind::Identifier
                && symbol.flags.intersects(
                    SymbolFlags::BlockScopedVariable
                        | SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::Function,
                )
                && !symbol.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeParameter
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::Alias,
                )
                && self.ts2304_reporting_allowed_for(type_name)
                && !self.has_same_named_type_symbol(type_name.text())
                && self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
            {
                let name_text = type_name.text().to_string();
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    tsox_core::diagnostics::messages_generated::
                        X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                    vec![name_text.clone(), name_text],
                ));
            }

            return self.error_type();
        }

        self.resolve_type_alias_reference_with_node(&symbol, type_arguments, Some(node))
    }

    pub(crate) fn interface_default_type_arguments(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("interface_default_type_arguments"); 
        let decl = symbol.declarations.iter().find(|d| {
            matches!(d.data, NodeData::InterfaceDeclaration(_))
        });
        let Some(decl) = decl else {
            return Vec::new();
        };
        let NodeData::InterfaceDeclaration(data) = &decl.data else {
            return Vec::new();
        };
        let Some(tps) = &data.type_parameters else {
            return Vec::new();
        };
        let has_any_default = tps.iter().any(|tp| {
            matches!(&tp.data, NodeData::TypeParameterDeclaration(td) if td.default_type.is_some())
        });
        if !has_any_default {
            return Vec::new();
        }
        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        self.push_scope(decl);
        let args: Vec<Arc<Type>> = tps
            .clone()
            .iter()
            .map(|tp| match &tp.data {
                NodeData::TypeParameterDeclaration(td) => match &td.default_type {
                    Some(default_node) => self.get_type_from_type_node(default_node),
                    None => self.get_unknown_type(),
                },
                _ => self.get_unknown_type(),
            })
            .collect();
        self.pop_scope();
        self.type_argument_stack = saved_stack;
        args
    }
}
