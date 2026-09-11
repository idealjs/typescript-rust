use tsox_lsp::fourslash::{self, Session};


#[test]
fn insert_var_after_empty_type_param_list() {
    let content = r#"class Dictionary<> { }
var x;
/**/"#;
    let mut s = Session::new_for_test("insertVarAfterEmptyTypeParamList", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "var y;\n");
}
