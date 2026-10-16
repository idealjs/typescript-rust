use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_named_export_equals_namespace_merged() {
    let content = r#"// @module: esnext
// @Filename: /b.d.ts
declare namespace N {
    export const foo: number;
}
declare module "n" {
    export = N;
}
// @Filename: /c.d.ts
declare namespace N {}
// @Filename: /a.ts
fo/**/"#;
    let mut s = Session::new_for_test("completionsImport_named_exportEqualsNamespace_merged", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
