use tsox_lsp::fourslash::{self, Session};


#[test]
fn basic_edit() {
    let content = r#"export {};
interface Point {
	x: number;
	y: number;
}
declare const p: Point;
p/*a*/"#;
    let mut s = Session::new_for_test("basicEdit", content);
    fourslash::go_to_marker(&mut s, "a");
    fourslash::insert(&mut s, ".");
    fourslash::go_to_eof(&mut s, );
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
