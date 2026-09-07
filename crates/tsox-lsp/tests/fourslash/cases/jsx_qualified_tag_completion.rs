use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsx_qualified_tag_completion() {
    let content = r#"//@Filename: file.tsx
declare var React: any;
namespace NS {
    export var Foo: any = null;
}
const j = <NS.Foo>Hello!/**/
"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "</");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
