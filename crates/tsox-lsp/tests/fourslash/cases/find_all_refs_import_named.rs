use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_import_named() {
    let content = r#"// @module: commonjs
// @Filename: f.ts
export { foo as foo }
function /*start*/foo(a: number, b: number) { }
// @Filename: b.ts
import x = require("./f");
x.foo(1, 2);"#;
    let mut s = Session::new_for_test("findAllRefsImportNamed", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "start")
}
