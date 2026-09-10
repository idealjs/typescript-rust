use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_use_big_int_literal_with_numeric_separators() {
    let content = r#"6_402_373_705_728_000;  // 18! < 2 ** 53
0x16_BE_EC_CA_73_00_00; // 18! < 2 ** 53"#;
    let mut s = Session::new_for_test("codeFixUseBigIntLiteralWithNumericSeparators", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "useBigintLiteral")
}
