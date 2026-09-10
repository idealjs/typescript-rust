use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_namespace_type_aliases() {
    let content = r#"
type BaseConfig = { host: string; port: number };

declare namespace Config/*1*/ {
    type Readonly<T> = { readonly [K in keyof T]: T[K] };
    type Optional<T> = { [K in keyof T]?: T[K] };
    type ServerConfig = Readonly<BaseConfig>;
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityNamespaceTypeAliases", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
