use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_triple_slash() {
    let content = r#"// @checkJs: true
// @Filename: /node_modules/@types/globals/index.d.ts
declare const someAmbientGlobal: unknown;
// @Filename: /a.ts
/// <reference path="b.ts/*1*/" />
/// <reference types="globals/*2*/" />
// @Filename: /b.ts
console.log("b.ts");
// @Filename: /c.js
require("./b");
require("globals");"#;
    let _s = Session::new_for_test("findAllReferencesTripleSlash", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
