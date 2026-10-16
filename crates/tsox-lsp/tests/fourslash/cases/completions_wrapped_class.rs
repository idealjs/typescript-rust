use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_wrapped_class() {
    let content = r#"class Client {
    private close() { }
    public open() { }
}
type Wrap<T> = T &
{
    [K in Extract<keyof T, string> as `${K}Wrapped`]: T[K];
};
class Service {
    method() {
        let service = undefined as unknown as Wrap<Client>;
        const { /*a*/ } = service;
    }
}"#;
    let mut s = Session::new_for_test("completionsWrappedClass", content);
    fourslash::verify_completions_exact_at(&mut s, Some("a"), &["open", "openWrapped"]);
}
