use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_destructuring() {
    let content = r#"const points = [{ x: 1, y: 2 }];
points.forEach(({ /*a*/ }) => { });
const { /*b*/ } = points[0];
for (const { /*c*/ } of points) {}"#;
    let mut s = Session::new_for_test("completionsDestructuring", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
