use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_mapped_type2() {
    let content = r#"type ListenerTemplate<T, S extends string, I extends string = "${1}"> = {
    [K in keyof T as K extends string
        ? S extends ` + "`" + `${infer F}${I}${infer R}` + "`" + ` ? ` + "`" + `${F}${K}${R}` + "`" + ` : K : K]
        : (listener: (payload: T[K]) => void) => void;
};
type ListenActionable<E> = ListenerTemplate<E, "add*Listener" | "remove*Listener", "*">;
type ClickEventSupport = ListenActionable<{ Click: 'some-click-event-payload' }>;

[|class C implements ClickEventSupport { }|]"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMappedType2", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
