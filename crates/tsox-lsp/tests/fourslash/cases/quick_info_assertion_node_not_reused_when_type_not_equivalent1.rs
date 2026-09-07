use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_assertion_node_not_reused_when_type_not_equivalent1() {
    let content = r#"// @strict: true
type Wrapper<T> = {
  _type: T;
};

function stringWrapper(): Wrapper<string> {
  return { _type: "" };
}

function objWrapper<T extends Record<string, Wrapper<any>>>(
  obj: T,
): Wrapper<T> {
  return { _type: obj };
}

const value = objWrapper({
  prop1: stringWrapper() as Wrapper<"hello">,
});

type Unwrap<T extends Wrapper<any>> = T["_type"] extends Record<
  string,
  Wrapper<any>
>
  ? { [Key in keyof T["_type"]]: Unwrap<T["_type"][Key]> }
  : T["_type"];

type Test/*1*/ = Unwrap<typeof value>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "type Test = {\n    prop1: \"hello\";\n}", "")
}
