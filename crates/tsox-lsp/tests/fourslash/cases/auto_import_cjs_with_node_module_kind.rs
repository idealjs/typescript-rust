use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn auto_import_cjs_with_node_module_kind() {
    let content = r#"// @Filename: /tsconfig.json
{
  "compilerOptions": {
    "allowJs": true,
    "module": "node20",
    "checkJs": true,
    "noEmit": true
  }
}
// @Filename: /package.json
{ "type": "commonjs" }
// @Filename: /lib.js
module.exports = { LIB_VERSION: 1 };
// @Filename: /main.js
module.exports.foo = 0;
LIB_VERSION/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn auto_import_cjs_with_node_module_kind_empty_file() {
    let content = r#"// @Filename: /tsconfig.json
{
  "compilerOptions": {
    "allowJs": true,
    "module": "node20",
    "checkJs": true,
    "noEmit": true
  }
}
// @Filename: /package.json
{ "type": "commonjs" }
// @Filename: /lib.js
module.exports = { LIB_VERSION: 1 };
// @Filename: /main.js
LIB_VERSION/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn auto_import_cjs_with_module_detection_force() {
    let content = r#"// @Filename: /tsconfig.json
{
  "compilerOptions": {
    "allowJs": true,
    "module": "preserve",
    "moduleDetection": "force",
    "checkJs": true,
    "noEmit": true
  }
}
// @Filename: /lib.js
export const LIB_VERSION = 1;
// @Filename: /main.js
const path = require("path");
LIB_VERSION/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
