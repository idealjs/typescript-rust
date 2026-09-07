use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyRangeAfterCodeFix"]
#[test]
fn code_fix_class_implement_class_method_via_heritage() {
    let content = r#"class C1 {
    f1() {}
}

class C2 extends C1 {

}

class C3 implements C2 {[| 
    |]f2(){}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `f1(): void{
}
