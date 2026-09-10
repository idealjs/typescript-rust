use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn refactor_convert_to_es_module_module_nodenext() {
    let content = r#"// @allowJs: true
// @target: esnext
// @module: node18
// @Filename: /a.js
module.exports = 0;
// @Filename: /b.ts
module.exports = 0;
// @Filename: /c.cjs
module.exports = 0;
// @Filename: /d.cts
module.exports = 0;"#;
    let mut s = Session::new_for_test("refactorConvertToEsModule_module_nodenext", content);
    fourslash::go_to_file(&mut s, "/a.js");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
    fourslash::go_to_file(&mut s, "/c.cjs");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
    fourslash::go_to_file(&mut s, "/d.cts");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
