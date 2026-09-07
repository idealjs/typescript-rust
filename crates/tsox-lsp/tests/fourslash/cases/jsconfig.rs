use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyErrorExistsAfterMarker"]
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
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/a.js");
    fourslash::unsupported("VerifyErrorExistsAfterMarker"); // f.VerifyErrorExistsAfterMarker(t, "")
}
