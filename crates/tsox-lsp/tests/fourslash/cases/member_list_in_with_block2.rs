use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_in_with_block2() {
    let content = r#"interface IFoo {
    a: number;
}

with (x) {
    var y: IFoo = { /*1*/ };
}"#;
    let mut s = Session::new_for_test("memberListInWithBlock2", content);
    fourslash::verify_completions_empty_at(&mut s, Some("1"));
}
