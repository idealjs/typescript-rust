use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn generic_parameter_help_type_references() {
    let content = r#"interface IFoo { }

class testClass<T extends IFoo, U, M extends IFoo> {
    constructor(a:T, b:U, c:M){ }
}

// Generic types
testClass</*type1*/
var x : testClass</*type2*/
class Bar<T> extends testClass</*type3*/
var x : testClass<,, /*type4*/any>;

interface I<T> {}
let i: I</*interface*/>;

type Ty<T> = T;
let t: Ty</*typeAlias*/>;"#;
    let mut s = Session::new_for_test("genericParameterHelpTypeReferences", content);
    fourslash::go_to_marker(&mut s, "type1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "testClass<T extends IFoo, U, M 
    fourslash::go_to_marker(&mut s, "type2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "testClass<T extends IFoo, U, M 
    fourslash::go_to_marker(&mut s, "type3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "testClass<T extends IFoo, U, M 
    fourslash::go_to_marker(&mut s, "type4");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "M", ParameterSpan: "M 
    fourslash::go_to_marker(&mut s, "interface");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "I<T>", ParameterName: "T", Para
    fourslash::go_to_marker(&mut s, "typeAlias");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Ty<T>", ParameterName: "T", Par
}
