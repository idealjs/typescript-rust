#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::tspath::Path;

use crate::ls::autoimport::RegistryCloneHost;
use crate::ls::autoimport_export::Export;
use crate::ls::autoimport_export::ExportSyntax;
use crate::ls::autoimport_export::ModuleID;
use crate::ls::autoimport_extract::CheckerLease;
use crate::ls::autoimport_extract::ExportExtractor;
use crate::ls::autoimport_extract::SymbolExtractor;
use tsox_frontend::ast::SourceFile;
use tsox_tsoptions::module::Resolver;

pub struct RegistryBuilder {
    pub to_path: Option<Arc<dyn Fn(&str) -> Path + Send + Sync>>,
}

impl RegistryBuilder {
    pub fn new_export_extractor(
        &self,
        package_name: &str,
        checker: Arc<tsox_checker::checker::Checker>,
        module_resolver: Option<Arc<Resolver>>,
        realpath: Option<Box<dyn Fn(&str) -> String + Send + Sync>>,
    ) -> ExportExtractor { ::tsox_core::fntrace::enter("new_export_extractor"); 
        let to_path: Option<Box<dyn Fn(&str) -> Path + Send + Sync>> = self.to_path.clone().map(|f| {
            Box::new(move |file_name: &str| f(file_name)) as Box<dyn Fn(&str) -> Path + Send + Sync>
        });
        ExportExtractor {
            symbol_extractor: crate::ls::autoimport_extract::new_symbol_extractor(
                package_name,
                checker,
                to_path,
                realpath,
            ),
            module_resolver,
        }
    }
}

pub fn is_non_pattern_ambient_module_declaration(
    _file: &SourceFile,
    _decl: &Arc<tsox_frontend::ast::Node>,
) -> bool { ::tsox_core::fntrace::enter("is_non_pattern_ambient_module_declaration"); 
    todo!("SourceFile.pattern_ambient_modules 与 Node.symbol 通道未移植（Go: 遍历 file.PatternAmbientModules 比对 module.Symbol == decl.Symbol）")
}

