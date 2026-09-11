use tsox_lsp::fourslash::{self, Session};


#[test]
fn rewrite_relative_import_extensions_project_references3() {
    let content = r#"// @Filename: src/tsconfig-base.json
{
    "compilerOptions": {
        "lib": ["es5"],
        "module": "nodenext",
        "composite": true,
        "rewriteRelativeImportExtensions": true,
    }
}
// @Filename: src/compiler/tsconfig.json
{
    "extends": "../tsconfig-base.json",
    "compilerOptions": {
        "lib": ["es5"],
        "rootDir": ".",
        "outDir": "../../dist/compiler",
}
// @Filename: src/compiler/parser.ts
export {};
// @Filename: src/services/tsconfig.json
{
    "extends": "../tsconfig-base.json",
    "compilerOptions": {
        "lib": ["es5"],
        "rootDir": ".",
        "outDir": "../../dist/services",
    },
    "references": [
        { "path": "../compiler" }
    ]
}
// @Filename: src/services/services.ts
import {} from "../compiler/parser.ts";"#;
    let mut s = Session::new_for_test("rewriteRelativeImportExtensionsProjectReferences3", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_file(&mut s, "/src/services/services.ts");
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
    // TODO: }
}
