use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn hover_self_re_exported_namespace_generic_no_crash() {
    let content = r#"// @filename: mod.ts
export interface Box<A> { content: Content<A> }
export type Content<A> = A
export * as Box from "./mod"
// @filename: main.ts
import { Box } from "./mod"
declare const b: Box<string>
const x = b./*1*/content
"#;
    let mut s = Session::new_for_test("hoverSelfReExportedNamespaceGenericNoCrash", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn hover_self_re_exported_namespace_generic_class_no_crash() {
    let content = r#"// @filename: mod.ts
export class Box<A> { content!: A }
export * as Box from "./mod"
// @filename: main.ts
import { Box } from "./mod"
declare const b: Box<string>
const x = b./*1*/content
"#;
    let mut s = Session::new_for_test("hoverSelfReExportedNamespaceGenericClassNoCrash", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn hover_namespace_export_generic_non_colliding() {
    let content = r#"// @filename: mod.ts
export interface Box<A> { content: A }
export * as BoxNS from "./mod"
// @filename: main.ts
import { Box } from "./mod"
declare const b: Box<string>
const x = b./*1*/content
"#;
    let mut s = Session::new_for_test("hoverNamespaceExportGenericNonColliding", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
