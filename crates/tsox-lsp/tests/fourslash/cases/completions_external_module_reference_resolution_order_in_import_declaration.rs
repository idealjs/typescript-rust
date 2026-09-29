use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_external_module_reference_resolution_order_in_import_declaration() {
    let content = r#"// @Filename: externalModuleRefernceResolutionOrderInImportDeclaration_file1.ts
export function foo() { };
// @Filename: externalModuleRefernceResolutionOrderInImportDeclaration_file2.ts
declare module "externalModuleRefernceResolutionOrderInImportDeclaration_file1" {
    export function bar();
}
// @Filename: externalModuleRefernceResolutionOrderInImportDeclaration_file3.ts
///<reference path='externalModuleRefernceResolutionOrderInImportDeclaration_file2.ts'/>
import file1 = require('externalModuleRefernceResolutionOrderInImportDeclaration_file1');
/*1*/"#;
    let mut s = Session::new_for_test("completionsExternalModuleReferenceResolutionOrderInImportDeclaration", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "file1.");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
