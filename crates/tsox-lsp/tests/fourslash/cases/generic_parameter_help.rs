use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_parameter_help() {
    let content = r#"interface IFoo { }

function testFunction<T extends IFoo, U, M extends IFoo>(a: T, b: U, c: M): M {
    return null;
}

// Function calls
testFunction</*1*/
testFunction<any, /*2*/
testFunction<any, any, any>(/*3*/
testFunction<any, any,/*4*/ any>(null, null, null);
testFunction<, ,/*5*/>(null, null, null);"#;
    let mut s = Session::new_for_test("genericParameterHelp", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "testFunction<T extends IFoo, U,
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "U", ParameterSpan: "U"
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "a", ParameterSpan: "a:
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "M", ParameterSpan: "M 
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "M", ParameterSpan: "M 
}
