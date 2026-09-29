use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_export_equals_global() {
    let content = r#"// @lib: es5
// @module: es6
// @Filename: /console.d.ts
 interface Console {}
 declare var console: Console;
 declare module "console" {
   export = console;
 }
// @Filename: /react-native.d.ts
 import 'console';
 declare global {
   interface Console {}
   var console: Console;
 }
// @Filename: /a.ts
conso/**/"#;
    let mut s = Session::new_for_test("completionsImport_exportEquals_global", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
