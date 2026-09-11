use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_arguments_property_name_in_js_mode2() {
    let content = r#"// @allowJs: true
// @filename: a.js
function /*1*/f(x) {
   arguments;
}

/*2*/f('');"#;
    let mut s = Session::new_for_test("quickInfoForArgumentsPropertyNameInJsMode2", content);
    // TODO: f.VerifyBaselineHover(t)
}
