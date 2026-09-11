use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_on_function_call_with_optional_argument() {
    let content = r#"declare function Foo(arg1?: Function): { q: number };
Foo(function () { } )./**/;"#;
    let mut s = Session::new_for_test("completionListOnFunctionCallWithOptionalArgument", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["q"]);
}
