use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_extends() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: dummy.js
/**
 * @extends {Thing<string>}
 */
class MyStringThing extends Thing {
    constructor() {
        var x = this.mine;
        x/**/;
    }
}
// @Filename: declarations.d.ts
declare class Thing<T> {
    mine: T;
}"#;
    let mut s = Session::new_for_test("jsDocExtends", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "(local var) x: string", "")
}
