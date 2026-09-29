use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_namespace_merged_with_class() {
    let content = r#"// @lib: es5
class C {
    static m() { }
}

class D extends C {}
namespace D {
    export type T = number;
}

let x: D./*type*/;
D./*value*/"#;
    let mut s = Session::new_for_test("completionsNamespaceMergedWithClass", content);
    fourslash::verify_completions_exact_at(&mut s, Some("type"), &["T"]);
    fourslash::go_to_marker(&mut s, "value");
    // TODO: f.VerifyCompletions(t, "value", &fourslash.CompletionsExpectedList{
}
