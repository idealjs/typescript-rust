use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_coalesce_imports_sort_specifiers_case_insensitive() {
    let content = r#"import { default as M, a as n, B, y, Z as O } from "lib";
M; n; B; y; O;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_sortSpecifiersCaseInsensitive", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_combine_side_effect_only() {
    let content = r#"import "lib";
import "lib";
void 0;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_combineSideEffectOnly", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_combine_namespace_imports_not_merged() {
    // TODO: // Namespace imports from the same module should not be merged into one.
    let content = r#"import * as x from "lib";
import * as y from "lib";
import { z } from "aaa";
x; y; z;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_combineNamespaceImportsNotMerged", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_combine_default_imports() {
    let content = r#"import x from "lib";
import y from "lib";
x; y;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_combineDefaultImports", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_combine_property_imports() {
    let content = r#"import { x } from "lib";
import { y as z } from "lib";
x; z;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_combinePropertyImports", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_side_effect_with_namespace() {
    // TODO: // Side-effect-only import and namespace import from same module should not be combined.
    let content = r#"import "lib";
import * as x from "lib";
import { z } from "aaa";
x; z;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_sideEffectWithNamespace", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_side_effect_with_default() {
    // TODO: // Side-effect-only import and default import from same module should not be combined.
    let content = r#"import "lib";
import x from "lib";
import { z } from "aaa";
x; z;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_sideEffectWithDefault", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_side_effect_with_property() {
    // TODO: // Side-effect-only import and property import from same module should not be combined.
    let content = r#"import "lib";
import { x } from "lib";
import { z } from "aaa";
x; z;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_sideEffectWithProperty", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_namespace_with_default() {
    // TODO: // Namespace import and default import from same module should be combined.
    let content = r#"import * as x from "lib";
import y from "lib";
x; y;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_namespaceWithDefault", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_namespace_with_property() {
    // TODO: // Namespace import and property import from same module should not be combined.
    let content = r#"import * as x from "lib";
import { y } from "lib";
import { z } from "aaa";
x; y; z;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_namespaceWithProperty", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_default_with_property() {
    // TODO: // Default import and property import from same module should be combined.
    let content = r#"import x from "lib";
import { y } from "lib";
x; y;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_defaultWithProperty", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_combine_many() {
    let content = r#"import "lib";
import * as y from "lib";
import w from "lib";
import { b } from "lib";
import "lib";
import * as x from "lib";
import z from "lib";
import { a } from "lib";
w; x; y; z; a; b;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_combineMany", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_two_namespaces_one_default() {
    // TODO: // Descriptive test: two namespace imports + one default should not combine.
    let content = r#"import * as x from "lib";
import * as y from "lib";
import z from "lib";
import { w } from "aaa";
x; y; z; w;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_twoNamespacesOneDefault", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_type_only_separate() {
    // TODO: // Type-only imports should be coalesced separately from value imports.
    let content = r#"import type { x } from "lib";
import type { y } from "lib";
import { z } from "lib";
x; y; z;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_typeOnlySeparate", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_type_only_kinds_not_combined() {
    // TODO: // Type-only default, namespace, and named imports should not be combined with each other.
    let content = r#"import type { x } from "lib";
import type * as y from "lib";
import type z from "lib";
x; y; z;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_typeOnlyKindsNotCombined", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_coalesce_imports_sort_specifiers_type_only_inline() {
    let content = r#"import { type z, y, type x, c, type b, a } from "lib";
z; y; x; c; b; a;"#;
    let _s = Session::new_for_test("organizeImports_coalesceImports_sortSpecifiersTypeOnlyInline", content);
    // TODO: f.VerifyOrganizeImports(
}
