#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use tsox_frontend::ast::is_global_scope_augmentation;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;

impl Checker {
    pub(crate) fn object_literal_to_string(
        &mut self,
        _t: &Arc<Type>,
        structured: &StructuredTypeData,
        flags: TypeFormatFlags,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        for sig in structured.call_signatures() {
            let params: Vec<String> = sig
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    let name = param.name.clone();
                    let param_type = self
                        .signature_instantiated_param_type(sig, i)
                        .unwrap_or_else(|| self.get_type_of_symbol(param));
                    let type_str = self.type_to_string_ex(&param_type, flags);
                    if param
                        .flags
                        .contains(tsox_frontend::ast::SymbolFlags::Optional)
                    {
                        format!("{}?: {}", name, type_str)
                    } else {
                        format!("{}: {}", name, type_str)
                    }
                })
                .collect();
            let ret_type = sig
                .resolved_return_type
                .get()
                .cloned()
                .unwrap_or_else(|| self.any_type());
            let ret_str = self.type_to_string_ex(&ret_type, flags);
            let tp = self.signature_type_param_prefix(sig);
            parts.push(format!("{tp}({}) => {}", params.join(", "), ret_str));
        }

        for sig in structured.construct_signatures() {
            let params: Vec<String> = sig
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    let param_type = self
                        .signature_instantiated_param_type(sig, i)
                        .unwrap_or_else(|| self.get_type_of_symbol(param));
                    format!(
                        "{}: {}",
                        param.name,
                        self.type_to_string_ex(&param_type, flags)
                    )
                })
                .collect();
            let ret_type = sig
                .resolved_return_type
                .get()
                .cloned()
                .unwrap_or_else(|| self.any_type());
            let ret_str = self.type_to_string_ex(&ret_type, flags);
            let tp = self.signature_type_param_prefix(sig);
            parts.push(format!("new {tp}({}) => {}", params.join(", "), ret_str));
        }

        for prop in &structured.properties {
            let name = prop.name.clone();

            let name = if prop.declarations.iter().any(|d| {
                d.name()
                    .is_some_and(|n| n.kind == SyntaxKind::StringLiteral)
            }) {
                format!("\"{name}\"")
            } else {
                name
            };
            let prop_type = self.get_type_of_symbol(prop);
            let type_str = self.type_to_string_ex(&prop_type, flags);
            let readonly = prop
                .check_flags
                .contains(tsox_frontend::ast::CheckFlags::Readonly);
            if prop.flags.contains(SymbolFlags::Optional) {
                let ro = if readonly { "readonly " } else { "" };
                parts.push(format!("{ro}{}?: {}", name, type_str));
            } else if readonly {
                parts.push(format!("readonly {}: {}", name, type_str));
            } else {
                parts.push(format!("{}: {}", name, type_str));
            }
        }

        for info in &structured.index_infos {
            let key_str = info
                .key_type
                .as_ref()
                .map(|k| self.type_to_string_ex(k, flags))
                .unwrap_or_else(|| "string".to_string());
            let val_str = info
                .value_type
                .as_ref()
                .map(|v| self.type_to_string_ex(v, flags))
                .unwrap_or_else(|| "any".to_string());

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

    // Go getSymbolChain 的别名感知限定：命名空间父级链显示时，
    // 遇显示上下文文件内解析到该命名空间的别名用别名名；到 enclosing 文件自身模块不加前缀
    pub(crate) fn namespace_qualifier_of(&mut self, symbol: &Arc<Symbol>) -> Option<String> {
        if std::env::var_os("TSOX_DEBUG_QI").is_some() {
            let p_info = symbol
                .parent
                .clone()
                .map(|p| format!("{}/flags={:?}", p.name, p.flags))
                .unwrap_or_else(|| "NONE".into());
            eprintln!("[nsq] sym={} parent={p_info}", symbol.name);
        }
        let mut parts: Vec<String> = Vec::new();
        let mut cur = symbol
            .parent
            .clone()
            .or_else(|| self.namespace_container_from_declarations(symbol));
        while let Some(ns) = cur {
            if !ns.flags.contains(SymbolFlags::ValueModule) {
                return None;
            }
            // declare global 增强容器不参与限定名
            if ns
                .declarations
                .iter()
                .any(|d| tsox_frontend::ast::is_global_scope_augmentation(d))
            {
                break;
            }
            // hover 在该命名空间声明内：符号就地可访问，无需限定
            if let Some(enclosing) = self.display_enclosing_node.clone()
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
            if let Some(file) = self.display_enclosing_file.clone()
                && let Some(alias_name) = self.alias_name_of_namespace_in_file(&ns, &file)
            {
                parts.insert(0, alias_name);
                return Some(parts.join("."));
            }
            if is_file_module {
                return if parts.is_empty() { None } else { Some(parts.join(".")) };
            }
            parts.insert(0, ns.name.clone());
            cur = ns.parent.clone();
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("."))
        }
    }

    // 类型成员限定名：只收集命名空间段（ValueModule 且非文件模块），不使用 alias 前缀
    // （tsc getSymbolChain：外部模块 root 的链段被跳过；值成员才经 alias 显示）
    pub(crate) fn namespace_only_qualifier_of(&mut self, symbol: &Arc<Symbol>) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();
        let mut cur = symbol.parent.clone();
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
            cur = ns.parent.clone();
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("."))
        }
    }

    // import X = A.B.C 形式别名：解析实体名链
    fn resolve_import_equals_entity(&mut self, alias: &Arc<Symbol>) -> Option<Arc<Symbol>> {
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

    fn module_specifier_of_require(&self, module_reference: &Arc<Node>) -> Option<String> {
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
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        if let Some(locals) = symbol_map.locals.get(&file.node.id())
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

    fn node_within(node: &Arc<Node>, ancestor: &Arc<Node>) -> bool {
        let mut cur = node.parent.clone();
        while let Some(n) = cur {
            if Arc::ptr_eq(&n, ancestor) {
                return true;
            }
            cur = n.parent.clone();
        }
        false
    }

    fn namespace_container_from_declarations(
        &self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        symbol.declarations.first().and_then(|decl| {
            let mut cur = decl.parent.as_ref();
            while let Some(n) = cur {
                match n.kind {
                    SyntaxKind::ModuleDeclaration | SyntaxKind::SourceFile => {
                        return symbol_map.symbol_of(n).map(Arc::clone);
                    }
                    _ => cur = n.parent.as_ref(),
                }
            }
            None
        })
    }

    fn alias_name_of_namespace_in_file(
        &mut self,
        ns: &Arc<Symbol>,
        file: &Arc<tsox_frontend::ast::SourceFile>,
    ) -> Option<String> {
        let symbol_map = self.program.symbol_map();
        let mut tables: Vec<tsox_frontend::ast::SymbolTable> = Vec::new();
        if let Some(locals) = symbol_map.locals.get(&file.node.id()) {
            tables.push(locals.clone());
        }
        if let Some(module_sym) = symbol_map.symbol_of(&file.node) {
            tables.push(module_sym.members.clone());
            tables.push(module_sym.exports.clone());
        }
        for table in tables {
            for (name, sym) in table.iter() {
                if !sym.flags.contains(SymbolFlags::Alias) {
                    continue;
                }
                let target = self
                    .resolve_import_equals_entity(sym)
                    .or_else(|| self.resolve_import_alias_target_symbol(sym));
                if let Some(target) = target {
                    if Arc::ptr_eq(&target, ns) {
                        return Some(sym.name.clone());
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
    ) -> String {
        if sym.flags.contains(SymbolFlags::ENUM) {
            return sym.name.clone();
        }

        let obj_data = match &t.data {
            TypeData::Object(o) => Some(o),
            TypeData::Interface(i) => Some(&i.object),
            _ => None,
        };

        if let Some(obj) = obj_data {
            if !obj.type_arguments.is_empty() {
                let args: Vec<String> = obj
                    .type_arguments
                    .iter()
                    .map(|ty| self.type_to_string_ex(ty, flags))
                    .collect();
                let qualified = self
                    .namespace_qualifier_of(sym)
                    .map(|q| format!("{q}.{}", sym.name))
                    .unwrap_or_else(|| sym.name.clone());
                return format!("{}<{}>", qualified, args.join(", "));
            }
        }

        if sym.flags.contains(SymbolFlags::Class) {
            if let Some(structured) = t.as_structured() {
                if !structured.construct_signatures().is_empty() {
                    return format!("typeof {}", sym.name);
                }
            }
        }

        if sym.flags.contains(SymbolFlags::ValueModule) {
            if sym
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::SourceFile)
            {
                let recorded = self.module_display_specifiers.get(&sym.id()).cloned();
                if let Some(spec) = recorded {
                    return format!("typeof import(\"{spec}\")");
                }
                return format!("typeof import(\"{}\")", module_specifier_of_name(&sym.name));
            }
            for d in &sym.declarations {
                if let NodeData::ModuleDeclaration(md) = &d.data
                    && md.name.kind == SyntaxKind::StringLiteral
                {
                    return format!(
                        "typeof import(\"{}\")",
                        md.name.text().trim_matches(['"', '\''])
                    );
                }
            }
            return format!("typeof {}", sym.name);
        }

        if sym.parent.as_ref().is_some_and(|p| p.flags.contains(SymbolFlags::ValueModule)) {
            return self
                .namespace_qualifier_of(sym)
                .map(|q| format!("{q}.{}", sym.name))
                .unwrap_or_else(|| sym.name.clone());
        }

        sym.name.clone()
    }

    pub(crate) fn needs_parens_in_union(&mut self, t: &Arc<Type>) -> bool {
        if let Some(structured) = t.as_structured() {
            if structured.call_signature_count > 0 && t.symbol.is_none() {
                return true;
            }
        }

        false
    }

    pub(crate) fn needs_parens_as_array_element(&mut self, t: &Arc<Type>) -> bool {
        if t.is_union() || t.is_intersection() {
            return true;
        }
        if matches!(&t.data, TypeData::Conditional(_) | TypeData::Index(_)) {
            return true;
        }
        self.needs_parens_in_union(t)
    }

    pub(crate) fn maybe_parenthesize_array_element_ex(
        &mut self,
        elem: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> String {
        let s = self.type_to_string_ex(elem, flags);
        if self.needs_parens_as_array_element(elem) {
            format!("({})", s)
        } else {
            s
        }
    }

    pub fn type_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        self.type_to_type_node_worker(t)
    }
}
