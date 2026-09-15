use tsox_lsp::fourslash::Session;


#[test]
fn auto_import_module_augmentation() {
    let content = r#"// @Filename: /a.ts
export interface Foo {
    x: number;
}

// @Filename: /b.ts
export {};
declare module "./a" {
    export const Foo: any;
}

// @Filename: /c.ts
Foo/**/
"#;
    let _s = Session::new_for_test("autoImportModuleAugmentation", content);
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}
