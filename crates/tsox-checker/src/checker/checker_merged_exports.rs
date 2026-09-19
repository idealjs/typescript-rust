#![allow(unused_imports)]

use crate::checker::checker::*;

// Go checkExportsOnMergedDeclarations：合并符号的声明必须全导出或全本地，
// default 导出与其余声明的公共空间相交另报 TS2390
impl Checker {
    pub(crate) fn check_exports_on_merged_declarations(&mut self, node: &Arc<Node>) {
        let Some(symbol) = self.program.symbol_map().symbol_of(node).cloned() else {
            return;
        };
        let mut any_exported = false;
        for d in symbol.declarations.iter() {
            if self
                .get_combined_modifier_flags(d)
                .contains(tsox_frontend::ast::ModifierFlags::Export)
            {
                any_exported = true;
                break;
            }
        }
        if !any_exported {
            return;
        }
        // Go：仅对同 kind 的首个声明跑一次
        if !symbol
            .declarations
            .iter()
            .any(|d| Arc::ptr_eq(d, node))
            || symbol
                .declarations
                .iter()
                .any(|d| d.kind == node.kind && d.loc.pos() < node.loc.pos())
        {
            return;
        }

        let mut exported = Spaces::NONE;
        let mut non_exported = Spaces::NONE;
        let mut default_exported = Spaces::NONE;
        for d in symbol.declarations.iter() {
            let spaces = Self::declaration_spaces(d);
            let flags = self.get_combined_modifier_flags(d);
            if flags.contains(tsox_frontend::ast::ModifierFlags::Export) {
                if flags.contains(tsox_frontend::ast::ModifierFlags::Default) {
                    default_exported |= spaces;
                } else {
                    exported |= spaces;
                }
            } else {
                non_exported |= spaces;
            }
        }
        let common_exports_locals = exported & non_exported;
        let non_default = exported | non_exported;
        let common_default = default_exported & non_default;
        if common_exports_locals.is_empty() && common_default.is_empty() {
            return;
        }
        for d in symbol.declarations.iter() {
            let spaces = Self::declaration_spaces(d);
            let name = tsox_frontend::ast::utilities::get_name_of_declaration(d)
                .unwrap_or_else(|| Arc::clone(d));
            let display = name.text().to_string();
            if !common_default.is_empty() && !spaces.intersect(common_default).is_empty() {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    tsox_core::diagnostics::messages_generated::
                        MERGED_DECLARATION_0_CANNOT_INCLUDE_A_DEFAULT_EXPORT_DECLARATION_CONSIDER_ADDING_A_SEPARATE_EXPORT_DEFAULT_0_DECLARATION_INSTEAD,
                    vec![display],
                ));
            } else if !common_exports_locals.is_empty()
                && !spaces.intersect(common_exports_locals).is_empty()
            {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    tsox_core::diagnostics::messages_generated::
                        INDIVIDUAL_DECLARATIONS_IN_MERGED_DECLARATION_0_MUST_BE_ALL_EXPORTED_OR_ALL_LOCAL,
                    vec![display],
                ));
            }
        }
    }

    // Go getDeclarationSpaces 的导出空间归类
    fn declaration_spaces(node: &Arc<Node>) -> Spaces {
        match node.kind {
            SyntaxKind::InterfaceDeclaration | SyntaxKind::TypeAliasDeclaration => Spaces::TYPE,
            SyntaxKind::ModuleDeclaration => {
                if tsox_frontend::ast::get_module_instance_state(node)
                    != tsox_frontend::ast::ModuleInstanceState::NonInstantiated
                {
                    Spaces::NAMESPACE | Spaces::VALUE
                } else {
                    Spaces::NAMESPACE
                }
            }
            SyntaxKind::ClassDeclaration | SyntaxKind::EnumDeclaration | SyntaxKind::EnumMember => {
                Spaces::TYPE | Spaces::VALUE
            }
            SyntaxKind::ExportAssignment | SyntaxKind::BinaryExpression => Spaces::VALUE,
            SyntaxKind::VariableDeclaration
            | SyntaxKind::BindingElement
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ImportSpecifier => Spaces::VALUE,
            _ => Spaces::NONE,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Spaces(u8);

impl Spaces {
    const NONE: Spaces = Spaces(0);
    const TYPE: Spaces = Spaces(1);
    const VALUE: Spaces = Spaces(2);
    const NAMESPACE: Spaces = Spaces(4);

    fn is_empty(self) -> bool {
        self.0 == 0
    }

    fn intersect(self, other: Spaces) -> Spaces {
        Spaces(self.0 & other.0)
    }
}

impl std::ops::BitOr for Spaces {
    type Output = Spaces;
    fn bitor(self, rhs: Spaces) -> Spaces {
        Spaces(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Spaces {
    fn bitor_assign(&mut self, rhs: Spaces) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for Spaces {
    type Output = Spaces;
    fn bitand(self, rhs: Spaces) -> Spaces {
        Spaces(self.0 & rhs.0)
    }
}
