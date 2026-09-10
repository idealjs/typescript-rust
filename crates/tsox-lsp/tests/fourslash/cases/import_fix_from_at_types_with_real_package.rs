use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // Another file already imports from `myLib` (resolving to @"]
#[test]
fn import_fix_from_at_types_with_real_package() {
    // TODO: // Simulate a project where both `myLib` (JS-only package) and `@types/myLib` (type declarations) ar
    // TODO: // Another file already imports from `myLib` (resolving to @types/myLib).
    // TODO: // The import fix should suggest importing from "myLib", not "@types/myLib".
    let content = r#"// @Filename: /node_modules/myLib/package.json
{"name":"myLib","version":"1.0.0","main":"index.js"}
// @Filename: /node_modules/myLib/index.js
module.exports = {};
// @Filename: /node_modules/@types/myLib/package.json
{"name":"@types/myLib","version":"1.0.0","types":"index.d.ts"}
// @Filename: /node_modules/@types/myLib/index.d.ts
export function f1(): void;
export function f2(): void;
// @Filename: /package.json
{"dependencies":{"myLib":"*"}}
// @Filename: /other.ts
import { f1 } from "myLib";
f1();
// @Filename: /index.ts
[|f2/*0*/();|]"#;
    let mut s = Session::new_for_test("importFixFromAtTypesWithRealPackage", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "0", []string{"myLib"}, nil /*preferences*/)
}

#[ignore = "generator: // Like the above test, but the real package has an exports "]
#[test]
fn import_fix_from_at_types_with_real_package_exports() {
    // TODO: // Like the above test, but the real package has an exports field pointing to JS files.
    // TODO: // This is the React 19 scenario: react has exports but no .d.ts, @types/react provides types.
    let content = r#"// @Filename: /node_modules/myLib/package.json
{"name":"myLib","version":"1.0.0","exports":{".":{"default":"./index.js"}}}
// @Filename: /node_modules/myLib/index.js
module.exports = {};
// @Filename: /node_modules/@types/myLib/package.json
{"name":"@types/myLib","version":"1.0.0","types":"index.d.ts"}
// @Filename: /node_modules/@types/myLib/index.d.ts
export function f1(): void;
export function f2(): void;
// @Filename: /package.json
{"dependencies":{"myLib":"*"}}
// @Filename: /other.ts
import { f1 } from "myLib";
f1();
// @Filename: /index.ts
[|f2/*0*/();|]"#;
    let mut s = Session::new_for_test("importFixFromAtTypesWithRealPackageExports", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "0", []string{"myLib"}, nil /*preferences*/)
}
