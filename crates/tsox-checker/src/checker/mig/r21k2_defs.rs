#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol, SyntaxKind};

pub fn is_entity_name(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_entity_name"); 
    matches!(
        node.kind,
        SyntaxKind::Identifier | SyntaxKind::QualifiedName
    )
}

pub fn get_class_like_declaration_of_symbol(symbol: &Symbol) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_class_like_declaration_of_symbol"); 
    for declaration in &symbol.declarations {
        if matches!(
            declaration.kind,
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
        ) {
            return Some(Arc::clone(declaration));
        }
    }
    None
}

pub fn get_host_signature_from_jsdoc(name: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_host_signature_from_jsdoc"); 
    let mut current = name.parent();
    while let Some(node) = current {
        match node.kind {
            SyntaxKind::QualifiedName | SyntaxKind::PropertyAccessExpression => {
                current = node.parent();
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature => return Some(node),
            _ => return None,
        }
    }
    None
}

impl Checker {
    pub fn resolve_external_module(
        &mut self,
        location: &Arc<Node>,
        module_name: &str,
        module_not_found_error: Option<&'static tsox_core::diagnostics::Message>,
        error_node: Option<&Arc<Node>>,
        is_for_augmentation: bool,
        import_attributes_type: Option<&Arc<Type>>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_external_module");
        // Go resolveExternalModule（checker.go:16292-16295）：@types/ 前缀导入
        // 追加 TS6260 提示后照常解析
        if let Some(error_node) = error_node
            && module_name.starts_with("@types/")
        {
            let without_prefix = &module_name["@types/".len()..];
            self.error_message(
                error_node,
                tsox_core::diagnostics::messages_generated::
                    CANNOT_IMPORT_TYPE_DECLARATION_FILES_CONSIDER_IMPORTING_0_INSTEAD_OF_1,
                &[without_prefix.to_string(), module_name.to_string()],
            );
        }
        // Go（checker.go:16296-16298）：缺省属性型视同空对象型
        let import_attributes_type = import_attributes_type
            .cloned()
            .unwrap_or_else(|| self.empty_object_type());
        // Go（checker.go:16300-16303）：exact 名 ambient 模块先于程序解析，
        // 且经 pattern/属性过滤（tryResolvePatternAmbientModule 对空属性型
        // 原样返回已解析符号，非 pattern 用例行为不变）
        if let Some(ambient) = self.try_find_ambient_module(module_name) {
            let resolved =
                self.try_resolve_pattern_ambient_module(Some(ambient), module_name, &import_attributes_type);
            return self.finish_external_module_resolution(resolved, is_for_augmentation);
        }
        let specifier = super::r19k8_defs::new_string_literal(module_name);
        // Go resolveExternalModule（checker.go:16291）经程序解析结果定位包
        // 入口；合成说明符节点无文件祖先，程序解析兜底的文件上下文取调用方
        // location（快速索引走查命中不了 package.json typings 指向的
        // dist/index.d.ts 形态入口）
        if let Some(symbol) = self
            .resolve_external_module_name(&specifier)
            .or_else(|| self.resolve_module_file_by_program_resolution(location, module_name))
        {
            let merged = self.get_merged_symbol(&symbol);
            let resolved =
                self.try_resolve_pattern_ambient_module(Some(merged), module_name, &import_attributes_type);
            return self.finish_external_module_resolution(resolved, is_for_augmentation);
        }
        // Go（checker.go:16229-16235）：程序解析未命中时 pattern ambient 模块
        // 兜底先于 TS2307 报错
        if let Some(pattern) =
            self.try_resolve_pattern_ambient_module(None, module_name, &import_attributes_type)
        {
            return self.finish_external_module_resolution(Some(pattern), is_for_augmentation);
        }
        if let Some(message) = module_not_found_error {
            let location = error_node.unwrap_or(location);
            // 模块未找到报错携带 moduleReference 实参
            self.error_message(location, *message, &[module_name.to_string()]);
        }
        None
    }

    fn finish_external_module_resolution(
        &mut self,
        symbol: Option<Arc<Symbol>>,
        is_for_augmentation: bool,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("finish_external_module_resolution");
        let Some(symbol) = symbol else {
            return None;
        };
        let resolved = self.resolve_external_module_symbol(&symbol, false);
        if is_for_augmentation && Arc::ptr_eq(&resolved, &self.unknown_symbol()) {
            return Some(symbol);
        }
        Some(resolved)
    }

    pub fn silent_never_signature(&mut self) -> Arc<Signature> { ::tsox_core::fntrace::enter("silent_never_signature"); 
        let silent_never = self.silent_never_type();
        self.new_signature(
            SignatureFlags::None,
            None,
            &[],
            None,
            &[],
            &silent_never,
            None,
            0,
        )
    }

    pub fn any_base_type_index_info(&mut self) -> Arc<IndexInfo> { ::tsox_core::fntrace::enter("any_base_type_index_info"); 
        let value_type = self.any_type();
        self.new_index_info(&self.string_type(), &value_type, false, None, &[])
    }
}
