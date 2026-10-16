use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_augments_and_extends() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: dummy.js
/**
 * @augments {Thing<number>}
 * [|@extends {Thing<string>}|]
 */
class MyStringThing extends Thing {
    constructor() {
        super();
        var x = this.mine;
        x/**/;
    }
}
// @Filename: declarations.d.ts
declare class Thing<T> {
    mine: T;
}"#;
    let mut s = Session::new_for_test("jsDocAugmentsAndExtends", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "(local var) x: number", "")
    // TODO: f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
