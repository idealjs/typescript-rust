use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_namespace_merged_with_object() {
    let content = r#"namespace N {
    export type T = number;
}
const N = { m() {} };
let x: N./*type*/;
N./*value*/;"#;
    let mut s = Session::new_for_test("completionsNamespaceMergedWithObject", content);
    fourslash::verify_completions_exact_at(&mut s, Some("type"), &["T"]);
    fourslash::verify_completions_exact_at(&mut s, Some("value"), &["m"]);
}
