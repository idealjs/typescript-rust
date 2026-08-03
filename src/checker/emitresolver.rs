//! Emit resolver: provides symbol/type information needed during emit.
//!
//! Ported from `internal/checker/emitresolver.go` (~1322 lines). The Go
//! implementation wraps the checker and provides thread-safe access to
//! symbol visibility, enum values, parameter optionality, and other
//! information required by the emitter/transformer.
//!
//! The Rust port implements the methods directly on `Checker` (since the
//! checker already uses interior mutability) and uses this module as the
//! organizational home for the emit-resolution logic. The visibility
//! tracking mirrors Go's `aliasMarkingVisitor` / `determineIfDeclarationIsVisible`
//! pipeline; the heavy `isSymbolAccessible` / declaration-emit node
//! construction is deferred until declaration emit is wired up.

use std::sync::Arc;

use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

use super::checker::Checker;
use super::types::{
    DeclarationFileLinks, DeclarationLinks, SymbolAccessibility, SymbolAccessibilityResult,
};

/// The emit resolver. Provides access to checker data needed during emit.
///
/// In Go, this is a separate struct that wraps the checker with a mutex.
/// In Rust, we implement the methods directly on `Checker` (since the
/// checker already uses interior mutability) and provide this module as
/// the organizational home for the emit-resolution logic.
impl Checker {
    // ────────────────────────────────────────────────────────────────────
    // Declaration visibility
    // ────────────────────────────────────────────────────────────────────

    /// Check if a declaration is visible (should be emitted).
    ///
    /// Mirrors Go's `EmitResolver.IsDeclarationVisible` (emitresolver.go ~L104).
    /// A declaration is visible if it's not purely a type-only declaration
    /// that's never used in a value position. Results are cached per-node in
    /// `declaration_links.is_visible` (a `Tristate`).
    pub fn is_declaration_visible(&mut self, node: &Arc<Node>) -> bool {
        // Cache lookup. Unknown → not yet computed.
        let cached = self
            .declaration_links
            .get(node)
            .map(|l| l.is_visible)
            .unwrap_or_default();
        if !cached.is_unknown() {
            return cached.is_true();
        }
        let result = self.determine_if_declaration_is_visible(node);
        self.declaration_links.get_or_default(node).is_visible = result.into();
        result
    }

