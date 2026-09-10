use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn signature_help_comments_function_expression() {
    let content = r#"/** lambdaFoo var comment*/
var lambdaFoo = /** this is lambda comment*/ (/**param a*/a: number, /**param b*/b: number) => a + b;
var lambddaNoVarComment = /** this is lambda multiplication*/ (/**param a*/a: number, /**param b*/b: number) => a * b;
lambdaFoo(/*5*/10, /*6*/20);
function anotherFunc(a: number) {
    /** documentation
        @param b {string} inner parameter */
    var lambdaVar = /** inner docs */(b: string) => {
        var localVar = "Hello ";
        return localVar + b;
    }
    return lambdaVar("World") + a;
}
/**
 * On variable
 * @param s the first parameter!
 * @returns the parameter's length
 */
var assigned = /**
                * Summary on expression
                * @param s param on expression
                * @returns return on expression
                */function(/** On parameter */s: string) {
  return s.length;
}
assigned(/*18*/"hey");"#;
    let mut s = Session::new_for_test("signatureHelpCommentsFunctionExpression", content);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
