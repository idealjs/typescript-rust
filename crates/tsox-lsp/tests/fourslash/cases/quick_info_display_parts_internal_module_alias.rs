use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_display_parts_internal_module_alias() {
    let content = r#"namespace m.m1 {
    export class c {
    }
}
namespace m2 {
    import /*1*/a1 = m;
    new /*2*/a1.m1.c();
    import /*3*/a2 = m.m1;
    new /*4*/a2.c();
    export import /*5*/a3 = m;
    new /*6*/a3.m1.c();
    export import /*7*/a4 = m.m1;
    new /*8*/a4.c();
}"#;
    let _s = Session::new_for_test("quickInfoDisplayPartsInternalModuleAlias", content);
    // TODO: f.VerifyBaselineHover(t)
}
