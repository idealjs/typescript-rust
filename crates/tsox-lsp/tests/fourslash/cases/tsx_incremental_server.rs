use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn tsx_incremental_server() {
    let content = r#"// @lib: es5
/**/"#;
    let mut s = Session::new_for_test("tsxIncrementalServer", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "<");
    fourslash::insert(&mut s, "div");
    fourslash::insert(&mut s, " ");
    fourslash::insert(&mut s, " id");
    fourslash::insert(&mut s, "=");
    fourslash::insert(&mut s, "\"foo");
    fourslash::insert(&mut s, "\"");
    fourslash::insert(&mut s, ">");
}
