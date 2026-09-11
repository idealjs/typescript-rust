use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_for_variable_in_extends_clause01() {
    let content = r#"/*1*/var /*2*/Base = class { };
class C extends /*3*/Base { }"#;
    let mut s = Session::new_for_test("findAllRefsForVariableInExtendsClause01", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
