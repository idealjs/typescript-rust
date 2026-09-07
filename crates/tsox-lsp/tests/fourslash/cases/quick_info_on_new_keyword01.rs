use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_on_new_keyword01() {
    let content = r#"class Cat {
  /**
   * NOTE: this constructor is private! Please use the factory function
   */
  private constructor() { }

  static makeCat() { new Cat(); }
}

ne/*1*/w Ca/*2*/t();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "constructor Cat(): Cat", "NOTE: this constructor is private! Please use
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "constructor Cat(): Cat", "NOTE: this constructor is private! Please use
}
