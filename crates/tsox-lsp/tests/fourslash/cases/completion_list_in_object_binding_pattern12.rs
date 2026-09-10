use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_object_binding_pattern12() {
    let content = r#"interface I {
    property1: number;
    property2: string;
}

function f({ property1, /**/ }: I): void {
}"#;
    let mut s = Session::new_for_test("completionListInObjectBindingPattern12", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
