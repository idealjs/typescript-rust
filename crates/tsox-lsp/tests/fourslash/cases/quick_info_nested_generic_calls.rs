use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_nested_generic_calls() {
    let content = r#"// @strict: true
/*1*/m({ foo: /*2*/$("foo") });
m({ foo: /*3*/$("foo") });
declare const m: <S extends string>(s: { [_ in S]: { $: NoInfer<S> } }) => void
declare const $: <S, T extends S>(s: T) => { $: S }
type NoInfer<T> = [T][T extends any ? 0 : never];"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "1",
        "const m: <\"foo\">(s: {\n    foo: {\n        $: \"foo\";\n    };\n}) => void",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "2",
        "const $: <unknown, string>(s: string) => {\n    $: unknown;\n}",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "3",
        "const $: <unknown, string>(s: string) => {\n    $: unknown;\n}",
        "",
    );
}