impl SymbolExtractor {
    pub fn try_resolve_symbol(
        &self,
        symbol: &Arc<tsox_frontend::ast::Symbol>,
        syntax: ExportSyntax,
        checker_lease: &mut CheckerLease,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> { ::tsox_core::fntrace::enter("try_resolve_symbol"); 
        if !tsox_frontend::ast::mig::m3g_2::is_non_local_alias(
            Some(symbol.as_ref()),
            tsox_frontend::ast::SymbolFlags::None,
        ) {
            return Some(symbol.clone());
        }

        let named_decl: Option<Arc<tsox_frontend::ast::Node>>;
        let assignment_decl: Option<Arc<tsox_frontend::ast::Node>>;
        let mut loc: Option<Arc<tsox_frontend::ast::Node>> = None;
        let mut name = String::new();
        match syntax {
            ExportSyntax::Named => {
                let Some(decl) = tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(
                    symbol,
                    tsox_frontend::ast::SyntaxKind::ExportSpecifier,
                ) else {
                    return None;
                };
                named_decl = Some(decl);
                let decl = named_decl.as_ref().unwrap();
                let module_specifier_is_none = decl
                    .parent()
                    .as_ref()
                    .and_then(|parent| parent.parent())
                    .is_some_and(|grand| {
                        matches!(
                            &grand.data,
                            tsox_frontend::ast::NodeData::ExportDeclaration(d)
                                if d.module_specifier.is_none()
                        )
                    });
                if module_specifier_is_none {
                    if let Some(n) = decl.name().or_else(|| {
                        tsox_frontend::ast::mig::m3b::property_name(decl)
                    }) {
                        if n.kind == tsox_frontend::ast::SyntaxKind::Identifier {
                            loc = Some(n.clone());
                            name = n.text().to_string();
                        }
                    }
                }
            }
            ExportSyntax::Equals if symbol.name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS => {
                assignment_decl = tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(
                    symbol,
                    tsox_frontend::ast::SyntaxKind::ExportAssignment,
                );
                if let Some(decl) = assignment_decl.as_ref() {
                    take_assignment_location(decl, &mut loc, &mut name);
                }
            }
            ExportSyntax::DefaultDeclaration => {
                assignment_decl = tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(
                    symbol,
                    tsox_frontend::ast::SyntaxKind::ExportAssignment,
                );
                if let Some(decl) = assignment_decl.as_ref() {
                    take_assignment_location(decl, &mut loc, &mut name);
                }
            }
            _ => {
                named_decl = None;
                assignment_decl = None;
            }
        }

        if let Some(loc) = loc {
            let local = self.resolve_local_name(&loc, &name);
            if let Some(local) = local {
                if !tsox_frontend::ast::mig::m3g_2::is_non_local_alias(
                    Some(local.as_ref()),
                    tsox_frontend::ast::SymbolFlags::None,
                ) {
                    return Some(local);
                }
            }
        }

        let checker = checker_lease.get_checker();
        let resolved = checker.get_aliased_symbol(symbol);
        if !checker.is_unknown_symbol(&resolved) {
            return Some(resolved);
        }
        None
    }

    fn resolve_local_name(
        &self,
        location: &Arc<tsox_frontend::ast::Node>,
        name: &str,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> { ::tsox_core::fntrace::enter("resolve_local_name"); 
        let mut resolver = tsox_checker::binder::nameresolver::NameResolver::new();
        resolver.compiler_options = Some(Arc::new(
            tsox_core::core::compiler_options::empty_compiler_options(),
        ));
        resolver.resolve(
            location,
            name,
            tsox_frontend::ast::SymbolFlags::all(),
            None,
            false,
            false,
        )
    }
}

fn take_assignment_location(
    decl: &tsox_frontend::ast::Node,
    loc: &mut Option<Arc<tsox_frontend::ast::Node>>,
    name: &mut String,
) { ::tsox_core::fntrace::enter("take_assignment_location"); 
    if let Some(expr) = decl.expression() {
        if expr.kind == tsox_frontend::ast::SyntaxKind::Identifier {
            *loc = Some(expr.clone());
            *name = expr.text().to_string();
        }
    }
}

pub fn extract_first_export(
    symbol: &tsox_frontend::ast::Symbol,
    ch: &mut tsox_checker::checker::Checker,
    module_id: ModuleID,
    module_file_name: &str,
    file: &SourceFile,
) -> Option<Export> { ::tsox_core::fntrace::enter("extract_first_export"); 
    let mut exports: Vec<Export> = Vec::new();
    let mut extractor = SymbolExtractor {
        package_name: String::new(),
        stats: Arc::new(crate::ls::autoimport_extract::ExtractorStats::default()),
        checker: None,
        to_path: None,
        realpath: None,
    };
    extractor.extract_from_symbol(
        &symbol.name,
        symbol,
        module_id,
        module_file_name,
        file,
        &mut exports,
    );
    exports.into_iter().next()
}

pub fn try_get_module_export(
    export_name: &str,
    target: &tsox_frontend::ast::Symbol,
    module_symbol: &Arc<tsox_frontend::ast::Symbol>,
    ch: &mut tsox_checker::checker::Checker,
    module_id: ModuleID,
    module_file_name: &str,
    file: &SourceFile,
) -> Option<Export> { ::tsox_core::fntrace::enter("try_get_module_export"); 
    let exported = ch.try_get_member_in_module_exports_and_properties(export_name, module_symbol)?;
    let merged = ch.get_merged_symbol(&ch.skip_alias(&exported));
    if merged.name == target.name && merged.declarations.len() == target.declarations.len() {
        return extract_first_export(exported.as_ref(), ch, module_id, module_file_name, file);
    }
    None
}

struct GetModuleResolverFs {
    host: Arc<dyn RegistryCloneHost>,
    realpath: Arc<dyn Fn(&str) -> String + Send + Sync>,
}

impl tsox_tsoptions::vfs::FS for GetModuleResolverFs {
    fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names"); 
        self.host.fs().use_case_sensitive_file_names()
    }
    fn file_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("file_exists"); 
        self.host.fs().file_exists(path)
    }
    fn read_file(&self, path: &str) -> Option<String> { ::tsox_core::fntrace::enter("read_file"); 
        self.host.fs().read_file(path)
    }
    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_file"); 
        self.host.fs().write_file(path, data)
    }
    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("append_file"); 
        self.host.fs().append_file(path, data)
    }
    fn remove(&self, path: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("remove"); 
        self.host.fs().remove(path)
    }
    fn directory_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("directory_exists"); 
        self.host.fs().directory_exists(path)
    }
    fn get_accessible_entries(&self, path: &str) -> tsox_tsoptions::vfs::Entries { ::tsox_core::fntrace::enter("get_accessible_entries"); 
        self.host.fs().get_accessible_entries(path)
    }
    fn stat(&self, path: &str) -> Option<tsox_tsoptions::vfs::FileInfo> { ::tsox_core::fntrace::enter("stat"); 
        self.host.fs().stat(path)
    }
    fn realpath(&self, path: &str) -> String { ::tsox_core::fntrace::enter("realpath"); 
        (self.realpath)(path)
    }
}

struct GetModuleResolverHost {
    fs: GetModuleResolverFs,
    current_directory: String,
}

impl tsox_tsoptions::module::ResolutionHost for GetModuleResolverHost {
    fn fs(&self) -> &dyn tsox_tsoptions::vfs::FS { ::tsox_core::fntrace::enter("fs"); 
        &self.fs
    }
    fn get_current_directory(&self) -> &str { ::tsox_core::fntrace::enter("get_current_directory"); 
        &self.current_directory
    }
}

pub fn get_module_resolver(
    host: Arc<dyn RegistryCloneHost>,
    realpath: impl Fn(&str) -> String + Send + Sync + 'static,
    opts: crate::ls::autoimport::ResolverOptions,
) -> Arc<Resolver> { ::tsox_core::fntrace::enter("get_module_resolver"); 
    let rh = GetModuleResolverHost {
        fs: GetModuleResolverFs {
            host: host.clone(),
            realpath: Arc::new(realpath),
        },
        current_directory: host.get_current_directory().to_string(),
    };
    new_resolver_with_options(
        Arc::new(rh),
        Arc::new(tsox_core::core::compiler_options::empty_compiler_options()),
        "",
        "",
        opts,
    )
}

pub fn new_resolver_with_options(
    host: Arc<dyn tsox_tsoptions::module::ResolutionHost + Send + Sync>,
    compiler_options: Arc<tsox_core::core::compiler_options::CompilerOptions>,
    typings_location: &str,
    project_name: &str,
    _opts: crate::ls::autoimport::ResolverOptions,
) -> Arc<Resolver> { ::tsox_core::fntrace::enter("new_resolver_with_options"); 
    Arc::new(Resolver::new(
        host,
        compiler_options,
        typings_location.to_string(),
        project_name.to_string(),
    ))
}
