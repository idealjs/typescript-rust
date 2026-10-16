use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_display_parts_external_modules() {
    let content = r#"export namespace /*1*/m {
    var /*2*/namespaceElemWithoutExport = 10;
    export var /*3*/namespaceElemWithExport = 10;
}
export var /*4*/a = /*5*/m;
export var /*6*/b: typeof /*7*/m;
export namespace /*8*/m1./*9*/m2 {
    var /*10*/namespaceElemWithoutExport = 10;
    export var /*11*/namespaceElemWithExport = 10;
}
export var /*12*/x = /*13*/m1./*14*/m2;
export var /*15*/y: typeof /*16*/m1./*17*/m2;"#;
    let _s = Session::new_for_test("quickInfoDisplayPartsExternalModules", content);
    // TODO: f.VerifyBaselineHover(t)
}
