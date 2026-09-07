use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Configure"]
#[test]
fn auto_import_symlink_case_sensitive() {
    let content = r#"// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "commonjs" } }
// @Filename: /node_modules/.pnpm/mobx@6.0.4/node_modules/MobX/Foo.d.ts
export declare function autorun(): void;
// @Filename: /index.ts
autorun/**/
// @Filename: /utils.ts
import "MobX/Foo";
// @link: /node_modules/.pnpm/mobx@6.0.4/node_modules/MobX -> /node_modules/MobX
// @link: /node_modules/.pnpm/mobx@6.0.4/node_modules/MobX -> /node_modules/.pnpm/cool-mobx-dependent@1.2.3/node_modules/MobX"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{AutoImportEntrypointDirectorySearch: core.TSTrue})
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
