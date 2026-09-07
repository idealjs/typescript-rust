use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn rewrite_relative_import_extensions_project_references1() {
    let content = r#"// @Filename: packages/common/tsconfig.json
{
    "compilerOptions": {
        "lib": ["es5"],
        "composite": true,
        "rootDir": "src",
        "outDir": "dist",
        "module": "nodenext",
        "resolveJsonModule": false,
    }
}
// @Filename: packages/common/package.json
{
    "name": "common",
    "version": "1.0.0",
    "type": "module",
    "exports": {
        ".": {
            "source": "./src/index.ts",
            "default": "./dist/index.js"
        }
    }
}
// @Filename: packages/common/src/index.ts
export {};
// @Filename: packages/main/tsconfig.json
{
    "compilerOptions": {
        "module": "nodenext",
        "rewriteRelativeImportExtensions": true,
        "lib": ["es5"],
        "rootDir": "src",
        "outDir": "dist",
        "resolveJsonModule": false,
    },
    "references": [
        { "path": "../common" }
    ]
}
// @Filename: packages/main/package.json
{ "type": "module" }
// @Filename: packages/main/src/index.ts
import {} from "../../common/src/index.ts";"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_file(&mut s, "/packages/main/src/index.ts");
    fourslash::unsupported("VerifyBaselineNonSuggestionDiagnostics"); // f.VerifyBaselineNonSuggestionDiagnostics(t)
}
