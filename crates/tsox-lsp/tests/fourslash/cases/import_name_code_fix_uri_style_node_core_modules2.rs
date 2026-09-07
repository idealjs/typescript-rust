use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_uri_style_node_core_modules2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @module: commonjs
// @Filename: /node_modules/@types/node/index.d.ts
declare module "fs" { function writeFile(): void }
declare module "fs/promises" { function writeFile(): Promise<void> }
declare module "node:fs" { export * from "fs"; }
declare module "node:fs/promises" { export * from "fs/promises"; }
// @Filename: /other.ts
import "node:fs/promises";
// @Filename: /index.ts
writeFile/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"node:fs", "node:fs/promises"}, nil /*preferences*
    fourslash::go_to_file(&mut s, "/other.ts");
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "\n")
    fourslash::go_to_file(&mut s, "/index.ts");
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"fs", "fs/promises", "node:fs", "node:fs/promises"
    fourslash::go_to_file(&mut s, "/other.ts");
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import \"node:fs/promises\";\n")
    fourslash::go_to_file(&mut s, "/index.ts");
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"node:fs", "node:fs/promises"}, nil /*preferences*
}
