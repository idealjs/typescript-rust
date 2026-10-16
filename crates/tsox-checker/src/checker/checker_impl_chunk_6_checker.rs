#![allow(unused_imports)]

use crate::checker::checker_get_excluded_symbol_flags::is_declaration_name;
use crate::checker::checker_impl_chunk_6::*;
use crate::checker::mig::m1b::jsnum_from_string;
use crate::checker::mig::m1b::parse_pseudo_big_int;
use crate::checker::mig::m1b_4::jsnum_negate;
use crate::checker::mig::m1f::r24k9b_expression_context::is_in_expression_context;
use tsox_core::diagnostics::messages_generated::OPERATOR_0_CANNOT_BE_APPLIED_TO_TYPE_1;

impl Checker {
    pub fn get_type_of_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_node");
        // Go getTypeOfNode（checker.go:33781-33784）：with 语句 statement 子树内
        // 不回答语义问题，一律 errorType（.types 渲染 any）
        if crate::checker::mig::m1f::r24k9b_expression_context::is_in_with_statement(node) {
            return self.error_type();
        }

        if node.kind == SyntaxKind::ThisKeyword {
            return self.compute_type_of_node(node);
        }

        // Go getTypeOfNode（checker.go:33776-33876）入口不读 typeNodeLinks，每次
        // 按分支链重算：isInRightSideOfImportOrExportAssignment（utilities.go:1235）
        // 标识符经 GetSymbolAtLocation（checker.go:33463-33465）→
        // GetSymbolOfNameOrPropertyAccessExpression 取符号，取
        // getDeclaredTypeOfSymbol 声明型（error 回落 getTypeOfSymbol）。check 阶段
        // checkExportAssignment（checker.go:5955）经 checkExpressionCached 驻留的
        // 是值侧构造型，若先查缓存该分支永不可达（export = a 的 a 渲染错
        // typeof a），故置于缓存查询之前；表达式上下文节点不受影响
        //（其缓存值与重算同源）
        if !is_in_expression_context(node)
            && crate::checker::utilities_is_private_within_ambient::
                is_in_right_side_of_import_or_export_assignment(node)
        {
            // Go GetSymbolOfNameOrPropertyAccessExpression（checker.go:33624-33646）：
            // export = 右侧全含义臂在前（33629-33638），import = 右侧
            // GetSymbolOfPartOfRightHandSideOfImportEquals 在后（33639-33645，
            // 整名按 Namespace、限定名按 Value|Type|Namespace、dontResolveAlias）。
            // 全含义就地重解会被同名局部 interface 以 Type 含义截走
            //（import Y = A 落 B 内 interface A 而非全局 namespace A）
            let resolved = self.get_symbol_of_name_or_property_access_expression(node);
            if let Some(symbol) = resolved
                && !self
                    .unknown_symbol
                    .as_ref()
                    .is_some_and(|u| std::sync::Arc::ptr_eq(u, &symbol))
            {
                let declared = self.get_declared_type_of_symbol(&symbol);
                if !crate::checker::utilities::is_type_error(&declared) {
                    return declared;
                }
                return self.get_type_of_symbol(&symbol);
            }
        }

