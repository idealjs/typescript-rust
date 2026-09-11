use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_js_property_assignment() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/** @type {{ p: "x" | "y" }} */
const x = { p: "x"  };
x.p = "[|/**/|]";"#;
    let mut s = Session::new_for_test("completionsJsPropertyAssignment", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
