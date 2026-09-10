use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_of_module() {
    let content = r#"namespace Foo {
  export class Bar {

  }


  export namespace Blah {

  }
}

var x: Foo./**/"#;
    let mut s = Session::new_for_test("memberListOfModule", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["Bar"]);
}
