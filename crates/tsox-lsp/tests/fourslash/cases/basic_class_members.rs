use tsox_lsp::fourslash::{self, Session};


#[test]
fn basic_class_members() {
    let content = r#"class n {
    constructor (public x: number, public y: number, private z: string) { }
}
var t = new n(0, 1, '');"#;
    let mut s = Session::new_for_test("basicClassMembers", content);
    // TODO: f.GoToEOF(t)
    fourslash::insert(&mut s, "t.");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
