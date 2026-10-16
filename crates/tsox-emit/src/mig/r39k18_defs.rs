use std::sync::Arc;

use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_frontend::ast::node_source_file::FileReference as AstFileReference;
use tsox_frontend::ast::SyntaxKind;
use tsox_core::core::compiler_options_kinds::ResolutionMode;
use tsox_core::core::text::TextRange;

use crate::mig::m3n_5::r33k8_defs::FileReference;

impl crate::mig::m4e::DeclarationEmitHost for crate::mig::m3n_5::r33k8_defs::DeclarationEmitHost {
    fn get_effective_declaration_flags(
        &self,
        node: &Arc<tsox_frontend::ast::Node>,
        flags: tsox_frontend::ast::node_flags::ModifierFlags,
    ) -> tsox_frontend::ast::node_flags::ModifierFlags { ::tsox_core::fntrace::enter("get_effective_declaration_flags"); 
        crate::mig::m3n_5::r33k8_defs::DeclarationEmitHost::get_effective_declaration_flags(
            self, node, flags,
        )
    }
}

pub fn file_reference_from_ast(reference: &AstFileReference) -> FileReference { ::tsox_core::fntrace::enter("file_reference_from_ast"); 
    FileReference {
        file_name: reference.file_name.clone(),
        text_range: reference.range,
        resolution_mode: Some(reference.resolution_mode),
        preserve: reference.preserve,
    }
}

pub fn file_reference_to_ast(reference: &FileReference) -> AstFileReference { ::tsox_core::fntrace::enter("file_reference_to_ast"); 
    AstFileReference {
        range: reference.text_range,
        file_name: reference.file_name.clone(),
        resolution_mode: reference.resolution_mode.unwrap_or(ResolutionMode::None),
        preserve: reference.preserve,
    }
}

pub fn source_file_is_js(script_kind: tsox_frontend::ast::node_source_file::ScriptKind) -> bool { ::tsox_core::fntrace::enter("source_file_is_js"); 
    matches!(
        script_kind,
        tsox_frontend::ast::node_source_file::ScriptKind::Js
            | tsox_frontend::ast::node_source_file::ScriptKind::Jsx
    )
}

pub trait R39K18EmitResolverExt {
    fn is_optional_parameter(&self, node: &Arc<tsox_frontend::ast::Node>) -> bool;
}

impl R39K18EmitResolverExt for EmitResolver {
    fn is_optional_parameter(&self, node: &Arc<tsox_frontend::ast::Node>) -> bool { ::tsox_core::fntrace::enter("is_optional_parameter"); 
        matches!(
            &node.data,
            tsox_frontend::ast::NodeData::ParameterDeclaration(d)
                if d.question_token
                    .as_ref()
                    .is_some_and(|t| t.kind == SyntaxKind::QuestionToken)
        )
    }
}
