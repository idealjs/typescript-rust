use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.GoToEOF"]
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
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
