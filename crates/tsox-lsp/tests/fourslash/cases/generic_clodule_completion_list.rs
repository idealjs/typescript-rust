use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_clodule_completion_list() {
    let content = r#"class D<T> { x: number }
namespace D { export function f() { } }
var d: D<number>;
d./**/"#;
    let mut s = Session::new_for_test("genericCloduleCompletionList", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["x"]);
}