    /// Core visibility decision. Mirrors Go's
    /// `EmitResolver.determineIfDeclarationIsVisible` (emitresolver.go ~L131).
    ///
    /// Walks the declaration kinds:
    /// - top-level declarations (variable/module/class/interface/type-alias/
    ///   function/enum/import=) are visible if exported and their container is
    ///   visible, or if they live in a global (non-module) source file;
    /// - properties/methods are visible if not private/protected and their
    ///   parent is visible;
    /// - import clauses / namespace imports / import specifiers default to
    ///   *not* visible (they are marked on demand by the alias marking visitor);
    /// - source files, namespace export declarations, and type parameters are
    ///   always visible;
    /// - export assignments are not visible (they don't bind a name).
    fn determine_if_declaration_is_visible(&mut self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::BindingElement => node
                .parent
                .clone()
                .and_then(|p| p.parent.clone())
                .map(|gp| self.is_declaration_visible(&gp))
                .unwrap_or(false),

            SyntaxKind::VariableDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ImportEqualsDeclaration => {
                // An empty binding pattern (`const {} = x`) is not visible.
                if node.kind == SyntaxKind::VariableDeclaration {
                    if let NodeData::VariableDeclaration(d) = &node.data {
                        let name = &d.name;
                        if name.kind == SyntaxKind::ObjectBindingPattern
                            || name.kind == SyntaxKind::ArrayBindingPattern
                        {
                            if let NodeData::BindingPattern(p) = &name.data {
                                if p.elements.nodes.is_empty() {
                                    return false;
                                }
                            }
                        }
                    }
                }
                // External module augmentation (`declare module "foo"` in a
                // module file) is always visible.
                if Self::is_external_module_augmentation(node) {
                    return true;
                }
                let parent = match Checker::get_declaration_container(node) {
                    Some(p) => p,
                    None => return false,
                };
                let is_exported = self
                    .get_combined_modifier_flags(node)
                    .contains(crate::ast::ModifierFlags::Export);
                // Ambient module elements (other than import=) inside a
                // non-source-file container are visible if exported.
                let is_ambient_element = node.kind != SyntaxKind::ImportEqualsDeclaration
                    && parent.kind != SyntaxKind::SourceFile
                    && parent.flags.contains(crate::ast::NodeFlags::Ambient);
                if !is_exported && !is_ambient_element {
                    // Not exported: visible only in a global (non-module) script.
                    return Self::is_global_source_file(&parent);
                }
                // Exported / ambient element: visible if its container is visible.
                self.is_declaration_visible(&parent)
            }

            SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature => {
                // Private/protected members are not visible.
                let flags = self.get_effective_declaration_flags(node);
                let private_protected = crate::ast::ModifierFlags::Private
                    .union(crate::ast::ModifierFlags::Protected)
                    .bits();
                if flags & private_protected != 0 {
                    return false;
                }
                node.parent
                    .clone()
                    .map(|p| self.is_declaration_visible(&p))
                    .unwrap_or(false)
            }

            SyntaxKind::Constructor
            | SyntaxKind::ConstructSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::Parameter
            | SyntaxKind::ModuleBlock
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::TypeLiteral
            | SyntaxKind::TypeReference
            | SyntaxKind::ArrayType
            | SyntaxKind::TupleType
            | SyntaxKind::UnionType
            | SyntaxKind::IntersectionType
            | SyntaxKind::ParenthesizedType
            | SyntaxKind::NamedTupleMember => node
                .parent
                .clone()
                .map(|p| self.is_declaration_visible(&p))
                .unwrap_or(false),

            // Default binding, namespace import and import specifier are
            // visible only on demand (marked by the alias marking visitor).
            SyntaxKind::ImportClause
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportSpecifier => false,

            // Type parameters are always visible.
            SyntaxKind::TypeParameter => true,

            // Source file and `export *` namespace export are always visible.
            SyntaxKind::SourceFile | SyntaxKind::NamespaceExportDeclaration => true,

            // `export =` does not bind a name outside the module.
            SyntaxKind::ExportAssignment => false,

            // An `export {X}` (without a module specifier) is a visible
            // re-export of the named binding; it contributes to the symbol's
            // external visibility.
            SyntaxKind::ExportSpecifier => {
                let export_decl = match node.parent.clone().and_then(|p| p.parent.clone()) {
                    Some(ed) if ed.kind == SyntaxKind::ExportDeclaration => ed,
                    _ => return false,
                };
                let has_module_specifier = match &export_decl.data {
                    NodeData::ExportDeclaration(d) => d.module_specifier.is_some(),
                    _ => false,
                };
                if has_module_specifier {
                    return false;
                }
                export_decl
                    .parent
                    .clone()
                    .map(|p| self.is_declaration_visible(&p))
                    .unwrap_or(false)
            }

            _ => false,
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Alias marking visitor (visibility pre-calculation)
    // ────────────────────────────────────────────────────────────────────

    /// Pre-calculate declaration-emit visibility for a file by running the
    /// alias marking visitor over its top-level statements.
    ///
    /// Mirrors Go's `EmitResolver.PrecalculateDeclarationEmitVisibility`
    /// (emitresolver.go ~L236). Idempotent: a file is only walked once.
    pub fn precalculate_declaration_emit_visibility(&mut self, file: &Arc<crate::ast::SourceFile>) {
        if self
            .declaration_file_links
            .get(file)
            .map(|l| l.aliases_marked)
            .unwrap_or(false)
        {
            return;
        }
        self.declaration_file_links
            .get_or_default(file)
            .aliases_marked = true;
        // Save/restore file context (mirrors check_source_file) so that
        // `resolve_name_in_file_scope` can find the file's symbol.
        let saved_file = self.current_file.take();
        let saved_file_id = self.current_file_id;
        let saved_file_symbol = self.current_file_symbol.take();
        let saved_scope_stack = self.scope_stack.clone();

        self.current_file = Some(Arc::clone(file));
        self.current_file_id = file.node.id();
        self.current_file_symbol = self.program.symbol_map().symbol_of(&file.node).cloned();
        // Push the source-file scope so name lookups can reach top-level
        // declarations via the scope chain as well.
        self.scope_stack.clear();
        self.scope_stack.push(file.node.id());

        // Collect children first so we don't hold a borrow of `self` while
        // mutating `declaration_links` inside the visitor.
        let children = collect_children(&file.node);
        for child in children {
            self.alias_marking_visitor(&child);
        }

        self.current_file = saved_file;
        self.current_file_id = saved_file_id;
        self.current_file_symbol = saved_file_symbol;
        self.scope_stack = saved_scope_stack;
    }

    /// The alias marking visitor. Marks declarations referenced by
    /// `export =`/`export {}`/CommonJS `module.exports =` as visible so that
    /// declaration emit can serialize them.
    ///
    /// Mirrors Go's `EmitResolver.aliasMarkingVisitorWorker` (~L260).
    fn alias_marking_visitor(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::BinaryExpression => {
                // CommonJS `module.exports = identifier` / `exports.x = identifier`.
                if Self::is_common_js_module_exports(node) {
                    if let NodeData::BinaryExpression(bin) = &node.data {
                        if bin.right.kind == SyntaxKind::Identifier {
                            self.mark_linked_aliases(&bin.right);
                        }
                    }
                }
            }
            SyntaxKind::ExportAssignment => {
                if let Some(expr) = node.expression() {
                    if expr.kind == SyntaxKind::Identifier {
                        self.mark_linked_aliases(expr);
                    }
                }
            }
            SyntaxKind::ExportSpecifier => {
                if let Some(name) = Self::export_specifier_name(node) {
                    self.mark_linked_aliases(&name);
                }
            }
            _ => {}
        }
        // Recurse.
        let children = collect_children(node);
        for child in children {
            self.alias_marking_visitor(&child);
        }
    }

