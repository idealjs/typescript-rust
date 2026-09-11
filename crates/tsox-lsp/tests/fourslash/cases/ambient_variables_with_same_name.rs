use tsox_lsp::fourslash::{self, Session};


#[test]
fn ambient_variables_with_same_name() {
    let content = r#"declare namespace M {
    export var x: string;
}
declare var x: number;"#;
    let mut s = Session::new_for_test("ambientVariablesWithSameName", content);
    // TODO: f.GoToEOF(t)
    // TODO: f.InsertLine(t, "")
    fourslash::verify_no_errors(&mut s, );
}
