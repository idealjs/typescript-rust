use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_variable_in_extends_clause02() {
    let content = r#"/*1*/interface /*2*/Base { }
namespace n {
    var Base = class { };
    interface I extends /*3*/Base { }
}"#;
    let mut s = Session::new_for_test("findAllRefsForVariableInExtendsClause02", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
