use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_triple_slash_ordering() {
    let content = r#"// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "skipDefaultLibCheck": false
    }
}
// @Filename: /a.ts
export const x = 0;
// @Filename: /b.ts
// some comment

/// <reference lib="es2017.string" />

const y = x + 1;
// @Filename: /c.ts
// some comment

/// <reference path="jquery-1.8.3.js" />

const y = x + 1;
// @Filename: /d.ts
// some comment

/// <reference types="node" />

const y = x + 1;
// @Filename: /f.ts
// some comment

/// <amd-module name="NamedModule" />

const y = x + 1;
// @Filename: /g.ts
// some comment

/// <amd-dependency path="legacy/moduleA" name="moduleA" />

const y = x + 1;"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/c.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/d.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/f.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/g.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
