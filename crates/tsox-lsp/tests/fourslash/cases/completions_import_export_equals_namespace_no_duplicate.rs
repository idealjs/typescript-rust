use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_export_equals_namespace_no_duplicate() {
    let content = r#"// @Filename: /node_modules/a/index.d.ts
declare namespace core {
    const foo: number;
}
declare module "a" {
    export = core;
}
declare module "a/alias" {
    export = core;
}
// @Filename: /user.ts
import * as a from "a";
/**/foo;"#;
    let mut s = Session::new_for_test("completionsImport_exportEqualsNamespace_noDuplicate", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
