use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
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
    let mut s = Session::new_for_test("autoImportModuleAugmentation", content);
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}
