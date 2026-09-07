use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"0", "1"}, &fourslash.CompletionsExpectedList{
}
