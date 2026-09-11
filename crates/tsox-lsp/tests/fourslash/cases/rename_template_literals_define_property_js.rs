use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_template_literals_define_property_js() {
    let content = r#"// @allowJs: true
// @Filename: a.js
let obj = {};

Object.defineProperty(obj, `[|prop|]`, { value: 0 });

obj = {
    [|[`[|{| "contextRangeIndex": 1 |}prop|]`]: 1|]
};

obj.[|prop|];
obj['[|prop|]'];
obj["[|prop|]"];
obj[`[|prop|]`];"#;
    let mut s = Session::new_for_test("renameTemplateLiteralsDefinePropertyJs", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "prop")
}
