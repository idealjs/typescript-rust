use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_display_parts_type_parameter_in_type_alias() {
    let content = r#"type /*0*/List</*1*/T> = /*2*/T[]
type /*3*/List2</*4*/T extends string> = /*5*/T[];"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsTypeParameterInTypeAlias", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
