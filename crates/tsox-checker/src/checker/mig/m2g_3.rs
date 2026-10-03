#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::types::TypeData;
use crate::checker::symboltracker::{NodeBuilderFlags, NodeBuilderInternalFlags};
use crate::checker::mig::m2f::VerbosityContext;
use crate::checker::mig::m2g::r21k9_defs::*;
use crate::checker::mig::m2g::r21k9_defs as tsox_printer;
use std::sync::Arc;
use tsox_frontend::ast::{get_source_file_of_node, Node, Symbol, SymbolFlags, SyntaxKind};

pub fn create_printer_with_defaults(emit_context: &EmitContext) -> Printer { ::tsox_core::fntrace::enter("create_printer_with_defaults"); 
    tsox_printer::new_printer(PrinterOptions::default(), PrintHandlers::default(), emit_context)
}

pub fn create_printer_with_remove_comments(emit_context: &EmitContext) -> Printer { ::tsox_core::fntrace::enter("create_printer_with_remove_comments"); 
    tsox_printer::new_printer(
        PrinterOptions {
            remove_comments: true,
            ..PrinterOptions::default()
        },
        PrintHandlers::default(),
        emit_context,
    )
}

pub fn create_printer_with_remove_comments_omit_trailing_semicolon(
    emit_context: &EmitContext,
) -> Printer { ::tsox_core::fntrace::enter("create_printer_with_remove_comments_omit_trailing_semicolon"); 
    tsox_printer::new_printer(
        PrinterOptions {
            remove_comments: true,
            omit_trailing_semicolon: true,
            ..PrinterOptions::default()
        },
        PrintHandlers::default(),
        emit_context,
    )
}

pub fn create_printer_with_remove_comments_omit_trailing_semicolon_never_ascii_escape(
    emit_context: &EmitContext,
) -> Printer { ::tsox_core::fntrace::enter("create_printer_with_remove_comments_omit_trailing_semicolon_never_ascii_escape"); 
    tsox_printer::new_printer(
        PrinterOptions {
            remove_comments: true,
            omit_trailing_semicolon: true,
            never_ascii_escape: true,
            ..PrinterOptions::default()
        },
        PrintHandlers::default(),
        emit_context,
    )
}

pub fn create_printer_with_remove_comments_never_ascii_escape(
    emit_context: &EmitContext,
) -> Printer { ::tsox_core::fntrace::enter("create_printer_with_remove_comments_never_ascii_escape"); 
    tsox_printer::new_printer(
        PrinterOptions {
            remove_comments: true,
            never_ascii_escape: true,
            ..PrinterOptions::default()
        },
        PrintHandlers::default(),
        emit_context,
    )
}

pub fn to_node_builder_flags(flags: TypeFormatFlags) -> NodeBuilderFlags { ::tsox_core::fntrace::enter("to_node_builder_flags"); 
    NodeBuilderFlags::from_bits_truncate(flags.bits() & TYPE_FORMAT_FLAGS_NODE_BUILDER_FLAGS_MASK)
}

impl Checker {
    pub fn expand_symbol_for_hover(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        vc: Option<&mut VerbosityContext>,
    ) -> String { ::tsox_core::fntrace::enter("expand_symbol_for_hover"); 
        let (mut node_builder, release) = self.get_node_builder();
        let old_verbosity = node_builder.verbosity.take();
        node_builder.verbosity = vc.as_deref().map(|vc| VerbosityContext {
            level: vc.level,
            max_truncation_length: vc.max_truncation_length,
            can_increase_verbosity: vc.can_increase_verbosity,
            truncated: vc.truncated,
        });
        let nodes = node_builder.expand_symbol_for_hover_nodes(symbol, meaning);
        if let (Some(vc), Some(vb)) = (vc, node_builder.verbosity.as_ref()) {
            if vb.can_increase_verbosity {
                vc.can_increase_verbosity = true;
            }
            if vb.truncated {
                vc.truncated = true;
            }
        }
        node_builder.verbosity = old_verbosity;
        release();
        if nodes.is_empty() {
            return String::new();
        }
        let p = create_printer_with_remove_comments(&node_builder.emit_context());
        let source_file = symbol
            .value_declaration
            .as_ref()
            .and_then(get_source_file_of_node);
        let mut b = String::new();
        for (i, node) in nodes.iter().enumerate() {
            if i > 0 {
                b.push('\n');
            }
            b.push_str(&p.emit(node, source_file.as_deref()));
        }
        b
    }

