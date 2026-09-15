use tsox_lsp::fourslash::Session;


#[test]
fn rename_js_special_assignment_rhs2() {
    let content = r#"// @allowJs: true
// @Filename: a.js
const foo = {
    set: function (x) {
        this._x = x;
    },
    copy: function ([|x|]) {
        this._x = [|x|].prop;
    }
};"#;
    let _s = Session::new_for_test("renameJsSpecialAssignmentRhs2", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/)
}
