use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn rewrite_relative_import_extensions_project_references2() {
    let content = r#"// @Filename: src/tsconfig-base.json
{
    "compilerOptions": {
        "lib": ["es5"],
        "module": "nodenext",
        "composite": true,
        "rootDir": ".",
        "outDir": "../dist",
        "rewriteRelativeImportExtensions": true,
    }
}
// @Filename: src/compiler/tsconfig.json
{
    "extends": "../tsconfig-base.json",
    "compilerOptions": { "lib": ["es5"] }
}
// @Filename: src/compiler/parser.ts
export {};
// @Filename: src/services/tsconfig.json
{
    "extends": "../tsconfig-base.json",
    "compilerOptions": { "lib": ["es5"] },
    "references": [
        { "path": "../compiler" }
    ]
}
// @Filename: src/services/services.ts
import {} from "../compiler/parser.ts";"#;
    let mut s = Session::new_for_test("rewriteRelativeImportExtensionsProjectReferences2", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_file(&mut s, "/src/services/services.ts");
    fourslash::unsupported("VerifyBaselineNonSuggestionDiagnostics"); // f.VerifyBaselineNonSuggestionDiagnostics(t)
}