    pub fn type_parameter_to_string_ex(
        &mut self,
        t: &Arc<Type>,
        enclosing_declaration: Option<&Arc<Node>>,
        vc: Option<&mut VerbosityContext>,
    ) -> String { ::tsox_core::fntrace::enter("type_parameter_to_string_ex"); 
        let (mut node_builder, release) = self.get_node_builder();
        let old_verbosity = node_builder.verbosity.take();
        node_builder.verbosity = vc.as_deref().map(|vc| VerbosityContext {
            level: vc.level,
            max_truncation_length: vc.max_truncation_length,
            can_increase_verbosity: vc.can_increase_verbosity,
            truncated: vc.truncated,
        });
        let type_param_node = node_builder.type_parameter_to_declaration(
            t,
            enclosing_declaration,
            NodeBuilderFlags::IGNORE_ERRORS,
            NodeBuilderInternalFlags::empty(),
            None,
        );
        if let (Some(vc), Some(vb)) = (vc, node_builder.verbosity.as_ref()) {
            if vb.can_increase_verbosity {
                vc.can_increase_verbosity = true;
            }
            if vb.truncated {
                vc.truncated = true;
            }
        }
        node_builder.verbosity = old_verbosity;
        release();
        let Some(type_param_node) = type_param_node else {
            return self.type_to_string(t);
        };
        let p = create_printer_with_remove_comments(&node_builder.emit_context());
        let source_file = enclosing_declaration.and_then(get_source_file_of_node);
        p.emit(&type_param_node, source_file.as_deref())
    }

    pub fn signature_to_signature_declaration(
        &mut self,
        signature: &Signature,
        kind: SyntaxKind,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("signature_to_signature_declaration"); 
        let (mut node_builder, release) = self.get_node_builder();
        let result = node_builder
            .signature_to_signature_declaration_ex(
                signature,
                kind,
                enclosing_declaration,
                flags,
                NodeBuilderInternalFlags::empty(),
            )
            .expect("signature declaration node");
        release();
        result
    }

    pub fn type_to_type_node_ex(
        &mut self,
        t: &Arc<Type>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        id_to_symbol: IdToSymbolMap,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("type_to_type_node_ex"); 
        let mut node_builder = self.get_node_builder_ex(id_to_symbol);
        node_builder
            .type_to_type_node_ex(t, enclosing_declaration, flags, internal_flags)
            .expect("type node")
    }

    pub fn type_predicate_to_type_predicate_node(
        &mut self,
        t: &TypePredicate,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        id_to_symbol: IdToSymbolMap,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("type_predicate_to_type_predicate_node"); 
        let mut node_builder = self.get_node_builder_ex(id_to_symbol);
        node_builder
            .type_predicate_to_type_predicate_node_ex(
                t,
                enclosing_declaration,
                flags,
                NodeBuilderInternalFlags::empty(),
            )
            .expect("type predicate node")
    }

    pub fn value_to_string(&mut self, value: &PseudoLiteralValue) -> String { ::tsox_core::fntrace::enter("value_to_string"); 
        value_to_string(value)
    }

