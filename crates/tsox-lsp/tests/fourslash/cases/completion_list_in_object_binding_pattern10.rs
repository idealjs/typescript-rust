use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_object_binding_pattern10() {
    let content = r#"interface I {
    propertyOfI_1: number;
    propertyOfI_2: string;
}
interface J {
    property1: I;
    property2: string;
}

var foo: J[];
var [{ property1: { propertyOfI_1, }, /*1*/ }, { /*2*/ }] = foo;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
