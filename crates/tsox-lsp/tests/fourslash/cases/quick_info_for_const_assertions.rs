use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_for_const_assertions() {
    let content = r#"const a = { a: 1 } as /*1*/const;
const b = 1 as /*2*/const;
const c = "c" as /*3*/const;
const d = [1, 2] as /*4*/const;"#;
    let mut s = Session::new_for_test("quickInfoForConstAssertions", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
