use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics24() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: a.js
function Person(age) {
    if (age >= 18) {
        this.canVote = true;
    } else {
        this.canVote = 23;
    }
}
let x = new Person(100);
x.canVote/**/;"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics24", content);
    fourslash::verify_quick_info_at(&mut s, "", "(property) Person.canVote: number | boolean", "");
}
