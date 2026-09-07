use tsox_lsp::fourslash::{self, Session};

#[test]
fn syntax_error_after_import1() {
    let content = r#"declare module "extmod" {
  namespace IntMod {
    class Customer {
      constructor(name: string);
    }
  }
}
import ext = require('extmod');
import int = ext.IntMod;
var x = new int/*0*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "0");
    fourslash::insert(&mut s, ".");
}
