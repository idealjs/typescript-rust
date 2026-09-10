use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: opts333 := f.GetOptions()"]
#[test]
fn formatting_space_before_function_paren() {
    let content = r#"/*1*/function foo() { }
/*2*/function boo  () { }
/*3*/var bar = function foo() { };
/*4*/var foo = { bar() { } };
/*5*/function tmpl <T> () { }
/*6*/var f = function*() { };
/*7*/function* g () { }"#;
    let mut s = Session::new_for_test("formattingSpaceBeforeFunctionParen", content);
    // TODO: opts333 := f.GetOptions()
    // TODO: opts333.FormatCodeSettings.InsertSpaceBeforeFunctionParenthesis = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts333)
    // TODO: opts414 := f.GetOptions()
    // TODO: opts414.FormatCodeSettings.InsertSpaceAfterFunctionKeywordForAnonymousFunctions = core.TSFalse
    fourslash::unsupported("Configure"); // f.Configure(t, opts414)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"function foo () { }"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"function boo () { }"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"var bar = function foo () { };"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"var foo = { bar () { } };"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"function tmpl<T> () { }"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"var f = function*() { };"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"function* g () { }"#);
}
