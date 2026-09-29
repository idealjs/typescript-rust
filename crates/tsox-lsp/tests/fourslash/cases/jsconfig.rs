use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsconfig() {
    let content = r#"// @Filename: /a.js
function f(/**/x) {
}
// @Filename: /jsconfig.json
{
    "compilerOptions": {
        "checkJs": true,
        "noImplicitAny": true
    }
}"#;
    let mut s = Session::new_for_test("jsconfig", content);
    fourslash::go_to_file(&mut s, "/a.js");
    // TODO: f.VerifyErrorExistsAfterMarker(t, "")
}
