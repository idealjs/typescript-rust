use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_non_module() {
    let content = r#"// @checkJs: true
// @Filename: /script.ts
console.log("I'm a script!");
// @Filename: /import.ts
import "./script/*1*/";
// @Filename: /require.js
require("./script/*2*/");
console.log("./script/*3*/");
// @Filename: /tripleSlash.ts
/// <reference path="script.ts" />
// @Filename: /stringLiteral.ts
console.log("./script");"#;
    let _s = Session::new_for_test("findAllRefsNonModule", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
