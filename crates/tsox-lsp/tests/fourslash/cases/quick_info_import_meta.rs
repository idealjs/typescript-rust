use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_import_meta() {
    let content = r#"// @module: esnext
// @Filename: foo.ts
/// <reference path='./bar.d.ts' />
im/*1*/port.me/*2*/ta;
//@Filename: bar.d.ts
/**
 * The type of `import.meta`.
 *
 * If you need to declare that a given property exists on `import.meta`,
 * this type may be augmented via interface merging.
 */
 interface ImportMeta {
}"#;
    let _s = Session::new_for_test("quickInfoImportMeta", content);
    // TODO: f.VerifyBaselineHover(t)
}
