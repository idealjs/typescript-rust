use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_after_numeric_literal() {
    let content = r#"// @Filename: f1.ts
0./*dotOnNumberExpressions1*/
// @Filename: f2.ts
0.0./*dotOnNumberExpressions2*/
// @Filename: f3.ts
0.0.0./*dotOnNumberExpressions3*/
// @Filename: f4.ts
0./** comment *//*dotOnNumberExpressions4*/
// @Filename: f5.ts
(0)./*validDotOnNumberExpressions1*/
// @Filename: f6.ts
(0.)./*validDotOnNumberExpressions2*/
// @Filename: f7.ts
(0.0)./*validDotOnNumberExpressions3*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"dotOnNumberExpressions1", "dotOnNumberExpressions4"}, nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"dotOnNumberExpressions2", "dotOnNumberExpressions3", "validDotOnNum
}
