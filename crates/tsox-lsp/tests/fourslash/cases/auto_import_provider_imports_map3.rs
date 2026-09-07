use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider_imports_map3() {
    let content = r##"// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "lib": ["es5"],
    "rootDir": "src",
    "outDir": "dist"
  }
}
// @Filename: /home/src/workspaces/project/package.json
{
  "type": "module",
  "imports": {
    "#internal/": "./dist/internal/"
  }
}
// @Filename: /home/src/workspaces/project/src/internal/foo.ts
export function something(name: string) {}
// @Filename: /home/src/workspaces/project/src/a.ts
something/**/"##;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"#internal/foo.js"}, nil /*preferences*/)
}
