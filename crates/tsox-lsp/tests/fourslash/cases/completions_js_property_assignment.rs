use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_js_property_assignment() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/** @type {{ p: "x" | "y" }} */
const x = { p: "x"  };
x.p = "[|/**/|]";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
