use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
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
    let mut s = Session::new_for_test("renameJsSpecialAssignmentRhs2", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/)
}
