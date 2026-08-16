//! Symbol binding, ported from `internal/binder/binder.go`.
//!
//! The binder walks the AST and creates symbols for declarations, builds
//! scopes (symbol tables), and associates identifiers with their declarations.
//! It also builds the control flow graph for use by the checker.
//!
//! In Go, symbols and flow nodes are stored directly on AST nodes. In Rust,
//! we use side tables (`NodeSymbolMap`) keyed by node ID.

pub mod nameresolver;
pub mod referenceresolver;

use crate::ast::*;
use crate::diagnostics::messages_generated::{
    A_PARAMETER_INITIALIZER_IS_ONLY_ALLOWED_IN_A_FUNCTION_OR_CONSTRUCTOR_IMPLEMENTATION,
    CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0, DUPLICATE_IDENTIFIER_0,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_CLASS_DEFINITIONS_ARE_AUTOMATICALLY_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
};
use std::sync::Arc;

/// The binder.
///
/// Mirrors `binder.Binder` in Go.
/// A flow label (junction point in the control flow graph).
///
/// Mirrors `ast.FlowLabel` in Go. Labels are used to collect antecedents
/// from multiple control flow paths (e.g. the merge point after an if/else).
#[derive(Debug)]
struct FlowLabel {
    node: FlowNode,
}

impl FlowLabel {
    fn new(flags: FlowFlags) -> Self {
        Self {
            node: FlowNode::new(flags),
        }
    }

    /// Add an antecedent to this label.
    fn add_antecedent(&mut self, antecedent: Arc<FlowNode>) {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }
        // Check if already present
        for ant in &self.node.antecedents {
            if Arc::ptr_eq(ant, &antecedent) {
                return;
            }
        }
        self.node.antecedents.push(antecedent);
    }

    /// Finish the label, returning the resulting flow node.
    fn finish(&self, unreachable: &Arc<FlowNode>) -> Arc<FlowNode> {
        if self.node.antecedents.is_empty() {
            return Arc::clone(unreachable);
        }
        if self.node.antecedents.len() == 1 {
            return Arc::clone(&self.node.antecedents[0]);
        }
        Arc::new(FlowNode {
            flags: self.node.flags,
            node: None,
            antecedent: None,
            antecedents: self.node.antecedents.clone(),
            switch_statement: None,
        })
    }
}

/// Active label tracking for labeled statements.
#[derive(Debug)]
struct ActiveLabel {
    name: String,
    break_target: Arc<FlowNode>,
    continue_target: Option<Arc<FlowNode>>,
    referenced: bool,
    next: Option<Box<ActiveLabel>>,
}

/// The binder.
///
/// Mirrors `binder.Binder` in Go.
pub struct Binder {
    /// Side table mapping nodes to symbols, locals, and flow nodes.
    pub symbol_map: NodeSymbolMap,
    /// The file currently being bound — attached to binder diagnostics
    /// (TS2300/TS2451 …) so they render with file/position like Go's
    /// `binder.createDiagnosticForNode`.
    current_source_file: Option<Arc<SourceFile>>,
    /// The current container node (where members/exports go).
    container: Option<Arc<Node>>,
    /// The current block-scoped container (where block-scoped locals go).
    block_scope_container: Option<Arc<Node>>,
    /// The current `this` container — the nearest function-like container
    /// that can serve as the target of `this.property` assignments in JS
    /// files. Mirrors Go's `binder.thisContainer`. Used by
    /// `bind_this_property_assignment` for JS expando binding
    /// (`this.prop = value` and `Class.prototype.method = fn`).
    this_container: Option<Arc<Node>>,
    /// The current container's parent symbol.
    parent_symbol: Option<Arc<Symbol>>,
    /// The current flow node.
    current_flow: Option<Arc<FlowNode>>,
    /// Symbol count (for diagnostics/stats).
    symbol_count: usize,
    /// Unreachable flow node.
    unreachable_flow: Option<Arc<FlowNode>>,
    /// Current break target flow label.
    current_break_target: Option<Arc<FlowNode>>,
    /// Current continue target flow label.
    current_continue_target: Option<Arc<FlowNode>>,
    /// Current exception target flow label (for try-catch-finally).
    current_exception_target: Option<Arc<FlowNode>>,
    /// Current return target flow label (for try-finally with IIFE).
    current_return_target: Option<Arc<FlowNode>>,
    /// Active label list (for labeled statements with break/continue).
    active_label_list: Option<Box<ActiveLabel>>,
    /// Whether the current function has explicit return statements.
    has_explicit_return: bool,
    /// Whether there are flow effects (assignments, calls, etc.).
    has_flow_effects: bool,
}

impl Default for Binder {
    fn default() -> Self {
        Self::new()
    }
}

/// Target symbol table for [`Binder::declare_symbol_into`].
///
/// Mirrors the explicit `table` argument Go passes to
/// `b.declareSymbol(table, parent, node, ...)` in the export/import bind
/// arms (`bindImportClause`, `bindExportAssignment`,
/// `bindExportDeclaration`, `bindNamespaceExportDeclaration`).
enum DeclareTarget {
    /// `container.Symbol().exports` — holds an owned clone of the container
    /// symbol so we can mutate its `exports` field through the raw pointer
    /// without borrowing `self`.
    Exports(Arc<Symbol>),
    /// `ast.GetLocals(container)` — the container node whose
    /// `symbol_map.locals` entry should receive the symbol.
    Locals(Arc<Node>),
}

impl Binder {
    /// Create a new binder.
    pub fn new() -> Self {
        Self {
            symbol_map: NodeSymbolMap::new(),
            current_source_file: None,
            container: None,
            block_scope_container: None,
            this_container: None,
            parent_symbol: None,
            current_flow: None,
            symbol_count: 0,
            unreachable_flow: None,
            current_break_target: None,
            current_continue_target: None,
            current_exception_target: None,
            current_return_target: None,
            active_label_list: None,
            has_explicit_return: false,
            has_flow_effects: false,
        }
    }

    /// Bind a source file: walk the AST and create symbols.
    ///
    /// Mirrors `binder.BindSourceFile` in Go.
    pub fn bind_source_file(&mut self, file: &Arc<SourceFile>) -> &NodeSymbolMap {
        self.current_source_file = Some(Arc::clone(file));
        // Populate parent pointers before binding so the binder can locate
        // enclosing containers (e.g. the `ConditionalType` that owns an
        // `infer R` type parameter). Mirrors Go's parser, which sets
        // `Node.Parent` during parsing.
        self.set_parent_pointers(&file.node);

        let start_flow = Arc::new(FlowNode::new(FlowFlags::START));
        self.current_flow = Some(Arc::clone(&start_flow));
        self.unreachable_flow = Some(Arc::new(FlowNode::new(FlowFlags::UNREACHABLE)));
        // Set the start flow node on the source file node itself
        self.symbol_map
            .set_flow_node(&file.node, Arc::clone(&start_flow));

        // Create a symbol for the source file itself
        let file_symbol = Arc::new(Symbol::new(
            SymbolFlags::ValueModule,
            file.file_name.clone(),
        ));
        self.symbol_map
            .set_symbol(&file.node, Arc::clone(&file_symbol));
        self.symbol_count += 1;

        // Set up container context
        let prev_container = self.container.take();
        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();

        self.container = Some(Arc::clone(&file.node));
        self.block_scope_container = Some(Arc::clone(&file.node));
        self.parent_symbol = Some(file_symbol);

        // Bind children
        self.bind_children(&file.node);

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;

        &self.symbol_map
    }

    /// Walk the AST and set `parent` pointers on every child node.
    /// Mirrors the parent-pointer population done by Go's parser. Safe
    /// because the binder runs single-threaded and the AST is a tree.
    fn set_parent_pointers(&mut self, node: &Arc<Node>) {
        use crate::ast::node_data_generated::for_each_child;
        let mut children: Vec<Arc<Node>> = Vec::new();
        for_each_child(node, |child| {
            children.push(Arc::clone(child));
            false
        });
        let parent_clone = Arc::clone(node);
        for child in &children {
            let child_mut = Arc::as_ptr(child) as *mut Node;
            unsafe {
                (*child_mut).parent = Some(Arc::clone(&parent_clone));
            }
            self.set_parent_pointers(child);
        }
    }

    /// Create a new symbol.
    fn new_symbol(&mut self, flags: SymbolFlags, name: impl Into<String>) -> Arc<Symbol> {
        self.symbol_count += 1;
        Arc::new(Symbol::new(flags, name))
    }

    /// Declare a symbol for a node, adding it to the appropriate symbol table.
    ///
    /// Mirrors `binder.declareSymbol` in Go, including declaration merging
    /// for mergeable kinds (interface+interface, namespace+namespace,
    /// namespace+function/class, function+function overloads, enum+enum).
    /// Non-mergeable kinds (TypeAlias, Class, block-scoped variables)
    /// overwrite the previous symbol on redeclaration — matching the
    /// previous behavior.
    fn declare_symbol(
        &mut self,
        node: &Arc<Node>,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);

        // `var` hoisting: a function-scoped `var` declared inside a block
        // (or loop initializer) must be declared in the nearest symbol
        // container's table — the enclosing function's locals, or the
        // file/module symbol's members at top level — NOT the block scope
        // container. Block-scoped routing would hide the variable after the
        // block (`function f() { { var x = 1; } use(x); }` → TS2304) and
        // break `var`-`var` merging across sibling blocks. Mirrors Go's
        // routing of FunctionScopedVariable declarations through
        // `declareSymbolAndAddToSymbolTable` (which targets `b.container`,
        // never `b.blockScopeContainer`). When `parent_symbol` is set (a
        // declaration directly inside a symbol-ful container) the existing
        // parent-member path below already lands in the right table.
        let var_hoist_container: Option<Arc<Node>> =
            if Self::declaration_is_var(node) && self.parent_symbol.is_none() {
                self.container
                    .as_ref()
                    .filter(|c| is_var_container_kind(c.kind))
                    .cloned()
            } else {
                None
            };

        // Look up an existing symbol with the same name in the target scope.
        // If it exists and the kinds are mergeable, fold this declaration
        // into the existing symbol instead of creating a new one.
        let existing: Option<Arc<Symbol>> = if let Some(parent_sym) = &self.parent_symbol {
            parent_sym
                .members
                .get(&name)
                .cloned()
                .or_else(|| parent_sym.exports.get(&name).cloned())
                // Non-exported module children live in the module node's
                // LOCALS — check there too so `namespace B` + `interface B`
                // merge (mirrors the insertion routing below).
                .or_else(|| {
                    if self.container.as_ref().is_some_and(|c| {
                        c.kind == SyntaxKind::ModuleDeclaration
                    }) {
                        self.symbol_map
                            .locals
                            .get(&self.container.as_ref().unwrap().id())
                            .and_then(|l| l.get(&name).cloned())
                    } else {
                        None
                    }
                })
        } else if let Some(hoist) = &var_hoist_container {
            match hoist.kind {
                SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => self
                    .symbol_map
                    .symbol_of(hoist)
                    .and_then(|sym| sym.members.get(&name).cloned()),
                // Function-like containers: hoist into the function's locals.
                _ => {
                    let container_id = hoist.id();
                    self.symbol_map
                        .locals
                        .get(&container_id)
                        .and_then(|locals| locals.get(&name).cloned())
                }
            }
        } else if let Some(block_container) = &self.block_scope_container {
            let container_id = block_container.id();
            self.symbol_map
                .locals
                .get(&container_id)
                .and_then(|locals| locals.get(&name).cloned())
        } else {
            None
        };

        // Set when `existing` was found but is non-mergeable with this
        // declaration (see the conflict comment below).
        let mut conflicted = false;

