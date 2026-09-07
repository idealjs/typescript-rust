use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_cross_package_paths_and_symlink() {
    let content = r#"// @Filename: /home/src/workspaces/project/packages/common/package.json
{
  "name": "@company/common",
  "version": "1.0.0",
  "main": "./lib/index.tsx"
}
// @Filename: /home/src/workspaces/project/packages/common/lib/index.tsx
export function Tooltip {};
// @Filename: /home/src/workspaces/project/packages/app/package.json
{
  "name": "@company/app",
  "version": "1.0.0",
  "dependencies": {
    "@company/common": "1.0.0"
  }
}
// @Filename: /home/src/workspaces/project/packages/app/tsconfig.json
{
  "compilerOptions": {
    "composite": true,
    "lib": ["es5"],
    "module": "esnext",
    "moduleResolution": "bundler",
    "paths": {
      "@/*": ["./*"]
    }
  }
}
// @Filename: /home/src/workspaces/project/packages/app/lib/index.ts
Tooltip/**/
// @link: /home/src/workspaces/project/packages/common -> /home/src/workspaces/project/node_modules/@company/common"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"@company/common"}, nil /*preferences*/)
}
