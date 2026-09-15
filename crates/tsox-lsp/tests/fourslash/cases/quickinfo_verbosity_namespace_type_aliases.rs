use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("quickinfoVerbosityNamespaceTypeAliases", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
