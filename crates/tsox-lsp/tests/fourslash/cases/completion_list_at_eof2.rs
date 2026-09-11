use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_eof2() {
    let content = r#"namespace Shapes {
    export class Point {
        constructor(public x: number, public y: number) { }
    }
}
var p = <Shapes."#;
    let mut s = Session::new_for_test("completionListAtEOF2", content);
    // TODO: f.GoToEOF(t)
    fourslash::verify_completions_exact_at(&mut s, None, &["Point"]);
}
