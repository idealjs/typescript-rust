use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_contextual_construct_signature_no_crash() {
    let content = r#"
type Obj = {
    foo: new () => object
}

let obj: Obj = {
    foo(/*constructOnly*/) {}
}
"#;
    let mut s = Session::new_for_test("signatureHelpContextualConstructSignatureNoCrash", content);
    // TODO: // When contextual type only has construct signatures (no call signatures),
    // TODO: // no signature help should be provided (and no panic should occur).
    fourslash::go_to_marker(&mut s, "constructOnly");
    // TODO: f.VerifyNoSignatureHelp(t)
}