        let cache_writes_enabled = !self.flow_back_edge_isolation;
        // Go getTypeOfNode（checker.go:33776-33876）入口零缓存读：发射侧
        //（types_emit_query）数组字面量不经节点缓存回放，每次重查
        // checkExpressionEx（TypeOnly）保鲜全链——check 期驻留的手写取型值
        //（widen_initializer_type 脱鲜）会压掉保鲜重查，arrayCast
        // ({ foo: string; } | {})[] 联合归约退化 {}[]；check 期照常读缓存
        let emit_array_literal_fresh_query =
            self.types_emit_query && node.kind == SyntaxKind::ArrayLiteralExpression;
        if !emit_array_literal_fresh_query
            && let Some(cached) = self
                .type_node_links
                .get(node)
                .and_then(|l| l.resolved_type.clone())
        {
            if matches!(
                node.kind,
                SyntaxKind::PropertyAccessExpression | SyntaxKind::QualifiedName
            ) && cached.flags == crate::checker::types::TypeFlags::Any
            {
                let result = self.compute_type_of_node(node);
                if cache_writes_enabled {
                    self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
                }
                return result;
            }
            // Go getTypeOfNode（checker.go:33776-33876）入口不读节点缓存，非表达式
            // 声明名每次经分支链重算（IsDeclarationNameOrImportPropertyName →
            // getSymbolAtLocation → getSymbolOfDeclaration 合并符号 →
            // getTypeOfSymbol 聚合全部声明）；check 期 check_function_declaration
            // 驻留的声明名型建于 resolve_identifier 就地解析，script 文件顶层声明
            // 与 lib 全局符（lib.dom alert + 本文件 declare function alert）合并的
            // 产物此时可未被该解析路径取到，缓存回放即呈单签名函数型而非聚合
            // 过载型，故声明名查询强制按分支链重算
            if node.kind == SyntaxKind::Identifier
                && !is_in_expression_context(node)
                && tsox_frontend::ast::mig::m3f_4::is_declaration_name_or_import_property_name(
                    node,
                )
            {
                let result = self.compute_type_of_node(node);
                if cache_writes_enabled {
                    self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
                }
                return result;
            }
            // Go getTypeOfNode（checker.go:33776-33876）入口不读 typeNodeLinks，
            // 表达式位标识符每次经 IsExpressionNode → getRegularTypeOfExpression
            //（checker.go:33967-33972）→ getTypeOfExpression（checker.go:7785-7806，
            // 仅 flowTypeCache 捷径）→ checkExpressionEx(CheckModeTypeOnly) →
            // checkIdentifier（checker.go:11650-11673）重算，命中符号检查终态
            // 缓存。本侧检查期 checkExpressionCached 族在环解析窗口驻留的表达式
            // 标识符瞬态型（自引用初始化式 export const pick = () => pick() 的
            // callee 环 errorType）发射期重放即呈 any；重算仅对缓存为 error 型
            // 的标识符生效（环驻留瞬态型），健康缓存回放保持检查期语义——整合
            // 波实证无差别重算在流收窄族（narrowByClauseExpressionInSwitchTrue9
            // 等）以查询时点收窄态偏离检查期缓存值；重算结果的驻留仅限查询侧
            //（types_emit_query，get_type_at_location 置位）
            if node.kind == SyntaxKind::Identifier
                && is_in_expression_context(node)
                && crate::checker::utilities::is_type_error(&cached)
            {
                let result = self.compute_type_of_node(node);
                if self.types_emit_query {
                    self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
                }
                return result;
            }
            return cached;
        }
        let result = self.compute_type_of_node(node);
        // 自引用函数建型的重入窗口内，返回类型推断经此处拿到的部分型
        //（partial_type_of_function_like 产物）是瞬态占位：一旦驻留节点缓存，
        // 挂接后的呈现/再查询会命中陈旧部分型（>f : (p: I) => any 的第二层
        // 来源），故窗口内跳过驻留
        let transient_partial = self.in_partial_fn_return.is_some()
            && self
                .fn_self_return_partials
                .values()
                .any(|p| Arc::ptr_eq(p, &result));
        if !transient_partial && cache_writes_enabled {
            // Go getTypeOfNode（checker.go:33776-33876）计算后写回节点缓存
            //（等价 emitTypeNodeLinks），check 期首查值跨消费点稳定：
            // 推断多趟定型（infer_type_arguments 清缓存重查、checkCallArguments
            // 读缓存）与赋值兼容重查都依赖首查驻留值，不驻留时每个消费点按
            // 当拍语境态重算，contextualTyping objc8.t9 重查呈 T[] TS2322、
            // inferentialTypingWithObjectLiteralProperties 二趟实参定型候选
            // 漂移呈 [] | [unknown]；发射侧保鲜由入口零缓存读承担（上方
            // emit_array_literal_fresh_query），不依赖抑制写回
            self.type_node_links
                .get_or_default(node)
                .resolved_type = Some(result.clone());
        }
        result
    }

    pub(crate) fn compute_type_of_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("compute_type_of_node"); 
        // Go getTypeOfNode（checker.go:33759）先解析 heritage 元素宿主类
        let heritage_host =
            tsox_frontend::ast::mig::m3g_3::try_get_class_implementing_or_extending_heritage_clause_element(node);
        let class_extends_element = node.kind == SyntaxKind::ExpressionWithTypeArguments
            && matches!(&heritage_host, Some((_, false)));
        // Go isTypeHeritageClause（parser.go:1952-1956）：class implements 元素与
        // interface extends 元素同样经 parseTypeHeritageClauseElement 解析为
        // TypeReferenceNode(typeName=QualifiedName)，限定名链各段是 QN 内部节点：
        // IsExpressionNode 恒假（ast/utilities.go:2235-2239，仅 TypeQuery/JSDoc
        // 引用/JSX 标签宿主为真），IsPartOfTypeNode 只随顶段成立
        //（utilities.go:2274-2311，isPartOfTypeNodeInParent 不跨 QN 传播），
        // getTypeOfNode（checker.go:33776-33876）重算穿透全部分支落 errorType。
        // 本侧 implements 链仍为表达式形态（EWT+属性访问链），链段按值表达式
        // 求值会取到命名空间值型（typeof X/typeof A），在此对齐 Go 的
        // errorType；class extends 链两侧同为表达式形态（Go 走值路径，基线
        // workaround type_symbol_baseline.go:397-402），不在此列
        if matches!(
            node.kind,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::Identifier | SyntaxKind::QualifiedName
        ) && is_in_class_implements_heritage_name_chain(node)
        {
            return self.error_type();
        }
        // Go getTypeOfNode（checker.go:33776）：类型节点整体委托
        // getTypeFromTypeNode（含 NamedTupleMember 等）；类 extends 子句中的
        // EWT 例外——IsPartOfTypeNode 对其返回 false
        //（isPartOfTypeExpressionWithTypeArguments：宿主为类且 token 为 extends），
        // 不走类型节点求值，落到下方宿主类分支取首个基类型
        if !class_extends_element && tsox_frontend::ast::is_type_node(node) {
            return self.get_type_from_type_node(node);
        }
        // Go getTypeOfNode 的 IsTypeDeclaration/IsTypeDeclarationName 前置分支：
        // 类型声明节点（名）取 getDeclaredTypeOfSymbol 声明型（枚举名→成员字面量联合，
        // 非经表达式链落 getTypeOfFuncClassEnumModule 的值侧对象型）
        if tsox_frontend::ast::mig::m3g_2::is_type_declaration(node) {
            return match self.get_symbol_of_declaration(node) {
                Some(sym) => self.get_declared_type_of_symbol(&sym),
                None => self.error_type(),
            };
        }
        if tsox_frontend::ast::mig::m3g_2::is_type_declaration_name(node) {
            // Go 基线行为（exportDefaultInterfaceClassAndValue.types:
            // >Foo : foo；.symbols: Symbol(foo, Decl×ea)）：default 修饰类的
            // 名字行呈现 default 导出槽首声明的声明名。Go 形态 export default
            // class 为带 default 修饰符的 ClassDeclaration（parser.go:4268-4285
            // default 随 export 入 modifiers），声明经 declareSymbolEx
            // （binder.go:162-168）恒名 default 并入 exports.default 合并符号
            // （ClassExcludes 不含 Alias，bindExportAssignment binder.go:911
            // EA 侧 Alias 位合流），类名 getDeclaredTypeOfSymbol（checker.go:
            // 25072 Class|Interface 位在先）取类实例型，渲染名经
            // getNameOfSymbolAsWritten（nodebuilderimpl.go:1014/1024）default
            // 名回退打印首声明（export default foo 的 EA）表达式标识符 foo；
            // 本侧类符号独立名 Foo，按观测行为取 exports.default 别名槽首 EA
            // 的表达式裸名呈现（Go 顶层 const 符号无 parent，呈现不限定；
            // 本侧 insert_symbol_into_container 给文件级符号挂文件符号父链，
            // resolve 目标经 ANY+alias 渲染门会呈限定名
            // nodebuilder_checker_7.rs:136-165，故按声明名文本造无父链符号），
            // 渲染门对齐 nodebuilderimpl.go:3422-3425
            if let Some(parent) = node.parent()
                && parent.kind == SyntaxKind::ClassDeclaration
                && parent.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Default)
                && let Some(default_sym) = self
                    .file_symbol_of(node)
                    .and_then(|f| f.exports.get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT).cloned())
                    .filter(|s| s.flags.intersects(tsox_frontend::ast::SymbolFlags::Alias))
            {
                let decl_name = default_sym.declarations.iter().find_map(|d| {
                    if let tsox_frontend::ast::NodeData::ExportAssignment(ea) = &d.data
                        && let tsox_frontend::ast::NodeData::Identifier(id) = &ea.expression.data
                    {
                        return Some(id.text.clone());
                    }
                    None
                });
                let target = match decl_name {
                    Some(text) => Arc::new(Symbol::new(SymbolFlags::Alias, text)),
                    None => self.resolve_alias_base(default_sym),
                };
                return Arc::new(crate::checker::types::Type {
                    flags: crate::checker::types::TypeFlags::Any,
                    object_flags: crate::checker::types::ObjectFlags::NonInferrableType,
                    id: crate::checker::types::next_type_id(),
                    symbol: None,
                    alias: Some(Box::new(crate::checker::types::TypeAlias::new(
                        Some(target),
                        Vec::new(),
                    ))),
                    data: crate::checker::types::TypeData::Intrinsic(
                        crate::checker::types::IntrinsicTypeData {
                            intrinsic_name: "error".to_string(),
                        },
                    ),
                });
            }
            return match self.get_symbol_at_location(node) {
                Some(sym) => self.get_declared_type_of_symbol(&sym),
                None => self.error_type(),
            };
        }
        // Go getTypeOfNode（checker.go:33806）：既非类型节点也非表达式的类 extends
        // 元素取宿主类首个基类型（基类型解析只消费 GetClassExtendsHeritageElement
        // 首元素，多基类 extends 全部元素同取首基类型），无基类型时 errorType；
        // 本侧类实例型为 Object 壳（ObjectFlags::Anonymous，无 Interface 型数据），
        // getBaseTypes 的 CLASS_OR_INTERFACE 守卫恒空表，首基型经
        // resolve_base_class_instance_type 按首 extends 元素求值
        //（Go resolveBaseTypesOfClass 的同一首元素消费点）
        if let Some((class_decl, false)) = heritage_host.as_ref() {
            let base_type = tsox_frontend::ast::get_class_extends_heritage_element(class_decl)
                .map(|element| self.resolve_base_class_instance_type(&element))
                .unwrap_or_else(|| self.error_type());
            return self.get_type_with_this_argument(&base_type, None, false);
        }
        // Go getTypeOfNode（checker.go:33851-33857）IsDeclarationNameOrImportPropertyName
        // → getTypeOfSymbol(getSymbolAtLocation)：字面量声明名（枚举成员数字名
        // 等）取声明符号类型，不做字面量求值；标识符声明名仍走下方 Identifier
        // 臂的同名分支（其含别名解析与纯类型命名空间守卫）。getSymbolAtLocation
        //（checker.go:33441-33448）对声明名取 getSymbolOfDeclaration(parent)
        //（checker.go:15181-15188）——恒过 getMergedSymbol，模块增广名经
        // recordMergedSymbol 映射到目标模块符号（ValueModule 位）
        // Go 解析形态分野：模块声明的 with 子句是 TypeLiteralNode（成员为
        // PropertySignature，声明名有符号与型，.types 呈属性型字面量
        // —— Go ast_generated.go ModuleDeclaration.Attributes）；import
        // 声明/导入型上的 attributes 是 ImportAttributes（ImportAttribute
        // 元素，名无符号 → getTypeOfNode 落 errorType，基线呈 error）。
        // 本仓两处同为 ImportAttribute 形态，此处按宿主补齐两臂；属性
        // 值节点非名位（Go IsExpressionNode 对 ImportAttribute 子代为假，
        // .types 不发射，取型同样 errorType）
        if node
            .parent()
            .as_ref()
            .is_some_and(|p| p.kind == SyntaxKind::ImportAttribute)
        {
            let attr = node.parent().unwrap();
            let is_name = matches!(&attr.data, NodeData::ImportAttribute(d) if Arc::ptr_eq(&d.name, node));
            if is_name {
                let host_is_module = attr
                    .parent()
                    .as_ref()
                    .and_then(|attrs| attrs.parent())
                    .is_some_and(|host| host.kind == SyntaxKind::ModuleDeclaration);
                if host_is_module {
                    if let NodeData::ImportAttribute(d) = &attr.data {
                        if let NodeData::StringLiteral(s) = &d.value.data {
                            return self.infer_string_literal_type(&s.text);
                        }
                    }
                }
            }
            return self.error_type();
        }
        if node.kind != SyntaxKind::Identifier
            && tsox_frontend::ast::mig::m3f_4::is_declaration_name_or_import_property_name(node)
        {
            let resolved = node
                .parent()
                .and_then(|parent| self.get_symbol_of_declaration(&parent))
                .map(|sym| self.get_merged_symbol(&sym));
            return match resolved {
                Some(sym) => self.get_type_of_symbol(&sym),
                None => self.error_type(),
            };
        }
        match node.kind {
            SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral
                if !is_in_expression_context(node) =>
            {
                // Go getTypeOfNode：数字系字面量属 IsExpressionNode 的
                // IsInExpressionContext 组（ast/utilities.go:2243-2245，
                // PropertyDeclaration 名 Initializer()==node 为 false），非表达式
                // 上下文落 IsDeclarationNameOrImportPropertyName（checker.go:33851）
                // → getSymbolAtLocation → getTypeOfSymbol——类成员 0: B 的名字取
                // 声明型 B 而非名字字面量 0
                match self.get_symbol_at_location(node) {
                    Some(sym) => self.get_type_of_symbol(&sym),
                    None => self.error_type(),
                }
            }
            SyntaxKind::NumericLiteral => {
                if let tsox_frontend::ast::NodeData::NumericLiteral(data) = &node.data {
                    let lit = self.infer_number_literal_type(&data.text);
                    return self.get_fresh_type_of_literal_type(&lit);
                }
                self.number_type()
            }
            SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
                if !is_in_expression_context(node) =>
            {
                // Go getTypeOfNode：非表达式上下文的字面量（声明名，如
                // declare module "m" 的模块名）不走表达式求值，落
                // IsDeclarationNameOrImportPropertyName → getSymbolOfDeclaration → getTypeOfSymbol；
                // getSymbolOfDeclaration（checker.go:15181-15188）取 getMergedSymbol，
                // 模块增广声明名经 recordMergedSymbol 映射到目标模块符号
                if is_declaration_name(node)
                    && let Some(parent) = node.parent()
                    && let Some(sym) = self.program.symbol_map().symbol_of(&parent).cloned()
                {
                    let sym = self.get_merged_symbol(&sym);
                    return self.get_type_of_symbol(&sym);
                }
                self.error_type()
            }
            SyntaxKind::StringLiteral => {
                if let tsox_frontend::ast::NodeData::StringLiteral(data) = &node.data {
                    let lit = self.infer_string_literal_type(&data.text);
                    return self.get_fresh_type_of_literal_type(&lit);
                }
                self.string_type()
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                if let tsox_frontend::ast::NodeData::NoSubstitutionTemplateLiteral(data) = &node.data
                {
                    let lit = self.infer_string_literal_type(&data.text);
                    return self.get_fresh_type_of_literal_type(&lit);
                }
                self.string_type()
            }
            SyntaxKind::TrueKeyword => self.get_fresh_type_of_literal_type(&self.true_type()),
            SyntaxKind::FalseKeyword => self.get_fresh_type_of_literal_type(&self.false_type()),
            SyntaxKind::NullKeyword => self.nullish_widening_type(self.null_type()),
            SyntaxKind::UndefinedKeyword => self.nullish_widening_type(self.undefined_type()),
            SyntaxKind::BigIntLiteral => {
                self.check_grammar_big_int_literal(node);
                let literal = self.get_big_int_literal_type(parse_pseudo_big_int(&node.text()));
                self.get_fresh_type_of_literal_type(&literal)
            }
            SyntaxKind::ArrayLiteralExpression => {
                // Go getTypeOfNode（checker.go:33806）IsExpressionNode 门：
                // getRegularTypeOfExpression → getTypeOfExpression →
                // checkExpressionEx（TypeOnly）。该重查只在发射/查询侧
                //（types_emit_query）安全：checkAssignmentCompat 的 RHS 与
                // call-arg 适用性的 arg 型都经 get_type_of_node 在 check 期
                // 取型，此时语境栈/推断上下文未随节点就位，TypeOnly 重查
                //（checkArrayLiteral → pushCachedContextualType → 元素
                // checkExpressionForMutableLocation）产出毒化元素型——
                // contextualTyping objc8.t9=[[,]] 呈 TS2322、
                // lambdaParamTypes/objectLiteralArraySpecialization 的
                // create([...]) 实参呈 'T[]'、spreadBooleanRespectsFreshness
                // 展开元素脱鲜呈 string|boolean。check 期改走手写
                // get_type_of_array_literal（自含元素取型，无重查副作用），
                // 结果照 Go 写回节点缓存（发射侧保鲜经入口零缓存读重查）
                if self.types_emit_query {
                    self.get_regular_type_of_expression(node)
                } else {
                    self.get_type_of_array_literal(node)
                }
            }
            SyntaxKind::ObjectLiteralExpression => {
                // Go getTypeOfNode（checker.go:33811-33813）统一走
                // getRegularTypeOfExpression → checkExpressionEx → checkObjectLiteral；
                // inConstContext（checker.go:14341-14349）成员挂 CheckFlagsReadonly、
                // 属性型经 checkExpressionForMutableLocation（checker.go:14632-14646）
                // 保 regular 字面量。本侧 getTypeOfNode 拉取与检查共用
                // type_node_links 缓存，非 const 上下文全量切 checkObjectLiteral
                // 会在推断上下文未就绪时带副作用物化成员符号型并驻留缓存，
                // 上下文参数型传播被陈旧无上下文型截断（w15-a0 全量 13 例
                // contextual 族回归）；仅 const 上下文（`x as const` 内层字面量，
                // get_const_assertion_type 经此处拉取，checker_literals.rs:102）
                // 走 checkObjectLiteral 语义修 readonly/字面量保持，非 const
                // 维持拉取专用路径（checker_literals.rs:179）
                if self.is_const_context(node) {
                    return self.get_regular_type_of_expression(node);
                }
                return self.get_type_of_object_literal(node);
            }
            SyntaxKind::PropertyAssignment => {
                if let tsox_frontend::ast::NodeData::PropertyAssignment(d) = &node.data
                    && let Some(literal) = node
                        .parent()
                        .as_ref()
                        .filter(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
                {
                    let name = self.get_property_name_from_node(&d.name);
                    return self.property_assignment_type(node, &d.initializer, literal, &name);
                }
                self.get_any_type()
            }
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
                // Go checkFunctionExpressionOrObjectLiteralMethod（checker.go:10707-10708）
                // 出口 getTypeOfSymbol(getSymbolOfDeclaration(node)) 节点型与符号型同
                // 对象——仅体自引（return somefn）使符号臂壳先行驻留时并入返回壳
                //（显示行单层展开），非自引保持按需建型（返回型非壳无双包，壳驻留
                // 不外溢出体自引形态）
                let base = self.get_type_of_function_like(node);
                let base = self.attach_expando_to_function_expression_type(node, base);
                match self.get_symbol_of_declaration(node) {
                    Some(symbol) => match self
                        .value_symbol_links
                        .get(&symbol)
                        .and_then(|l| l.resolved_type.clone())
                    {
                        Some(shell) => shell,
                        None => base,
                    },
                    None => base,
                }
            }
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
                self.get_type_of_class_declaration(node)
            }
            SyntaxKind::RegularExpressionLiteral => self.global_regexp_type(),
            SyntaxKind::FunctionDeclaration => self.get_type_of_function_like(node),
            SyntaxKind::Identifier => {
                // Go getTypeOfNode：IsTypeDeclarationName → getSymbolAtLocation + getDeclaredTypeOfSymbol
                //（getSymbolAtLocation 对声明名取 getSymbolOfDeclaration(parent)）
                if tsox_frontend::ast::mig::m3g_2::is_type_declaration_name(node) {
                    let symbol = node
                        .parent()
                        .and_then(|parent| self.get_symbol_of_declaration(&parent));
                    return match symbol {
                        Some(symbol) => self.get_declared_type_of_symbol(&symbol),
                        None => self.error_type(),
                    };
                }
                // Go getTypeOfNode（checker.go:33862-33871）：import/export 赋值
                // 右侧实体名不是表达式节点（IsInExpressionContext 对 export=
                // 表达式父级走 default → IsExpressionNode(ExportAssignment) 为
                // false），取 getDeclaredTypeOfSymbol 声明型——export = C 的 a
                // 显示实例型 C 而非值路径的 typeof C；声明型为 error 时回落
                // getTypeOfSymbol
                if !is_in_expression_context(node)
                    && crate::checker::utilities_is_private_within_ambient::
                        is_in_right_side_of_import_or_export_assignment(node)
                {
                    if let Some(symbol) = self.get_symbol_at_location(node) {
                        let declared = self.get_declared_type_of_symbol(&symbol);
                        if !crate::checker::utilities::is_type_error(&declared) {
                            return declared;
                        }
                        return self.get_type_of_symbol(&symbol);
                    }
                }
                // Go getTypeOfNode（checker.go:33851-33857）：声明名先于表达式
                // 查询被 IsDeclarationNameOrImportPropertyName 截获
                // → getSymbolAtLocation + getTypeOfSymbol——符号类型含声明期
                // widening（let 初始化式的 fresh 字面量宽化）；IsExpressionNode
                // 的 IsInExpressionContext（ast/utilities.go:2247-2252，父级
                // VariableDeclaration 时 initializer==node 才是表达式）已把
                // 表达式上下文的标识符排除在外。缺此分支时声明名走下方
                // get_type_of_identifier 的流查询，取到 flow initial type
                //（初始化式表达式原始型，未宽化）
                if !is_in_expression_context(node)
                    && tsox_frontend::ast::mig::m3f_4::
                        is_declaration_name_or_import_property_name(node)
                {
                    return match self.get_symbol_at_location(node) {
                        Some(symbol) => self.get_type_of_symbol(&symbol),
                        None => self.error_type(),
                    };
                }
                self.get_type_of_identifier(node)
            }
            SyntaxKind::MetaProperty => self.get_type_of_meta_property(node),

            SyntaxKind::BinaryExpression => self.get_type_of_binary_expression(node),
            SyntaxKind::PrefixUnaryExpression => {
                if let tsox_frontend::ast::NodeData::PrefixUnaryExpression(data) = &node.data {
                    if data.operator == SyntaxKind::MinusToken
                        && data.operand.kind == SyntaxKind::BigIntLiteral
                    {
                        self.check_grammar_big_int_literal(&data.operand);
                        let positive = parse_pseudo_big_int(&data.operand.text());
                        let negative =
                            tsox_core::jsnum::PseudoBigInt::new(&positive.to_string(), true);
                        let literal = self.get_big_int_literal_type(negative);
                        return self.get_fresh_type_of_literal_type(&literal);
                    }
                    if data.operator == SyntaxKind::PlusToken
                        && data.operand.kind == SyntaxKind::BigIntLiteral
                    {
                        self.check_grammar_big_int_literal(&data.operand);
                        self.error_message(
                            &data.operand,
                            OPERATOR_0_CANNOT_BE_APPLIED_TO_TYPE_1,
                            &["+".to_string(), "bigint".to_string()],
                        );
                        return self.number_type();
                    }
                    match data.operator {
                        SyntaxKind::MinusToken
                            if data.operand.kind == SyntaxKind::NumericLiteral =>
                        {
                            let value =
                                jsnum_negate(jsnum_from_string(data.operand.text()));
                            let literal = self.get_number_literal_type(value);
                            return self.get_fresh_type_of_literal_type(&literal);
                        }
                        SyntaxKind::PlusToken
                            if data.operand.kind == SyntaxKind::NumericLiteral =>
                        {
                            let value = jsnum_from_string(data.operand.text());
                            let literal = self.get_number_literal_type(value);
                            return self.get_fresh_type_of_literal_type(&literal);
                        }
                        SyntaxKind::ExclamationToken => {
                            // Go checkPrefixUnaryExpression（checker.go:11511-11520）：
                            // operandType := c.checkExpression(operand)——操作数走
                            // 检查链缓存（checkExpressionCached），! 按 truthiness
                            // facts 折叠 falseType/trueType
                            let operand = Arc::clone(&data.operand);
                            let operand_type = self.check_expression_cached(&operand);
                            let facts = self.get_type_facts(
                                &operand_type,
                                crate::checker::types::TypeFacts::TRUTHY
                                    | crate::checker::types::TypeFacts::FALSY,
                            );
                            if facts == crate::checker::types::TypeFacts::TRUTHY {
                                return self.false_type();
                            }
                            if facts == crate::checker::types::TypeFacts::FALSY {
                                return self.true_type();
                            }
                            return self.boolean_type();
                        }

                        SyntaxKind::DeleteKeyword => return self.boolean_type(),

                        SyntaxKind::VoidKeyword => return self.undefined_type(),

                        _ => return self.number_type(),
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::PostfixUnaryExpression => self.number_type(),
            // Go getTypeOfNode 表达式臂（checker.go:33967-33972 → getTypeOfExpression
            // checker.go:7785 → checkExpressionEx）按表达式检查分派（checker.go:8245-8248）：
            // 动态 import() 调用走 checkImportCallExpression 取 Promise<T>，不经
            // 签名解析
            SyntaxKind::CallExpression => {
                if tsox_frontend::ast::is_import_call(node) {
                    self.check_import_call_expression(node)
                } else {
                    self.get_return_type_of_call_expression(node)
                }
            }
            SyntaxKind::NewExpression => self.get_return_type_of_new_expression(node),
            SyntaxKind::PropertyAccessExpression => {
                // Go getTypeOfNode → IsExpressionNode → getRegularTypeOfExpression
                //（checker.go:33806/33967-33972）：表达式查询型把 fresh 字面量
                // 解包为 regular（枚举成员访问表达式 O.A.X 呈声明型 O.A）
                let t = self.get_type_of_property_access(node);
                self.get_regular_type_of_literal_type(&t)
            }
            SyntaxKind::QualifiedName => {
                // Go getTypeOfNode：IsExpressionNode（utilities.go:2228-2232，
                // 限定名链顶宿主为 TypeQuery/JSDoc/JSX 时为真）→
                // getRegularTypeOfExpression → getTypeOfExpression →
                // checkQualifiedName（checker.go:8604-8609，typeof this.x 的
                // this 根走 checkThisExpression）——typeof 实体名的 .types
                // 行按表达式链求值，不落 errorType 渲染 any
                let t = self.get_type_of_expression(node);
                self.get_regular_type_of_literal_type(&t)
            }
            SyntaxKind::ElementAccessExpression => self.get_type_of_element_access(node),
            SyntaxKind::ParenthesizedExpression => {
                if let tsox_frontend::ast::NodeData::ParenthesizedExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
                }
                self.get_any_type()
            }
            SyntaxKind::AsExpression => {
                if let tsox_frontend::ast::NodeData::AsExpression(data) = &node.data {
                    if data.type_node.kind == SyntaxKind::ConstKeyword {
                        return self.get_const_assertion_type(&data.expression);
                    }
                    return self.get_type_from_type_node(&data.type_node);
                }
                self.get_any_type()
            }
            SyntaxKind::SatisfiesExpression => {
                if let tsox_frontend::ast::NodeData::SatisfiesExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
                }
                self.get_any_type()
            }
            SyntaxKind::TypeAssertionExpression => {
                if let tsox_frontend::ast::NodeData::TypeAssertion(data) = &node.data {
                    // Go getTypeOfExpressionOfTypeAssertion：isConstTypeReference
                    // 不解析类型名，取表达式类型并保留字面量（const 断言）
                    if crate::checker::utilities_has_only_expression_initialization::is_const_type_reference(&data.type_node) {
                        return self.get_type_of_node(&data.expression);
                    }
                    return self.get_type_from_type_node(&data.type_node);
                }
                self.get_any_type()
            }
            SyntaxKind::NonNullExpression => {
                if let tsox_frontend::ast::NodeData::NonNullExpression(data) = &node.data {
                    let operand_type = self.get_type_of_node(&data.expression);
                    return self.get_non_nullable_type(&operand_type);
                }
                self.get_any_type()
            }
            SyntaxKind::ConditionalExpression => {
                if let tsox_frontend::ast::NodeData::ConditionalExpression(data) = &node.data {
                    let true_type = self.get_type_of_node(&data.when_true);
                    let false_type = self.get_type_of_node(&data.when_false);
                    let types = vec![true_type, false_type];
                    // Go checkConditionalExpression：UnionReductionSubtype（移除可赋给
                    // 其他成员的 structured 成员，如 any[] ⊑ number[] → number[]）
                    let reduced = self.remove_subtype_redundant_members(types);
                    return self.get_union_type(reduced);
                }
                self.get_any_type()
            }
            SyntaxKind::TemplateExpression => self.get_type_of_expression(node),
            SyntaxKind::TaggedTemplateExpression => {
                if let tsox_frontend::ast::NodeData::TaggedTemplateExpression(data) = &node.data {
                    let tag_type = self.get_type_of_node(&data.tag);
                    if let Some(structured) = tag_type.as_structured() {
                        for sig in structured.call_signatures() {
                            if let Some(rt) = self.get_return_type_of_signature(sig) {
                                return rt;
                            }
                            return self.get_any_type();
                        }
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::DeleteExpression => self.boolean_type(),
            SyntaxKind::VoidExpression => self.undefined_type(),
            SyntaxKind::OmittedExpression => Arc::clone(&self.undefined_widening_type),
            SyntaxKind::TypeOfExpression => self.typeof_type(),
            SyntaxKind::YieldExpression => {
                // Go checkYieldExpression：yield* 的类型 = 操作数迭代器的 TReturn
                if let tsox_frontend::ast::NodeData::YieldExpression(data) = &node.data
                    && data.asterisk_token.is_some()
                    && let Some(expr) = &data.expression
                {
                    let operand_type = self.get_type_of_node(expr);
                    if let Some(t) = self.get_yield_star_return_type(&operand_type) {
                        return t;
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::AwaitExpression => {
                if let tsox_frontend::ast::NodeData::AwaitExpression(data) = &node.data {
                    let operand = Arc::clone(&data.expression);

                    // Go checkAwaitExpression（checker.go:11443-11453）：
                    // checkExpression(operand) → checkAwaitedType——操作数为动态
                    // import() 时经 checkImportCallExpression 得
                    // Promise<synthetic>，await 取其 awaited 型（含 synthetic
                    // default 包装），不走裸模块命名空间型捷径
                    let operand_type = self.check_expression_cached(&operand);
                    return match self.get_awaited_type(&operand_type) {
                        Some(awaited) => awaited,
                        None => operand_type,
                    };
                }
                self.get_any_type()
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => {
                if node.kind == SyntaxKind::ThisKeyword {
                    return self.this_expression_type(node);
                }
                if node.kind == SyntaxKind::SuperKeyword
                    && self.super_in_computed_name_of_innermost_class(node)
                    && self.enclosing_class_stack.len() >= 2
                {
                    return self
                        .this_type_stack
                        .get(self.this_type_stack.len() - 2)
                        .cloned()
                        .unwrap_or_else(|| self.get_any_type());
                }

                let is_call_expression = node.parent().is_some_and(|p| {
                    matches!(&p.data,
                        tsox_frontend::ast::NodeData::CallExpression(c) if Arc::ptr_eq(&c.expression, node))
                });
                // Go checkSuperExpression（checker.go:8361-8387）：isLegalUsageOfSuperExpression
                // 门先于一切取型——super() 调用容器必须是构造器，super 属性访问容器
                // 必须是类/对象字面量的合法成员；不合法一律 errorType（.types 渲染 any）
                let (_, super_container) = crate::checker::checker_super_expressions::super_containers_of(node, is_call_expression);
                if !crate::checker::checker_super_expressions::is_legal_super_container(
                    is_call_expression,
                    &super_container,
                ) {
                    return self.error_type();
                }
                // Go checkSuperExpression（checker.go:8391-8394）：对象字面量成员的 super 取 anyType
                if super_container
                    .as_ref()
                    .and_then(|c| c.parent())
                    .is_some_and(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
                {
                    return self.get_any_type();
                }
                let syntactic = crate::checker::checker_super_expressions::super_container_class_and_staticity(node);
                if let Some(class) = syntactic
                    .as_ref()
                    .map(|(c, _)| Arc::clone(c))
                    .or_else(|| self.enclosing_class_stack.last().cloned())
                {
                    let is_static_member = syntactic
                        .as_ref()
                        .map(|(_, s)| *s)
                        .unwrap_or(self.this_container_stack.last() == Some(&ThisContainerKind::StaticMember));
                    if let Some(heritage) =
                        crate::checker::checker_classes_ctor_super_calls::class_extends_heritage_element(&class)
                    {
                        // Go checkSuperExpression（checker.go:8401-8406）：
                        // classDeclarationExtendsNull 先于 IsStatic/isCallExpression 的
                        // 构造型取值——super() 调用为 errorType（.types 渲染 any），
                        // super 属性访问为 nullWideningType，与静态性无关
                        if self.class_declaration_extends_null(&class) {
                            return if is_call_expression {
                                self.error_type()
                            } else {
                                self.null_widening_type()
                            };
                        }
                        // Go checkSuperExpression（checker.go:8416-8419）：
                        // classType = getDeclaredTypeOfSymbol(class 符号)
                        let Some(sym) = self.get_symbol_of_declaration(&class) else {
                            return self.error_type();
                        };
                        let class_type = self.get_declared_type_of_symbol(&sym);
                        // Go checkSuperExpression（checker.go:8416-8420）：
                        // 构造器参数初始化器中的 super 报 TS2336 并取
                        // errorType（.types 渲染 any），先于静态/调用分支；
                        // 诊断由 check_super_expression 侧发射，此处仅取型
                        if crate::checker::checker_super_expressions::get_super_container(node, true)
                            .is_some_and(|c| {
                                c.kind == SyntaxKind::Constructor
                                    && crate::checker::checker_super_expressions::is_in_constructor_argument_initializer(node, &c)
                            })
                        {
                            return self.error_type();
                        }
                        if is_static_member || is_call_expression {
                            // Go checkSuperExpression（checker.go:8440-8441）：
                            // IsStatic(container) || isCallExpression →
                            // getBaseConstructorTypeOfClass(classType)
                            return self
                                .get_base_constructor_type_of_class(&class_type)
                                .unwrap_or_else(|| self.error_type());
                        }
                        // Go checkSuperExpression（checker.go:8419-8421/8444-8445）：
                        // baseClassType = getBaseTypes(classType) 首基型，非静态
                        // 属性访问取 getTypeWithThisArgument(baseClassType,
                        // classType.thisType)；本侧类实例型为 Object 壳，
                        // getBaseTypes 的 CLASS_OR_INTERFACE 守卫恒空表，首基型
                        // 经 resolve_base_class_instance_type 按首 extends 元素
                        // 求值（Go resolveBaseTypesOfClass 同一首元素消费点）
                        let class_this = class_type
                            .as_interface_type()
                            .and_then(|i| i.this_type.clone());
                        let base_class_type = self.resolve_base_class_instance_type(&heritage);
                        return self.get_type_with_this_argument(
                            &base_class_type,
                            class_this.as_ref(),
                            false,
                        );
                    }
                }
                let r = self
                    .this_type_stack
                    .last()
                    .cloned()
                    .unwrap_or_else(|| self.get_any_type());
                r
            }
            SyntaxKind::SpreadElement => {
                // Go getTypeOfNode：KindSpreadElement 属 IsExpressionNode
                //（ast/utilities.go:2223）→ getRegularTypeOfExpression
                //（checker.go:33967）→ getTypeOfExpression →
                // checkSpreadExpression（checker.go:11551），spread 行呈现
                // 操作数的迭代元素型（如 ...robotAInfo : string | number）
                let t = self.get_type_of_expression(node);
                self.get_regular_type_of_literal_type(&t)
            }
            SyntaxKind::OmittedExpression => {
                // Go getTypeOfNode：KindOmittedExpression 属 IsExpressionNode
                //（ast/utilities.go:2223）→ getRegularTypeOfExpression
                //（checker.go:33967）→ getTypeOfExpression → checkExpression
                //（checker.go:8286 undefinedWideningType）；字面量 regular 化
                // 对 undefined no-op。解构赋值目标数组字面量的空洞元素
                // 据此呈 undefined 而非兜底 any
                let t = self.get_type_of_expression(node);
                self.get_regular_type_of_literal_type(&t)
            }
            SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement => {
                // Go getTypeOfNode（checker.go:33806）IsExpressionNode 门：
                // JSX 元素走 getRegularTypeOfExpression → checkExpressionWorker
                // 的 checkJsxElement/checkJsxSelfClosingElement
                self.get_regular_type_of_expression(node)
            }
            _ => self.get_any_type(),
        }
    }

    fn file_symbol_of(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let mut cur = Arc::clone(node);
        loop {
            if cur.kind == SyntaxKind::SourceFile {
                return self.program.symbol_map().symbol_of(&cur).cloned();
            }
            let parent = cur.parent()?;
            cur = parent;
        }
    }

    pub(crate) fn get_type_of_identifier(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_identifier");
        // Go checkIdentifier 前置分支（checker.go:11652-11654）：
        // IsThisInTypeQuery(node) → checkThisExpression(node)——typeof this /
        // typeof this.x 的 this 段按 this 型求值；检查链 check_identifier 已有
        // 同型分支（m1b_2.rs:107-109），getTypeOfNode 拉取链在此对齐，否则按
        // 名字在作用域查 this 落 errorType 渲染 any
        if tsox_frontend::ast::mig::m3g_2::is_this_in_type_query(node) {
            return self.check_this_expression(node);
        }
        // Go getTypeOfNode（checker.go:33806）IsExpressionNode 门：仅表达式
        // 标识符走 getRegularTypeOfExpression（checker.go:33967-33972）——
        // 属性访问/限定名右侧的标识符取宿主表达式类型，并将 fresh 字面量
        // 解包为 regular（O.A.X 的 X 随宿主呈枚举声明型，O.x 的 x 呈 number）；
        // 类型位置限定名链（C.a.B.Id 的 C/a/B，IsPartOfTypeNode 只命中右段顶）
        // 既非表达式也非声明名，落 errorType 渲染 any
        let in_expression =
            crate::checker::mig::m1f::r24k9b_expression_context::is_expression_node(node);
        if in_expression
            && tsox_frontend::ast::mig::m3g_2::is_right_side_of_qualified_name_or_property_access(
                node,
            )
            && let Some(parent) = node.parent()
        {
            let t = self.get_type_of_expression(&parent);
            return self.get_regular_type_of_literal_type(&t);
        }
        // Go getTypeOfNode：限定名左侧标识符非类型节点（IsPartOfTypeNode 只认
        // 右侧标识符，utilities.go:2298-2301）也非表达式（IsExpressionNode
        // KindQualifiedName 分支仅 TypeQuery/JSDoc 引用/JSX 标签名宿主为真，
        // utilities.go:2235-2239），类型引用内（declare let a: A.F 的 A）穿过
        // 全部分支落 errorType；typeof A.B 的 A 仍走表达式路径
        if let Some(parent) = node.parent()
            && parent.kind == SyntaxKind::QualifiedName
            && matches!(&parent.data, tsox_frontend::ast::NodeData::QualifiedName(q) if Arc::ptr_eq(&q.left, node))
        {
            let mut top = parent;
            while let Some(p) = top.parent() {
                if p.kind != SyntaxKind::QualifiedName {
                    break;
                }
                top = p;
            }
            let in_expression_name_position = top.parent().is_some_and(|p| match p.kind {
                SyntaxKind::TypeQuery
                | SyntaxKind::JSDocNameReference
                | SyntaxKind::JSDocLink
                | SyntaxKind::JSDocLinkCode
                | SyntaxKind::JSDocLinkPlain => true,
                SyntaxKind::JsxOpeningElement => matches!(
                    &p.data,
                    tsox_frontend::ast::NodeData::JsxOpeningElement(d)
                        if Arc::ptr_eq(&d.tag_name, &top)
                ),
                SyntaxKind::JsxClosingElement => matches!(
                    &p.data,
                    tsox_frontend::ast::NodeData::JsxClosingElement(d)
                        if Arc::ptr_eq(&d.tag_name, &top)
                ),
                SyntaxKind::JsxSelfClosingElement => matches!(
                    &p.data,
                    tsox_frontend::ast::NodeData::JsxSelfClosingElement(d)
                        if Arc::ptr_eq(&d.tag_name, &top)
                ),
                _ => false,
            });
            if !in_expression_name_position {
                return self.error_type();
            }
        }
        // Go getTypeOfNode：IsTypeDeclarationName → getDeclaredTypeOfSymbol(
        // getSymbolAtLocation)——类/接口/枚举/别名/类型参数的声明名取声明型
        //（类名是实例型 C，而非值路径的构造型 typeof C）；符号经
        // GetSymbolOfNameOrPropertyAccessExpression → getSymbolOfNode
        // （checker.go:33626-33627）取合并符号
        if tsox_frontend::ast::mig::m3g_2::is_type_declaration_name(node) {
            let sym = node
                .parent()
                .and_then(|parent| self.get_symbol_of_declaration_merged(&parent));
            return match sym {
                Some(sym) => self.get_declared_type_of_symbol(&sym),
                None => self.error_type(),
            };
        }
        // Go getTypeOfNode：IsDeclarationNameOrImportPropertyName →
        // getTypeOfSymbol(getSymbolAtLocation(node))——声明名经声明符号取类型
        //（访问器符号走 getTypeOfAccessors 等价路径，成员名不做作用域解析）；
        // 符号经 GetSymbolOfNameOrPropertyAccessExpression → getSymbolOfNode
        //（checker.go:33626-33627、15192-15199）取合并符号——script 文件顶层
        // 声明与 lib 全局符（如 alert）合并后节点侧符号缺 lib 声明集，
        // 未归并则声明名型呈单声明函数型而非聚合过载型
        if tsox_frontend::ast::mig::m3f_4::is_declaration_name_or_import_property_name(node) {
            let resolved = node
                .parent()
                .and_then(|parent| self.get_symbol_of_declaration_merged(&parent));
            let Some(sym) = resolved else {
                return self.error_type();
            };
            // Go getTypeOfSymbol 分派（checker.go:17476-17491）：无任何值位
            //（Function|Class|Enum|ValueModule|Variable|Property 均无，纯类型
            // 命名空间）的符号无对应分支落 anyType 渲染 any；export= 符号
            //（Property 位，bindCommonJSTypeExports 后兼 NamespaceModule）
            // 有值意义，按别名符号型正常取值
            let effective = if sym.flags.intersects(SymbolFlags::Alias) {
                self.resolve_alias_base(Arc::clone(&sym))
            } else {
                Arc::clone(&sym)
            };
            if effective.flags.contains(SymbolFlags::NamespaceModule)
                && !effective.flags.intersects(SymbolFlags::VALUE)
            {
                return self.error_type();
            }
            return self.get_type_of_symbol(&sym);
        }
        // Go getTypeOfNode（checker.go:33867-33876）：import/export 赋值右侧实体名
        // 经 GetSymbolAtLocation（checker.go:33463-33465）→
        // GetSymbolOfNameOrPropertyAccessExpression（checker.go:33624-33646）取符——
        // 整名按 NAMESPACE、限定名按 VALUE|TYPE|NAMESPACE 意义且 dontResolveAlias
        // （checker.go:15272-15291），符号取不到即落 errorType（checker.go:33886）。
        // 此前此处按全含义名字解析（resolve_identifier），import r = undefined 的
        // undefined 以 VALUE 含义命中全局 undefined 符号呈 undefined，而 Go 命名
        // 空间含义过滤后取不到渲染 any
        if crate::checker::utilities_is_private_within_ambient::is_in_right_side_of_import_or_export_assignment(node)
            && let Some(symbol) = self.get_symbol_of_name_or_property_access_expression(node)
        {
            let declared = self.get_declared_type_of_symbol(&symbol);
            if !self.is_error_type(&declared) {
                return declared;
            }
            return self.get_type_of_symbol(&symbol);
        }
        let resolved_symbol = if in_expression {
            self.resolve_identifier(node)
        } else {
            None
        };
        if let Some(symbol) = resolved_symbol {
            // Go checkIdentifier（checker.go:11669）：解析出的本地符号先经
            // getExportSymbolOfValueSymbolIfExported 映射——declareModuleMember
            // 的 default 导出 local（ExportValue 位，export_symbol 链接合并
            // default 导出符号）按导出符号取型（new C() 取构造型而非 any）
            let symbol = self.get_export_symbol_of_value_symbol_if_exported(&symbol);
            let module_without_value = if symbol.flags.intersects(SymbolFlags::Alias) {
                let effective = self.resolve_alias_base(Arc::clone(&symbol));
                effective.flags.contains(SymbolFlags::NamespaceModule)
                    && !effective.flags.intersects(SymbolFlags::VALUE)
            } else {
                symbol.flags.contains(SymbolFlags::NamespaceModule)
                    && !symbol.flags.intersects(SymbolFlags::VALUE)
            };
            if module_without_value {
                return self.error_type();
            }
            let declared = if symbol.flags == SymbolFlags::Alias {
                match self.type_of_imported_symbol(&symbol) {
                    Some(t) => t,
                    None => self.get_type_of_symbol(&symbol),
                }
            } else {
                self.get_type_of_symbol(&symbol)
            };
            // Go checkIdentifier（checker.go:11685-11713）：赋值目标位符号非变量
            //（JS 文件 value module 除外）或只读（const/readonly/getter/枚举成员）
            // 时不进入收窄与字面量放宽，返回 errorType 渲染 any；错误由诊断遍报告
            let target_kind = get_assignment_target_kind(node);
            if target_kind != AssignmentKind::None {
                let value_module_assignment = tsox_frontend::ast::is_in_js_file(node)
                    && symbol.flags.contains(SymbolFlags::ValueModule);
                if !symbol.flags.intersects(SymbolFlags::VARIABLE) && !value_module_assignment {
                    return self.error_type();
                }
                if self.is_readonly_symbol(&symbol) {
                    return self.error_type();
                }
            }
            let compound_like =
                target_kind == AssignmentKind::Definite && is_in_compound_like_assignment(node);
            let narrowed =
                if symbol.flags.intersects(
                    SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::BlockScopedVariable
                        | SymbolFlags::Alias,
                ) && target_kind != AssignmentKind::Definite
                {
                    let flow = self.program.symbol_map().flow_node_of(node).map(Arc::clone);
                    let narrowable = self.get_narrowable_type_for_reference(&declared, node);
                    // Go checkIdentifier initialType 门（checker.go:11742-11777）：
                    // auto/autoArray declared 的读引用在 !assumeInitialized 时
                    // initialType 为 undefinedType，流图回到 FlowStart 取 initial
                    // 呈 undefined 而非 declared(auto) 渲染 any
                    let initial_override = self
                        .reference_read_initial_type_override(
                            node,
                            &symbol,
                            &narrowable,
                            flow.as_ref(),
                        )
                        .or_else(|| {
                            self.check_identifier_parameter_initial_type(&symbol, &narrowable)
                        });
                    self.get_narrowed_type_of_symbol_with_declared(
                        &symbol,
                        flow.as_ref(),
                        narrowable,
                        Some(node),
                        initial_override,
                    )
                } else {
                    declared
                };

            if narrowed.object_flags.contains(ObjectFlags::EvolvingArray)
                && self.is_evolving_array_operation_target(node)
            {
                return self.auto_array_type();
            }

            let final_type = self.finalize_evolving_array_type(&narrowed);

            if target_kind == AssignmentKind::None
                && let Some(declared) = self.definite_assignment_violation_type(node, &symbol)
            {
                return declared;
            }
            if compound_like || target_kind == AssignmentKind::Compound {
                let base = self.get_base_type_of_literal_type(&final_type);
                return self.get_regular_type_of_literal_type(&base);
            }
            // Go getRegularTypeOfExpression（checker.go:33967-33972）：getTypeOfNode
            // 对表达式标识符取 getRegularTypeOfLiteralType(getTypeOfExpression)
            //（26782-26786 fresh 字面量解包 regularType）——fresh 枚举成员型
            // E.a 解包为 regular 后与枚举声明型同一对象，渲染按声明型名 E 呈现
            //（nodebuilderimpl.go:3456-3458 身份判定；声明名行不经此解包，仍呈
            // 成员限定名 E.a）
            self.get_regular_type_of_literal_type(&final_type)
        } else {
            self.error_type()
        }
    }

    pub(crate) fn is_evolving_array_operation_target(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_evolving_array_operation_target"); 
        let root = self.get_reference_root(node);
        let Some(parent) = root.parent() else {
            return false;
        };

        if let NodeData::PropertyAccessExpression(pa) = &parent.data {
            if Arc::ptr_eq(&pa.expression, &root) {
                let name = pa.name.text();
                if name == "length" {
                    return true;
                }
                if name == "push" || name == "unshift" {
                    if let Some(grandparent) = parent.parent() {
                        if matches!(grandparent.kind, SyntaxKind::CallExpression) {
                            return true;
                        }
                    }
                }
            }
        }

        if let NodeData::ElementAccessExpression(ea) = &parent.data {
            if Arc::ptr_eq(&ea.expression, &root) {
                if let Some(grandparent) = parent.parent() {
                    if let NodeData::BinaryExpression(bin) = &grandparent.data {
                        if bin.operator_token.kind == SyntaxKind::EqualsToken
                            && Arc::ptr_eq(&bin.left, &parent)
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub(crate) fn get_reference_root(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_reference_root"); 
        let Some(parent) = node.parent() else {
            return Arc::clone(node);
        };
        let recurse = match &parent.data {
            NodeData::ParenthesizedExpression(_) => true,
            NodeData::BinaryExpression(bin) => {
                (bin.operator_token.kind == SyntaxKind::EqualsToken && Arc::ptr_eq(&bin.left, node))
                    || (bin.operator_token.kind == SyntaxKind::CommaToken
                        && Arc::ptr_eq(&bin.right, node))
            }
            _ => false,
        };
        if recurse {
            self.get_reference_root(&parent)
        } else {
            Arc::clone(node)
        }
    }
}

impl Checker {
    // Go removeSubtypes 的受限版：structured/instantiable 成员若可赋给另一成员则移除
    pub(crate) fn remove_subtype_redundant_members(&mut self, types: Vec<Arc<Type>>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("remove_subtype_redundant_members"); 
        if types.len() < 2 {
            return types;
        }
        // Go removeSubtypes：从后往前遍历，位置靠前的成员在互相可赋时优先保留
        let mut keep: Vec<bool> = vec![true; types.len()];
        for i in (0..types.len()).rev() {
            if !keep[i] {
                continue;
            }
            let source = &types[i];
            // 非严格模式下 widening null/undefined 可赋给任何类型，参与缩减
            let nullable_source = !self.strict_null_checks
                && source.flags.intersects(TypeFlags::Null | TypeFlags::Undefined);
            if !nullable_source
                && !source.flags.intersects(
                    TypeFlags::Object
                        | TypeFlags::Union
                        | TypeFlags::Intersection
                        | TypeFlags::TypeParameter,
                )
            {
                continue;
            }
            for (j, target) in types.iter().enumerate() {
                if i == j || !keep[j] {
                    continue;
                }
                if Arc::ptr_eq(source, target) {
                    continue;
                }
                // Go requireOptionalProperties 排除元组/数组源：其必需成员
                // （数字索引名）不因空对象成员参与子型缩减，保住联合有序性
                if (self.is_array_type(source) || self.is_tuple_type(source))
                    && !self.is_array_type(target)
                    && !self.is_tuple_type(target)
                    && self.is_empty_object_type(target)
                {
                    continue;
                }
                // Go removeSubtypes：strictSubtypeRelation 判缩减（assignable 会
                // 把 {a} 对 {} 误判为可缩减，fresh {} 目标须挡住）
                if self.is_type_strict_subtype_of(source, target) {
                    keep[i] = false;
                    break;
                }
            }
        }
        let result: Vec<Arc<Type>> = types
            .into_iter()
            .zip(keep)
            .filter(|(_, k)| *k)
            .map(|(t, _)| t)
            .collect();
        result
    }
}

/// Go 侧 class implements 元素的限定名链各段位于 TypeReferenceNode 的
/// QualifiedName 内部（parseTypeHeritageClauseElement，parser.go:1957-1989）；
/// 本侧同位置为 EWT.expression 的属性访问链（EWT 解析形态）。判定节点是否
/// 处于该链上：沿属性访问 expression/name 与限定名 left/right 上攀，抵达
/// class implements 元素 EWT 的 expression 位为真；type_arguments 子树
/// （Go 里仍属类型节点）与 extends 链不命中
fn is_in_class_implements_heritage_name_chain(node: &Arc<Node>) -> bool {
    let mut cur = Arc::clone(node);
    loop {
        let Some(parent) = cur.parent() else {
            return false;
        };
        match &parent.data {
            NodeData::PropertyAccessExpression(pa)
                if Arc::ptr_eq(&pa.expression, &cur) || Arc::ptr_eq(&pa.name, &cur) =>
            {
                cur = Arc::clone(&parent);
            }
            NodeData::QualifiedName(qn)
                if Arc::ptr_eq(&qn.left, &cur) || Arc::ptr_eq(&qn.right, &cur) =>
            {
                cur = Arc::clone(&parent);
            }
            NodeData::ExpressionWithTypeArguments(ewt) if Arc::ptr_eq(&ewt.expression, &cur) => {
                let Some(heritage) = parent.parent() else {
                    return false;
                };
                let implements = matches!(
                    &heritage.data,
                    NodeData::HeritageClause(d) if d.token == SyntaxKind::ImplementsKeyword
                );
                if !implements {
                    return false;
                }
                return heritage
                    .parent()
                    .is_some_and(|g| tsox_frontend::ast::is_class_like(g.as_ref()));
            }
            _ => return false,
        }
    }
}
