use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_parameter_help_constructor_calls() {
    let content = r#"interface IFoo { }

class testClass<T extends IFoo, U, M extends IFoo> {
    constructor(a:T, b:U, c:M){ }
}

// Constructor calls
new testClass</*constructor1*/
new testClass<IFoo, /*constructor2*/
new testClass</*constructor3*/>(null, null, null)
new testClass<,,/*constructor4*/>(null, null, null)
new testClass<IFoo,/*constructor5*/IFoo,IFoo>(null, null, null)"#;
    let mut s = Session::new_for_test("genericParameterHelpConstructorCalls", content);
    fourslash::go_to_marker(&mut s, "constructor1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "testClass<T extends IFoo, U, M 
    fourslash::go_to_marker(&mut s, "constructor2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "U", ParameterSpan: "U"
    fourslash::go_to_marker(&mut s, "constructor3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "T", ParameterSpan: "T 
    fourslash::go_to_marker(&mut s, "constructor4");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "M", ParameterSpan: "M 
    fourslash::go_to_marker(&mut s, "constructor5");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "U", ParameterSpan: "U"
}
