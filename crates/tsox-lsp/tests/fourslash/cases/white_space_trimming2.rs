use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCurrentFileContent"]
#[test]
fn white_space_trimming2() {
    let content = r#"let noSubTemplate = ` + "`" + `/*    /*1*/` + "`" + `;
let templateHead = ` + "`" + `/*    /*2*/${1 + 2}` + "`" + `;
let templateMiddle = ` + "`" + `/*    ${1 + 2    /*3*/}` + "`" + `;
let templateTail = ` + "`" + `/*    ${1 + 2}    /*4*/` + "`" + `;"#;
    let mut s = Session::new_for_test("whiteSpaceTrimming2", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "\n");
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert(&mut s, "\n");
    fourslash::go_to_marker(&mut s, "3");
    fourslash::insert(&mut s, "\n");
    fourslash::go_to_marker(&mut s, "4");
    fourslash::insert(&mut s, "\n");
    fourslash::unsupported("VerifyCurrentFileContent"); // f.VerifyCurrentFileContent(t, "let noSubTemplate = `/*    \n`;\nlet templateHead = `/*    \n${1 + 2}
}
