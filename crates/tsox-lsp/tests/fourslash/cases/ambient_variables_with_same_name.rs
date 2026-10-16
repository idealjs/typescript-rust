use tsox_lsp::fourslash::{self, Session};


#[test]
fn ambient_variables_with_same_name() {
    let content = r#"declare namespace M {
    export var x: string;
}
declare var x: number;"#;
    let mut s = Session::new_for_test("ambientVariablesWithSameName", content);
    fourslash::go_to_eof(&mut s, );
    fourslash::insert_line(&mut s, "");
    fourslash::verify_no_errors(&mut s, );
}
