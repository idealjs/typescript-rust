use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_for_variable_in_implements_clause01() {
    let content = r#"var Base = class { };
class C extends Base implements /**/Base { }"#;
    let _s = Session::new_for_test("findAllRefsForVariableInImplementsClause01", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
