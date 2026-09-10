use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_multiline_comments_with_tabs1() {
    let content = r#"var f = function (j) {

	switch (j) {
		case 1:
/*1*/				/* when current checkbox has focus, Firefox has changed check state already
/*2*/				on SPACE bar press only
/*3*/				IE does not have issue, use the CSS class
/*4*/				input:focus[type=checkbox] (z-index = 31290)
/*5*/				to determine whether checkbox has focus or not
				*/
			break;
		case 2:
		break;
	}
}"#;
    let mut s = Session::new_for_test("formattingMultilineCommentsWithTabs1", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"            /* when current checkbox has focus, Firefox has changed check state already"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"            on SPACE bar press only"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"            IE does not have issue, use the CSS class"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"            input:focus[type=checkbox] (z-index = 31290)"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"            to determine whether checkbox has focus or not"#);
}