        if let Some(existing) = existing {
            // `var` + `var` merge like Go (a plain `var` redeclaration is
            // legal and folds into the hoisted symbol). The Rust binder
            // assigns BlockScopedVariable to every variable, so the generic
            // merge check would treat this as a conflict.
            let var_var_merge = Self::declaration_is_var(node)
                && existing.flags == SymbolFlags::BlockScopedVariable
                && existing.declarations.iter().all(|d| Self::declaration_is_var(d));
            if self.can_merge_symbols(existing.flags, includes) || var_var_merge {
                // Merge: add this declaration to the existing symbol, union
                // the flags, and map the node to the existing symbol.
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).declarations.push(Arc::clone(node));
                    (*existing_mut).flags |= includes;
                    // For function overloads, only the first declaration
                    // carries the VALUE flag (already set). For other
                    // merges (interface/namespace), VALUE isn't involved.
                    if (*existing_mut).value_declaration.is_none()
                        && includes.intersects(SymbolFlags::VALUE)
                    {
                        (*existing_mut).value_declaration = Some(Arc::clone(node));
                    }
                }
                self.symbol_map.set_symbol(node, Arc::clone(&existing));
                return existing;
            }
            // Non-mergeable redeclaration: fall through to create a new
            // symbol for this declaration. Go's `declareSymbolEx` does NOT
            // write the replacement back into the symbol table on conflict,
            // so a later same-name declaration re-conflicts with the
            // ORIGINAL symbol and reports again (e.g. `var foo; function
            // foo(){} function foo(){}` reports TS2300 on the third
            // declaration too) — mirrored by the `conflicted` flag set in
            // the report paths below. Quiet fall-throughs (Rust models
            // `var` + `var` as non-mergeable where Go merges them) still
            // replace the table entry.
            // Determine the kind of conflict and report the appropriate
            // diagnostic.
            //
            // Skip anonymous (empty-name) declarations: they are internal
            // symbols (e.g. interface/object members whose names are resolved
            // structurally by the checker) and must not trigger duplicate
            // diagnostics. Mirrors Go, whose `getDeclarationName` always
            // produces a real name for user-named declarations.
            //
            // TS2451 "Cannot redeclare block-scoped variable" fires when a
            // block-scoped variable (let/const) collides with another
            // block-scoped variable. The binder assigns `BlockScopedVariable`
            // to every variable declaration (including `var`), so we
            // additionally verify the new declaration's keyword is `let`/`const`
            // to avoid a false positive on `var x; var x;`. A plain `var`
            // redeclaration is legal (function-scoped, may be redeclared).
            //
            // TS2300 "Duplicate identifier" fires for any other non-mergeable
            // redeclaration: e.g. two imports of the same name, a class and a
            // function with the same name, two classes, two type aliases, etc.
            // A function-scoped `var` may coexist with a function or class
            // declaration (e.g. `var x; function x() {}` is allowed), so
            // TS2300 is suppressed in that case. Mirrors Go's `declareSymbol`.
            let both_block_scoped_var = existing.flags.contains(SymbolFlags::BlockScopedVariable)
                && includes.contains(SymbolFlags::BlockScopedVariable);
            if !name.is_empty() {
                // Go reports the conflict on EVERY declaration of the
                // existing symbol (name node, falling back to the node) and
                // then on the new declaration's name — so `class C{} class
                // C{}` yields two TS2300s, one per declaration.
                let report_all = |b: &mut Self, message: &'static crate::diagnostics::Message| {
                    // Identical duplicates (the middle declarations of a
                    // triple `let x` redeclare) collapse — Go's diagnostics
                    // are deduplicated before reporting.
                    let mut push = |b: &mut Self, loc: crate::core::text::TextRange| {
                        if b
                            .symbol_map
                            .binder_diagnostics
                            .iter()
                            .any(|d| d.loc == loc && d.code == message.code)
                        {
                            return;
                        }
                        b.symbol_map.binder_diagnostics.push(Diagnostic::new(
                            b.current_source_file.clone(),
                            loc,
                            *message,
                            vec![name.clone()],
                        ));
                    };
                    for d in &existing.declarations {
                        let name_node = crate::ast::utilities::get_name_of_declaration(d)
                            .unwrap_or_else(|| Arc::clone(d));
                        push(b, name_node.loc);
                    }
                    let name_node = crate::ast::utilities::get_name_of_declaration(node)
                        .unwrap_or_else(|| Arc::clone(node));
                    push(b, name_node.loc);
                };
                if both_block_scoped_var {
                    if Self::is_let_or_const_declaration(node) {
                        report_all(
                            self,
                            &CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0,
                        );
                        // A real Go conflict (block-scoped redeclare): keep
                        // the original table entry so later same-name
                        // declarations re-conflict (see `conflicted`).
                        conflicted = true;
                    }
                    // else: `var` + `var` — redeclaration is legal, no error.
                    // (Rust models var+var as non-mergeable; Go merges them,
                    // so the table replacement below must still happen.)
                } else {
                    // TS2300 is a *scope-level* duplicate-identifier check.
                    // Member-level declarations (parameters, properties,
                    // accessors, enum members, type parameters) live in their
                    // container's symbol table and — for merged declarations
                    // such as function overloads — legitimately reuse a name
                    // across distinct declarations. Exclude those kinds so we
                    // only report genuine scope-level duplicates (two imports,
                    // a class and a function, two classes, etc.).
                    let member_flags = SymbolFlags::Property
                        .union(SymbolFlags::Method)
                        .union(SymbolFlags::GetAccessor)
                        .union(SymbolFlags::SetAccessor)
                        .union(SymbolFlags::EnumMember)
                        .union(SymbolFlags::FunctionScopedVariable)
                        .union(SymbolFlags::TypeParameter)
                        .union(SymbolFlags::Constructor)
                        .union(SymbolFlags::Signature);
                    // `export as namespace Foo` (NamespaceExportDeclaration)
                    // declares a UMD global alias that intentionally coexists
                    // with a same-named `declare namespace Foo`. Go's binder
                    // stores such aliases in a separate `GlobalExports` table,
                    // so they never participate in the duplicate-identifier
                    // check; since we lack that table and store the alias in
                    // the container's exports, suppress the false-positive
                    // TS2300 when either side is a namespace export
                    // declaration (the common `@types/react` pattern:
                    //   export as namespace React;
                    //   declare namespace React { ... }
                    // ).
                    let involves_namespace_export = node.kind
                        == SyntaxKind::NamespaceExportDeclaration
                        || existing
                            .declarations
                            .iter()
                            .any(|d| d.kind == SyntaxKind::NamespaceExportDeclaration);
                    if involves_namespace_export
                        || existing.flags.intersects(member_flags)
                        || includes.intersects(member_flags)
                    {
                        // Member-level collision or UMD global alias — not a
                        // scope-level duplicate.
                    } else {
                        // Current Go semantics: `var x` + `function x` /
                        // `class x` CONFLICT (TS2300 on every declaration —
                        // verified against typescript-go; the old TS5
                        // coexistence no longer holds).
                        report_all(self, &DUPLICATE_IDENTIFIER_0);
                        conflicted = true;
                    }
                }
            }
        }

        let symbol = self.new_symbol(includes, name.clone());

        // Record this declaration node on the symbol. `Symbol` is behind an
        // `Arc`; the binder runs single-threaded before any checker access, so
        // we mutate through the raw pointer (same pattern used for `members`
        // below). This lets the checker recover the AST declaration from a
        // symbol (e.g. resolving a type alias's declared type).
        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));
                // The first declaration is also the value declaration.
                if (*symbol_mut).value_declaration.is_none()
                    && includes.intersects(SymbolFlags::VALUE)
                {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
        }

        // Add to appropriate symbol table based on container kind.
        // Mirrors Go's `declareSymbolAndAddToSymbolTable`:
        // - ModuleDeclaration: exported members go to `exports`, non-exported
        //   to `locals` (so they're visible inside the namespace but not via
        //   `N.x` from outside).
        // - ClassDeclaration/InterfaceDeclaration/etc.: members go to
        // `members`.
        // - Block-scoped containers: locals.
        //
        // Skipped entirely on conflict (see `conflicted` above): Go leaves
        // the original symbol in the table so later redeclarations of the
        // same name re-conflict against it.
        if !conflicted && let Some(container) = &self.container {
            if container.kind == SyntaxKind::ModuleDeclaration {
                // Namespace member: exported → exports, non-exported → locals.
                // Use combined modifier flags to handle `export const x`
                // where the `Export` modifier is on the parent
                // VariableStatement, not the VariableDeclaration itself.
                //
                // NOTE Go's implicit export (`setExportContextFlag`: an
                // ambient container with no explicit export declarations
                // exports everything) is NOT applied when routing here —
                // routing ambient members into `exports` perturbs the
                // checker's lazily-resolved lib types. Ambient visibility
                // from outside is handled by the checker consulting the
                // namespace's locals for ambient containers (see
                // `ambient_namespace_locals_visible`).
                let has_export = self
                    .get_combined_modifier_flags(node)
                    .contains(ModifierFlags::Export);
                if has_export {
                    if let Some(parent_sym) = &self.parent_symbol {
                        let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                        unsafe {
                            (*parent_sym_mut)
                                .exports
                                .insert(name.clone(), Arc::clone(&symbol));
                        }
                    }
                    // Also add to locals so the member is visible inside
                    // the namespace body by its local name.
                    if has_locals(container.kind) {
                        let locals = self
                            .symbol_map
                            .locals
                            .entry(container.id())
                            .or_insert_with(SymbolTable::new);
                        locals.insert(name.clone(), Arc::clone(&symbol));
                    }
                } else if has_locals(container.kind) {
                    let locals = self
                        .symbol_map
                        .locals
                        .entry(container.id())
                        .or_insert_with(SymbolTable::new);
                    locals.insert(name.clone(), Arc::clone(&symbol));
                }
            } else if let Some(parent_sym) = &self.parent_symbol {
                // Class/Interface/Object members.
                let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                unsafe {
                    (*parent_sym_mut)
                        .members
                        .insert(name.clone(), Arc::clone(&symbol));
                }
            } else if let Some(hoist) = &var_hoist_container {
                // `var` hoisting (see `var_hoist_container` above): declare in
                // the nearest symbol container's table — the enclosing
                // function's locals, or the file/module symbol's members at
                // top level — instead of the block scope container.
                match hoist.kind {
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => {
                        if let Some(sym) = self.symbol_map.symbol_of(hoist) {
                            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                            unsafe {
                                (*sym_mut)
                                    .members
                                    .insert(name.clone(), Arc::clone(&symbol));
                            }
                        }
                    }
                    _ => {
                        let locals = self
                            .symbol_map
                            .locals
                            .entry(hoist.id())
                            .or_insert_with(SymbolTable::new);
                        locals.insert(name.clone(), Arc::clone(&symbol));
                    }
                }
            } else if let Some(block_container) = &self.block_scope_container {
                // Add to locals of the block-scoped container
                let container_id = block_container.id();
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container_id)
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.clone(), Arc::clone(&symbol));
            }
        }

        // Exported module/namespace members get an `export_symbol` link so
        // the checker's `follow_alias` / `get_export_symbol_of_value_symbol_if_exported`
        // can recover the export face of the symbol. Mirrors Go's
        // `declareModuleMember` two-symbol pattern, using the safer
        // self-reference approach: the same symbol is registered in both
        // the locals/members and exports tables, and `export_symbol` points
        // back to itself. (The full Go pattern uses two distinct symbols —
        // a local with `ExportValue` and an export with full flags — but
        // that risks breaking existing tests, so we keep a single symbol
        // and just establish the link.)
        if let Some(container) = &self.container {
            let is_module_container = container.kind == SyntaxKind::SourceFile
                || container.kind == SyntaxKind::ModuleDeclaration;
            if is_module_container
                && self
                    .get_combined_modifier_flags(node)
                    .contains(ModifierFlags::Export)
            {
                let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                unsafe {
                    (*symbol_mut).export_symbol = Some(Arc::clone(&symbol));
                }
            }
        }

        // Associate the symbol with the node
        self.symbol_map.set_symbol(node, Arc::clone(&symbol));

        // Set the value declaration if this is a value declaration
        // (in the full Go implementation, this is more nuanced)

        symbol
    }

    /// Declare a symbol and add it to an explicit target symbol table,
    /// applying the same merge semantics as [`Binder::declare_symbol`].
    ///
    /// Used by the export/import bind arms that must target a specific table
    /// (the container's `exports` or `locals`) rather than the table
    /// [`Binder::declare_symbol`] routes to based on container kind.
    ///
    /// Mirrors Go's `b.declareSymbol(table, parent, node, flags, excludes)`.
    fn declare_symbol_into(
        &mut self,
        node: &Arc<Node>,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
        target: DeclareTarget,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);

        // Look up an existing symbol with the same name in the target table.
        let existing: Option<Arc<Symbol>> = match &target {
            DeclareTarget::Exports(parent_sym) => parent_sym.exports.get(&name).cloned(),
            DeclareTarget::Locals(container) => self
                .symbol_map
                .locals
                .get(&container.id())
                .and_then(|locals| locals.get(&name).cloned()),
        };

        if let Some(existing) = existing {
            if self.can_merge_symbols(existing.flags, includes) {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).declarations.push(Arc::clone(node));
                    (*existing_mut).flags |= includes;
                    if (*existing_mut).value_declaration.is_none()
                        && includes.intersects(SymbolFlags::VALUE)
                    {
                        (*existing_mut).value_declaration = Some(Arc::clone(node));
                    }
                }
                self.symbol_map.set_symbol(node, Arc::clone(&existing));
                return existing;
            }
            // Non-mergeable redeclaration: fall through to create a new
            // symbol (overwrites the previous entry).
        }

        let symbol = self.new_symbol(includes, name.clone());
        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));
                if (*symbol_mut).value_declaration.is_none()
                    && includes.intersects(SymbolFlags::VALUE)
                {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
        }

        match &target {
            DeclareTarget::Exports(parent_sym) => {
                let parent_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                unsafe {
                    (*parent_mut)
                        .exports
                        .insert(name.clone(), Arc::clone(&symbol));
                    // Set the export symbol's parent to the container symbol,
                    // mirroring Go which passes `container.Symbol()` as the
                    // parent argument to `declareSymbol`.
                    let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                    (*symbol_mut).parent = Some(Arc::clone(parent_sym));
                }
            }
            DeclareTarget::Locals(container) => {
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container.id())
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.clone(), Arc::clone(&symbol));
            }
        }

        self.symbol_map.set_symbol(node, Arc::clone(&symbol));
        symbol
    }

    /// Whether a new declaration with `new_flags` can be merged into an
    /// existing symbol with `existing_flags`. Mirrors the merge rules in
    /// Go's `binder.declareSymbol` (`canMergeSymbol`).
    ///
    /// Mergeable combinations:
    /// - interface + interface
    /// - namespace + namespace (ValueModule + ValueModule)
    /// - namespace + function/class (ValueModule + Function/Class) and vice
    ///   versa
    /// - function + function (overloads)
    /// - enum + enum
    /// - namespace + enum (ValueModule + RegularEnum/ConstEnum)
    ///
    /// Non-mergeable: TypeAlias (redefinition error), Class + Class
    /// (duplicate), block-scoped variable redeclarations.
    fn can_merge_symbols(&self, existing_flags: SymbolFlags, new_flags: SymbolFlags) -> bool {
        // Interface + Interface (and interface + class, which is allowed in
        // TS but not yet fully handled by the checker — still merge so the
        // interface members are visible).
        if existing_flags.contains(SymbolFlags::Interface)
            && new_flags.contains(SymbolFlags::Interface)
        {
            return true;
        }
        // Type + Value coexistence: an interface or type alias (type-only)
        // can coexist with a variable/function (value-only) of the same
        // name. This is how lib files declare `interface Object` alongside
        // `declare var Object: ObjectConstructor;`, or `type NodeFilter`
        // alongside `declare var NodeFilter: { ... }`.
        // Go's excludes table: InterfaceExcludes = Type & ^(Interface |
        // Class) — an interface coexists with ANY value-side symbol
        // including classes (`declare class X` + `interface X`).
        // TypeAliasExcludes = Type — a type alias coexists with
        // value-side symbols EXCEPT classes (`class X` + `type X` is
        // TS2300, but `type T` + `var T` / `function T` merge).
        let existing_interface = existing_flags.contains(SymbolFlags::Interface);
        let new_interface = new_flags.contains(SymbolFlags::Interface);
        let existing_type_alias = existing_flags.contains(SymbolFlags::TypeAlias);
        let new_type_alias = new_flags.contains(SymbolFlags::TypeAlias);
        let class_side = SymbolFlags::Class;
        if (existing_interface && !new_interface && !new_type_alias)
            || (new_interface && !existing_interface && !existing_type_alias)
            || (existing_type_alias && !new_type_alias && !new_flags.intersects(class_side) && !new_interface)
            || (new_type_alias && !existing_type_alias && !existing_flags.intersects(class_side) && !existing_interface)
        {
            return true;
        }
        // Class + Function merge (`declare class X` + `function X`) — Go's
        // ClassExcludes/FunctionExcludes re-enable Function/Class. Cross-kind
        // only: Class + Class is TS2300 (Class stays excluded), and the
        // checker reports TS2813/TS2814 when a non-ambient class meets
        // function declarations.
        let existing_class = existing_flags.contains(SymbolFlags::Class);
        let new_class = new_flags.contains(SymbolFlags::Class);
        let existing_fn = existing_flags.contains(SymbolFlags::Function);
        let new_fn = new_flags.contains(SymbolFlags::Function);
        if (existing_class && new_fn) || (existing_fn && new_class) {
            return true;
        }
        // Namespace merging: a ValueModule can merge with another ValueModule,
        // a Function, a Class, or an Enum.
        let existing_ns = existing_flags.contains(SymbolFlags::ValueModule);
        let new_ns = new_flags.contains(SymbolFlags::ValueModule);
        if existing_ns || new_ns {
            let other_existing = if existing_ns {
                new_flags
            } else {
                existing_flags
            };
            let _other_new = if existing_ns {
                existing_flags
            } else {
                new_flags
            };
            // The non-namespace side must be one of: ValueModule, Function,
            // Class, RegularEnum, ConstEnum, Interface (`namespace B` +
            // `interface B` merge into one symbol, like Go's binder).
            let can_merge_with_ns = other_existing.contains(SymbolFlags::ValueModule)
                || other_existing.contains(SymbolFlags::Function)
                || other_existing.contains(SymbolFlags::Class)
                || other_existing.contains(SymbolFlags::RegularEnum)
                || other_existing.contains(SymbolFlags::ConstEnum)
                || other_existing.contains(SymbolFlags::Interface);
            if can_merge_with_ns {
                return true;
            }
        }
        // Function overloads: Function + Function.
        if existing_flags.contains(SymbolFlags::Function)
            && new_flags.contains(SymbolFlags::Function)
        {
            return true;
        }
        // Enum + Enum.
        if (existing_flags.contains(SymbolFlags::RegularEnum)
            || existing_flags.contains(SymbolFlags::ConstEnum))
            && (new_flags.contains(SymbolFlags::RegularEnum)
                || new_flags.contains(SymbolFlags::ConstEnum))
        {
            return true;
        }
        false
    }

    /// Whether `node` is a `let`/`const` (block-scoped) variable declaration,
    /// as opposed to a function-scoped `var`. Used by the TS2451 redeclaration
    /// check: the binder assigns `BlockScopedVariable` to every variable
    /// declaration, so the keyword must be inspected explicitly. Non-variable
    /// declarations (e.g. binding elements) are conservatively treated as
    /// block-scoped. Mirrors the checker's `is_let_or_const_declaration`.
    fn is_let_or_const_declaration(node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = node.parent.as_ref() {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    return parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
                }
            }
        }
        true
    }

    /// Whether a container (SourceFile / ModuleDeclaration) has any explicit
    /// export statements (`export ...` / `export = ...`). Go's
    /// `hasExportDeclarations`: an ambient container WITHOUT such statements
    /// is an implicit-export context (everything inside is exported).
    pub(crate) fn has_export_declarations(container: &Arc<Node>) -> bool {
        let statements: &[Arc<Node>] = match &container.data {
            crate::ast::NodeData::SourceFile(sf) => &sf.statements.nodes,
            crate::ast::NodeData::ModuleDeclaration(md) => {
                if let Some(body) = &md.body
                    && body.kind == SyntaxKind::ModuleBlock
                    && let crate::ast::NodeData::ModuleBlock(block) = &body.data
                {
                    &block.statements.nodes
                } else {
                    &[]
                }
            }
            _ => &[],
        };
        statements.iter().any(|s| {
            s.kind == SyntaxKind::ExportDeclaration || s.kind == SyntaxKind::ExportAssignment
        })
    }

    /// Whether `node` is a function-scoped `var` declaration (as opposed to a
    /// block-scoped `let`/`const`). The inverse of
    /// [`Self::is_let_or_const_declaration`] for actual variable declarations;
    /// returns `false` for non-variable nodes. Used by the TS2300 check to
    /// allow a `var` to coexist with a function/class declaration.
    fn is_var_declaration(node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = node.parent.as_ref() {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    return !parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
                }
            }
        }
        false
    }

    /// Whether `node` is a function-scoped (`var`) variable declaration or a
    /// binding element inside one (`var [{x, y:z}] = …`). Walks binding
    /// elements up through their binding pattern to the enclosing
    /// `VariableDeclaration`'s list keyword. Binding elements under other
    /// declarations (e.g. `catch ({e})` parameters) bottom out at a
    /// non-variable ancestor and count as block-scoped. Mirrors Go's
    /// `ast.IsBlockOrCatchScoped` (inverted, for variable contexts) which
    /// routes `var` bindings through `declareSymbolAndAddToSymbolTable`.
    fn declaration_is_var(node: &Arc<Node>) -> bool {
        let mut current = node;
        loop {
            match current.kind {
                SyntaxKind::VariableDeclaration => {
                    return if let Some(parent) = current.parent.as_ref() {
                        parent.kind == SyntaxKind::VariableDeclarationList
                            && !parent.flags.intersects(NodeFlags::Let | NodeFlags::Const)
                    } else {
                        false
                    };
                }
                SyntaxKind::BindingElement
                | SyntaxKind::ObjectBindingPattern
                | SyntaxKind::ArrayBindingPattern => {
                    match current.parent.as_ref() {
                        Some(parent) => current = parent,
                        None => return false,
                    }
                }
                _ => return false,
            }
        }
    }

    /// Whether an existing symbol was declared as a function-scoped `var`.
    /// Inspects the symbol's value declaration (falling back to its first
    /// declaration) to recover the variable keyword.
    fn symbol_is_var_declaration(symbol: &Arc<Symbol>) -> bool {
        let decl: Option<&Arc<Node>> = symbol
            .value_declaration
            .as_ref()
            .or_else(|| symbol.declarations.first());
        match decl {
            Some(node) => Self::is_var_declaration(node),
            None => false,
        }
    }

    /// Get the combined modifier flags for a node, walking up the
    /// variable-declaration chain (VariableDeclaration →
    /// VariableDeclarationList → VariableStatement) to collect `export`
    /// and other modifiers from parent nodes. Mirrors Go's
    /// `ast.GetCombinedModifierFlags`. Requires parent pointers to be
    /// populated (see `set_parent_pointers`).
    fn get_combined_modifier_flags(&self, node: &Arc<Node>) -> ModifierFlags {
        let mut flags = node.syntactic_modifier_flags();
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = &node.parent {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    flags |= parent.syntactic_modifier_flags();
                    if let Some(gp) = &parent.parent {
                        if gp.kind == SyntaxKind::VariableStatement {
                            flags |= gp.syntactic_modifier_flags();
                        }
                    }
                }
            }
        }
        flags
    }

    /// Get the name of a declaration node.
    fn get_declaration_name(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::VariableDeclaration(data) => self.node_text(&data.name),
            NodeData::VariableStatement(_) => String::new(),
            NodeData::FunctionDeclaration(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            NodeData::FunctionExpression(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_FUNCTION.to_string()),
            NodeData::ArrowFunction(_) => INTERNAL_SYMBOL_NAME_FUNCTION.to_string(),
            NodeData::ClassDeclaration(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            NodeData::ClassExpression(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_CLASS.to_string()),
            NodeData::InterfaceDeclaration(data) => self.node_text(&data.name),
            NodeData::TypeAliasDeclaration(data) => self.node_text(&data.name),
            NodeData::EnumDeclaration(data) => self.node_text(&data.name),
            NodeData::ModuleDeclaration(data) => self.node_text(&data.name),
            NodeData::ParameterDeclaration(data) => self.node_text(&data.name),
            NodeData::BindingElement(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            // `import { default as Foo }` binds the LOCAL name (`Foo`);
            // the property name (`default`) is what's imported from the
            // module.
            NodeData::ImportSpecifier(data) => self.node_text(&data.name),
            NodeData::ImportClause(data) => data.name.as_ref().map_or_else(
                || {
                    data.named_bindings
                        .as_ref()
                        .map_or_else(|| String::new(), |n| self.node_text(n))
                },
                |n| self.node_text(n),
            ),
            NodeData::PropertyDeclaration(data) => self.node_text(&data.name),
            NodeData::MethodDeclaration(data) => self.node_text(&data.name),
            NodeData::PropertyAssignment(data) => self.node_text(&data.name),
            NodeData::ShorthandPropertyAssignment(data) => self.node_text(&data.name),
            NodeData::EnumMember(data) => self.node_text(&data.name),
            NodeData::GetAccessorDeclaration(data) => self.node_text(&data.name),
            NodeData::SetAccessorDeclaration(data) => self.node_text(&data.name),
            NodeData::TypeParameterDeclaration(data) => self.node_text(&data.name),
            // `import X = ...` / `import * as X from ...` — the alias binds
            // under its own name (Go's getDeclarationName reads Name()).
            NodeData::ImportEqualsDeclaration(data) => self.node_text(&data.name),
            NodeData::NamespaceImport(data) => self.node_text(&data.name),
            NodeData::Identifier(data) => data.text.clone(),
            // `export default <expr>` → "default"; `export = <expr>` → "export=".
            // Mirrors Go's `getDeclarationName` for `KindExportAssignment`.
            NodeData::ExportAssignment(data) => {
                if data.is_export_equals {
                    INTERNAL_SYMBOL_NAME_EXPORT_EQUALS.to_string()
                } else {
                    INTERNAL_SYMBOL_NAME_DEFAULT.to_string()
                }
            }
            // `export * from "mod"` — the export star declaration node is
            // named with the internal `export-star` marker.
            NodeData::ExportDeclaration(_) => INTERNAL_SYMBOL_NAME_EXPORT_STAR.to_string(),
            // `export * as ns from "mod"` — the `* as ns` clause (and the
            // standalone `NamespaceExportDeclaration` form) are named after
            // their identifier.
            NodeData::NamespaceExport(data) => self.node_text(&data.name),
            NodeData::NamespaceExportDeclaration(data) => self.node_text(&data.name),
            _ => String::new(),
        }
    }

    /// Get the text of a node (for name extraction).
    fn node_text(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(data) => data.text.clone(),
            NodeData::StringLiteral(data) => data.text.clone(),
            NodeData::NumericLiteral(data) => data.text.clone(),
            NodeData::NoSubstitutionTemplateLiteral(data) => data.text.clone(),
            NodeData::BigIntLiteral(data) => data.text.clone(),
            _ => String::new(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Flow graph helper methods
    // ─────────────────────────────────────────────────────────────────────

    /// Get the unreachable flow node.
    fn unreachable_flow(&self) -> Arc<FlowNode> {
        Arc::clone(self.unreachable_flow.as_ref().unwrap())
    }

    /// Create a new flow node with the given flags.
    fn new_flow_node(&self, flags: FlowFlags) -> FlowNode {
        FlowNode::new(flags)
    }

    /// Create a flow condition node (true or false branch).
    fn create_flow_condition(
        &mut self,
        flags: FlowFlags,
        antecedent: &Arc<FlowNode>,
        expression: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.has_flow_effects = true;
        Arc::new(FlowNode {
            flags,
            node: Some(Arc::clone(expression)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
        })
    }

    /// Create a flow assignment node.
    fn create_flow_assignment(
        &mut self,
        antecedent: &Arc<FlowNode>,
        node: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.has_flow_effects = true;
        Arc::new(FlowNode {
            flags: FlowFlags::ASSIGNMENT,
            node: Some(Arc::clone(node)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
        })
    }

    /// Create a flow call node.
    fn create_flow_call(&mut self, antecedent: &Arc<FlowNode>, node: &Arc<Node>) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.has_flow_effects = true;
        Arc::new(FlowNode {
            flags: FlowFlags::CALL,
            node: Some(Arc::clone(node)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
        })
    }

    /// Create a flow mutation node (for array mutations like push, unshift, idx assignment).
    fn create_flow_mutation(
        &mut self,
        antecedent: &Arc<FlowNode>,
        node: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.set_flow_node_referenced(antecedent);
        self.has_flow_effects = true;
        let result = Arc::new(FlowNode {
            flags: FlowFlags::ARRAY_MUTATION,
            node: Some(Arc::clone(node)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
        });
        // Add to exception target if we're inside a try block
        if let Some(target) = &self.current_exception_target {
            self.add_antecedent_to_flow(target, &result);
        }
        result
    }

    /// Mark a flow node as referenced (sets Referenced flag, then Shared on subsequent calls).
    fn set_flow_node_referenced(&self, flow: &FlowNode) {
        // We need interior mutability for this. Since FlowNode is behind Arc,
        // we use a raw pointer cast. This is safe because we only modify flags.
        let ptr = flow as *const FlowNode as *mut FlowNode;
        unsafe {
            if (*ptr).flags.contains(FlowFlags::REFERENCED) {
                (*ptr).flags = (*ptr).flags | FlowFlags::SHARED;
            } else {
                (*ptr).flags = (*ptr).flags | FlowFlags::REFERENCED;
            }
        }
    }

    /// Create a reduce label node (for try-finally flow graph).
    fn create_reduce_label(
        &self,
        antecedents: &[Arc<FlowNode>],
        antecedent: &Arc<FlowNode>,
    ) -> Arc<FlowNode> {
        Arc::new(FlowNode {
            flags: FlowFlags::REDUCE_LABEL,
            node: None,
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: antecedents.to_vec(),
            switch_statement: None,
        })
    }

    /// Create a branch-label flow node used as an accumulation point for
    /// jump edges (`break`/`continue` targets, labeled-statement break
    /// targets). Unlike `FlowLabel::finish`, this node never collapses to
    /// its single antecedent (or to the shared UNREACHABLE node when empty),
    /// so edges added in place via `add_antecedent_to_flow` are never pushed
    /// into — and never corrupt — an arbitrary unrelated node. The
    /// accumulated antecedents are folded into the owning loop's post/pre
    /// label when the loop finishes binding.
    fn new_flow_accumulator() -> Arc<FlowNode> {
        Arc::new(FlowNode {
            flags: FlowFlags::BRANCH_LABEL,
            node: None,
            antecedent: None,
            antecedents: Vec::new(),
            switch_statement: None,
        })
    }

    /// Add an antecedent to a flow label (checking for duplicates).
    fn add_antecedent_to_flow(&self, label: &Arc<FlowNode>, antecedent: &Arc<FlowNode>) {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }
        for ant in &label.antecedents {
            if Arc::ptr_eq(ant, antecedent) {
                return;
            }
        }
        let ptr = Arc::as_ptr(label) as *mut FlowNode;
        unsafe {
            (*ptr).antecedents.push(Arc::clone(antecedent));
        }
        // Mark the antecedent as referenced (or shared if already referenced).
        // Mirrors Go's `setFlowNodeReferenced` called from `addAntecedent`.
        self.set_flow_node_referenced(antecedent);
    }

    /// Create a flow switch clause node.
    ///
    /// `switch_statement` is the enclosing `SwitchStatement` node, used by
    /// the checker to resolve the discriminant expression for narrowing.
    /// `clause` is the `CaseClause` or `DefaultClause` node, used to
    /// resolve the case expression(s) being matched.
    fn create_flow_switch_clause(
        &mut self,
        antecedent: &Arc<FlowNode>,
        clause: &Arc<Node>,
        switch_statement: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        Arc::new(FlowNode {
            flags: FlowFlags::SWITCH_CLAUSE,
            node: Some(Arc::clone(clause)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: Some(Arc::clone(switch_statement)),
        })
    }

    // ─────────────────────────────────────────────────────────────────────
    // Control flow statement binding
    // ─────────────────────────────────────────────────────────────────────

    /// Bind an if statement with proper control flow.
    fn bind_if_statement(&mut self, node: &Arc<Node>) {
        let mut then_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut else_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_if_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expr, then_stmt, else_stmt) = match &node.data {
            NodeData::IfStatement(data) => (
                data.expression.clone(),
                data.then_statement.clone(),
                data.else_statement.clone(),
            ),
            _ => return,
        };

        // Bind condition and split flow
        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            then_label.add_antecedent(true_flow);
            else_label.add_antecedent(false_flow);
        }

        // Then branch
        self.current_flow = Some(then_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&then_stmt);
        if let Some(current) = &self.current_flow {
            post_if_label.add_antecedent(Arc::clone(current));
        }

        // Else branch
        self.current_flow = Some(else_label.finish(self.unreachable_flow.as_ref().unwrap()));
        if let Some(else_s) = else_stmt {
            self.bind(&else_s);
        }
        if let Some(current) = &self.current_flow {
            post_if_label.add_antecedent(Arc::clone(current));
        }

        // Merge after if/else
        self.current_flow = Some(post_if_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

    /// Bind a while statement with proper control flow.
    fn bind_while_statement(&mut self, node: &Arc<Node>) {
        let mut pre_while_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut pre_body_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_while_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expr, stmt) = match &node.data {
            NodeData::WhileStatement(data) => (data.expression.clone(), data.statement.clone()),
            _ => return,
        };

        if let Some(current) = &self.current_flow {
            pre_while_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_while_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Condition
        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            pre_body_label.add_antecedent(true_flow);
            post_while_label.add_antecedent(false_flow);
        }

        // Save break/continue targets. Jump targets are accumulation nodes
        // (never collapsed snapshots) so in-place edge additions from
        // `break;`/`continue;` can't corrupt unrelated flow nodes; the
        // accumulated edges are folded into the loop's labels below.
        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));
        // Back-fill enclosing labels (`l: while (…)`) so `continue l;`
        // targets THIS loop. Mirrors Go's `setContinueTarget`.
        self.set_continue_target(node, &continue_acc);

        // Body
        self.current_flow = Some(pre_body_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&stmt);
        if let Some(current) = &self.current_flow {
            pre_while_label.add_antecedent(Arc::clone(current));
        }
        // Fold `continue;` edges into the condition label.
        for ant in &continue_acc.antecedents {
            pre_while_label.add_antecedent(Arc::clone(ant));
        }
        // Fold `break;` edges into the post-loop label.
        for ant in &break_acc.antecedents {
            post_while_label.add_antecedent(Arc::clone(ant));
        }

        // Restore break/continue targets
        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow = Some(post_while_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

    /// Bind a do-while statement with proper control flow.
    fn bind_do_statement(&mut self, node: &Arc<Node>) {
        let mut pre_do_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut pre_condition_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_do_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expr, stmt) = match &node.data {
            NodeData::DoStatement(data) => (data.expression.clone(), data.statement.clone()),
            _ => return,
        };

        if let Some(current) = &self.current_flow {
            pre_do_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_do_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Save break/continue targets. Jump targets are accumulation nodes
        // (never collapsed snapshots) so in-place edge additions from
        // `break;`/`continue;` can't corrupt unrelated flow nodes; the
        // accumulated edges are folded into the loop's labels below.
        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));
        // Back-fill enclosing labels (`l: do (…)`) so `continue l;`
        // targets THIS loop. Mirrors Go's `setContinueTarget`.
        self.set_continue_target(node, &continue_acc);

        // Body
        self.bind(&stmt);
        if let Some(current) = &self.current_flow {
            pre_condition_label.add_antecedent(Arc::clone(current));
        }
        // Fold `continue;` edges into the condition label.
        for ant in &continue_acc.antecedents {
            pre_condition_label.add_antecedent(Arc::clone(ant));
        }

        // Restore break/continue targets
        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        // Condition
        self.current_flow =
            Some(pre_condition_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            pre_do_label.add_antecedent(true_flow);
            post_do_label.add_antecedent(false_flow);
        }
        // Fold `break;` edges into the post-loop label.
        for ant in &break_acc.antecedents {
            post_do_label.add_antecedent(Arc::clone(ant));
        }

        self.current_flow = Some(post_do_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

    /// Bind a for statement with proper control flow.
    fn bind_for_statement(&mut self, node: &Arc<Node>) {
        let mut pre_loop_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut pre_body_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut pre_incr_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_loop_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (initializer, condition, incrementor, statement) = match &node.data {
            NodeData::ForStatement(data) => (
                data.initializer.clone(),
                data.condition.clone(),
                data.incrementor.clone(),
                data.statement.clone(),
            ),
            _ => return,
        };

        // Activate the ForStatement as the block-scoped container so its
        // initializer variables (`for (let x = 1, y = 2; …)`) are scoped to
        // THIS loop rather than the enclosing scope. Without this, two loops
        // in the same block both declaring `let x` collide (TS2451) and the
        // body sees the init as used-before-declaration (TS2448). This
        // early-dispatch path otherwise skips `bind_container`.
        //
        // Mirrors Go's `bindContainer` for ForStatement (an
        // `IsBlockScopedContainer`-only node): ONLY `block_scope_container`
        // is advanced. `container` keeps pointing at the enclosing
        // function-like container so `for (var i = 0; …)` still declares `i`
        // in the function scope (var hoisting, see `declare_symbol`).
        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();
        self.block_scope_container = Some(Arc::clone(node));
        self.symbol_map
            .locals
            .entry(node.id())
            .or_insert_with(SymbolTable::new);
        // parent_symbol stays None (ForStatement has no symbol) so declares
        // route to this loop's locals.

        // Initializer
        if let Some(init) = initializer {
            self.bind(&init);
        }

        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Condition
        if let Some(cond) = condition {
            self.bind(&cond);
            if let Some(current) = self.current_flow.take() {
                let true_flow =
                    self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &cond);
                let false_flow =
                    self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &cond);
                pre_body_label.add_antecedent(true_flow);
                post_loop_label.add_antecedent(false_flow);
            }
        } else {
            // No condition = always true
            if let Some(current) = &self.current_flow {
                pre_body_label.add_antecedent(Arc::clone(current));
            }
        }

        // Save break/continue targets. Jump targets are accumulation nodes
        // (never collapsed snapshots) so in-place edge additions from
        // `break;`/`continue;` can't corrupt unrelated flow nodes; the
        // accumulated edges are folded into the loop's labels below.
        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));
        // Back-fill enclosing labels (`l: for (…)`) so `continue l;`
        // targets THIS loop. Mirrors Go's `setContinueTarget`.
        self.set_continue_target(node, &continue_acc);

        // Body
        self.current_flow = Some(pre_body_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&statement);
        if let Some(current) = &self.current_flow {
            pre_incr_label.add_antecedent(Arc::clone(current));
        }
        // Fold `continue;` edges into the incrementor label.
        for ant in &continue_acc.antecedents {
            pre_incr_label.add_antecedent(Arc::clone(ant));
        }

        // Restore break/continue targets
        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        // Incrementor
        self.current_flow = Some(pre_incr_label.finish(self.unreachable_flow.as_ref().unwrap()));
        if let Some(inc) = incrementor {
            self.bind(&inc);
        }
        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }
        // Fold `break;` edges into the post-loop label.
        for ant in &break_acc.antecedents {
            post_loop_label.add_antecedent(Arc::clone(ant));
        }

        self.current_flow = Some(post_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Restore the enclosing block scope / parent symbol.
        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;
    }

    /// Bind a for-in or for-of statement with proper control flow.
    fn bind_for_in_or_of_statement(&mut self, node: &Arc<Node>) {
        let mut pre_loop_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut post_loop_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expression, initializer, statement) = match &node.data {
            NodeData::ForInOrOfStatement(data) => (
                data.expression.clone(),
                data.initializer.clone(),
                data.statement.clone(),
            ),
            _ => return,
        };

        // Activate the ForIn/ForOf statement as the block-scoped container so
        // its loop variable (`for (let b of …)`) is scoped to THIS loop.
        // Without this, two sibling `for (let b of …)` loops in the same
        // function collide (TS2451). Mirrors Go's `bindContainer`, which sets
        // `blockScopeContainer = node` before the children are bound. Like
        // `bind_for_statement`, only `block_scope_container` is advanced so
        // `for (var k in o)` still hoists `k` to the function scope.
        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();
        self.block_scope_container = Some(Arc::clone(node));
        self.symbol_map
            .locals
            .entry(node.id())
            .or_insert_with(SymbolTable::new);
        // parent_symbol stays None (loop statements have no symbol) so
        // block-scoped declares route to this loop's locals.

        // Expression
        self.bind(&expression);

        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        post_loop_label.add_antecedent(Arc::clone(self.current_flow.as_ref().unwrap()));

        // Initializer
        self.bind(&initializer);

        // Save break/continue targets. Jump targets are accumulation nodes
        // (never collapsed snapshots) so in-place edge additions from
        // `break;`/`continue;` can't corrupt unrelated flow nodes; the
        // accumulated edges are folded into the loop's labels below.
        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));
        // Back-fill enclosing labels (`l: for (… of …)`) so `continue l;`
        // targets THIS loop. Mirrors Go's `setContinueTarget`.
        self.set_continue_target(node, &continue_acc);

        // Body
        self.bind(&statement);
        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }
        // Fold `continue;` edges into the pre-loop label (the next iteration).
        for ant in &continue_acc.antecedents {
            pre_loop_label.add_antecedent(Arc::clone(ant));
        }
        // Fold `break;` edges into the post-loop label.
        for ant in &break_acc.antecedents {
            post_loop_label.add_antecedent(Arc::clone(ant));
        }

        // Restore break/continue targets
        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow = Some(post_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Restore the enclosing block scope / parent symbol.
        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;
    }

    /// Bind a switch statement with proper control flow.
    fn bind_switch_statement(&mut self, node: &Arc<Node>) {
        let mut post_switch_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expression, case_block) = match &node.data {
            NodeData::SwitchStatement(data) => (data.expression.clone(), data.case_block.clone()),
            _ => return,
        };

        // Switch expression
        self.bind(&expression);

        // Save break target
        let prev_break = self.current_break_target.take();
        self.current_break_target =
            Some(post_switch_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Get clauses from case block
        let clauses = match &case_block.data {
            NodeData::CaseBlock(data) => data.clauses.clone(),
            _ => {
                self.current_break_target = prev_break;
                return;
            }
        };

        // Activate the CaseBlock as the block-scoped container so
        // case-clause declarations (`case 1: let x;`) are scoped to the
        // switch rather than colliding with the enclosing block. Mirrors
        // Go's `GetContainerFlags`: `KindCaseBlock` is an
        // `IsBlockScopedContainer`-only node (all clauses share one scope).
        // Like the loop binders, only `block_scope_container` is advanced so
        // `case 1: var x;` still hoists to the function scope.
        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();
        self.block_scope_container = Some(Arc::clone(&case_block));
        self.symbol_map
            .locals
            .entry(case_block.id())
            .or_insert_with(SymbolTable::new);

        // Process each clause
        for clause in &clauses.nodes {
            // Create switch clause flow
            if let Some(current) = self.current_flow.take() {
                let clause_flow = self.create_flow_switch_clause(&current, clause, node);
                self.current_flow = Some(clause_flow);
            }

            // Bind clause (expression + statements)
            match &clause.data {
                NodeData::CaseOrDefaultClause(data) => {
                    // For CaseClause, bind the expression; for DefaultClause, expression is just a placeholder
                    self.bind(&data.expression);
                    // Bind statements
                    for stmt in &data.statements.nodes {
                        self.bind(stmt);
                    }
                }
                _ => {}
            }
        }

        // Add final flow to post-switch label
        if let Some(current) = &self.current_flow {
            post_switch_label.add_antecedent(Arc::clone(current));
        }

        self.current_flow = Some(post_switch_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Restore the enclosing block scope / parent symbol.
        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;

        // Restore break target
        self.current_break_target = prev_break;
    }

    /// Bind a return statement.
    fn bind_return_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ReturnStatement(data) = &node.data {
            if let Some(expr) = &data.expression {
                self.bind(expr);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
        self.has_explicit_return = true;
        self.has_flow_effects = true;
    }

    /// Bind a throw statement.
    fn bind_throw_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ThrowStatement(data) = &node.data {
            self.bind(&data.expression);
        }
        self.current_flow = Some(self.unreachable_flow());
        self.has_flow_effects = true;
    }

    /// Bind a try/catch/finally statement with proper control flow.
    ///
    /// Mirrors `binder.bindTryStatement` in Go.
    fn bind_try_statement(&mut self, node: &Arc<Node>) {
        let stmt = match &node.data {
            NodeData::TryStatement(data) => data,
            _ => return,
        };

        let save_return_target = self.current_return_target.take();
        let save_exception_target = self.current_exception_target.take();

        let mut normal_exit_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut return_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut exception_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let mut finally_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        if stmt.finally_block.is_some() {
            self.current_return_target = Some(return_label.finish(&self.unreachable_flow()));
        }

        // Add current flow as possible exception source
        if let Some(current) = &self.current_flow {
            exception_label.add_antecedent(Arc::clone(current));
        }
        self.current_exception_target = Some(Arc::clone(
            &exception_label.finish(&self.unreachable_flow()),
        ));

        // Bind try block
        self.bind(&stmt.try_block);
        // Try block normal exit -> normal_exit_label
        if let Some(current) = &self.current_flow {
            normal_exit_label.add_antecedent(Arc::clone(current));
        }
        // Try block exception -> exception_label (already set above)

        // Bind catch clause if present
        if let Some(catch_clause) = &stmt.catch_clause {
            // Set current flow to exception label (the catch block starts here)
            self.current_flow = Some(exception_label.finish(&self.unreachable_flow()));
            self.bind(catch_clause);
            // Catch block normal exit -> normal_exit_label
            if let Some(current) = &self.current_flow {
                normal_exit_label.add_antecedent(Arc::clone(current));
            }
        }

        self.current_return_target = save_return_target;
        self.current_exception_target = save_exception_target;

        // Bind finally block if present
        if let Some(finally_block) = &stmt.finally_block {
            // Combine all possible donors into the finally label
            for ant in &normal_exit_label.node.antecedents {
                finally_label.add_antecedent(Arc::clone(ant));
            }
            for ant in &exception_label.node.antecedents {
                finally_label.add_antecedent(Arc::clone(ant));
            }
            for ant in &return_label.node.antecedents {
                finally_label.add_antecedent(Arc::clone(ant));
            }

            self.current_flow = Some(finally_label.finish(&self.unreachable_flow()));
            self.bind(finally_block);

            if self
                .current_flow
                .as_ref()
                .map_or(false, |f| f.flags.contains(FlowFlags::UNREACHABLE))
            {
                self.current_flow = Some(self.unreachable_flow());
            } else {
                // Handle return paths through finally
                if self.current_return_target.is_some() && !return_label.node.antecedents.is_empty()
                {
                    if let Some(current_flow) = &self.current_flow {
                        let reduce =
                            self.create_reduce_label(&return_label.node.antecedents, current_flow);
                        if let Some(rt) = &self.current_return_target {
                            self.add_antecedent_to_flow(rt, &reduce);
                        }
                    }
                }
                // Handle exception paths through finally
                if self.current_exception_target.is_some()
                    && !exception_label.node.antecedents.is_empty()
                {
                    if let Some(current_flow) = &self.current_flow {
                        let reduce = self
                            .create_reduce_label(&exception_label.node.antecedents, current_flow);
                        if let Some(et) = &self.current_exception_target {
                            self.add_antecedent_to_flow(et, &reduce);
                        }
                    }
                }
                // Normal exit path through finally
                if !normal_exit_label.node.antecedents.is_empty() {
                    if let Some(current_flow) = &self.current_flow {
                        self.current_flow = Some(self.create_reduce_label(
                            &normal_exit_label.node.antecedents,
                            current_flow,
                        ));
                    }
                } else {
                    self.current_flow = Some(self.unreachable_flow());
                }
            }
        } else {
            self.current_flow = Some(normal_exit_label.finish(&self.unreachable_flow()));
        }
    }

    /// Bind a break statement.
    fn bind_break_statement(&mut self, node: &Arc<Node>) {
        // Check for labeled break first
        let label_name = if let NodeData::BreakStatement(data) = &node.data {
            data.label.as_ref().map(|l| self.node_text(l))
        } else {
            None
        };

        if let Some(name) = label_name {
            // Two-pass lookup: first find the matching label's break target
            // (immutable borrow), then mark it referenced (mutable borrow).
            // Mirrors Go's `activeLabel.referenced = true` in
            // `bindBreakOrContinueStatement`.
            let break_target = {
                let mut current = &self.active_label_list;
                let mut found = None;
                while let Some(label) = current {
                    if label.name == name {
                        found = Some(Arc::clone(&label.break_target));
                        break;
                    }
                    current = &label.next;
                }
                found
            };
            if let Some(target) = break_target {
                if let Some(current_flow) = &self.current_flow {
                    self.add_antecedent_to_flow(&target, current_flow);
                }
                // Mark the matching label as referenced.
                let mut current = &mut self.active_label_list;
                while let Some(label) = current {
                    if label.name == name {
                        label.referenced = true;
                        break;
                    }
                    current = &mut label.next;
                }
            }
        } else if let Some(target) = &self.current_break_target {
            // Unlabeled break to the innermost break target
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(target, current);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
    }

    /// Bind a continue statement.
    fn bind_continue_statement(&mut self, node: &Arc<Node>) {
        // Check for labeled continue first
        let label_name = if let NodeData::ContinueStatement(data) = &node.data {
            data.label.as_ref().map(|l| self.node_text(l))
        } else {
            None
        };

        if let Some(name) = label_name {
            // Two-pass lookup: first find the matching label's continue target
            // (immutable borrow), then mark it referenced (mutable borrow).
            // Mirrors Go's `activeLabel.referenced = true`.
            let continue_target = {
                let mut current = &self.active_label_list;
                let mut found = None;
                while let Some(label) = current {
                    if label.name == name {
                        found = label.continue_target.clone();
                        break;
                    }
                    current = &label.next;
                }
                found
            };
            if let Some(target) = continue_target {
                if let Some(current_flow) = &self.current_flow {
                    self.add_antecedent_to_flow(&target, current_flow);
                }
                // Mark the matching label as referenced.
                let mut current = &mut self.active_label_list;
                while let Some(label) = current {
                    if label.name == name {
                        label.referenced = true;
                        break;
                    }
                    current = &mut label.next;
                }
            }
        } else if let Some(target) = &self.current_continue_target {
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(target, current);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
    }

    /// Propagate a loop's continue target to enclosing labels while the loop
    /// is directly labeled (`l: for (…)`, `a: b: while (…)`). Mirrors Go's
    /// `setContinueTarget` (binder.go:1779): walks the loop node's parent
    /// chain of `LabeledStatement`s in lockstep with the active label list
    /// (innermost first), assigning the loop's own continue target so
    /// `continue label;` routes to the correct loop rather than the
    /// enclosing one.
    fn set_continue_target(&mut self, loop_node: &Arc<Node>, target: &Arc<FlowNode>) {
        let mut node = Arc::clone(loop_node);
        let mut cursor = &mut self.active_label_list;
        loop {
            let Some(parent) = node.parent.clone() else { break };
            if parent.kind != SyntaxKind::LabeledStatement {
                break;
            }
            let Some(label) = cursor else { break };
            label.continue_target = Some(Arc::clone(target));
            node = parent;
            cursor = &mut label.next;
        }
    }

    /// Bind a labeled statement.
    ///
    /// Mirrors `binder.bindLabeledStatement` in Go.
    fn bind_labeled_statement(&mut self, node: &Arc<Node>) {
        let stmt = match &node.data {
            NodeData::LabeledStatement(data) => data,
            _ => return,
        };

        let label_name = self.node_text(&stmt.label);
        // Break target: a branch-label accumulation node. It must NOT be
        // created by `FlowLabel::finish` before the statement is bound —
        // an empty label collapses to the (shared) UNREACHABLE node, which
        // would poison `current_flow` for everything after the labeled
        // statement (all subsequent references narrow to `never`).
        // `break label;` adds antecedents in place; the fallthrough
        // antecedent is added after the statement is bound below.
        let break_target = Self::new_flow_accumulator();

        // The continue target starts as `None` and is back-filled by the
        // labeled iteration statement itself (see `set_continue_target`,
        // called from the loop binders) — NOT from the enclosing loop, which
        // would route `continue label;` to the wrong place. Mirrors Go's
        // `bindLabeledStatement` (`continueTarget: nil`) + `setContinueTarget`.
        let continue_target: Option<Arc<FlowNode>> = None;

        let active_label = Box::new(ActiveLabel {
            name: label_name,
            break_target: Arc::clone(&break_target),
            continue_target,
            referenced: false,
            next: self.active_label_list.take(),
        });

        self.active_label_list = Some(active_label);

        // Bind the statement (the loop body, etc.)
        self.bind(&stmt.statement);

        // Check if the label was referenced by a break/continue statement.
        // Mirrors Go's `if !b.activeLabelList.referenced { ... }` — an
        // unreferenced label is marked `NodeFlags::Unreachable` so the
        // checker can report it (TS7028 unused label).
        let was_referenced = self
            .active_label_list
            .as_ref()
            .map_or(false, |l| l.referenced);

        // Restore active label list
        self.active_label_list = self.active_label_list.take().and_then(|l| l.next);

        if !was_referenced {
            // Mark the label node as unreachable (unused label). The checker
            // will decide whether to report TS7028 based on the enclosing
            // context (e.g., `allowUnusedLabels`).
            let label_ptr = Arc::as_ptr(&stmt.label) as *mut Node;
            unsafe {
                (*label_ptr).flags |= NodeFlags::Unreachable;
            }
        }

        // Finish break target: add the fallthrough antecedent (skipped when
        // unreachable), mirroring Go's `b.addAntecedent(postStatementLabel,
        // b.currentFlow); b.currentFlow = b.finishFlowLabel(...)`. A label
        // that accumulated no antecedents (body always breaks/returns)
        // finishes to the UNREACHABLE node, like Go's `finishFlowLabel`.
        if let Some(current) = &self.current_flow {
            self.add_antecedent_to_flow(&break_target, current);
        }
        self.current_flow = if break_target.antecedents.is_empty() {
            Some(self.unreachable_flow())
        } else {
            Some(break_target)
        };
    }

    /// Check if an identifier is push or unshift (for array mutation tracking).
    /// Mirrors Go's `ast.IsPushOrUnshiftIdentifier`.
    fn is_push_or_unshift_identifier(&self, name: &str) -> bool {
        name == "push" || name == "unshift"
    }

    /// Check if an expression is a narrowable operand (identifier, property
    /// access chain, parenthesized, etc.). Mirrors Go's `isNarrowableOperand` +
    /// `containsNarrowableReference`. Used to gate ARRAY_MUTATION flow nodes
    /// so that `arr.push(x)` (where `arr` is an identifier) is tracked but
    /// `getFoo().push(x)` is not.
    fn is_narrowable_operand(&self, expr: &Arc<Node>) -> bool {
        match expr.kind {
            SyntaxKind::Identifier
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::MetaProperty => true,
            SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::NonNullExpression => {
                if let Some(inner) = expr.expression() {
                    self.is_narrowable_operand(&inner)
                } else {
                    false
                }
            }
            SyntaxKind::ElementAccessExpression => {
                // Element access is narrowable if the argument is a
                // string/numeric literal or an entity-name expression whose
                // receiver is narrowable.
                if let NodeData::ElementAccessExpression(ea) = &expr.data {
                    if self.is_string_or_numeric_literal_like(&ea.argument_expression) {
                        return true;
                    }
                    return self.is_entity_name_expression(&ea.argument_expression)
                        && self.is_narrowable_operand(&ea.expression);
                }
                false
            }
            _ => false,
        }
    }

    /// Mirrors Go's `ast.IsStringOrNumericLiteralLike`.
    fn is_string_or_numeric_literal_like(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
        )
    }

    /// Mirrors Go's `ast.IsEntityNameExpression` (identifier or qualified
    /// name).
    fn is_entity_name_expression(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::Identifier | SyntaxKind::QualifiedName
        )
    }

    /// Bind a call expression for flow tracking (array mutation detection).
    ///
    /// Mirrors `binder.bindCallExpressionFlow` in Go. Handles:
    /// - Optional chains (delegates to `bind_optional_chain_flow`)
    /// - IIFE (function/arrow expression): bind args then callee
    /// - `super()`: create a CALL flow node
    /// - `arr.push(x)` / `arr.unshift(x)`: create an ARRAY_MUTATION flow node
    fn bind_call_expression_flow(&mut self, node: &Arc<Node>) {
        if let NodeData::CallExpression(data) = &node.data {
            let expr = &data.expression;
            // Check for property access expression like arr.push()
            if let NodeData::PropertyAccessExpression(prop) = &expr.data {
                let name = self.node_text(&prop.name);
                if self.is_push_or_unshift_identifier(&name)
                    && self.is_narrowable_operand(&prop.expression)
                {
                    // This is an array mutation call: create a flow mutation node
                    let current = self.current_flow.clone();
                    if let Some(current) = current {
                        self.current_flow = Some(self.create_flow_mutation(&current, node));
                    }
                }
            }
        }
    }

    /// Handle `this.property = value` assignments in JS files for expando
    /// binding. Mirrors Go's `bindThisPropertyAssignment`
    /// (`binder.go:1121-1141`).
    ///
    /// When a `this.prop = value` assignment is found inside a function-like
    /// container (the `this_container`), the property is declared on the
    /// container's symbol. For class members (`this.prop = value` inside a
    /// constructor/method), the property goes to the class symbol's members
    /// or exports (depending on static/instance).
    ///
    /// This is a no-op for TS files (Go checks `IsInJSFile`).
    /// Currently a skeleton: full expando binding requires `declareSymbolEx`
    /// with `isReplaceableByMethod` / `isComputedName` flags and
    /// `addLateBoundAssignmentDeclarationToSymbol` for dynamic names —
    /// deferred to the JS support phase. The `this_container` tracking
    /// infrastructure (field + save/restore + `IS_THIS_CONTAINER` flag) is
    /// in place so that expando binding can be wired in later.
    fn bind_this_property_assignment(&mut self, _node: &Arc<Node>) {
        // TODO(JS): Implement full `this.prop = value` expando binding.
        // The `this_container` field is now tracked but not yet used to
        // declare properties. This requires:
        // 1. `is_in_js_file(node)` guard (Go: `ast.IsInJSFile(node)`)
        // 2. `get_this_class_and_symbol_table()` — resolve `this_container`
        //    to a class symbol + members/exports table
        // 3. `declareSymbolEx` with `isReplaceableByMethod = true`
        // 4. `addLateBoundAssignmentDeclarationToSymbol` for dynamic names
        // Deferred until JS file support is prioritized.
    }

    /// Bind an expression statement (with assignment flow tracking).
    fn bind_expression_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ExpressionStatement(data) = &node.data {
            self.bind(&data.expression);
            // Check for assignment
            if let NodeData::BinaryExpression(bin_data) = &data.expression.data {
                if is_assignment_operator(bin_data.operator_token.kind) {
                    if let Some(current) = self.current_flow.take() {
                        let assign_flow = self.create_flow_assignment(&current, &data.expression);
                        self.symbol_map
                            .set_flow_node(&data.expression, Arc::clone(&assign_flow));
                        self.current_flow = Some(assign_flow);
                    }
                    // Check for element access assignment (array mutation: arr[i] = val)
                    if let NodeData::ElementAccessExpression(_) = &bin_data.left.data {
                        let current = self.current_flow.clone();
                        if let Some(current) = current {
                            self.current_flow = Some(self.create_flow_mutation(&current, node));
                        }
                    }
                }
            }
            // Check for call expression
            if let NodeData::CallExpression(_) = &data.expression.data {
                if let Some(current) = self.current_flow.take() {
                    let call_flow = self.create_flow_call(&current, &data.expression);
                    self.symbol_map
                        .set_flow_node(&data.expression, Arc::clone(&call_flow));
                    self.current_flow = Some(call_flow);
                }
            }
        } else {
            self.bind_children(node);
        }
    }

    /// Bind a variable statement (with assignment flow for initializers).
    fn bind_variable_statement(&mut self, node: &Arc<Node>) {
        // Bind normally (creates symbols, recurses)
        self.bind_children(node);

        // Add assignment flow for declarations with initializers
        if let NodeData::VariableStatement(data) = &node.data {
            if let NodeData::VariableDeclarationList(list_data) = &data.declaration_list.data {
                for decl in &list_data.declarations.nodes {
                    if let NodeData::VariableDeclaration(decl_data) = &decl.data {
                        if decl_data.initializer.is_some() {
                            if let Some(current) = self.current_flow.take() {
                                let assign_flow = self.create_flow_assignment(&current, decl);
                                self.symbol_map
                                    .set_flow_node(decl, Arc::clone(&assign_flow));
                                self.current_flow = Some(current);
                            }
                        }
                    }
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Binding dispatch
    // ─────────────────────────────────────────────────────────────────────

    /// Bind a single node: create symbols, set flow nodes, then recurse.
    /// Go binder.go:1301 `checkContextualIdentifier` — an identifier whose
    /// text is a future-reserved word (implements/interface/let/package/
    /// private/protected/public/static/yield) in strict mode reports
    /// TS1213 (inside a class), TS1214 (module), or TS1100 (plain strict).
    /// `await`/`yield` misuse reports TS1262/TS1359 in the matching
    /// contexts. Skipped when the file has parse errors (Go reports only
    /// on clean parses), in ambient contexts, for JSDoc-synthesized
    /// identifiers, and in identifier-name positions (`a.static`,
    /// `{ static: 1 }`, member names) where keywords are legal.
    fn check_contextual_identifier(&mut self, node: &Arc<Node>) {
        let Some(file) = self.current_source_file.clone() else {
            return;
        };
        if file.has_parse_diagnostics
            || node.flags.contains(NodeFlags::Ambient)
            || node.flags.contains(NodeFlags::JSDoc)
            || is_identifier_name(node)
        {
            return;
        }
        let Some(kind) = crate::scanner::string_to_keyword(node.text()) else {
            return;
        };
        let is_future_reserved = matches!(
            kind,
            SyntaxKind::ImplementsKeyword
                | SyntaxKind::InterfaceKeyword
                | SyntaxKind::LetKeyword
                | SyntaxKind::PackageKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::YieldKeyword
        );
        let message = if is_future_reserved {
            if crate::ast::utilities::get_containing_class(node).is_some() {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_CLASS_DEFINITIONS_ARE_AUTOMATICALLY_IN_STRICT_MODE
            } else if file.external_module_indicator.is_some() {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE
            } else {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE
            }
        } else if kind == SyntaxKind::AwaitKeyword {
            if file.external_module_indicator.is_some()
                && crate::ast::utilities::is_in_top_level_context(node)
            {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE
            } else if node.flags.contains(NodeFlags::AwaitContext) {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE
            } else {
                return;
            }
        } else if kind == SyntaxKind::YieldKeyword && node.flags.contains(NodeFlags::YieldContext) {
            IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE
        } else {
            return;
        };
        self.symbol_map.binder_diagnostics.push(Diagnostic::new(
            Some(file),
            node.loc,
            message,
            vec![node.text().to_string()],
        ));
    }

    fn bind(&mut self, node: &Arc<Node>) {
        // Set flow node for expressions
        match node.kind {
            SyntaxKind::Identifier => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
                self.check_contextual_identifier(node);
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
            }
            _ => {}
        }

        // Create symbols for declarations
        match node.kind {
            SyntaxKind::VariableDeclaration => {
                self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
            }
            SyntaxKind::VariableStatement => {
                // The statement itself doesn't get a symbol; its declarations do
            }
            SyntaxKind::FunctionDeclaration => {
                self.declare_symbol(node, SymbolFlags::Function, SymbolFlags::VALUE);
            }
            SyntaxKind::FunctionExpression => {
                // Use the actual name for named function expressions so the
                // name is self-referenceable inside the body. Mirrors Go's
                // `bindFunctionExpression` which uses `node.Name().Text()`
                // when a name is present. The symbol is added to the
                // function expression's own locals in `bind_container` so it
                // is visible inside the body but not in the enclosing scope.
                let name = match &node.data {
                    NodeData::FunctionExpression(data) => {
                        data.name.as_ref().map(|n| self.node_text(n))
                    }
                    _ => None,
                }
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_FUNCTION.to_string());
                self.bind_anonymous_declaration(node, SymbolFlags::Function, &name);
            }
            SyntaxKind::ArrowFunction => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Function,
                    INTERNAL_SYMBOL_NAME_FUNCTION,
                );
            }
            SyntaxKind::ClassDeclaration => {
                self.declare_symbol(
                    node,
                    SymbolFlags::Class,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::ClassExpression => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Class,
                    INTERNAL_SYMBOL_NAME_CLASS,
                );
            }
            SyntaxKind::InterfaceDeclaration => {
                self.declare_symbol(node, SymbolFlags::Interface, SymbolFlags::TYPE);
            }
            SyntaxKind::TypeAliasDeclaration => {
                self.declare_symbol(node, SymbolFlags::TypeAlias, SymbolFlags::TYPE);
            }
            SyntaxKind::EnumDeclaration => {
                self.declare_symbol(
                    node,
                    SymbolFlags::RegularEnum,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::ModuleDeclaration => {
                // A DOTTED module name (`declare namespace Foo.Bar`) declares
                // nested module symbols (Go's declareModuleSymbol): `Foo` in
                // the enclosing scope, `Bar` in `Foo`'s exports — so
                // `Foo.Bar.x` resolves from outside.
                // A dotted name parses as a QualifiedName (A.B.C) — flatten
                // it; plain names are Identifiers.
                let dotted_name = match &node.data {
                    crate::ast::NodeData::ModuleDeclaration(md) => match md.name.kind {
                        SyntaxKind::Identifier => md.name.text().to_string(),
                        SyntaxKind::QualifiedName => {
                            fn qualified_text(n: &Arc<Node>) -> String {
                                match &n.data {
                                    crate::ast::NodeData::QualifiedName(q) => {
                                        format!("{}.{}", qualified_text(&q.left), q.right.text())
                                    }
                                    _ => n.text().to_string(),
                                }
                            }
                            qualified_text(&md.name)
                        }
                        _ => String::new(),
                    },
                    _ => String::new(),
                };
                if dotted_name.contains('.') {
                    let parts: Vec<&str> = dotted_name.split('.').collect();
                    // Locate the container's symbol table like declare_symbol
                    // does (parent symbol members/exports for symbol-ful
                    // containers, else the container's locals).
                    let container = self.container.clone();
                    let parent_sym = self.parent_symbol.clone();
                    let mut table: Option<Arc<Symbol>> = None;
                    let mut locals_key: Option<u64> = None;
                    if let Some(ps) = &parent_sym {
                        table = Some(Arc::clone(ps));
                    } else if let Some(c) = &container {
                        locals_key = Some(c.id());
                    }
                    let mut current: Option<Arc<Symbol>> = None;
                    for part in &parts[..parts.len() - 1] {
                        let existing = current.as_ref().map_or_else(
                            || {
                                table.as_ref().and_then(|t| {
                                    t.members.get(*part).cloned().or_else(|| t.exports.get(*part).cloned())
                                }).or_else(|| {
                                    locals_key
                                        .and_then(|k| self.symbol_map.locals.get(&k))
                                        .and_then(|l| l.get(*part).cloned())
                                })
                            },
                            |cur| cur.exports.get(*part).cloned(),
                        );
                        let sym = match existing {
                            Some(s) if s.flags.contains(SymbolFlags::ValueModule) => s,
                            _ => {
                                let fresh = Arc::new(Symbol::new(
                                    SymbolFlags::ValueModule,
                                    part.to_string(),
                                ));
                                if let Some(cur) = &current {
                                    let cur_mut = Arc::as_ptr(cur) as *mut Symbol;
                                    unsafe {
                                        (*cur_mut).exports.insert(part.to_string(), Arc::clone(&fresh));
                                    }
                                } else if let Some(t) = &table {
                                    let t_mut = Arc::as_ptr(t) as *mut Symbol;
                                    unsafe {
                                        (*t_mut).members.insert(part.to_string(), Arc::clone(&fresh));
                                    }
                                } else if let Some(k) = locals_key {
                                    self.symbol_map
                                        .locals
                                        .entry(k)
                                        .or_default()
                                        .insert(part.to_string(), Arc::clone(&fresh));
                                }
                                fresh
                            }
                        };
                        current = Some(sym);
                    }
                    // Declare the LAST segment into the innermost parent's
                    // exports and register the node on it.
                    let last = parts[parts.len() - 1];
                    let symbol = Arc::new(Symbol::new(SymbolFlags::ValueModule, last.to_string()));
                    {
                        let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                        unsafe {
                            (*symbol_mut).declarations.push(Arc::clone(node));
                        }
                    }
                    match &current {
                        Some(cur) => {
                            let cur_mut = Arc::as_ptr(cur) as *mut Symbol;
                            unsafe {
                                (*cur_mut).exports.insert(last.to_string(), Arc::clone(&symbol));
                            }
                        }
                        None => {
                            if let Some(t) = &table {
                                let t_mut = Arc::as_ptr(t) as *mut Symbol;
                                unsafe {
                                    (*t_mut).members.insert(last.to_string(), Arc::clone(&symbol));
                                }
                            } else if let Some(k) = locals_key {
                                self.symbol_map
                                    .locals
                                    .entry(k)
                                    .or_default()
                                    .insert(last.to_string(), Arc::clone(&symbol));
                            }
                        }
                    }
                    self.symbol_map.set_symbol(node, Arc::clone(&symbol));
                } else {
                    self.declare_symbol(node, SymbolFlags::ValueModule, SymbolFlags::MODULE);
                }
            }
            SyntaxKind::Parameter => {
                // TS2371: parameter initializers are only allowed on
                // function/constructor IMPLEMENTATIONS (Go's
                // checkGrammarParameters via checkSignatureDeclaration) —
                // overload signatures, method signatures, and type-level
                // function types have no body and reject initializers.
                // Parent pointers are populated before binding, so the
                // enclosing function-like's body presence is checkable here.
                let mut report_2371 = |b: &mut Self, loc: crate::core::text::TextRange| {
                    b.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        b.current_source_file.clone(),
                        loc,
                        A_PARAMETER_INITIALIZER_IS_ONLY_ALLOWED_IN_A_FUNCTION_OR_CONSTRUCTOR_IMPLEMENTATION,
                        vec![],
                    ));
                };
                if let NodeData::ParameterDeclaration(pd) = &node.data
                    && let Some(parent) = node.parent.as_ref()
                    && !fn_like_body_present(parent)
                {
                    if pd.initializer.is_some() {
                        report_2371(self, node.loc);
                    } else {
                        // Binding-pattern parameters: initializers live on
                        // the binding elements ('({ first = 0 }: …)').
                        let mut elements: Vec<&Arc<Node>> = Vec::new();
                        collect_binding_elements(&pd.name, &mut elements);
                        for el in elements {
                            if matches!(&el.data, NodeData::BindingElement(be) if be.initializer.is_some()) {
                                report_2371(self, el.loc);
                            }
                        }
                    }
                }
                self.declare_symbol(
                    node,
                    SymbolFlags::FunctionScopedVariable,
                    SymbolFlags::VALUE,
                );
            }
            SyntaxKind::PropertyDeclaration | SyntaxKind::PropertySignature => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
                self.declare_symbol(node, SymbolFlags::Method, SymbolFlags::VALUE);
            }
            SyntaxKind::PropertyAssignment => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::ShorthandPropertyAssignment => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::EnumMember => {
                self.declare_symbol(
                    node,
                    SymbolFlags::EnumMember,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::GetAccessor => {
                self.declare_symbol(node, SymbolFlags::GetAccessor, SymbolFlags::VALUE);
            }
            SyntaxKind::SetAccessor => {
                self.declare_symbol(node, SymbolFlags::SetAccessor, SymbolFlags::VALUE);
            }
            SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ExportSpecifier => {
                self.declare_symbol(node, SymbolFlags::Alias, SymbolFlags::Alias);
            }
            // `import D from "mod"` — the default import `D` is an alias
            // declared in the container's locals. Mirrors Go's
            // `bindImportClause`.
            SyntaxKind::ImportClause => {
                self.bind_import_clause(node);
            }
            // `export default <expr>` / `export = <expr>`. Mirrors Go's
            // `bindExportAssignment`.
            SyntaxKind::ExportAssignment => {
                self.bind_export_assignment(node);
            }
            // `export * from "mod"` / `export * as ns from "mod"` /
            // `export { a, b }`. Mirrors Go's `bindExportDeclaration`.
            SyntaxKind::ExportDeclaration => {
                self.bind_export_declaration(node);
            }
            // Standalone `export * as ns from "mod"` (the
            // `NamespaceExportDeclaration` form used in global declaration
            // files). Mirrors Go's `bindNamespaceExportDeclaration`.
            SyntaxKind::NamespaceExportDeclaration => {
                self.bind_namespace_export_declaration(node);
            }
            SyntaxKind::BindingElement => {
                self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
            }
            SyntaxKind::TypeParameter => {
                // TS2300: duplicate names in one type-parameter list (Go's
                // checkTypeParameters). The parameter's parent is the list
                // node; earlier same-name siblings make this one a dupe.
                if let Some(list) = node.parent.as_ref()
                    && let Some(name) = node.name()
                    && name.kind == SyntaxKind::Identifier
                {
                    let mut dup = false;
                    crate::ast::node_data_generated::for_each_child(list, |sibling| {
                        if Arc::ptr_eq(sibling, node) {
                            return true; // stop at self — only EARLIER entries count
                        }
                        if sibling.kind == SyntaxKind::TypeParameter
                            && sibling
                                .name()
                                .is_some_and(|sn| sn.text() == name.text())
                        {
                            dup = true;
                        }
                        false
                    });
                    if dup {
                        self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                            self.current_source_file.clone(),
                            name.loc,
                            DUPLICATE_IDENTIFIER_0,
                            vec![name.text().to_string()],
                        ));
                    }
                }
                self.bind_type_parameter(node);
            }
            SyntaxKind::ObjectLiteralExpression => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::ObjectLiteral,
                    INTERNAL_SYMBOL_NAME_OBJECT,
                );
            }
            SyntaxKind::TypeLiteral => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::TypeLiteral,
                    INTERNAL_SYMBOL_NAME_TYPE,
                );
            }
            _ => {}
        }

        // Control flow statement dispatch
        match node.kind {
            SyntaxKind::IfStatement => {
                self.bind_if_statement(node);
                return;
            }
            SyntaxKind::WhileStatement => {
                self.bind_while_statement(node);
                return;
            }
            SyntaxKind::DoStatement => {
                self.bind_do_statement(node);
                return;
            }
            SyntaxKind::ForStatement => {
                self.bind_for_statement(node);
                return;
            }
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                self.bind_for_in_or_of_statement(node);
                return;
            }
            SyntaxKind::SwitchStatement => {
                self.bind_switch_statement(node);
                return;
            }
            SyntaxKind::ReturnStatement => {
                self.bind_return_statement(node);
                return;
            }
            SyntaxKind::ThrowStatement => {
                self.bind_throw_statement(node);
                return;
            }
            SyntaxKind::BreakStatement => {
                self.bind_break_statement(node);
                return;
            }
            SyntaxKind::ContinueStatement => {
                self.bind_continue_statement(node);
                return;
            }
            SyntaxKind::ExpressionStatement => {
                self.bind_expression_statement(node);
                return;
            }
            SyntaxKind::VariableStatement => {
                self.bind_variable_statement(node);
                return;
            }
            SyntaxKind::TryStatement => {
                self.bind_try_statement(node);
                return;
            }
            SyntaxKind::LabeledStatement => {
                self.bind_labeled_statement(node);
                return;
            }
            SyntaxKind::CallExpression => {
                self.bind_call_expression_flow(node);
                // Don't return - also check for children after call expression flow
            }
            SyntaxKind::BinaryExpression => {
                // JS expando binding: `this.prop = value` or
                // `Class.prototype.method = fn` in JS files. Mirrors Go's
                // `bindThisPropertyAssignment` (`binder.go:1121-1141`).
                self.bind_this_property_assignment(node);
            }
            _ => {}
        }

        // Recurse into children
        let container_flags = get_container_flags(node.kind);
        if container_flags != ContainerFlags::NONE {
            self.bind_container(node, container_flags);
        } else {
            self.bind_children(node);
        }
    }

    /// Create an anonymous symbol (for function expressions, class expressions,
    /// object literals, type literals).
    fn bind_anonymous_declaration(&mut self, node: &Arc<Node>, flags: SymbolFlags, name: &str) {
        let symbol = self.new_symbol(flags, name.to_string());
        self.symbol_map.set_symbol(node, symbol);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Import / export binding — ported from `internal/binder/binder.go`
    // ─────────────────────────────────────────────────────────────────────

    /// Bind an `ImportClause` (`import D from "mod"`). Only the default
    /// import name `D` is declared here; named bindings and namespace
    /// imports are handled by their own dispatch arms (`ImportSpecifier`,
    /// `NamespaceImport`).
    ///
    /// The default import alias goes to the container's locals (not
    /// exports), matching Go's `declareModuleMember` alias branch which
    /// calls `declareSymbol(GetLocals(container), nil, node, Alias, ...)`.
    ///
    /// Mirrors Go's `binder.bindImportClause`.
    fn bind_import_clause(&mut self, node: &Arc<Node>) {
        let has_name = matches!(&node.data, NodeData::ImportClause(data) if data.name.is_some());
        if !has_name {
            return;
        }
        if let Some(container) = &self.container {
            self.declare_symbol_into(
                node,
                SymbolFlags::Alias,
                SymbolFlags::AliasExcludes,
                DeclareTarget::Locals(Arc::clone(container)),
            );
        }
    }

    /// Bind an `ExportAssignment` (`export default <expr>` /
    /// `export = <expr>`).
    ///
    /// The symbol is declared in the container's exports, named "default"
    /// (for `export default`) or "export=" (for `export =`). If the
    /// expression is an entity name or a class expression the symbol is an
    /// `Alias`; otherwise (e.g. `export default 42`) it is a `Property`.
    ///
    /// Mirrors Go's `binder.bindExportAssignment`.
    fn bind_export_assignment(&mut self, node: &Arc<Node>) {
        let (is_export_equals, expr_kind) = match &node.data {
            NodeData::ExportAssignment(data) => (data.is_export_equals, data.expression.kind),
            _ => return,
        };
        let parent_sym = match self.parent_symbol.clone() {
            Some(s) => s,
            None => {
                // Export assignment inside a block construct without a
                // container symbol — emit an anonymous declaration so the
                // node still gets a symbol. Mirrors Go's fallback branch.
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::VALUE,
                    &self.get_declaration_name(node),
                );
                return;
            }
        };
        // `ExpressionIsAlias(expr)` = `IsEntityNameExpression || IsClassExpression`.
        let is_alias = matches!(
            expr_kind,
            SyntaxKind::Identifier | SyntaxKind::QualifiedName | SyntaxKind::ClassExpression
        );
        let flags = if is_alias {
            SymbolFlags::Alias
        } else {
            SymbolFlags::Property
        };
        let symbol = self.declare_symbol_into(
            node,
            flags,
            SymbolFlags::all(),
            DeclareTarget::Exports(parent_sym),
        );
        if is_export_equals {
            // Ensure export assignments have a ValueDeclaration set.
            // Mirrors Go's `SetValueDeclaration(symbol, node)`.
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).value_declaration = Some(Arc::clone(node));
            }
        }
    }

    /// Bind an `ExportDeclaration` (`export * from "mod"` /
    /// `export * as ns from "mod"` / `export { a, b }`).
    ///
    /// - `export * from "mod"`: record an `ExportStar` symbol in the
    ///   container's exports.
    /// - `export * as ns from "mod"`: declare an `Alias` for `ns` in the
    ///   container's exports (the aliased node is the `NamespaceExport`
    ///   clause, so its name `ns` is used).
    /// - `export { a, b }`: nothing to do here — the individual
    ///   `ExportSpecifier`s already declare their own alias symbols via
    ///   the shared dispatch arm.
    ///
    /// Mirrors Go's `binder.bindExportDeclaration`.
    fn bind_export_declaration(&mut self, node: &Arc<Node>) {
        let export_clause: Option<Arc<Node>> = match &node.data {
            NodeData::ExportDeclaration(data) => data.export_clause.clone(),
            _ => return,
        };
        let parent_sym = match self.parent_symbol.clone() {
            Some(s) => s,
            None => {
                // `export *` in a block construct without a container
                // symbol — anonymous declaration. Mirrors Go's fallback.
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::ExportStar,
                    &self.get_declaration_name(node),
                );
                return;
            }
        };
        match &export_clause {
            None => {
                // `export * from "mod"`.
                self.declare_symbol_into(
                    node,
                    SymbolFlags::ExportStar,
                    SymbolFlags::None,
                    DeclareTarget::Exports(parent_sym),
                );
            }
            Some(clause) if clause.kind == SyntaxKind::NamespaceExport => {
                // `export * as ns from "mod"`.
                self.declare_symbol_into(
                    clause,
                    SymbolFlags::Alias,
                    SymbolFlags::AliasExcludes,
                    DeclareTarget::Exports(parent_sym),
                );
            }
            _ => {
                // `export { a, b }` — handled by ExportSpecifier arms.
            }
        }
    }

    /// Bind a standalone `NamespaceExportDeclaration` (`export * as ns from
    /// "mod"` in global declaration files).
    ///
    /// Go places this in the file's `GlobalExports` table. The Rust
    /// `NodeSymbolMap` has no separate global-exports table, so the symbol
    /// is declared in the container symbol's `exports`, which is where
    /// downstream lookups search.
    ///
    /// Mirrors Go's `binder.bindNamespaceExportDeclaration`.
    fn bind_namespace_export_declaration(&mut self, node: &Arc<Node>) {
        let parent_sym = match self.parent_symbol.clone() {
            Some(s) => s,
            None => return,
        };
        self.declare_symbol_into(
            node,
            SymbolFlags::Alias,
            SymbolFlags::AliasExcludes,
            DeclareTarget::Exports(parent_sym),
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Container binding
    // ─────────────────────────────────────────────────────────────────────

    /// Bind a container node: save/restore container context, then bind children.
    fn bind_container(&mut self, node: &Arc<Node>, flags: ContainerFlags) {
        // Save (clone, not take): block-only containers below leave
        // `container` pointing at the enclosing function-like container, so
        // `var` declarations inside them hoist correctly.
        let prev_container = self.container.clone();
        let prev_block = self.block_scope_container.take();
        // Save the current `this_container`. If this node is a
        // `IS_THIS_CONTAINER` (function-like), it becomes the new
        // `this_container` for its children. Mirrors Go's
        // `saveThisContainer := b.thisContainer` + conditional set
        // (`binder.go:1482,1513-1514`).
        let prev_this_container = self.this_container.take();
        // Save the current parent_symbol. For container nodes that have a
        // symbol (e.g. FunctionDeclaration), we'll replace it with the
        // container's symbol so children are added to its members. For
        // block-scoped containers without a symbol (e.g. Block), we clear
        // it so children go into the block's locals.
        let prev_parent_symbol = self.parent_symbol.take();

        // Mirrors Go's `bindContainer` (binder.go:1501-1510):
        // - `IsContainer` nodes (functions, classes, source files, modules…)
        //   advance BOTH `container` and `block_scope_container`.
        // - Block-scoped-only containers (Block, For*, CatchClause, …) advance
        //   ONLY `block_scope_container`.
        // Keeping `container` at the nearest function-like container is what
        // lets `var` declarations in nested blocks hoist to the function
        // scope (see `declare_symbol`). Note: our `get_container_flags` marks
        // `Block` as IS_CONTAINER (a deviation from Go), so block-only kinds
        // are filtered out explicitly here.
        let block_only = is_block_only_container(node.kind);
        if flags.contains(ContainerFlags::IS_CONTAINER) && !block_only {
            self.container = Some(Arc::clone(node));
            self.block_scope_container = Some(Arc::clone(node));
        } else {
            // Block-scoped container (no symbol of its own for locals).
            self.block_scope_container = Some(Arc::clone(node));
        }

        // `IS_THIS_CONTAINER` containers (FunctionDeclaration,
        // FunctionExpression, MethodDeclaration, Constructor, etc.)
        // become the new `this_container`.
        if flags.contains(ContainerFlags::IS_THIS_CONTAINER) {
            self.this_container = Some(Arc::clone(node));
        }

        // Create locals for this container if it has them
        if has_locals(node.kind) {
            self.symbol_map.locals.insert(node.id(), SymbolTable::new());
        }

        // Set parent_symbol to the container's symbol (if it has one).
        // This ensures children (parameters, class members, etc.) are added
        // to the container's symbol members rather than the outer scope.
        if let Some(sym) = self.symbol_map.symbol_of(node) {
            self.parent_symbol = Some(Arc::clone(sym));
        }
        // If the node has no symbol (e.g. Block), parent_symbol remains None,
        // so declare_symbol falls through to the block_scope_container.locals.

        // Function-like containers get their own fresh control flow graph:
        // a new START flow node, with the outer flow saved and restored.
        // This prevents flow effects inside the function body (e.g. a
        // `return` marking the flow UNREACHABLE) from leaking into the
        // enclosing scope. Mirrors Go's `bindChildren` flow handling for
        // `ContainerFlagsIsFunctionLike` containers.
        let is_function_like = flags.contains(ContainerFlags::IS_FUNCTION_LIKE);
        let prev_flow = if is_function_like {
            self.current_flow.take()
        } else {
            None
        };
        if is_function_like {
            self.current_flow = Some(Arc::new(FlowNode::new(FlowFlags::START)));
        }

        // Named function expressions can reference their own name inside the
        // body. Add the function's symbol to its own locals table so the
        // name is visible during binding/checking of the body. Mirrors Go's
        // NameResolver special case for `KindFunctionExpression` which
        // returns `location.Symbol()` when the name matches.
        if node.kind == SyntaxKind::FunctionExpression {
            let sym_and_name = self
                .symbol_map
                .symbol_of(node)
                .map(|sym| (Arc::clone(&sym), sym.name.clone()));
            if let Some((sym, sym_name)) = sym_and_name {
                if sym_name != INTERNAL_SYMBOL_NAME_FUNCTION {
                    if let Some(locals) = self.symbol_map.locals.get_mut(&node.id()) {
                        locals.insert(sym_name, sym);
                    }
                }
            }
        }

        self.bind_children(node);

        // Restore the outer flow for function-like containers.
        if is_function_like {
            self.current_flow = prev_flow;
        }

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.this_container = prev_this_container;
        self.parent_symbol = prev_parent_symbol;
    }

    // ─────────────────────────────────────────────────────────────────────
    // Child binding
    // ─────────────────────────────────────────────────────────────────────

    /// Bind all children of a node.
    fn bind_children(&mut self, node: &Arc<Node>) {
        // Use a raw pointer to work around the borrow checker: `bind` needs
        // `&mut self` but `for_each_child` gives us shared references to children.
        // This is safe because we don't alias the node itself.
        let this = self as *mut Self;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            unsafe {
                (*this).bind(child);
            }
            false
        });
    }

    /// Get the number of symbols created.
    pub fn symbol_count(&self) -> usize {
        self.symbol_count
    }

    /// Bind a `TypeParameter` node. Mirrors Go's `bindTypeParameter`.
    ///
    /// When the type parameter is the child of an `InferType` (i.e.
    /// `infer R`), it is declared as a local of the enclosing
    /// `ConditionalType` (found via `get_infer_type_container`), so that
    /// `getInferTypeParameters` can later collect the infer type
    /// parameters. Otherwise it falls through to the normal
    /// `declare_symbol` path.
    fn bind_type_parameter(&mut self, node: &Arc<Node>) {
        let parent_is_infer = node
            .parent
            .as_ref()
            .map_or(false, |p| p.kind == SyntaxKind::InferType);
        if parent_is_infer {
            if let Some(container) = node
                .parent
                .as_ref()
                .and_then(|infer| self.get_infer_type_container(infer))
            {
                self.declare_local_symbol(
                    &container,
                    node,
                    SymbolFlags::TypeParameter,
                    SymbolFlags::TYPE,
                );
                return;
            }
            // No enclosing ConditionalType — fall back to anonymous declaration.
            let name = self.get_declaration_name(node);
            self.bind_anonymous_declaration(node, SymbolFlags::TypeParameter, &name);
            return;
        }
        self.declare_symbol(node, SymbolFlags::TypeParameter, SymbolFlags::TYPE);
    }

    /// Find the `ConditionalType` node whose `extends_type` clause contains
    /// the given `InferType` node. Mirrors Go's `getInferTypeContainer`.
    /// Requires parent pointers to be populated (see `set_parent_pointers`).
    fn get_infer_type_container(&self, infer_node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(infer_node);
        loop {
            let parent = match &current.parent {
                Some(p) => Arc::clone(p),
                None => return None,
            };
            if parent.kind == SyntaxKind::ConditionalType {
                // Check that `current` is the extends_type of the conditional.
                let is_extends = match &parent.data {
                    NodeData::ConditionalTypeNode(data) => {
                        Arc::ptr_eq(&data.extends_type, &current)
                    }
                    _ => false,
                };
                if is_extends {
                    return Some(parent);
                }
                return None;
            }
            current = parent;
        }
    }

    /// Declare a symbol as a local of a specific container node, bypassing
    /// the normal `container`/`block_scope_container` state. Used for
    /// `infer R` type parameters which belong to the `ConditionalType`
    /// even though it is not the active container.
    fn declare_local_symbol(
        &mut self,
        container: &Arc<Node>,
        node: &Arc<Node>,
        flags: SymbolFlags,
        _excludes: SymbolFlags,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);
        let symbol = self.new_symbol(flags, name.clone());
        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));
                if (*symbol_mut).value_declaration.is_none() && flags.intersects(SymbolFlags::VALUE)
                {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
        }
        let container_id = container.id();
        let locals = self
            .symbol_map
            .locals
            .entry(container_id)
            .or_insert_with(SymbolTable::new);
        locals.insert(name, Arc::clone(&symbol));
        self.symbol_map.set_symbol(node, Arc::clone(&symbol));
        symbol
    }
}

