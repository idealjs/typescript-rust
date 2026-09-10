use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn no_completion_list_on_comments_inside_object_literals() {
    let content = r#"namespace ObjectLiterals {
	interface MyPoint {
		x1: number;
		y1: number;
	}

	var p1: MyPoint = {
		/* /*1*/ Comment /*2*/ */
	};
}"#;
    let mut s = Session::new_for_test("noCompletionListOnCommentsInsideObjectLiterals", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), nil)
}
