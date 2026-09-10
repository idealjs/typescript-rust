use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
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
    let mut s = Session::new_for_test("findAllReferencesTripleSlash", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
