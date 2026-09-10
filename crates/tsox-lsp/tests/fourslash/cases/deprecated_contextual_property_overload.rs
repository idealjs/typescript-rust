use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn deprecated_contextual_property_overload() {
    let content = r#"interface DeprecatedOptions {
    kind: "deprecated";
    /** @deprecated */
    value: number;
}
interface CurrentOptions {
    kind: "current";
    value: number;
}
declare function select(options: DeprecatedOptions): void;
declare function select(options: CurrentOptions): void;

select({ kind: "current", value: 1 });

/** @deprecated */
declare const deprecatedValue: number;
select({ kind: "current", value: [|deprecatedValue|] });

interface DeprecatedContainer {
    /** @deprecated */
    value: number;
}
declare const deprecatedContainer: DeprecatedContainer;
select({ kind: "current", value: deprecatedContainer.[|value|] });

/** @deprecated */
declare function deprecatedCall(): number;
select({ kind: "current", value: [|deprecatedCall|]() });

interface DeprecatedAccessorOptions {
    /** @deprecated */
    value: number;
}
declare function accessor(options: DeprecatedAccessorOptions): void;
accessor({ get [|value|]() { return 1; } });
accessor({ set [|value|](_value: number) {} });

interface DeprecatedNamedOptions {
    /** @deprecated */
    "string-name": number;
    /** @deprecated */
    1: number;
}
declare function named(options: DeprecatedNamedOptions): void;
named({ [|"string-name"|]: 1, [|1|]: 1 });

interface FirstDeprecatedOptions {
    kind: "first";
    /** @deprecated */
    value: number;
}
interface SecondDeprecatedOptions {
    kind: "second";
    /** @deprecated */
    value: number;
}
declare function selectDeprecated(options: FirstDeprecatedOptions): void;
declare function selectDeprecated(options: SecondDeprecatedOptions): void;

selectDeprecated({ kind: "second", [|value|]: 1 });"#;
    let mut s = Session::new_for_test("deprecatedContextualPropertyOverload", content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
