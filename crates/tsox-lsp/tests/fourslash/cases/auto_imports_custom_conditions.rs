use tsox_lsp::fourslash::Session;


#[test]
fn auto_imports_custom_conditions() {
    let content = r#"// @module: esnext
// @moduleResolution: bundler
// @customConditions: custom
// @Filename: /node_modules/dep/package.json
{
  "name": "dep",
  "version": "1.0.0",
  "exports": {
    ".": {
      "custom": "./dist/index.js"
    }
  }
}
// @Filename: /node_modules/dep/dist/index.d.ts
export const dep: number;
// @Filename: /index.ts
dep/**/"#;
    let _s = Session::new_for_test("autoImportsCustomConditions", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"dep"}, nil /*preferences*/)
}
