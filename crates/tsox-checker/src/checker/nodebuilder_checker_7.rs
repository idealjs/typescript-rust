#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;

impl Checker {
    pub fn type_to_string(&mut self, t: &Arc<Type>) -> String { ::tsox_core::fntrace::enter("type_to_string");
        // Go TypeToString（printer.go:49-57）：旗标 = AllowUniqueESSymbolType |
        // UseAliasDefinedOutsideCurrentScope；后者使 symbolToTypeNode 的
        // yieldModuleSymbol 为 false，错误消息中符号链外模块根段被剔除
        //（同文件模块局部型呈裸名）
        let saved_yield = self.display_yield_module_symbol;
        self.display_yield_module_symbol = false;
        let result = self.type_to_string_ex(
            t,
            TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE
                .union(TypeFormatFlags::NO_TRUNCATION)
                .union(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE),
        );
        self.display_yield_module_symbol = saved_yield;
        result
    }

    // Go TypeToStringEx(t, enclosingDeclaration, flags)（printer.go:64-102）：
    // enclosingDeclaration 经 nodeBuilder.TypeToTypeNode 传入，限定名按其判定
    // （nodebuilderimpl.go:1110/1127-1135）；Rust 以 type_render_enclosing
    // 承载，仅 namespace_qualifier_of 的命名空间内就地判定消费
    pub fn type_to_string_ex_with_enclosing(
        &mut self,
        t: &Arc<Type>,
        enclosing_declaration: Option<&Arc<tsox_frontend::ast::Node>>,
        flags: TypeFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("type_to_string_ex_with_enclosing");
        // Go .types typeWriter 短路臂（type_symbol_baseline.go:394-404）：
        // 无错误基线时 Any 型打印 IntrinsicName（errorType 呈 "error"），越过
        // nodebuilder 的 Any 旗标 any 臂；本函数是 .types 基线通道唯一入口
        if let Some(intrinsic) = self.type_writer_intrinsic_name(t) {
            return intrinsic;
        }
        let saved = self.type_render_enclosing.take();
        self.type_render_enclosing = enclosing_declaration.cloned();
        // Go .types 基线通道（type_symbol_baseline.go:406-409）ctx.Reset 后经
        // NodeBuilder.TypeToTypeNode 直传 GenerateNamesForShadowedTypeParams；
        // 本函数是该通道在本仓的唯一入口（调用方仅 types_baseline.rs），故在
        // 此并入该位并重置型参命名会话（对应 ctx.Reset）
        let flags = flags.union(TypeFormatFlags::GENERATE_NAMES_FOR_SHADOWED_TYPE_PARAMS);
        let saved_shadowed = self.display_generate_shadowed_names;
        self.display_generate_shadowed_names = true;
        self.display_type_parameter_names.clear();
        self.display_type_param_names_by_text.clear();
        self.display_type_param_next_name_count.clear();
        let result = self.type_to_string_ex(t, flags);
        self.display_generate_shadowed_names = saved_shadowed;
        self.type_render_enclosing = saved;
        result
    }

    /// Go cloneNodeBuilderContext（nodebuilderscopes.go:10-33，经
    /// enterSignatureScope/enterNewScope 挂到每个签名显示）：签名级命名
    /// 作用域——作用域内新增的型参名（typeParameterNames/namesByText/
    /// nextNameCount 的 CopyOnWrite EnterScope 语义）随 cleanup 丢弃，
    /// 兄弟签名各自独立取母名，联合成员重名型参互不改名
    pub(crate) fn push_display_naming_scope(&self) -> DisplayNamingScope {
        DisplayNamingScope {
            type_parameter_names: self.display_type_parameter_names.clone(),
            names_by_text: self.display_type_param_names_by_text.clone(),
            next_name_count: self.display_type_param_next_name_count.clone(),
        }
    }

    pub(crate) fn pop_display_naming_scope(&mut self, scope: DisplayNamingScope) {
        self.display_type_parameter_names = scope.type_parameter_names;
        self.display_type_param_names_by_text = scope.names_by_text;
        self.display_type_param_next_name_count = scope.next_name_count;
    }

