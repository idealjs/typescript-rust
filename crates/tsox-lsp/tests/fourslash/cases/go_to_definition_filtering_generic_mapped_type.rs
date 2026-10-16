use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_filtering_generic_mapped_type() {
    let content = r#"const obj = {
  get /*def*/id() {
    return 1;
  },
  name: "test",
};

type Omit2<T, DroppedKeys extends PropertyKey> = {
  [K in keyof T as Exclude<K, DroppedKeys>]: T[K];
};

declare function omit2<O, Mask extends { [K in keyof O]?: true }>(
  obj: O,
  mask: Mask
): Omit2<O, keyof Mask>;

const obj2 = omit2(obj, {
  name: true,
});

obj2.[|/*ref*/id|];"#;
    let _s = Session::new_for_test("goToDefinition_filteringGenericMappedType", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "ref")
}
