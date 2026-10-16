use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_non_existent_export_binding() {
    let content = r#"// @Filename: /tsconfig.json
 { "compilerOptions": { "module": "commonjs" } }
// @filename: /bar.ts
import { Foo/**/ } from "./foo";
// @filename: /foo.ts
export { Foo }"#;
    let _s = Session::new_for_test("findAllReferencesNonExistentExportBinding", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
