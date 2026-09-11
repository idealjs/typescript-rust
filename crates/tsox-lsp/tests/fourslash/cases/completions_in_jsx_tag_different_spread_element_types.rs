use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_in_jsx_tag_different_spread_element_types() {
    let content = r#"
// @Filename: /completionsWithDifferentSpreadTypes.tsx
// @strict: true

// A reasonable type to spread.
export function ComponentObjectX(props: { x: string }) {
    return <SomeComponent {...props} /*objectX*//>;
}

// A questionable but valid type to spread.
export function ComponentObjectXOrY(props: { x: string } | { y: string }) {
    return <SomeComponent {...props} /*objectXOrY*//>;
}

// A very unexpected type to spread (a union containing a primitive).
export function ComponentNumberOrObjectX(props: number | { x: string }) {
    return <SomeComponent {...props} /*numberOrObjectX*//>;
}

// Very unexpected, but still structured (union) types.
// 'boolean' is 'true | false' and an optional 'null' is really 'null | undefined'.
export function ComponentBoolean(props: boolean) {
    return <SomeComponent {...props} /*boolean*//>;
}
export function ComponentOptionalNull(props?: null) {
    return <SomeComponent {...props} /*optNull*//>;
}

// Primitive types (non-structured).
export function ComponentAny(props: any) {
    return <SomeComponent {...props} /*any*//>;
}
export function ComponentUnknown(props: unknown) {
    return <SomeComponent {...props} /*unknown*//>;
}
export function ComponentNever(props: never) {
    return <SomeComponent {...props} /*never*//>;
}
export function ComponentUndefined(props: undefined) {
    return <SomeComponent {...props} /*undefined*//>;
}
export function ComponentNumber(props: number) {
    return <SomeComponent {...props} /*number*//>;
}
"#;
    let mut s = Session::new_for_test("completionsInJsxTagDifferentSpreadElementTypes", content);
    // TODO: f.GoToEachMarker(t, nil, func(marker *fourslash.Marker, index int) {
}
