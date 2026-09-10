use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_reexport_transient() {
    let content = r#"// @lib: es5
// @esModuleInterop: true
// @Filename: /transient.d.ts
declare const map: { [K in "one"]: number };
export = map;
// @Filename: /r1.ts
export { one } from "./transient";
// @Filename: /r2.ts
export { one } from "./r1";
// @Filename: /index.ts
one/**/"#;
    let mut s = Session::new_for_test("completionsImport_reexportTransient", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