    /// Mark the chain of declarations reachable from `node` (an identifier) as
    /// visible. Follows `import d = a.b.c` chains.
    ///
    /// Mirrors Go's `EmitResolver.markLinkedAliases` (~L278). This is a
    /// simplified port: it resolves the export name in the file's symbol
    /// members (or the enclosing export specifier's target) and marks each
    /// declaration of the resolved symbol as visible, following
    /// `ImportEqualsDeclaration` chains.
    fn mark_linked_aliases(&mut self, node: &Arc<Node>) {
        let export_symbol = self.resolve_export_symbol_for_alias(node);
        let mut export_symbol = export_symbol;
        // Guard against circular import chains.
        let mut visited: Vec<u64> = Vec::new();
        while let Some(sym) = export_symbol {
            if visited.contains(&sym.id()) {
                break;
            }
            visited.push(sym.id());

            let mut next_symbol: Option<Arc<Symbol>> = None;
            let declarations = sym.declarations.clone();
            for declaration in declarations.iter() {
                self.declaration_links
                    .get_or_default(declaration)
                    .is_visible = true.into();

                // Follow `import d = a.b.c` chains: the first identifier of
                // the module reference is the next link to resolve.
                if declaration.kind == SyntaxKind::ImportEqualsDeclaration {
                    if let NodeData::ImportEqualsDeclaration(d) = &declaration.data {
                        let first_id = Self::first_identifier_of(&d.module_reference);
                        if let Some(first_id) = first_id {
                            // Resolve the first identifier in the scope of
                            // the import declaration.
                            let saved = self.scope_stack.clone();
                            // Push the import's enclosing source file scope.
                            if let Some(parent) = declaration.parent.clone() {
                                self.scope_stack.push(parent.id());
                            }
                            let resolved = self.resolve_identifier_with_meaning(
                                &first_id,
                                SymbolFlags::VALUE
                                    | SymbolFlags::TYPE
                                    | SymbolFlags::NAMESPACE
                                    | SymbolFlags::Alias,
                            );
                            self.scope_stack = saved;
                            next_symbol = resolved;
                        }
                    }
                }
            }
            export_symbol = next_symbol;
        }
    }

    /// Resolve the export symbol that an alias-marking target points at.
    ///
    /// For `export = identifier` and CommonJS `module.exports = identifier`,
    /// resolve the identifier name in the enclosing source-file scope. For an
    /// `export { X }` specifier, look up `X` in the file symbol's members.
    fn resolve_export_symbol_for_alias(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let parent = node.parent.clone()?;
        match parent.kind {
            SyntaxKind::ExportAssignment | SyntaxKind::BinaryExpression => {
                let name = node.text();
                self.resolve_name_in_file_scope(name)
            }
            SyntaxKind::ExportSpecifier => {
                // `export { X }` — look up X in the file's symbol members.
                let spec_name = Self::export_specifier_name(&parent)?;
                let name = spec_name.text();
                self.resolve_name_in_file_scope(name)
            }
            _ => None,
        }
    }