    pub fn type_to_string_ex(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String { ::tsox_core::fntrace::enter("type_to_string_ex");
        let reduced = self.get_reduced_type(t);
        let t: &Arc<Type> = &reduced;
        let key = Arc::as_ptr(t) as usize;
        if self.type_print_stack.len() >= 300 || self.type_print_stack.contains(&key) {
            if self.type_print_stack.contains(&key)
                && let Some(s) = self.self_referential_fn_symbol_to_string(t)
            {
                return s;
            }
            // Go createElidedInformationPlaceholder（nodebuilderimpl.go:354-361）：
            // 环复现截断占位按 NoTruncation 分流——.types 基线通道呈 any，
            // 截断通道呈 ...
            if self.type_print_stack.contains(&key)
                && flags.contains(TypeFormatFlags::NO_TRUNCATION)
            {
                return "any".to_string();
            }
            return "...".to_string();
        }
        if self.serialization_level >= MAX_SERIALIZATION_LEVEL {
            return "?".to_string();
        }
        self.type_print_stack.push(key);
        let outermost = self.type_print_stack.len() == 1;
        let saved_length = self.display_approximate_length;
        let saved_truncating = self.display_truncating;
        // Go 显示入口带 enclosingDeclaration（.types 基线 harness 传 node.Parent，
        // type_symbol_baseline.go:409）；无 hover/relater 显示上下文时以上一次
        // 类型查询节点的父声明补位，限定名可及性判定随上下文收敛
        let saved_display_enclosing_node = if outermost
            && self.display_enclosing_node.is_none()
            && self.display_enclosing_file.is_none()
        {
            let from_query = self
                .last_type_query_node
                .as_ref()
                .and_then(|n| n.parent());
            from_query.map(|p| self.display_enclosing_node.replace(p))
        } else {
            None
        };
        if outermost {
            self.display_approximate_length = 0;
            self.display_truncating = false;
        }
        let result = self.type_to_string_ex_worker(t, flags);
        if let Some(saved) = saved_display_enclosing_node {
            self.display_enclosing_node = saved;
        }
        if outermost {
            self.display_approximate_length = saved_length;
            self.display_truncating = saved_truncating;
        }
        self.type_print_stack.pop();
        result
    }

    // Go visitAndTransformType（nodebuilderimpl.go:3307-3375）的身份三分流
    // （3330-3341）：带节点引用按引用节点 id、条件按 root 节点 id、其余按
    // 符号 id；仅在对象展开/联合/条件/带节点引用四个包裹位计数
    fn symbol_depth_identity(t: &Arc<Type>) -> Option<(bool, usize, usize)> {
        let is_constructor_object = t.object_flags.contains(ObjectFlags::ANONYMOUS)
            && t
                .symbol
                .as_ref()
                .is_some_and(|s| s.flags.contains(SymbolFlags::Class));
        if t.object_flags.contains(ObjectFlags::Reference)
            && let Some(node) = t.as_type_reference().and_then(|d| d.node.clone())
        {
            return Some((false, 0, tsox_frontend::ast::mig::m3f::get_node_id(&node) as usize));
        }
        if let TypeData::Conditional(c) = &t.data
            && let Some(node) = c.root.as_ref().and_then(|r| r.node.clone())
        {
            return Some((false, 0, tsox_frontend::ast::mig::m3f::get_node_id(&node) as usize));
        }
        if let Some(sym) = t.symbol.as_ref() {
            return Some((
                is_constructor_object,
                tsox_frontend::ast::mig::m3f::get_symbol_id(sym) as usize,
                0,
            ));
        }
        None
    }

    // Go visitAndTransformType 的进入/退出配平（nodebuilderimpl.go:3343-3372）：
    // depth>10 返回 None 由调用方落 createElidedInformationPlaceholder
    fn with_symbol_depth<R>(
        &mut self,
        t: &Arc<Type>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> Option<R> {
        let identity = Self::symbol_depth_identity(t);
        let saved = identity
            .as_ref()
            .map(|k| *self.type_print_symbol_depth.entry(*k).or_insert(0));
        if saved.is_some_and(|depth| depth > 10) {
            return None;
        }
        if let Some(k) = identity.as_ref() {
            let depth = saved.unwrap_or(0);
            self.type_print_symbol_depth.insert(*k, depth + 1);
        }
        let result = f(self);
        if let (Some(k), Some(depth)) = (identity, saved) {
            self.type_print_symbol_depth.insert(k, depth);
        }
        Some(result)
    }

    // Go createElidedInformationPlaceholder（nodebuilderimpl.go:354-361）：
    // NoTruncation 呈 any（elided 注释被 printer 剔除），截断通道呈 ...
    fn elided_information_placeholder(flags: TypeFormatFlags) -> String {
        if flags.contains(TypeFormatFlags::NO_TRUNCATION) {
            "any".to_string()
        } else {
            "...".to_string()
        }
    }

    pub(crate) fn type_to_string_ex_worker(
        &mut self,
        t: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("type_to_string_ex_worker"); 
        // Go typeToTypeNode（nodebuilderimpl.go:3298）：InTypeAlias 单层消费，
        // 仅别名右值顶层跳过按别名名呈现，嵌套类型恢复常规判定
        let in_type_alias = flags.contains(TypeFormatFlags::IN_TYPE_ALIAS);
        let flags = flags.without(TypeFormatFlags::IN_TYPE_ALIAS);
        if t.flags.intersects(TypeFlags::ANY) {
            // Go typeToTypeNode（nodebuilderimpl.go:3431-3441）：TypeFlagsAny 带
            // 别名时无条件按别名引用呈现（ToTypeReferenceNode→
            // symbolToEntityNameNode 按符号父链限定，nodebuilderimpl.go:671-679，
            // 无可及性门），错误型挂未解析别名符号即呈限定名（A.B）；无别名时
            // error/任意 intrinsic 关键字照旧
            if let Some(alias) = &t.alias {
                if let Some(sym) = &alias.symbol {
                    let args: Vec<String> = alias
                        .type_arguments
                        .iter()
                        .map(|a| self.type_to_string_ex(a, flags))
                        .collect();
                    if args.len() == 1 && sym.name == "Array" {
                        let elem = self
                            .maybe_parenthesize_array_element_ex(&alias.type_arguments[0], flags);
                        return format!("{elem}[]");
                    }
                    let mut name = sym.name.clone();
                    let mut cur = sym.parent();
                    while let Some(p) = cur {
                        name = format!("{}.{}", p.name, name);
                        cur = p.parent();
                    }
                    if args.is_empty() {
                        return name;
                    }
                    return format!("{}<{}>", name, args.join(", "));
                }
                return "any".to_string();
            }
            if let Some(name) = t.intrinsic_name() {
                if name == "error" {
                    return "any".to_string();
                }
                return name.to_string();
            }
            return "any".to_string();
        }
        if let Some(name) = t.intrinsic_name() {
            // Go nodebuilderimpl.go:3422-3430：ANY 位先查别名（ToTypeReferenceNode
            // → symbolToEntityNameNode 按符号链限定名），unresolved 引用的
            // errorType 由此呈现为实体名链；无别名才落内在名/error→any
            if t.flags.contains(TypeFlags::Any) {
                if let Some(sym) = t.alias.as_ref().and_then(|a| a.symbol.clone()) {
                    let args: Vec<String> = t
                        .alias
                        .as_ref()
                        .map(|a| {
                            a.type_arguments
                                .iter()
                                .map(|arg| self.type_to_string_ex(arg, flags))
                                .collect()
                        })
                        .unwrap_or_default();
                    let base = crate::checker::mig::m1f_3::get_symbol_path(&sym);
                    return if args.is_empty() {
                        base
                    } else {
                        format!("{}<{}>", base, args.join(", "))
                    };
                }
            }
            if name == "error" {
                return "any".to_string();
            }
            return name.to_string();
        }

        // Go typeToTypeNodeHelper（nodebuilderimpl.go:3447-3450）：TypeFlagsBoolean
        // 且无别名的类型按 boolean 关键字呈现——false/true 双字面量联合的置旗
        // 产物（checker.go:27291-27292）与内建 boolean 单例都经此渲染，联合
        // 处理臂之前生效；.types 基线通道此前缺此臂，置旗联合仍按成员列表展开
        if t.flags.contains(TypeFlags::Boolean) && t.alias.is_none() {
            return "boolean".to_string();
        }

        if t.flags.intersects(TypeFlags::ENUM_LIKE)
            && !t.is_union()
            && let Some(sym) = &t.symbol
            && sym
                .flags
                .intersects(tsox_frontend::ast::SymbolFlags::EnumMember)
            && let Some(parent) = sym.parent()
        {
            // Go nodebuilderimpl.go:3453-3456：父名经 symbolToTypeNode 取可及链
            //（SymbolFlagsType，链头为 import 别名时按别名渲染，Outer.A → O.A）；
            // 字面量型正是枚举符号的声明型时按父名呈现
            let parent_name = self
                .accessible_chain_full_name(&parent, SymbolFlags::TYPE)
                .unwrap_or_else(|| self.namespace_qualified_name(&parent, SymbolFlags::TYPE));
            let declared = self.get_declared_type_of_symbol(&parent);
            if Arc::ptr_eq(&declared, t) {
                return parent_name;
            }
            // Go nodebuilderimpl.go:3459-3471：标识符成员名走父名限定引用
            //（E.A），非标识符名（字符串名/缺名成员）按索引访问呈现
            // (typeof E)["name"]，字符串字面量经非 ASCII 转义。Go 内部名前缀
            // 为原始 0xFE 字节，printer escapeStringWorker 经 utf8 解码为
            // RuneError 后转义输出 \uFFFD；本仓以合法字符 U+00FE 承载该前缀，
            // 展示侧先映射回 U+FFFD 再转义
            let display_name = sym.name.replace('\u{FE}', "\u{FFFD}");
            if crate::checker::checker_get_excluded_symbol_flags::is_valid_identifier_text(
                &display_name,
            ) {
                return format!("{}.{}", parent_name, display_name);
            }
            let escaped = tsox_frontend::format::mig::m4t_2::escape_non_ascii_string(
                &display_name,
                tsox_frontend::format::mig::m4t_2::QuoteChar::DoubleQuote,
            );
            return format!("(typeof {})[\"{}\"]", parent_name, escaped);
        }

        if t.flags.intersects(TypeFlags::Enum | TypeFlags::EnumLiteral)
            && !t.is_union()
            && let Some(sym) = &t.symbol
            && sym.flags.intersects(tsox_frontend::ast::SymbolFlags::ENUM)
            && !sym.flags.intersects(tsox_frontend::ast::SymbolFlags::EnumMember)
        {
            // Go typeToTypeNodeWorker EnumLike 非成员分支
            // （nodebuilderimpl.go:3473-3474）：非联合枚举声明型（空枚举
            // createComputedEnumType，字面量值为 None）按枚举名呈现，
            // 不落字面量值分支
            return self
                .accessible_chain_full_name(sym, tsox_frontend::ast::SymbolFlags::VALUE)
                .unwrap_or_else(|| {
                    self.namespace_qualified_name(sym, tsox_frontend::ast::SymbolFlags::VALUE)
                });
        }

        if let Some(val) = t.literal_value() {
            return self.literal_value_to_string(val);
        }

        if t.flags.contains(TypeFlags::UniqueESSymbol) {
            if let TypeData::UniqueESSymbol(sym) = &t.data {
                if flags.contains(TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE) {
                    return format!("unique symbol");
                }
                return format!("typeof {}", sym.name);
            }
        }

        if t.flags.contains(TypeFlags::Never) {
            return "never".to_string();
        }

        if t.is_union()
            && let Some(sym) = &t.symbol
            && sym.flags.intersects(tsox_frontend::ast::SymbolFlags::ENUM)
        {
            // Go typeToTypeNode EnumLike 非成员分支（nodebuilderimpl.go:3473-3474）
            // → symbolToTypeNode(SymbolFlagsType)：枚举名从渲染上下文取可及链
            //（getSymbolChain）；链不可用时限定名回退与 symbol_type_to_string 的
            // typeof 通道同名源（namespace_qualified_name）——跨命名空间引用
            //（m1.e3）随符号父容器带段，容器内/hover 就地呈裸名
            return self
                .accessible_chain_full_name(sym, SymbolFlags::TYPE)
                .unwrap_or_else(|| self.namespace_qualified_name(sym, SymbolFlags::TYPE));
        }

        // Go typeToTypeNode：alias 可达时先于 union/intersection/类型参数/类接口
        // 按别名引用呈现（命名空间限定 + 类型实参；Array 单实参写 []）。
        // 门控按 Go nodebuilderimpl.go:3552 全条件：inTypeAlias==0 且
        // （UseAliasDefinedOutsideCurrentScope 或
        // IsTypeSymbolAccessible(alias.Symbol, enclosingDeclaration)）。
        // InTypeAlias 置位一律跳过按目标展开（hover 随
        // UseAliasDefinedOutsideCurrentScope 同传、.types 基线重渲染单传
        // InTypeAlias，两处均走展开）；无 UseAliasDefinedOutsideCurrentScope
        // 时别名名呈现以别名符号对 enclosingDeclaration 的可及性为前提，
        // 函数体局部别名（如 callsOnComplexSignatures 的 stringType1）自
        // 别名声明处不可及，展开为底层联合
        let alias_gate_enclosing = self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone());
        // Go typeToTypeNode（nodebuilderimpl.go:3552）别名门裁决一次全函数共用：
        // 门未开时本函数后续任何别名/引用名字形态（Reference 名呈现、别名展开
        // 体呈名）都不得触发，按底层结构展开（createTypeNodeFromObjectType）。
        // Go 实例化别名产物恒为 instantiateAnonymousType 匿名壳
        // （checker.go:23785-23804），门未开即落成员展开
        let alias_gate_open = if in_type_alias {
            true
        } else {
            match &t.alias {
                Some(alias) => match &alias.symbol {
                    Some(sym) => {
                        flags.contains(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE)
                            || self.is_type_symbol_accessible(sym, alias_gate_enclosing.as_ref())
                    }
                    None => true,
                },
                None => true,
            }
        };
        if !in_type_alias
            && let Some(alias) = &t.alias
            && let Some(sym) = &alias.symbol
            && alias_gate_open
        {
            let args: Vec<String> = alias
                .type_arguments
                .iter()
                .map(|a| self.type_to_string_ex(a, flags))
                .collect();
            if args.len() == 1 && sym.name == "Array" {
                let elem =
                    self.maybe_parenthesize_array_element_ex(&alias.type_arguments[0], flags);
                return format!("{elem}[]");
            }
            // Go 别名分支以 symbolToTypeNode(SymbolFlagsType) 呈现
            //（nodebuilderimpl.go:3563）：名字从渲染上下文取可及链
            //（getSymbolChain），跨文件模块成员链根为模块符号，按 import
            // 说明符呈现（index.ts 下 wrapClass.Constructor →
            // import("./wrapClass").Constructor）。链判定以 ctx 内
            // enclosingDeclaration 为前提（lookupSymbolChainWorker
            // nodebuilderimpl.go:1112-1120 无 enclosing 直接链=[符号]）；
            // 本仓错误消息通道无 .types 基线的 node.Parent 上下文
            //（type_render_enclosing 为 None，旧径按 current_file 判定），
            // 该通道维持 namespace_qualifier_of 原行为，仅发射通道走链
            let qualified = if self.type_render_enclosing.is_some() {
                self.symbol_chain_display_name(sym, tsox_frontend::ast::SymbolFlags::TYPE)
                    .or_else(|| {
                        self.namespace_qualifier_of(sym)
                            .map(|q| format!("{q}.{}", sym.name))
                    })
                    .unwrap_or_else(|| sym.name.clone())
            } else {
                self.namespace_qualifier_of(sym)
                    .map(|q| format!("{q}.{}", sym.name))
                    .unwrap_or_else(|| sym.name.clone())
            };
            if args.is_empty() {
                return qualified;
            }
            return format!("{}<{}>", qualified, args.join(", "));
        }

        if !alias_gate_open
            && t.flags.contains(TypeFlags::Object)
            && t.object_flags.contains(ObjectFlags::Mapped)
        {
            // Go 别名门未开的实例化映射壳按结构展开前先解析成员
            //（createTypeNodeFromObjectType → resolveStructuredTypeMembers）：
            // instantiateAnonymousType 建壳为惰性，直读 structured 会落空对象
            self.resolve_mapped_type_members(t);
        }

        if t.is_union() {
            return self.union_to_string(t, flags);
        }

        if t.is_intersection() {
            return self.intersection_to_string(t, flags);
        }

        if t.is_type_parameter() {
            return self.type_parameter_to_string(t);
        }

        if let TypeData::IndexedAccess(ia) = &t.data {
            return self.indexed_access_to_string(ia, flags);
        }

        if let TypeData::TemplateLiteral(tl) = &t.data {
            return self.template_literal_to_string(tl, flags);
        }

        if let TypeData::Index(i) = &t.data {
            let target = i
                .target
                .as_ref()
                .map(|tt| self.type_to_string_ex(tt, flags))
                .unwrap_or_else(|| "any".to_string());
            return format!("keyof {target}");
        }
        if let TypeData::StringMapping(s) = &t.data {
            let target = s
                .target
                .as_ref()
                .map(|tt| self.type_to_string_ex(tt, flags))
                .unwrap_or_else(|| "any".to_string());
            let name = t
                .symbol
                .as_ref()
                .map(|sym| sym.name.clone())
                .unwrap_or_default();
            if name.is_empty() {
                return target;
            }
            return format!("{name}<{target}>");
        }
        if let TypeData::Mapped(_) = &t.data {
            // Go createTypeNodeFromObjectType（nodebuilderimpl.go:2869-2872）：
            // 仅泛型映射型（或含 error）按映射形态呈现；实例化产物先经
            // resolveStructuredTypeMembers（checker.go:20204-20208）Mapped 臂
            // 解析成员，落下方 structured 分支按 type literal 呈现
            if !t.object_flags.contains(ObjectFlags::MembersResolved)
                && !self.is_generic_mapped_type_for_display(t)
            {
                self.resolve_mapped_type_members(t);
            }
        }
        if let TypeData::Mapped(m) = &t.data
            && self.is_generic_mapped_type_for_display(t)
        {
            if !in_type_alias
                && let Some(alias) = &t.alias
                && let Some(sym) = &alias.symbol
            {
                let args: Vec<String> = alias
                    .type_arguments
                    .iter()
                    .map(|a| self.type_to_string_ex(a, flags))
                    .collect();
                if args.is_empty() {
                    return sym.name.clone();
                }
                return format!("{}<{}>", sym.name, args.join(", "));
            }

            let mut decl_tp_name: Option<String> = None;
            let mut decl_constraint: Option<String> = None;
            let mut decl_constraint_is_reference = false;
            if let Some(decl) = m.declaration.as_ref()
                && let tsox_frontend::ast::NodeData::MappedTypeNode(md) = &decl.data
                && let tsox_frontend::ast::NodeData::TypeParameterDeclaration(tpd) =
                    &md.type_parameter.data
            {
                decl_tp_name = Some(tpd.name.text().to_string());
                if let Some(c) = &tpd.constraint {
                    decl_constraint_is_reference = c.kind == SyntaxKind::TypeReference;
                    decl_constraint = self.node_source_text(c);
                }
            }
            let tp = decl_tp_name
                .filter(|n| !n.is_empty())
                .or_else(|| {
                    m.type_parameter
                        .as_ref()
                        .and_then(|tp| tp.symbol.as_ref().map(|s| s.name.clone()))
                })
                .unwrap_or_else(|| "K".to_string());
            let constraint = if Self::is_mapped_type_with_keyof_constraint_declaration(t) {
                let modifiers = self.get_modifiers_type_from_mapped_type(t);
                format!("keyof {}", self.type_to_string_ex(&modifiers, flags))
            } else {
                match self.get_constraint_type_from_mapped_type(t) {
                    Some(c) if !decl_constraint_is_reference => self.type_to_string_ex(&c, flags),
                    _ => decl_constraint.unwrap_or_else(|| "keyof any".to_string()),
                }
            };
            let as_clause = m
                .name_type
                .as_ref()
                .map(|n| format!(" as {}", self.type_to_string_ex(n, flags)))
                .unwrap_or_default();
            let template = match self.get_template_type_from_mapped_type(t) {
                Some(tt) => self.type_to_string_ex(&tt, flags),
                None => "any".to_string(),
            };
            return format!("{{ [{tp} in {constraint}{as_clause}]: {template}; }}");
        }
        if let TypeData::Substitution(sub) = &t.data {
            if let Some(base) = &sub.base_type {
                let base_str = self.type_to_string_ex(base, flags);
                if self.is_no_infer_type(t) {
                    return format!("NoInfer<{base_str}>");
                }
                return base_str;
            }
            if let Some(c) = &sub.constraint {
                return self.type_to_string_ex(c, flags);
            }
        }
        if let TypeData::Conditional(c) = &t.data {
            // Go conditionalTypeToTypeNode：已解析/可解析的条件显示解析值，
            // 别名形态只留给泛型挂起的条件
            // Go 3697 的 visitAndTransformType 计数只覆盖别名名呈现未接住
            // 的条件（别名路由 3552 先返），别名臂命中时不计数
            let alias_arm = t
                .alias
                .as_ref()
                .and_then(|a| a.symbol.as_ref())
                .is_some();
            let body = |s: &mut Self| -> String {
                let resolved = c
                    .resolved_true_type
                    .get()
                    .or_else(|| c.resolved_false_type.get())
                    .cloned()
                    .or_else(|| s.resolve_conditional_type(t));
                if let Some(resolved) = resolved {
                    return s.type_to_string_ex(&resolved, flags);
                }
                if let Some(alias) = &t.alias
                    && let Some(sym) = &alias.symbol
                {
                    let args: Vec<String> = alias
                        .type_arguments
                        .iter()
                        .map(|a| s.type_to_string_ex(a, flags))
                        .collect();
                    if args.is_empty() {
                        return sym.name.clone();
                    }
                    return format!("{}<{}>", sym.name, args.join(", "));
                }
            let root = c.root.as_ref();
            let check = root
                .and_then(|r| r.check_type.clone())
                .or_else(|| c.check_type.clone())
                .map(|ct| {
                    // Go conditionalTypeToTypeNode：函数/构造形态的 check 需要
                    // 括号分组（(...t: T) => void extends ...）
                    let ct_str = s.type_to_string_ex(&ct, flags);
                    if ct.as_structured().is_some_and(|st| !st.signatures.is_empty()) {
                        format!("({ct_str})")
                    } else {
                        ct_str
                    }
                })
                .unwrap_or_else(|| "unknown".to_string());
            let extends = root
                .and_then(|r| r.extends_type.clone())
                .or_else(|| c.extends_type.clone())
                .map(|et| s.type_to_string_ex(&et, flags))
                .unwrap_or_else(|| "unknown".to_string());

            let (cond_node, true_node, false_node) = root
                .and_then(|r| r.node.as_ref())
                .map(|n| match &n.data {
                    tsox_frontend::ast::NodeData::ConditionalTypeNode(d) => (
                        Some(Arc::clone(n)),
                        Some(Arc::clone(&d.true_type)),
                        Some(Arc::clone(&d.false_type)),
                    ),
                    _ => (None, None, None),
                })
                .unwrap_or((None, None, None));
            if let Some(cn) = &cond_node {
                s.push_scope(cn);
            }
            let true_t = c
                .resolved_true_type
                .get()
                .map(|tt| s.type_to_string_ex(tt, flags))
                .or_else(|| {
                    true_node.map(|n| {
                        let t = s.get_type_from_type_node(&n);
                        s.type_to_string_ex(&t, flags)
                    })
                })
                .unwrap_or_else(|| "...".to_string());
            let false_t = c
                .resolved_false_type
                .get()
                .map(|ft| s.type_to_string_ex(ft, flags))
                .or_else(|| {
                    false_node.map(|n| {
                        let t = s.get_type_from_type_node(&n);
                        s.type_to_string_ex(&t, flags)
                    })
                })
                .unwrap_or_else(|| "...".to_string());
            if cond_node.is_some() {
                s.pop_scope();
            }
            format!("{check} extends {extends} ? {true_t} : {false_t}")
            };
            return if alias_arm {
                body(self)
            } else {
                self.with_symbol_depth(t, body)
                    .unwrap_or_else(|| Self::elided_information_placeholder(flags))
            };
        }

        // 泛型函数内声明的类经调用位实例化：f<string>.C 形态
        // （alias=外层函数+实参，symbol=类）
        if let Some(alias) = &t.alias
            && let Some(fn_sym) = &alias.symbol
            && fn_sym.flags.contains(tsox_frontend::ast::SymbolFlags::Function)
            && let Some(class_sym) = &t.symbol
            && class_sym.flags.contains(tsox_frontend::ast::SymbolFlags::Class)
            && !alias.type_arguments.is_empty()
        {
            let args: Vec<String> = alias
                .type_arguments
                .iter()
                .map(|a| self.type_to_string_ex(a, flags))
                .collect();
            return format!("{}<{}>.{}", fn_sym.name, args.join(", "), class_sym.name);
        }

        // Go typeToString：hover flags 带 UseAliasDefinedOutsideCurrentScope 时
        // 优先按别名符号打印（Name<args>）
        if flags.contains(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE)
            && let Some(alias) = &t.alias
            && let Some(sym) = &alias.symbol
        {
            let args: Vec<String> = alias
                .type_arguments
                .iter()
                .map(|a| self.type_to_string_ex(a, flags))
                .collect();
            if args.is_empty() {
                return sym.name.clone();
            }
            return format!("{}<{}>", sym.name, args.join(", "));
        }

        if t.object_flags.contains(ObjectFlags::Tuple) {
            return self.tuple_to_string(t, flags);
        }

        if t.object_flags.contains(ObjectFlags::Reference) {
            // Go 3582 的 visitAndTransformType 计数仅覆盖带节点引用（延迟
            // 引用），且别名名呈现（3552 先返）不计数
            let has_node = t
                .as_type_reference()
                .and_then(|d| d.node.clone())
                .is_some();
            let alias_named = t
                .alias
                .as_ref()
                .and_then(|a| a.symbol.as_ref())
                .is_some();
            if has_node && !alias_named {
                return self
                    .with_symbol_depth(t, |s: &mut Self| s.reference_to_string(t, flags))
                    .unwrap_or_else(|| Self::elided_information_placeholder(flags));
            }
            return self.reference_to_string(t, flags);
        }

        // Go typeToTypeNode（nodebuilderimpl.go:3552）：别名名呈现恒过
        // UseAliasDefinedOutsideCurrentScope 或 IsTypeSymbolAccessible 门；
        // 门未过（全局合并冲突下被顶替的别名符号，如 interface/namespace A
        // 与 type A 跨文件同名）与 in_type_alias 置位（.types 重渲染）一样
        // 落展开形态，不回退按别名名呈现
        if !in_type_alias
            && let Some(alias) = &t.alias
            && let Some(sym) = &alias.symbol
            && (flags.contains(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE)
                || self.is_type_symbol_accessible(sym, alias_gate_enclosing.as_ref()))
        {
            let args: Vec<String> = alias
                .type_arguments
                .iter()
                .map(|a| self.type_to_string_ex(a, flags))
                .collect();
            if args.is_empty() {
                return sym.name.clone();
            }
            return format!("{}<{}>", sym.name, args.join(", "));
        }

        if t.object_flags.contains(ObjectFlags::INSTANTIATION_EXPRESSION_TYPE) {
            // Go createAnonymousTypeNodeEx（nodebuilderimpl.go:2999-3023）：
            // 实例化表达式型优先复用原 typeof 节点——原节点为 TypeQuery 且
            // 解出的型与当前型恒等时按节点呈现（typeof Class<T>），复用不成
            // 再落匿名对象展开（createTypeNodeFromObjectType 回退）
            let existing = match &t.data {
                TypeData::InstantiationExpression(ie) => ie.node.clone(),
                _ => None,
            };
            if let Some(existing) = existing
                && existing.kind == SyntaxKind::TypeQuery
                && Arc::ptr_eq(&self.get_type_from_type_node(&existing), t)
                && let Some(text) = self.node_source_text(&existing)
            {
                return text;
            }
        }

        if let Some(structured) = t.as_structured() {
            if !self.symbol_type_printed_by_name(t)
                && structured.signatures.len() == 1
                && structured.properties.is_empty()
                && structured.index_infos.is_empty()
            {
                // Go 3060：单签名匿名型经 createTypeNodeFromObjectType 在
                // visitAndTransformType 包裹内展开
                return self
                    .with_symbol_depth(t, |s: &mut Self| {
                        s.function_type_to_string(t, structured, flags)
                    })
                    .unwrap_or_else(|| Self::elided_information_placeholder(flags));
            }
        }

        if self.symbol_type_printed_by_name(t)
            && let Some(sym) = &t.symbol
        {
            return self.symbol_type_to_string(t, sym, flags);
        }

        if let Some(structured) = t.as_structured() {
            if !structured.properties.is_empty()
                || !structured.call_signatures().is_empty()
                || !structured.construct_signatures().is_empty()
                || !structured.index_infos.is_empty()
            {
                // Go 3060：匿名对象结构展开经 createTypeNodeFromObjectType 在
                // visitAndTransformType 包裹内计数（到达此位时别名路由已放过）
                return self
                    .with_symbol_depth(t, |s: &mut Self| {
                        s.object_literal_to_string(t, structured, flags)
                    })
                    .unwrap_or_else(|| Self::elided_information_placeholder(flags));
            }
            // 空匿名对象字面量形态（Go createTypeNodeFromObjectType 无成员 TypeLiteral）：
            // 内部名匿名符号（þobject 等）与 TypeLiteral 符号同按 {} 展示
            if t
                .symbol
                .as_ref()
                .is_none_or(|s| s.name.starts_with('\u{FE}') || s.flags.contains(SymbolFlags::TypeLiteral))
            {
                return "{}".to_string();
            }
        }

        if t.flags.contains(TypeFlags::Object) {
            return "object".to_string();
        }
        if t.flags.contains(TypeFlags::Unknown) {
            return "unknown".to_string();
        }

        "<unknown type>".to_string()
    }

    pub(crate) fn literal_value_to_string(&mut self, val: &LiteralValue) -> String { ::tsox_core::fntrace::enter("literal_value_to_string"); 
        match val {
            // Go ValueToString：字符串值经 printer.EscapeString('"') 转义
            LiteralValue::String(s) => {
                format!("\"{}\"", crate::checker::escape_string::escape_string(s, crate::checker::escape_string::QuoteChar::DoubleQuote))
            }
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::BigInt(b) => format!("{}n", b.to_string()),
            LiteralValue::Boolean(true) => "true".to_string(),
            LiteralValue::Boolean(false) => "false".to_string(),
            LiteralValue::None => String::new(),
        }
    }

    pub(crate) fn is_fn_symbol_anonymous_type(&self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_fn_symbol_anonymous_type"); 
        t.object_flags.contains(ObjectFlags::Anonymous)
            && t.symbol.as_ref().is_some_and(|s| {
                s.flags.contains(SymbolFlags::Function)
                    && !s
                        .flags
                        .intersects(SymbolFlags::Class | SymbolFlags::ENUM | SymbolFlags::ValueModule)
            })
    }

    pub(crate) fn symbol_type_printed_by_name(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("symbol_type_printed_by_name");
        let by_name = t.symbol.as_ref().is_some_and(|sym| {
            !sym.flags.contains(SymbolFlags::TypeLiteral)
                && sym.flags.intersects(
                    SymbolFlags::Class
                        | SymbolFlags::ENUM
                        | SymbolFlags::ValueModule
                        | SymbolFlags::Interface,
                )
        });
        if !by_name {
            return false;
        }
        // Go shouldEmitTypeOfSymbol（nodebuilderimpl.go:2982-2991）：基构造型
        // 变量存在的类符号（mixin 形态，getBaseTypeVariableOfClass 非空）
        // 不按名呈现，落 createTypeNodeFromObjectType 结构展开；门只作用于
        // createAnonymousTypeNodeEx 可达的匿名片构型——实例侧（declared type
        // 与 IsClassInstanceClone，isClassInstanceSide）恒按名。判定走节点
        // 直算版（get_base_type_variable_of_class_node）——符号→声明型间接链
        // 在 .types 拉取上下文恒 None（progress_notes.md 三轮行为证据）
        if let Some(sym) = &t.symbol
            && sym.flags.contains(SymbolFlags::Class)
            && !crate::checker::mig::m2f_3::is_class_instance_side(self, t)
        {
            let is_mixin =
                crate::checker::mig::m2c::r21k2_defs::get_class_like_declaration_of_symbol(sym)
                    .and_then(|d| self.get_base_type_variable_of_class_node(&d))
                    .is_some();
            if is_mixin {
                return false;
            }
        }
        true
    }

    fn self_referential_fn_symbol_to_string(&mut self, t: &Arc<Type>) -> Option<String> { ::tsox_core::fntrace::enter("self_referential_fn_symbol_to_string");
        if !self.is_fn_symbol_anonymous_type(t) {
            return None;
        }
        let mut symbol = t.symbol.as_ref()?.clone();
        if !symbol.flags.contains(SymbolFlags::Function) {
            return None;
        }
        // Go shouldWriteTypeOfFunctionSymbol（nodebuilderimpl.go:2944-2968）：
        // 顶层变量语句直挂的具名函数表达式同为 non-local（isFunctionExpressionSymbol）
        let mut is_function_expression_symbol = false;
        let mut is_non_local = symbol.parent().is_some();
        if !is_non_local {
            for declaration in &symbol.declarations {
                let Some(parent) = declaration.parent() else {
                    continue;
                };
                if parent.kind == SyntaxKind::SourceFile || parent.kind == SyntaxKind::ModuleBlock {
                    is_non_local = true;
                    break;
                }
                let grand = parent.parent();
                let great = grand.as_ref().and_then(|g| g.parent());
                let great2 = great.as_ref().and_then(|g| g.parent());
                if tsox_frontend::ast::is_function_expression_or_arrow_function(declaration)
                    && parent.kind == SyntaxKind::VariableDeclaration
                    && grand.as_ref().is_some_and(|g| g.kind == SyntaxKind::VariableDeclarationList)
                    && great.as_ref().is_some_and(|g| g.kind == SyntaxKind::VariableStatement)
                    && great2.as_ref().is_some_and(|g| {
                        g.kind == SyntaxKind::SourceFile || g.kind == SyntaxKind::ModuleBlock
                    })
                {
                    is_non_local = true;
                    is_function_expression_symbol = true;
                    break;
                }
            }
        }
        if !is_non_local {
            return None;
        }
        if is_function_expression_symbol {
            // Go nodebuilderimpl.go:2971-2974：ValueDeclaration.Parent !=
            // enclosingDeclaration 时重绑 getMergedSymbol(ValueDeclaration.Parent.Symbol())
            if let Some(value_declaration) = symbol.value_declaration.as_ref()
                && let Some(parent) = value_declaration.parent()
            {
                let enclosing_same = self
                    .type_render_enclosing
                    .as_ref()
                    .or(self.display_enclosing_node.as_ref())
                    .is_some_and(|e| Arc::ptr_eq(e, &parent));
                if !enclosing_same {
                    if let Some(parent_symbol) = self.get_symbol_of_declaration(&parent) {
                        symbol = self.get_merged_symbol(&parent_symbol);
                    }
                }
            }
        }
        Some(format!("typeof {}", symbol.name))
    }
}

pub(crate) struct DisplayNamingScope {
    type_parameter_names: std::collections::HashMap<u32, String>,
    names_by_text: std::collections::HashSet<String>,
    next_name_count: std::collections::HashMap<String, usize>,
}