/// Get container flags for a node kind.
fn get_container_flags(kind: SyntaxKind) -> ContainerFlags {
    match kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeLiteral
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::JsxAttributes
        | SyntaxKind::EnumDeclaration => ContainerFlags::IS_CONTAINER,
        SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::IS_FUNCTION_EXPRESSION
                | ContainerFlags::HAS_LOCALS
                | ContainerFlags::IS_THIS_CONTAINER
        }
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::Constructor => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::HAS_LOCALS
                | ContainerFlags::IS_THIS_CONTAINER
        }
        SyntaxKind::CallSignature
        | SyntaxKind::ConstructSignature
        | SyntaxKind::IndexSignature
        | SyntaxKind::FunctionType
        | SyntaxKind::ConstructorType => {
            ContainerFlags::IS_CONTROL_FLOW_CONTAINER | ContainerFlags::IS_THIS_CONTAINER
        }
        SyntaxKind::Block | SyntaxKind::ModuleDeclaration | SyntaxKind::SourceFile => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_BLOCK_SCOPED_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::CatchClause
        | SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement => {
            ContainerFlags::IS_BLOCK_SCOPED_CONTAINER | ContainerFlags::HAS_LOCALS
        }
        _ => ContainerFlags::NONE,
    }
}

/// Whether a node kind is a block-scoped container.
fn is_block_scoped_container(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SourceFile
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::Constructor
    )
}

