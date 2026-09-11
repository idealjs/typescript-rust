use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_js_special_assignment_rhs1() {
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
    let mut s = Session::new_for_test("renameJsSpecialAssignmentRhs1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/)
}
