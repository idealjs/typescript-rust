use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn type_reference_and_import_deprecated() {
    let content = r#"// @filename: types.ts
/** @deprecated */
export type SelectorMap<T extends Record<string, (...params: unknown[]) => unknown>> = {
    [key in keyof T]: T[key];
};
// @filename: index.ts
/** @deprecated */
export type SelectorMap<T extends Record<string, (...params: unknown[]) => unknown>> = {
    [key in keyof T]: T[key];
};

export declare const value2: {
    sliceSelectors: <FuncMap extends [|import('./types').SelectorMap<FuncMap>|]>(selectorsBySlice: FuncMap) => { [P in keyof FuncMap]: Parameters<FuncMap[P]> };
};

export declare const value3: {
    sliceSelectors: <FuncMap extends [|SelectorMap<FuncMap>|]>(selectorsBySlice: FuncMap) => { [P in keyof FuncMap]: Parameters<FuncMap[P]> };
};"#;
    let mut s = Session::new_for_test("typeReferenceAndImportDeprecated", content);
    fourslash::go_to_file(&mut s, "index.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
