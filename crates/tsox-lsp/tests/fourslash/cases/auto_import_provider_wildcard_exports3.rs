use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_provider_wildcard_exports3() {
    let content = r#"// @Filename: /home/src/workspaces/project/packages/ui/package.json
{
  "name": "@repo/ui",
  "version": "1.0.0",
  "exports": {
    "./*": "./src/*.tsx"
  }
}
// @Filename: /home/src/workspaces/project/packages/ui/src/Card.tsx
export const Card = () => null;
// @Filename: /home/src/workspaces/project/apps/web/package.json
{
  "name": "web",
  "version": "1.0.0",
  "dependencies": {
    "@repo/ui": "workspace:*"
  }
}
// @Filename: /home/src/workspaces/project/apps/web/tsconfig.json
{
  "compilerOptions": {
    "module": "esnext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "jsx": "preserve",
    "lib": ["es5"]
  },
 "include": ["app"]
}
// @Filename: /home/src/workspaces/project/apps/web/app/index.tsx
(<Card/**/ />);
// @link: /home/src/workspaces/project/packages/ui -> /home/src/workspaces/project/apps/web/node_modules/@repo/ui"#;
    let mut s = Session::new_for_test("autoImportProvider_wildcardExports3", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
