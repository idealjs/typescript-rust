use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsx_qualified_tag_completion() {
    let content = r#"//@Filename: file.tsx
declare var React: any;
namespace NS {
    export var Foo: any = null;
}
const j = <NS.Foo>Hello!/**/
"#;
    let mut s = Session::new_for_test("jsxQualifiedTagCompletion", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "</");
    fourslash::verify_completions_exact_at(&mut s, None, &["NS.Foo>"]);
}
