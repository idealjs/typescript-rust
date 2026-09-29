use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("noCompletionListOnCommentsInsideObjectLiterals", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), nil)
}