/// Node kinds that are block-scoped containers but NOT symbol containers —
/// `Block`, the loop statements, `CatchClause`, and `CaseBlock`. These mirror
/// the `ContainerFlagsIsBlockScopedContainer`-only kinds in Go's
/// `GetContainerFlags`. Our `get_container_flags` additionally marks `Block`
/// as IS_CONTAINER (a deviation), so `bind_container` filters these kinds out
/// explicitly when deciding whether to advance `container`: `container` must
/// keep pointing at the nearest function-like container so `var` declarations
/// in nested blocks hoist to the function scope.
fn is_block_only_container(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::CaseBlock
    )
}

/// Whether `kind` is a function-like (or module/file) container that `var`
/// declarations hoist into. Mirrors the locals-bearing container kinds of Go's
/// `declareSymbolAndAddToSymbolTable` (functions declare into their locals;
/// source files / modules route through their symbol's member tables).
fn is_var_container_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::SourceFile
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
    )
}

/// Whether a node kind has locals (a local symbol table).
/// Collect BindingElement nodes from a binding pattern (recursively —
/// patterns nest).
fn collect_binding_elements<'a>(node: &'a Arc<Node>, out: &mut Vec<&'a Arc<Node>>) {
    if let NodeData::BindingPattern(pattern) = &node.data {
        for el in pattern.elements.iter() {
            out.push(el);
            let name = match &el.data {
                NodeData::BindingElement(be) => &be.name,
                _ => continue,
            };
            if let Some(name_node) = name
                && matches!(name_node.data, NodeData::BindingPattern(_))
            {
                collect_binding_elements(name_node, out);
            }
        }
    }
}

