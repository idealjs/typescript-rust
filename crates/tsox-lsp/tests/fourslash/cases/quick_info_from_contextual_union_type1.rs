use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "1", "(property) someProp: string", "");
}
