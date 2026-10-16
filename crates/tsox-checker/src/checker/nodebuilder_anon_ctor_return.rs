#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
use std::sync::Arc;

use crate::checker::checker::Checker;
use crate::checker::types::Type;

impl Checker {
    /// 构造签名返回型的显示：模板态匿名类实例型（默认构造签名的
    /// resolvedReturnType 即声明实例型，is_class_instance_side 成立）呈裸
    /// 书写名 (Anonymous class)——Go 基线同字面量内 new (...): (Anonymous
    /// class) 与 prototype: Mixin1.(Anonymous class) 并存（15 例 mixin 族，
    /// doubleMixinConditionalTypeBaseClassWorks 行9）；实例化态（声明型指针
    /// 不等）与具名类照常走链式渲染
    pub(crate) fn construct_sig_return_string(
        &mut self,
        ret_type: &Arc<Type>,
        flags: TypeFormatFlags,
    ) -> String {
        let anon_class_declared_instance = ret_type
            .symbol
            .as_ref()
            .is_some_and(|s| {
                s.flags.contains(tsox_frontend::ast::SymbolFlags::Class)
                    && s.name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_CLASS
                    && crate::checker::mig::m2f_3::is_class_instance_side(self, ret_type)
            });
        if anon_class_declared_instance {
            return self
                .internal_symbol_written_name(ret_type.symbol.as_ref().unwrap())
                .unwrap_or_else(|| "(Anonymous class)".to_string());
        }
        self.type_to_string_ex(ret_type, flags)
    }
}
