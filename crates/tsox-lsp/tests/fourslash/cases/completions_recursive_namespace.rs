use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_recursive_namespace() {
    let content = r#"declare namespace N {
    export import M = N;
}
type T = N./**/"#;
    let mut s = Session::new_for_test("completionsRecursiveNamespace", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
