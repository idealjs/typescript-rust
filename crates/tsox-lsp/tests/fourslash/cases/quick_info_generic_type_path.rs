use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_generic_type_path() {
    let content = r#"
function f<T>(x: T) {
  class C {
    value = x
  }
  return new C()
}

class Box<T> {
  public value: T;
  constructor(value: T) {
    this.value = value;
  }
}

const instance = f/*callF*/("hello");
const b1/*b1*/ = new Box/*newBox*/(instance);
declare const b2/*b2*/: Box<typeof instance>;
"#;
    let mut s = Session::new_for_test("quickInfoGenericTypePath", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
