use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_nested_calls() {
    let content = r#"function foo(s: string) { return s; }
function bar(s: string) { return s; }
let s = foo(/*a*/ /*b*/bar/*c*/(/*d*/"hello"/*e*/)/*f*/);"#;
    let mut s = Session::new_for_test("signatureHelpNestedCalls", content);
    // TODO: // Markers a, b, c should show foo (outer call)
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(s: string): string"})
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(s: string): string"})
    fourslash::go_to_marker(&mut s, "c");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(s: string): string"})
    // TODO: // Markers d, e should show bar (inside inner call, including the end boundary)
    fourslash::go_to_marker(&mut s, "d");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "bar(s: string): string"})
    fourslash::go_to_marker(&mut s, "e");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "bar(s: string): string"})
    // TODO: // Marker f should show foo (after the inner call closes)
    fourslash::go_to_marker(&mut s, "f");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(s: string): string"})
}

#[test]
fn signature_help_empty_inner_call() {
    let content = r#"function foo(s: string) { return s; }
function bar(s: string) { return s; }
let s = foo(bar(/*a*/));"#;
    let mut s = Session::new_for_test("signatureHelpEmptyInnerCall", content);
    // TODO: // Marker a should show bar even though the inner argument list is empty.
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "bar(s: string): string"})
}