    /// Resolve a name in the current file symbol's members (top-level
    /// declarations of the enclosing source file).
    fn resolve_name_in_file_scope(&self, name: &str) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        let file_id = self.current_file_id;
        if let Some(file_sym) = symbol_map.symbols.get(&file_id) {
            if let Some(sym) = file_sym.members.get(name) {
                return self.follow_alias(sym);
            }
        }
        // Fall back to walking the scope stack (top-level locals).
        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    return self.follow_alias(sym);
                }
            }
            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                if let Some(sym) = container_sym.members.get(name) {
                    return self.follow_alias(sym);
                }
            }
        }
        None
    }

    /// Whether a binary expression is a CommonJS `module.exports = ...` or
    /// `exports.x = ...` assignment at the top level of a CommonJS module.
    /// Mirrors Go's `isCommonJSModuleExports`.
    fn is_common_js_module_exports(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::BinaryExpression {
            return false;
        }
        // Walk: node → ExpressionStatement → SourceFile, and check the source
        // file would be treated as a CommonJS module. The Rust SourceFile does
        // not yet track a `CommonJSModuleIndicator`, so we approximate by
        // checking the left-hand side spelling.
        let NodeData::BinaryExpression(bin) = &node.data else {
            return false;
        };
        let left_is_module_exports = matches!(&bin.left.data,
            NodeData::PropertyAccessExpression(pa)
                if pa.expression.kind == SyntaxKind::Identifier
                && pa.expression.text() == "module"
                && pa.name.text() == "exports");
        let left_is_exports_dot = matches!(&bin.left.data,
            NodeData::PropertyAccessExpression(pa)
                if pa.expression.kind == SyntaxKind::Identifier
                && pa.expression.text() == "exports");
        left_is_module_exports || left_is_exports_dot
    }

    /// The local-name or property-name node of an `ExportSpecifier`.
    fn export_specifier_name(node: &Arc<Node>) -> Option<Arc<Node>> {
        let NodeData::ExportSpecifier(d) = &node.data else {
            return None;
        };
        Some(if let Some(pn) = &d.property_name {
            Arc::clone(pn)
        } else {
            Arc::clone(&d.name)
        })
    }

    /// Walk a (possibly qualified) entity-name/expression to its first
    /// identifier. Mirrors Go's `ast.GetFirstIdentifier`.
    fn first_identifier_of(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(node);
        loop {
            match &current.data {
                NodeData::Identifier(_) => return Some(current),
                NodeData::QualifiedName(q) => current = Arc::clone(&q.left),
                NodeData::PropertyAccessExpression(p) => current = Arc::clone(&p.expression),
                _ => return None,
            }
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Entity-name visibility
    // ────────────────────────────────────────────────────────────────────

    /// Check if an entity name is visible from `enclosing_declaration`.
    ///
    /// Mirrors Go's `EmitResolver.isEntityNameVisible` (~L340). This is a
    /// simplified port: it resolves the first identifier and checks
    /// `has_visible_declarations`. Full alias-to-make-visible computation is
    /// deferred (it requires the complete `getTargetOfExportSpecifier` and
    /// `IsSymbolAccessible` machinery, which depends on binder features not
    /// yet migrated).
    pub fn is_entity_name_visible(
        &mut self,
        entity_name: &Arc<Node>,
        enclosing_declaration: &Arc<Node>,
    ) -> SymbolAccessibilityResult {
        let meaning = Self::meaning_of_entity_name_reference(entity_name);
        let first_identifier =
            Self::first_identifier_of(entity_name).unwrap_or_else(|| Arc::clone(entity_name));

        let symbol =
            self.resolve_name_in_enclosure(enclosing_declaration, first_identifier.text(), meaning);

        if let Some(sym) = &symbol {
            if sym.flags.contains(SymbolFlags::TypeParameter) && meaning.contains(SymbolFlags::TYPE)
            {
                return SymbolAccessibilityResult {
                    accessibility: SymbolAccessibility::Accessible,
                    ..Default::default()
                };
            }
        }

        let symbol = match symbol {
            Some(s) => s,
            None => {
                return SymbolAccessibilityResult {
                    accessibility: SymbolAccessibility::NotResolved,
                    error_symbol_name: first_identifier.text().to_string(),
                    error_node: Some(first_identifier),
                    ..Default::default()
                };
            }
        };

        match self.has_visible_declarations(&symbol) {
            Some(result) => result,
            None => SymbolAccessibilityResult {
                accessibility: SymbolAccessibility::NotAccessible,
                error_symbol_name: first_identifier.text().to_string(),
                error_node: Some(first_identifier),
                ..Default::default()
            },
        }
    }

    /// Resolve `name` in the scope of `enclosing_declaration`.
    fn resolve_name_in_enclosure(
        &self,
        enclosing_declaration: &Arc<Node>,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        // Walk up from the enclosing declaration, checking locals and member
        // tables of each container. This mirrors the scope-chain walk done by
        // Go's `resolveName` for the alias/visibility case (without the full
        // NameResolver feature set).
        let mut current: Option<Arc<Node>> = Some(Arc::clone(enclosing_declaration));
        while let Some(n) = current {
            if let Some(sym) = symbol_map.symbol_of(&n) {
                if let Some(found) = sym.members.get(name) {
                    if found.flags.intersects(meaning) {
                        return self.follow_alias(found);
                    }
                }
            }
            if let Some(locals) = symbol_map.locals.get(&n.id()) {
                if let Some(found) = locals.get(name) {
                    if found.flags.intersects(meaning) {
                        return self.follow_alias(found);
                    }
                }
            }
            current = n.parent.clone();
        }
        // Fall back to the file symbol.
        if let Some(file_sym) = symbol_map.symbols.get(&self.current_file_id) {
            if let Some(found) = file_sym.members.get(name) {
                if found.flags.intersects(meaning) {
                    return self.follow_alias(found);
                }
            }
        }
        None
    }

    /// Determine the meaning (value/type/namespace) of an entity-name
    /// reference based on its syntactic position. Mirrors Go's
    /// `getMeaningOfEntityNameReference` (~L311).
    fn meaning_of_entity_name_reference(entity_name: &Arc<Node>) -> SymbolFlags {
        let parent = match &entity_name.parent {
            Some(p) => p,
            None => return SymbolFlags::TYPE,
        };
        // `typeof x`, `x as E` (ExpressionWithTypeArguments in a value
        // position, e.g. `class extends Foo<T>`), computed property name,
        // `x is T` predicate LHS, binary expr.
        let is_value_position = matches!(
            parent.kind,
            SyntaxKind::TypeQuery
                | SyntaxKind::ComputedPropertyName
                | SyntaxKind::BinaryExpression
                | SyntaxKind::ExpressionWithTypeArguments
        ) || (parent.kind == SyntaxKind::TypePredicate
            && matches!(&parent.data, NodeData::TypePredicateNode(tp) if tp.parameter_name.id() == entity_name.id()));
        if is_value_position {
            return SymbolFlags::VALUE | SymbolFlags::ExportValue;
        }
        // Left identifier of a qualified name / property access, or the
        // entity name of `import d = a.b.c`.
        let is_namespace_position = entity_name.kind == SyntaxKind::QualifiedName
            || entity_name.kind == SyntaxKind::PropertyAccessExpression
            || parent.kind == SyntaxKind::ImportEqualsDeclaration
            || (parent.kind == SyntaxKind::QualifiedName
                && matches!(&parent.data, NodeData::QualifiedName(q) if q.left.id() == entity_name.id()))
            || (parent.kind == SyntaxKind::PropertyAccessExpression
                && matches!(&parent.data, NodeData::PropertyAccessExpression(pa) if pa.expression.id() == entity_name.id()))
            || (parent.kind == SyntaxKind::ElementAccessExpression
                && matches!(&parent.data, NodeData::ElementAccessExpression(ea) if ea.expression.id() == entity_name.id()));
        if is_namespace_position {
            return SymbolFlags::NAMESPACE;
        }
        SymbolFlags::TYPE
    }

    /// Whether all declarations of `symbol` are visible (or can be made
    /// visible via aliases). Mirrors Go's `hasVisibleDeclarations` (~L384).
    ///
    /// Returns `Some(Accessible)` if visible, `None` if not visible. The
    /// aliases-to-make-visible computation is simplified: unexported imports
    /// and variable statements whose parent is visible are marked visible.
    pub fn has_visible_declarations(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<SymbolAccessibilityResult> {
        let declarations = symbol.declarations.clone();
        for declaration in declarations.iter() {
            if declaration.kind == SyntaxKind::Identifier {
                continue;
            }
            if self.is_declaration_visible(declaration) {
                continue;
            }
            // Try to mark an unexported alias as visible if its parent is
            // visible (these aliases can name types in a declaration file).
            if let Some(any_import) = Checker::get_any_import_syntax(declaration) {
                let is_exported =
                    any_import.has_syntactic_modifier(crate::ast::ModifierFlags::Export);
                if !is_exported {
                    if let Some(parent) = any_import.parent.clone() {
                        if self.is_declaration_visible(&parent) {
                            self.declaration_links
                                .get_or_default(declaration)
                                .is_visible = true.into();
                            continue;
                        }
                    }
                }
            }
            // Unexported variable statement whose container is visible.
            if declaration.kind == SyntaxKind::VariableDeclaration {
                let var_list = declaration.parent.clone();
                let var_stmt = var_list.as_ref().and_then(|p| p.parent.clone());
                if let Some(vs) = &var_stmt {
                    if vs.kind == SyntaxKind::VariableStatement
                        && !vs.has_syntactic_modifier(crate::ast::ModifierFlags::Export)
                    {
                        if let Some(container) = vs.parent.clone() {
                            if self.is_declaration_visible(&container) {
                                self.declaration_links
                                    .get_or_default(declaration)
                                    .is_visible = true.into();
                                continue;
                            }
                        }
                    }
                }
            }
            // Late-visibility-painted top-level statement whose parent is visible.
            if Checker::is_late_visibility_painted_statement(declaration)
                && !declaration.has_syntactic_modifier(crate::ast::ModifierFlags::Export)
            {
                if let Some(parent) = declaration.parent.clone() {
                    if self.is_declaration_visible(&parent) {
                        self.declaration_links
                            .get_or_default(declaration)
                            .is_visible = true.into();
                        continue;
                    }
                }
            }
            // Not visible.
            return None;
        }
        Some(SymbolAccessibilityResult {
            accessibility: SymbolAccessibility::Accessible,
            ..Default::default()
        })
    }

    // ────────────────────────────────────────────────────────────────────
    // Enum member values
    // ────────────────────────────────────────────────────────────────────

    /// Get the constant value of an enum member.
    ///
    /// Mirrors Go's `EmitResolver.GetEnumMemberValue` (emitresolver.go ~L89).
    /// Returns the enum member's value as a string (for numeric enums) or
    /// the string literal (for string enums).
    pub fn get_enum_member_value_string(&mut self, node: &Arc<Node>) -> Option<String> {
        // Look for the enum member's initializer expression.
        let NodeData::EnumMember(data) = &node.data else {
            return None;
        };
        let initializer = data.initializer.as_ref()?;
        match initializer.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(s) = &initializer.data {
                    Some(format!("\"{}\"", s.text))
                } else {
                    None
                }
            }
            SyntaxKind::NumericLiteral => {
                if let NodeData::NumericLiteral(n) = &initializer.data {
                    Some(n.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::PrefixUnaryExpression => {
                // Handle `-1`, `+1` etc.
                if let NodeData::PrefixUnaryExpression(unary) = &initializer.data {
                    let operand_text = match &unary.operand.data {
                        NodeData::NumericLiteral(n) => n.text.clone(),
                        _ => return None,
                    };
                    let op = match unary.operator {
                        SyntaxKind::MinusToken => "-",
                        SyntaxKind::PlusToken => "+",
                        _ => return None,
                    };
                    Some(format!("{}{}", op, operand_text))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Parameter optionality
    // ────────────────────────────────────────────────────────────────────

    /// Check if a parameter is optional.
    ///
    /// Mirrors Go's `EmitResolver.IsOptionalParameter` (emitresolver.go ~L65).
    pub fn is_optional_parameter(&self, node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ParameterDeclaration(data) => {
                // A parameter is optional if it has a question mark or
                // if it's a rest parameter.
                data.question_token.is_some() || node.kind == SyntaxKind::RestType
            }
            _ => false,
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Literal const declarations
    // ────────────────────────────────────────────────────────────────────

    /// Check if a declaration is a literal const declaration.
    ///
    /// Mirrors Go's `EmitResolver.IsLiteralConstDeclaration` (emitresolver.go ~L639).
    /// A `const` declaration with a literal initializer (e.g. `const x = "foo"`)
    /// is a literal const declaration.
    pub fn is_literal_const_declaration(&self, node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::VariableDeclaration {
            return false;
        }
        let NodeData::VariableDeclaration(data) = &node.data else {
            return false;
        };
        // Check if the parent is a const declaration.
        // This requires checking the parent VariableStatement's modifiers,
        // but since we don't have parent pointers, we check if the
        // declaration's type is a literal type.
        if data.initializer.is_none() {
            return false;
        }
        let initializer = data.initializer.as_ref().unwrap();
        matches!(
            initializer.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::PrefixUnaryExpression
        )
    }

    // ────────────────────────────────────────────────────────────────────
    // Constant values
    // ────────────────────────────────────────────────────────────────────

    /// Get the constant value of a node (for enum members, const assertions).
    ///
    /// Mirrors Go's `EmitResolver.GetConstantValue` (emitresolver.go ~L1157).
    pub fn get_constant_value(&mut self, node: &Arc<Node>) -> Option<String> {
        if node.kind == SyntaxKind::EnumMember {
            return self.get_enum_member_value_string(node);
        }
        match node.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(s) = &node.data {
                    Some(format!("\"{}\"", s.text))
                } else {
                    None
                }
            }
            SyntaxKind::NumericLiteral => {
                if let NodeData::NumericLiteral(n) = &node.data {
                    Some(n.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::TrueKeyword => Some("true".to_string()),
            SyntaxKind::FalseKeyword => Some("false".to_string()),
            SyntaxKind::NullKeyword => Some("null".to_string()),
            _ => None,
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Alias / import declarations
    // ────────────────────────────────────────────────────────────────────

    /// Check if an import declaration is referenced (and thus should be emitted).
    ///
    /// Mirrors Go's `EmitResolver.IsReferencedAliasDeclaration` (emitresolver.go ~L689).
    /// Returns true once the alias marking visitor (or a reference) has marked
    /// the declaration visible; otherwise returns true as a conservative
    /// default (the full referenced-alias tracking requires the reference
    /// resolver, which is not yet migrated).
    pub fn is_referenced_alias_declaration(&self, node: &Arc<Node>) -> bool {
        if let Some(links) = self.declaration_links.get(node) {
            if links.is_visible.is_true() {
                return true;
            }
        }
        // Conservative default: keep the alias. This matches Go's behavior
        // when `canCollectSymbolAliasAccessibilityData` is false.
        true
    }

    /// Check if an alias declaration is a value alias (not type-only).
    ///
    /// Mirrors Go's `EmitResolver.IsValueAliasDeclaration` (emitresolver.go ~L715).
    pub fn is_value_alias_declaration(&self, node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ImportSpecifier(data) => !data.is_type_only,
            _ => true,
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Declaration flags
    // ────────────────────────────────────────────────────────────────────

    /// Get the effective modifier flags for a declaration.
    ///
    /// Mirrors Go's `EmitResolver.GetEffectiveDeclarationFlags` (emitresolver.go ~L1143).
    /// Returns the node's combined syntactic modifier flags.
    pub fn get_effective_declaration_flags(&self, node: &Arc<Node>) -> u32 {
        node.syntactic_modifier_flags().bits()
    }

    // ────────────────────────────────────────────────────────────────────
    // Symbol access
    // ────────────────────────────────────────────────────────────────────

    /// Get the symbol of a declaration node.
    ///
    /// Mirrors Go's `Checker.getSymbolOfDeclaration`.
    pub fn get_symbol_of_declaration(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.program.symbol_map().symbol_of(node).cloned()
    }

    /// Check if a symbol is a const enum member.
    pub fn is_const_enum_member(&self, symbol: &Symbol) -> bool {
        symbol.flags.contains(SymbolFlags::ConstEnum)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ────────────────────────────────────────────────────────────────────────────

/// Collect the direct children of `node` into a `Vec`, so the caller can
/// iterate without holding a borrow into `node`'s data (the alias marking
/// visitor mutates `self` while recursing).
fn collect_children(node: &Arc<Node>) -> Vec<Arc<Node>> {
    let mut children: Vec<Arc<Node>> = Vec::new();
    for_each_child(node, |child| {
        children.push(Arc::clone(child));
        false
    });
    children
}
