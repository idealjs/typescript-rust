use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn auto_import_package_json_imports_pattern_ts_ts() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#*.ts": "./src/*.ts"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /a.ts
something/**/"##;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"#something.ts"}, nil /*preferences*/)
}
