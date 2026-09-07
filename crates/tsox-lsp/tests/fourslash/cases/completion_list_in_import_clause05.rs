use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_import_clause05() {
    let content = r#"// @Filename: app.ts
import * as A from "/*1*/";
// @Filename: /node_modules/@types/a__b/index.d.ts
declare module "@e/f" { function fun(): string; }
// @Filename: /node_modules/@types/c__d/index.d.ts
export declare let x: number;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