    pub fn signature_to_string_ex(
        &mut self,
        signature: &Signature,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: TypeFormatFlags,
        vc: Option<&mut VerbosityContext>,
    ) -> String { ::tsox_core::fntrace::enter("signature_to_string_ex"); 
        let is_constructor = signature.flags.contains(SignatureFlags::Construct)
            && !flags.contains(TypeFormatFlags::WriteCallStyleSignature);
        let sig_output = if flags.contains(TypeFormatFlags::WriteArrowStyleSignature) {
            if is_constructor {
                SyntaxKind::ConstructorType
            } else {
                SyntaxKind::FunctionType
            }
        } else if is_constructor {
            SyntaxKind::ConstructSignature
        } else {
            SyntaxKind::CallSignature
        };
        let (mut node_builder, release) = self.get_node_builder();
        let old_verbosity = node_builder.verbosity.take();
        node_builder.verbosity = vc.as_deref().map(|vc| VerbosityContext {
            level: vc.level,
            max_truncation_length: vc.max_truncation_length,
            can_increase_verbosity: vc.can_increase_verbosity,
            truncated: vc.truncated,
        });
        let combined_flags = to_node_builder_flags(flags)
            | NodeBuilderFlags::IGNORE_ERRORS
            | NodeBuilderFlags::WriteTypeParametersInQualifiedName;
        let sig = node_builder
            .signature_to_signature_declaration_ex(
                signature,
                sig_output,
                enclosing_declaration,
                combined_flags,
                NodeBuilderInternalFlags::empty(),
            )
            .expect("signature declaration node");
        if let (Some(vc), Some(vb)) = (vc, node_builder.verbosity.as_ref()) {
            if vb.can_increase_verbosity {
                vc.can_increase_verbosity = true;
            }
            if vb.truncated {
                vc.truncated = true;
            }
        }
        node_builder.verbosity = old_verbosity;
        release();
        let p = create_printer_with_remove_comments_omit_trailing_semicolon_never_ascii_escape(
            &node_builder.emit_context(),
        );
        let source_file = enclosing_declaration.and_then(get_source_file_of_node);
        if flags.contains(TypeFormatFlags::MultilineObjectLiterals) {
            let mut writer = TextWriter::new("\n", 0);
            p.write(&sig, source_file.as_deref(), &mut writer);
            return writer.string();
        }
        let mut writer = single_line_string_writer();
        p.write(&sig, source_file.as_deref(), &mut writer);
        writer.string()
    }

    pub fn type_predicate_to_string_ex(
        &mut self,
        type_predicate: &TypePredicate,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: TypeFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("type_predicate_to_string_ex"); 
        let mut writer = single_line_string_writer();
        let (mut node_builder, release) = self.get_node_builder();
        let combined_flags = to_node_builder_flags(flags)
            | NodeBuilderFlags::IGNORE_ERRORS
            | NodeBuilderFlags::WriteTypeParametersInQualifiedName;
        let predicate = node_builder
            .type_predicate_to_type_predicate_node_ex(
                type_predicate,
                enclosing_declaration,
                combined_flags,
                NodeBuilderInternalFlags::empty(),
            )
            .expect("type predicate node");
        release();
        let printer_ = create_printer_with_remove_comments(&node_builder.emit_context());
        let source_file = enclosing_declaration.and_then(get_source_file_of_node);
        printer_.write(&predicate, source_file.as_deref(), &mut writer);
        writer.string()
    }

    pub fn format_union_types(
        &mut self,
        types: &[Arc<Type>],
        expanding_enum: bool,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("format_union_types"); 
        let mut result: Vec<Arc<Type>> = Vec::new();
        let mut flags = TypeFlags::empty();
        let mut i = 0;
        while i < types.len() {
            let t = &types[i];
            flags |= t.flags;
            if !t.flags.contains(TypeFlags::NULLABLE) {
                if t.flags.contains(TypeFlags::BooleanLiteral)
                    || (!expanding_enum && t.flags.contains(TypeFlags::ENUM_LIKE))
                {
                    let base_type = if t.flags.contains(TypeFlags::BooleanLiteral) {
                        Arc::clone(self.boolean_type.get().unwrap())
                    } else {
                        self.get_base_type_of_enum_like_type(t)
                    };
                    if base_type.flags.contains(TypeFlags::UNION) {
                        let count = match &base_type.data {
                            TypeData::Union(u) => u.union_or_intersection.types.len(),
                            _ => 0,
                        };
                        if i + count <= types.len()
                            && Arc::ptr_eq(
                                &self.get_regular_type_of_literal_type(&types[i + count - 1]),
                                &self.get_regular_type_of_literal_type(
                                    &match &base_type.data {
                                    TypeData::Union(u) => &u.union_or_intersection.types,
                                    _ => unreachable!(),
                                }[count - 1],
                                ),
                            )
                        {
                            result.push(base_type);
                            i += count;
                            continue;
                        }
                    }
                }
                result.push(Arc::clone(t));
            }
            i += 1;
        }
        if flags.contains(TypeFlags::Null) {
            result.push(Arc::clone(self.null_type.get().unwrap()));
        }
        if flags.contains(TypeFlags::UNDEFINED) {
            result.push(Arc::clone(self.undefined_type.get().unwrap()));
        }
        result
    }
}
