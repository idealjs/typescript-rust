use tsox_lsp::fourslash::Session;


#[test]
fn completions_destructuring() {
    let content = r#"const points = [{ x: 1, y: 2 }];
points.forEach(({ /*a*/ }) => { });
const { /*b*/ } = points[0];
for (const { /*c*/ } of points) {}"#;
    let _s = Session::new_for_test("completionsDestructuring", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
