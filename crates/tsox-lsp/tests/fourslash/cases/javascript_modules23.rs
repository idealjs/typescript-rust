use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn javascript_modules23() {
    let content = r#"// @Filename: mod.ts
var foo = {a: "test"};
export = foo;
// @Filename: app.ts
import {a} from "./mod"
a./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
