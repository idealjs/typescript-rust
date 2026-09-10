use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
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
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "testFunction<T extends IFoo, U,
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "U", ParameterSpan: "U"
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "a", ParameterSpan: "a:
    fourslash::go_to_marker(&mut s, "4");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "M", ParameterSpan: "M 
    fourslash::go_to_marker(&mut s, "5");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "M", ParameterSpan: "M 
}
