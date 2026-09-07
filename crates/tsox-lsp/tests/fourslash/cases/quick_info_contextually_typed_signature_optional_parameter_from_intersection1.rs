use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_contextually_typed_signature_optional_parameter_from_intersection1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: true
const optionals: ((a?: number) => unknown) & ((b?: string) => unknown) = (
  arg,
) =/**/> {};"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "",
        "function(arg: string | number | undefined): void",
        "",
    );
}
