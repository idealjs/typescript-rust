use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_nested_scopes() {
    let content = r#"/*1*/        namespace      My.App      {
/*2*/export      var appModule =      angular.module("app", [
/*3*/            ]).config([() =>            {
/*4*/                        configureStates
/*5*/($stateProvider);
/*6*/}]).run(My.App.setup);
/*7*/      }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"namespace My.App {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(
        &mut s,
        r#"    export var appModule = angular.module("app", ["#,
    );
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    ]).config([() => {"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"        configureStates"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"            ($stateProvider);"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"    }]).run(My.App.setup);"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
