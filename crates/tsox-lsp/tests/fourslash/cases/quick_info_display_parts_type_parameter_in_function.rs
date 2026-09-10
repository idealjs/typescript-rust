use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_display_parts_type_parameter_in_function() {
    let content = r#"function /*1*/foo</*2*/U>(/*3*/a: /*4*/U) {
    return /*5*/a;
}
/*6*/foo("Hello");
function /*7*/foo2</*8*/U extends string>(/*9*/a: /*10*/U) {
    return /*11*/a;
}
/*12*/foo2("hello");"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsTypeParameterInFunction", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
