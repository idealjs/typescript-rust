use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_function_parameter() {
    let content = r#"var x;
var n;

function n(x: number, /*1*/n: number) {
    /*2*/n = 32;
    x = /*3*/n;
}"#;
    let mut s = Session::new_for_test("referencesForFunctionParameter", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
