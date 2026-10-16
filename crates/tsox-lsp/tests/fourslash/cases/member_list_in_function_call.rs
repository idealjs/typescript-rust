use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_in_function_call() {
    let content = r#"function aa(x: any) {}
aa({
  "1": function () {
    var b = "";
    b/**/;
  }
});"#;
    let mut s = Session::new_for_test("memberListInFunctionCall", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ".");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["charAt"], &[]);
}
