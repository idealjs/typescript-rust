use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Backspace"]
#[test]
fn get_java_script_completions16() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: file.js
"use strict";

class Something {

    /**
     * @param {number} a
     */
    constructor(a, b) {
        a/*body*/
    }

    /**
     * @param {number} a
     */
    method(a) {
        a/*method*/
    }
}
let x = new Something(/*sig*/);"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "body");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("Backspace"); // f.Backspace(t, 1)
    fourslash::go_to_marker(&mut s, "sig");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Something(a: number, b: any): S
    fourslash::go_to_marker(&mut s, "method");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
