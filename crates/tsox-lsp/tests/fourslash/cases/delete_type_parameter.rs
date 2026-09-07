use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn delete_type_parameter() {
    let content = r#"interface Query<T> {
    groupBy(): Query</**/T>;
}
interface Query2<T> {
    groupBy(): Query2<Query<T>>;
}
var q1: Query<number>;
var q2: Query2<number>;
q1 = q2;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 1)
}
