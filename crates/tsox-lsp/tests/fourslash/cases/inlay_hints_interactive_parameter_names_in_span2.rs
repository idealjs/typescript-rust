use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: end := f.MarkerByName(t, 'h')"]
#[test]
fn inlay_hints_interactive_parameter_names_in_span2() {
    let content = r#"function foo1 (a: number, b: number) {}
function foo2 (c: number, d: number) {}
function foo3 (e: number, f: number) {}
function foo4 (g: number, h: number) {}
function foo5 (i: number, j: number) {}
function foo6 (k: number, l: number) {}

foo1(/*a*/1, /*b*/2);
foo2(/*c*/1, /*d*/2);
foo3(/*e*/1, /*f*/2);
foo4(/*g*/1, /*h*/2);
foo5(/*i*/1, /*j*/2);
foo6(/*k*/1, /*l*/2);"#;
    let mut s = Session::new(content);
    // TODO: start := f.MarkerByName(t, "c")
    // TODO: end := f.MarkerByName(t, "h")
    // TODO: span := &lsproto.Range{Start: start.LSPosition, End: end.LSPosition}
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, span, &lsutil.UserPreferences{
}
