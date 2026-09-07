use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // Marker f should show foo (after the inner call closes)"]
#[test]
fn signature_help_nested_calls() {
    let content = r#"function foo(s: string) { return s; }
function bar(s: string) { return s; }
let s = foo(/*a*/ /*b*/bar/*c*/(/*d*/"hello"/*e*/)/*f*/);"#;
    let mut s = Session::new(content);
    // TODO: // Markers a, b, c should show foo (outer call)
    fourslash::go_to_marker(&mut s, "a");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(s: string): string"})
    fourslash::go_to_marker(&mut s, "b");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(s: string): string"})
    fourslash::go_to_marker(&mut s, "c");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(s: string): string"})
    // TODO: // Markers d, e should show bar (inside inner call, including the end boundary)
    fourslash::go_to_marker(&mut s, "d");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "bar(s: string): string"})
    fourslash::go_to_marker(&mut s, "e");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "bar(s: string): string"})
    // TODO: // Marker f should show foo (after the inner call closes)
    fourslash::go_to_marker(&mut s, "f");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(s: string): string"})
}

#[ignore = "generator: // Marker a should show bar even though the inner argument l"]
#[test]
fn signature_help_empty_inner_call() {
    let content = r#"function foo(s: string) { return s; }
function bar(s: string) { return s; }
let s = foo(bar(/*a*/));"#;
    let mut s = Session::new(content);
    // TODO: // Marker a should show bar even though the inner argument list is empty.
    fourslash::go_to_marker(&mut s, "a");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "bar(s: string): string"})
}
