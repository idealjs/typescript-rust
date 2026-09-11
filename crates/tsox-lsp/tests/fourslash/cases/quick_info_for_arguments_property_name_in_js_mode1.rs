use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_arguments_property_name_in_js_mode1() {
    let content = r#"// @allowJs: true
// @filename: a.js
const foo = {
    f1: (params) => { }
}

function /*1*/f2(x) {
   foo.f1({ x, arguments: [] });
}

/*2*/f2('');"#;
    let mut s = Session::new_for_test("quickInfoForArgumentsPropertyNameInJsMode1", content);
    // TODO: f.VerifyBaselineHover(t)
}
