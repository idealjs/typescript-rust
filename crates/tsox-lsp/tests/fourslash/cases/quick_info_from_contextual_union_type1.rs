use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_from_contextual_union_type1() {
    let content = r#"// @strict: true
// based on https://github.com/microsoft/TypeScript/issues/55495
type X =
  | {
      name: string;
      [key: string]: any;
    }
  | {
      name: "john";
      someProp: boolean;
    };

const obj = { name: "john", /*1*/someProp: "foo" } satisfies X;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) someProp: string", "")
}
