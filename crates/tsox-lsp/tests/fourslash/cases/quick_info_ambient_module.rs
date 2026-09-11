use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_ambient_module() {
    let content = r#"declare module "*.css"/*1*/;"#;
    let mut s = Session::new_for_test("quickInfoAmbientModule", content);
    fourslash::verify_quick_info_at(&mut s, "1", r#"module "*.css""#, "");
}

#[test]
fn quick_info_pattern_ambient_module_with_import_attributes() {
    let content = r#"declare module "*.css"/*1*/ with { type: "css" } {
    const styles: { readonly [className: string]: string };
    export default styles;
}"#;
    let mut s = Session::new_for_test("quickInfoPatternAmbientModuleWithImportAttributes", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}

#[test]
fn quick_info_merged_pattern_ambient_module_with_import_attributes() {
    let content = r#"// @Filename: /first.d.ts
declare module "*.asset"/*css*/ with { type: "css" } {
    export const cssOnly: "css";
}
declare module "*.asset"/*text*/ with { type: "text" } {
    export const textOnly: "text";
}

// @Filename: /second.d.ts
declare module "*.asset" with { type: "css" } {
    export const cssAlso: "css-also";
}
declare module "*.asset" with { type: "text" } {
    export const textAlso: "text-also";
}"#;
    let mut s = Session::new_for_test("quickInfoMergedPatternAmbientModuleWithImportAttributes", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"css": {0, 1}, "text": {0, 1}})
}
