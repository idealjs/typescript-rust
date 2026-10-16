use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_display_parts_parameters() {
    let content = r#"/** @return *crunch* */
function /*1*/foo(/*2*/param: string, /*3*/optionalParam?: string, /*4*/paramWithInitializer = "hello", .../*5*/restParam: string[]) {
    /*6*/param = "Hello";
    /*7*/optionalParam = "World";
    /*8*/paramWithInitializer = "Hello";
    /*9*/restParam[0] = "World";
}"#;
    let _s = Session::new_for_test("quickInfoDisplayPartsParameters", content);
    // TODO: f.VerifyBaselineHover(t)
}
