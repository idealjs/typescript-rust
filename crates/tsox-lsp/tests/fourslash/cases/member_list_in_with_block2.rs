use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn member_list_in_with_block2() {
    let content = r#"interface IFoo {
    a: number;
}

with (x) {
    var y: IFoo = { /*1*/ };
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", nil)
}
