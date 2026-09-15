use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_for_variable_in_extends_clause02() {
    let content = r#"/*1*/interface /*2*/Base { }
namespace n {
    var Base = class { };
    interface I extends /*3*/Base { }
}"#;
    let _s = Session::new_for_test("findAllRefsForVariableInExtendsClause02", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
