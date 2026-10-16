use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_literal() {
    let content = r#"interface point {
    x: number;
    y: number;
}
interface thing {
    name: string;
    pos: point;
}
var t: thing;
t.pos = { x: 4, y: 3 + t./**/ };"#;
    let mut s = Session::new_for_test("completionListInObjectLiteral", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["name", "pos"]);
}
