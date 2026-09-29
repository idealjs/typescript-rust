use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_member_ordering() {
    let content = r#"// @lib: es2017
/** asdf */
interface I {
    1;
    2;
    3;
    4;
    5;
    6;
    7;
    8;
    9;
    10;
    11;
    12;
    13;
    14;
    15;
    16;
    17;
    18;
    19;
    20;
    21;
    22;
    /** a nice safe prime */
    23;
}
class C implements I {}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceMemberOrdering", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
