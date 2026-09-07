use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn import_name_code_fix_barrel_export3() {
    let content = r#"// @module: commonjs
// @Filename: /foo/a.ts
export const A = 0;
// @Filename: /foo/b.ts
export {};
A/*sibling*/
// @Filename: /foo/index.ts
export * from "./a";
export * from "./b";
// @Filename: /index.ts
export * from "./foo";
export * from "./src";
// @Filename: /src/a.ts
export {};
A/*parent*/
// @Filename: /src/index.ts
export * from "./a";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "sibling", []string{"./a", "./index", "../index"}, &lsutil.User
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "parent", []string{"../foo/a", "../foo/index", "../index"}, &ls
}
