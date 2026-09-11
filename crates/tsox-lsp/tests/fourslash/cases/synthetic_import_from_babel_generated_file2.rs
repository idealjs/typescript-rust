use tsox_lsp::fourslash::{self, Session};


#[test]
fn synthetic_import_from_babel_generated_file2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @allowSyntheticDefaultImports: true
// @Filename: /a.js
Object.defineProperty(exports, "__esModule", {
    value: true
});
exports.default = f;
/**
 * Run this function
 * @param {string} t
 */
function f(t) {}
// @Filename: /b.js
import f from "./a"
/**/f"#;
    let mut s = Session::new_for_test("syntheticImportFromBabelGeneratedFile2", content);
    fourslash::verify_quick_info_at(&mut s, "", "(alias) function f(t: string): void\nimport f", "Run this function");
}