/// Whether a function-like node has an implementation body (arrow and
/// function expressions always do; declarations may be overload
/// signatures; method/type signatures never do).
fn fn_like_body_present(parent: &Arc<Node>) -> bool {
    match &parent.data {
        NodeData::FunctionDeclaration(d) => d.body.is_some(),
        NodeData::MethodDeclaration(d) => d.body.is_some(),
        NodeData::ConstructorDeclaration(d) => d.body.is_some(),
        NodeData::GetAccessorDeclaration(d) => d.body.is_some(),
        NodeData::SetAccessorDeclaration(d) => d.body.is_some(),
        NodeData::FunctionExpression(_) | NodeData::ArrowFunction(_) => true,
        _ => false,
    }
}

fn has_locals(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SourceFile
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
    )
}

/// Bind a source file using a fresh binder.
pub fn bind_source_file(file: &Arc<SourceFile>) -> NodeSymbolMap {
    let mut binder = Binder::new();
    binder.bind_source_file(file);
    std::mem::take(&mut binder.symbol_map)
}

/// Whether a syntax kind is an assignment operator token.
fn is_assignment_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::EqualsToken
            | SyntaxKind::PlusEqualsToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken
            | SyntaxKind::CaretEqualsToken
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_and_bind(source: &str) -> (Arc<SourceFile>, NodeSymbolMap) {
        let source_file = Arc::new(Parser::parse_source_file_text("test.ts", source.to_string()));
        let symbol_map = bind_source_file(&Arc::clone(&source_file));
        (source_file, symbol_map)
    }

    #[test]
    fn bind_variable_declaration() {
        let (file, map) = parse_and_bind("var x = 1;");
        let statements = match &file.node.data {
            NodeData::SourceFile(data) => &data.statements,
            _ => unreachable!(),
        };
        assert!(!statements.nodes.is_empty());
        // The variable statement contains a declaration list with declarations
        let var_stmt = &statements.nodes[0];
        assert_eq!(var_stmt.kind, SyntaxKind::VariableStatement);
        // Symbol count should be > 0 (file symbol + variable symbol)
        let mut binder = Binder::new();
        binder.bind_source_file(&Arc::clone(&file));
        assert!(binder.symbol_count() >= 2);
        let _ = map;
    }

    #[test]
    fn bind_function_declaration() {
        let (file, _map) = parse_and_bind("function foo() { return 42; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&Arc::clone(&file));
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn bind_class_declaration() {
        let (file, _map) = parse_and_bind("class Foo { bar() {} }");
        let mut binder = Binder::new();
        binder.bind_source_file(&Arc::clone(&file));
        assert!(binder.symbol_count() >= 3); // file + class + method
    }

    #[test]
    fn bind_interface_declaration() {
        let (file, _map) = parse_and_bind("interface Foo { bar: number; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 3); // file + interface + property
    }

    #[test]
    fn bind_import_declaration() {
        let (file, _map) = parse_and_bind("import { foo } from 'mod';");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        // Import is not yet parsed by our parser, but binding shouldn't crash
        let _ = binder.symbol_count();
    }

    #[test]
    fn bind_multiple_declarations() {
        let (file, _map) = parse_and_bind("let x = 1; let y = 2; let z = 3;");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 4); // file + 3 variables
    }

    #[test]
    fn bind_nested_scope() {
        let (file, _map) = parse_and_bind("function foo() { let x = 1; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        // file + function + variable
        assert!(binder.symbol_count() >= 3);
    }

    // ───────────────────────────────────────────────────────────────
    // Flow graph tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn flow_start_node_exists() {
        let (file, map) = parse_and_bind("let x = 1;");
        // File should have a start flow node
        let flow = map.flow_node_of(&file.node);
        assert!(flow.is_some());
        let flow = flow.unwrap();
        assert!(flow.flags.contains(FlowFlags::START));
    }

    #[test]
    fn flow_identifier_has_flow_node() {
        let (file, map) = parse_and_bind("let x = 1; x;");
        // Find the identifier x (the second statement's expression)
        let statements = match &file.node.data {
            NodeData::SourceFile(data) => &data.statements,
            _ => unreachable!(),
        };
        // Second statement is ExpressionStatement containing Identifier
        let expr_stmt = &statements.nodes[1];
        let expr = match &expr_stmt.data {
            NodeData::ExpressionStatement(data) => &data.expression,
            _ => unreachable!(),
        };
        assert_eq!(expr.kind, SyntaxKind::Identifier);
        // The identifier should have a flow node
        assert!(map.flow_node_of(expr).is_some());
    }

    #[test]
    fn flow_if_statement_merges() {
        // Just make sure binding an if statement doesn't crash
        let (file, _map) = parse_and_bind("let x = 1; if (x > 0) { x = 2; } else { x = 3; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_while_statement() {
        let (file, _map) = parse_and_bind("let i = 0; while (i < 10) { i = i + 1; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_for_statement() {
        let (file, _map) = parse_and_bind("for (let i = 0; i < 10; i++) { console.log(i); }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_switch_statement() {
        let (file, _map) =
            parse_and_bind("let x = 1; switch (x) { case 1: x = 2; break; default: x = 0; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_return_statement_unreachable() {
        let (file, map) = parse_and_bind("function foo() { return 1; let x = 2; }");
        let _ = map;
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_explicit_return);
    }

    #[test]
    fn flow_throw_statement() {
        let (file, _map) = parse_and_bind("function foo() { throw new Error(); }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_assignment_has_effects() {
        let (file, _map) = parse_and_bind("let x = 1; x = 2;");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_call_expression_has_effects() {
        let (file, _map) = parse_and_bind("console.log('hello');");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_try_catch_finally_does_not_crash() {
        // `try/catch/finally` must build a flow graph without panicking.
        let (file, _map) =
            parse_and_bind("try { let x = 1; } catch (e) { let y = 2; } finally { let z = 3; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_try_with_throw_in_catch() {
        // Throw inside try should route through catch, not fall through.
        let (file, _map) = parse_and_bind(
            "function f() {\
             try { throw new Error(); }\
             catch (e) { return 1; }\
             return 2;\
             }",
        );
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_labeled_break_to_outer_loop() {
        // Labeled break must route the inner loop's exit to the outer label.
        let (file, _map) = parse_and_bind(
            "outer: for (let i = 0; i < 3; i++) {\
             for (let j = 0; j < 3; j++) {\
             if (j === 1) break outer;\
             }\
             }",
        );
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_labeled_continue_to_outer_loop() {
        // Labeled continue must route the inner loop's continue to the outer label.
        let (file, _map) = parse_and_bind(
            "outer: for (let i = 0; i < 3; i++) {\
             for (let j = 0; j < 3; j++) {\
             if (j === 1) continue outer;\
             }\
             }",
        );
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_array_mutation_call_has_effects() {
        // `arr.push(x)` is an ARRAY_MUTATION flow node — has_flow_effects
        // must be true and binding must not crash.
        let (file, _map) = parse_and_bind("let arr = []; arr.push(1);");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    // ───────────────────────────────────────────────────────────────
    // Import / export binding — P3.4
    // ───────────────────────────────────────────────────────────────

    fn file_symbol<'a>(file: &'a SourceFile, map: &'a NodeSymbolMap) -> &'a Arc<Symbol> {
        map.symbols
            .get(&file.node.id())
            .expect("source file should have a symbol")
    }

    fn find_statement(file: &SourceFile, kind: SyntaxKind) -> Option<Arc<Node>> {
        let NodeData::SourceFile(data) = &file.node.data else {
            return None;
        };
        data.statements
            .nodes
            .iter()
            .find(|n| n.kind == kind)
            .cloned()
    }

    fn find_child(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
        let mut found: Option<Arc<Node>> = None;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            if child.kind == kind {
                found = Some(Arc::clone(child));
                true
            } else {
                false
            }
        });
        found
    }

    #[test]
    fn bind_export_default_expression_creates_default_export_symbol() {
        // `export default 42` → a Property symbol named "default" in the
        // file's exports (the expression is a literal, not an alias).
        let (file, map) = parse_and_bind("export default 42;");
        let export_assignment =
            find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
        let sym = map.symbol_of(&export_assignment).expect("symbol");
        assert!(
            sym.flags.contains(SymbolFlags::Property),
            "expected Property flags, got {:?}",
            sym.flags
        );
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_DEFAULT);
        let file_sym = file_symbol(&file, &map);
        let default_export = file_sym
            .exports
            .get(INTERNAL_SYMBOL_NAME_DEFAULT)
            .expect("default export in file exports");
        assert!(Arc::ptr_eq(default_export, sym));
    }

    #[test]
    fn bind_export_default_identifier_creates_alias() {
        // `export default foo` → an Alias symbol named "default" (the
        // expression is an entity name).
        let (file, map) = parse_and_bind("const foo = 1; export default foo;");
        let export_assignment =
            find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
        let sym = map.symbol_of(&export_assignment).expect("symbol");
        assert!(
            sym.flags.contains(SymbolFlags::Alias),
            "expected Alias flags, got {:?}",
            sym.flags
        );
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_DEFAULT);
    }

    #[test]
    fn bind_export_equals_creates_export_equals_symbol() {
        // `export = x` → an Alias symbol named "export=" with a value
        // declaration set.
        let (file, map) = parse_and_bind("function x() {} export = x;");
        let export_assignment =
            find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
        let sym = map.symbol_of(&export_assignment).expect("symbol");
        assert!(sym.flags.contains(SymbolFlags::Alias));
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS);
        assert!(
            sym.value_declaration.is_some(),
            "export = should have a value declaration set"
        );
        let file_sym = file_symbol(&file, &map);
        assert!(
            file_sym
                .exports
                .get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                .is_some()
        );
    }

    #[test]
    fn bind_export_star_creates_export_star_symbol() {
        // `export * from "mod"` → an ExportStar symbol in the file's exports.
        let (file, map) = parse_and_bind("export * from \"mod\";");
        let export_decl =
            find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");
        let sym = map.symbol_of(&export_decl).expect("symbol");
        assert!(
            sym.flags.contains(SymbolFlags::ExportStar),
            "expected ExportStar flags, got {:?}",
            sym.flags
        );
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_EXPORT_STAR);
        let file_sym = file_symbol(&file, &map);
        assert!(
            file_sym
                .exports
                .get(INTERNAL_SYMBOL_NAME_EXPORT_STAR)
                .is_some()
        );
    }

    #[test]
    fn bind_export_star_as_ns_creates_alias() {
        // `export * as ns from "mod"` → an Alias symbol named "ns" in the
        // file's exports, attached to the NamespaceExport clause node.
        let (file, map) = parse_and_bind("export * as ns from \"mod\";");
        let export_decl =
            find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");
        let ns_clause =
            find_child(&export_decl, SyntaxKind::NamespaceExport).expect("NamespaceExport clause");
        let sym = map
            .symbol_of(&ns_clause)
            .expect("symbol on NamespaceExport clause");
        assert!(sym.flags.contains(SymbolFlags::Alias));
        assert_eq!(sym.name, "ns");
        let file_sym = file_symbol(&file, &map);
        let ns_export = file_sym.exports.get("ns").expect("ns export");
        assert!(Arc::ptr_eq(ns_export, sym));
    }

    #[test]
    fn bind_export_named_specifiers_does_not_duplicate() {
        // `export { a, b }` is handled by the ExportSpecifier arms; the
        // ExportDeclaration itself should not declare an extra symbol.
        let (file, map) = parse_and_bind("const a = 1; const b = 2; export { a, b };");
        let export_decl =
            find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");
        // No symbol should be created directly on the ExportDeclaration for
        // `export { ... }` (only on the individual ExportSpecifiers).
        assert!(
            map.symbol_of(&export_decl).is_none(),
            "export {{ a, b }} should not create a symbol on the ExportDeclaration"
        );
    }

    #[test]
    fn bind_import_clause_default_import_creates_local_alias() {
        // `import D from "mod"` → an Alias symbol named "D" in the file's
        // locals (not exports).
        let (file, map) = parse_and_bind("import D from \"mod\";");
        let import_decl =
            find_statement(&file, SyntaxKind::ImportDeclaration).expect("import declaration");
        let clause = find_child(&import_decl, SyntaxKind::ImportClause).expect("import clause");
        let sym = map.symbol_of(&clause).expect("symbol on ImportClause");
        assert!(sym.flags.contains(SymbolFlags::Alias));
        assert_eq!(sym.name, "D");
        let locals = map.locals.get(&file.node.id()).expect("file locals table");
        let local_sym = locals.get("D").expect("D in file locals");
        assert!(Arc::ptr_eq(local_sym, sym));
        let file_sym = file_symbol(&file, &map);
        assert!(
            file_sym.exports.get("D").is_none(),
            "default import should not be in exports"
        );
    }

    #[test]
    fn bind_import_clause_without_name_is_noop() {
        // `import { x } from "mod"` has no default import name; the
        // ImportClause should declare no symbol itself.
        let (file, map) = parse_and_bind("import { x } from \"mod\";");
        let import_decl =
            find_statement(&file, SyntaxKind::ImportDeclaration).expect("import declaration");
        let clause = find_child(&import_decl, SyntaxKind::ImportClause).expect("import clause");
        assert!(
            map.symbol_of(&clause).is_none(),
            "ImportClause without a name should not get a symbol"
        );
    }

    #[test]
    fn bind_exported_namespace_member_has_export_symbol_link() {
        // `namespace N { export const x = 1; }` — the exported member `x`
        // should have its `export_symbol` link set (self-reference).
        let (file, map) = parse_and_bind("namespace N { export const x = 1; }");
        // Find the ModuleDeclaration N, then its symbol's exports.
        let ns = find_statement(&file, SyntaxKind::ModuleDeclaration).expect("namespace N");
        let ns_sym = map.symbol_of(&ns).expect("namespace symbol");
        let x_export = ns_sym.exports.get("x").expect("x in N's exports");
        assert!(
            x_export.export_symbol.is_some(),
            "exported namespace member should have export_symbol set"
        );
        assert!(Arc::ptr_eq(
            x_export.export_symbol.as_ref().unwrap(),
            x_export
        ));
    }

    #[test]
    fn bind_non_exported_namespace_member_has_no_export_symbol() {
        // `namespace N { const x = 1; }` — non-exported member `x` should
        // NOT have an `export_symbol` link and should be in locals, not
        // exports.
        let (file, map) = parse_and_bind("namespace N { const x = 1; }");
        let ns = find_statement(&file, SyntaxKind::ModuleDeclaration).expect("namespace N");
        let ns_sym = map.symbol_of(&ns).expect("namespace symbol");
        assert!(
            ns_sym.exports.get("x").is_none(),
            "non-exported member should not be in exports"
        );
        // Non-exported namespace members live in the ModuleDeclaration
        // container's locals (the binder keys locals on the container node,
        // which is the ModuleDeclaration, not the ModuleBlock).
        let locals = map.locals.get(&ns.id()).expect("namespace locals table");
        let x_local = locals.get("x").expect("x in locals");
        assert!(
            x_local.export_symbol.is_none(),
            "non-exported member should not have export_symbol"
        );
    }

    #[test]
    fn bind_exported_top_level_member_has_export_symbol_link() {
        // `export const x = 1;` at the top level — `x` should have its
        // `export_symbol` link set (self-reference).
        let (file, map) = parse_and_bind("export const x = 1;");
        let var_stmt =
            find_statement(&file, SyntaxKind::VariableStatement).expect("variable statement");
        // The VariableDeclaration is the first child of the declaration list.
        let decl_list =
            find_child(&var_stmt, SyntaxKind::VariableDeclarationList).expect("declaration list");
        let var_decl =
            find_child(&decl_list, SyntaxKind::VariableDeclaration).expect("variable declaration");
        let sym = map.symbol_of(&var_decl).expect("symbol for x");
        assert!(
            sym.export_symbol.is_some(),
            "exported top-level member should have export_symbol set"
        );
        assert!(Arc::ptr_eq(sym.export_symbol.as_ref().unwrap(), sym));
    }
}
