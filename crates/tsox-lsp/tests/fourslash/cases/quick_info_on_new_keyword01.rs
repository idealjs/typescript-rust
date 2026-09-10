use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("quickInfoOnNewKeyword01", content);
    fourslash::verify_quick_info_at(&mut s, "1", "constructor Cat(): Cat", "NOTE: this constructor is private! Please use the factory function");
    fourslash::verify_quick_info_at(&mut s, "2", "constructor Cat(): Cat", "NOTE: this constructor is private! Please use the factory function");
}
