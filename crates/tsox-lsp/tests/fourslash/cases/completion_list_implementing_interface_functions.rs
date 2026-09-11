use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_implementing_interface_functions() {
    let content = r#"interface I1 {
    a(): void;
    b(): void;
}

var imp1: I1 = {
    a() {},
    /*0*/
}

var imp2: I1 = {
    a: () => {},
    /*1*/
}"#;
    let mut s = Session::new_for_test("completionListImplementingInterfaceFunctions", content);
    // TODO: f.VerifyCompletions(t, []string{"0", "1"}, &fourslash.CompletionsExpectedList{
}
