use tsox_lsp::fourslash::{self, Session};

#[test]
fn enum_update1() {
    let content = r#"namespace M {
	export enum E {
		A = 1,
		B = 2,
		C = 3,
		/*1*/
	}
}
namespace M {
	function foo(): M.E {
		return M.E.A;
	}
}"#;
    let mut s = Session::new(content);
    fourslash::verify_no_errors(&mut s);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "D = C << 1,");
    fourslash::verify_no_errors(&mut s);
}
