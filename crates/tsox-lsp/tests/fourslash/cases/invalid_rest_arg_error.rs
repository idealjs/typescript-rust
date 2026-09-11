use tsox_lsp::fourslash::{self, Session};


#[test]
fn invalid_rest_arg_error() {
    let content = r#"function b(.../*1*/)/*2*/ {}  "#;
    let mut s = Session::new_for_test("invalidRestArgError", content);
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
}
