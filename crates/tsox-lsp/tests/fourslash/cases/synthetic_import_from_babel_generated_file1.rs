use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn synthetic_import_from_babel_generated_file1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @allowSyntheticDefaultImports: true
// @Filename: /a.js
exports.__esModule = true;
exports.default = f;
/**
 * Run this function
 * @param {string} t
 */
function f(t) {}
// @Filename: /b.js
import f from "./a"
/**/f"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(alias) function f(t: string): void\nimport f", "Run this function")
}
