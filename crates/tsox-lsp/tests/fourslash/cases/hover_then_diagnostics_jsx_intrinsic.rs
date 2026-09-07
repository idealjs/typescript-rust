use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // ...then a subsequent diagnostic pull must not invent a TS"]
#[test]
fn hover_then_diagnostics_jsx_intrinsic() {
    let content = r#"// @Filename: /tsconfig.json
{ "compilerOptions": { "strict": true, "jsx": "preserve" } }
// @Filename: /jsx.d.ts
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: any;
    }
}
// @Filename: /file.tsx
export default function Home() {
    return <di/*1*/v>hi</div>;
}"#;
    let mut s = Session::new(content);
    // TODO: // Hover on the intrinsic element first...
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) JSX.IntrinsicElements.div: any", "")
    // TODO: // ...then a subsequent diagnostic pull must not invent a TS2304 for `div`.
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
