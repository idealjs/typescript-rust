use std::sync::Arc;

use tsox_frontend::ast::Node;

use crate::checker::checker::*;
use crate::checker::types::LiteralValue;

impl Checker {
    // Go getPropertyTypeForIndexType 尾部（checker.go:28771-28793）：const enum
    // 对象类型被 checker.go:28746 的 !isConstEnumObjectType 挡在 noImplicitAny
    // 错误块之外，fall through 到此按索引类型分派报错
    pub(crate) fn const_enum_element_access_fallthrough(
        &mut self,
        obj_type: &Arc<Type>,
        arg_expr: &Arc<Node>,
        arg_type: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("const_enum_element_access_fallthrough");
        let index_display = self.type_to_string(arg_type);
        if arg_expr.kind != SyntaxKind::BigIntLiteral
            && arg_type
                .flags
                .intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral)
        {
            let value = match arg_type.literal_value() {
                Some(LiteralValue::String(s)) => s.clone(),
                Some(LiteralValue::Number(n)) => n.to_string(),
                _ => String::new(),
            };
            let obj_display = self.type_to_string(obj_type);
            self.emit_index_diagnostic(
                arg_expr.loc,
                tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                vec![value, obj_display],
            );
        } else if arg_type.flags.intersects(TypeFlags::String | TypeFlags::Number) {
            let obj_display = self.type_to_string(obj_type);
            self.emit_index_diagnostic(
                arg_expr.loc,
                tsox_core::diagnostics::messages_generated::
                    TYPE_0_HAS_NO_MATCHING_INDEX_SIGNATURE_FOR_TYPE_1,
                vec![obj_display, index_display],
            );
        } else {
            let type_display = if arg_expr.kind == SyntaxKind::BigIntLiteral {
                "bigint".to_string()
            } else {
                index_display
            };
            self.emit_index_diagnostic(
                arg_expr.loc,
                tsox_core::diagnostics::messages_generated::TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                vec![type_display],
            );
        }
        if arg_type.flags.intersects(TypeFlags::Any) {
            return Arc::clone(arg_type);
        }
        self.error_type()
    }
}
