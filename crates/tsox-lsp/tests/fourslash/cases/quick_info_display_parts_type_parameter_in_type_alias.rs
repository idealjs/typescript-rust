use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_display_parts_type_parameter_in_type_alias() {
    let content = r#"type /*0*/List</*1*/T> = /*2*/T[]
type /*3*/List2</*4*/T extends string> = /*5*/T[];"#;
    let _s = Session::new_for_test("quickInfoDisplayPartsTypeParameterInTypeAlias", content);
    // TODO: f.VerifyBaselineHover(t)
}
