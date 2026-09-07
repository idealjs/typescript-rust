use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_and_member_list_on_commented_dot() {
    let content = r#"namespace M {
  export class C { public pub = 0; private priv = 1; }
  export var V = 0;
}


var c = new M.C();

c. // test on c.

//Test for comment
//c./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", nil)
}
