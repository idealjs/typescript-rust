use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoExists"]
#[test]
fn quick_info_type_argument_inference_with_method_without_body() {
    let content = r#"interface ProxyHandler<T extends object> {
    getPrototypeOf?(target: T): object | null;
}
interface ProxyConstructor {
    new <T extends object>(target: T, handler: ProxyHandler<T>): T;
}
declare var Proxy: ProxyConstructor;
let target = {}
let proxy = new /**/Proxy(target, {
    getPrototypeOf()
})"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
}
