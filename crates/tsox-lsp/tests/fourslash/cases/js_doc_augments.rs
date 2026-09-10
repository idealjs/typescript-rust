use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_augments() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: dummy.js
/**
 * @augments {Thing<string>}
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
    let mut s = Session::new_for_test("jsDocAugments", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(local var) x: string", "")
}
