use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("getJavaScriptCompletions16", content);
    fourslash::go_to_marker(&mut s, "body");
    fourslash::insert(&mut s, ".");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    // TODO: f.Backspace(t, 1)
    fourslash::go_to_marker(&mut s, "sig");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Something(a: number, b: any): S
    fourslash::go_to_marker(&mut s, "method");
    fourslash::insert(&mut s, ".");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
