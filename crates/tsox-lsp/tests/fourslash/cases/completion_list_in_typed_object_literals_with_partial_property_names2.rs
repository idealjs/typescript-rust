use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_typed_object_literals_with_partial_property_names2() {
    let content = r#"interface MyPoint {
    x1: number;
    y1: number;
}
var p15: MyPoint = {
    /**/x1: 0,
};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
