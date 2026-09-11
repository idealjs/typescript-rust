use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_from_contextual_union_type3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: true
declare const foo1: <D extends Foo1<D>>(definition: D) => D;

type Foo1<D, Bar = Prop<D, "bar">> = {
  bar: {
    [K in keyof Bar]: Bar[K] extends boolean
      ? Bar[K]
      : "Error: bar should be boolean";
  };
};

declare const foo2: <D extends Foo2<D>>(definition: D) => D;

type Foo2<D, Bar = Prop<D, "bar">> = {
  bar?: {
    [K in keyof Bar]: Bar[K] extends boolean
      ? Bar[K]
      : "Error: bar should be boolean";
  };
};

type Prop<T, K> = K extends keyof T ? T[K] : never;

foo1({ bar: { /*1*/X: "test" } });

foo2({ bar: { /*2*/X: "test" } });"#;
    let mut s = Session::new_for_test("quickInfoFromContextualUnionType3", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) X: \"Error: bar should be boolean\"", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) X: \"Error: bar should be boolean\"", "");
}
