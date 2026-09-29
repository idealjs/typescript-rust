use tsox_lsp::fourslash::{self, Session};


#[test]
fn basic_interface_members() {
    let content = r#"export {};
interface Point {
	x: number;
	y: number;
}
declare const p: Point;
p./*a*/"#;
    let mut s = Session::new_for_test("basicInterfaceMembers", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
