use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_default_pascal_case() {
    let content = r#"// @jsx: react
// @module: esnext
// @moduleResolution: bundler

// @Filename: /src/components/ChargerHeader.tsx
export default function ChargerHeader() {
  return null;
}

// @Filename: /src/screens/SomeScreen.tsx
export function SomeScreen() {
  return <ChargerHeader/*1*/
}
"#;
    let mut s = Session::new_for_test("autoImportDefaultPascalCase", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{"1"})
}

#[test]
fn auto_import_default_pascal_case_anonymous() {
    let content = r#"// @jsx: react
// @module: esnext
// @moduleResolution: bundler

// @Filename: /src/components/ChargerHeader.tsx
export default function() {
  return null;
}

// @Filename: /src/screens/SomeScreen.tsx
export function SomeScreen() {
  return <ChargerHeader/*1*/
}
"#;
    let mut s = Session::new_for_test("autoImportDefaultPascalCaseAnonymous", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{"1"})
}

#[test]
fn auto_import_default_pascal_case_case_insensitive() {
    let content = r#"// @jsx: react
// @module: esnext
// @moduleResolution: bundler
// @useCaseSensitiveFileNames: false

// @Filename: /src/components/ChargerHeader.tsx
export default function ChargerHeader() {
  return null;
}

// @Filename: /src/screens/SomeScreen.tsx
export function SomeScreen() {
  return <ChargerHeader/*1*/
}
"#;
    let mut s = Session::new_for_test("autoImportDefaultPascalCaseCaseInsensitive", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{"1"})
}

#[test]
fn auto_import_default_pascal_case_anonymous_case_insensitive() {
    let content = r#"// @jsx: react
// @module: esnext
// @moduleResolution: bundler
// @useCaseSensitiveFileNames: false

// @Filename: /src/components/ChargerHeader.tsx
export default function() {
  return null;
}

// @Filename: /src/screens/SomeScreen.tsx
export function SomeScreen() {
  return <ChargerHeader/*1*/
}
"#;
    let mut s = Session::new_for_test("autoImportDefaultPascalCaseAnonymousCaseInsensitive", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{"1"})
}

#[test]
fn auto_import_default_pascal_case_reexport_case_insensitive() {
    let content = r#"// @jsx: react
// @module: esnext
// @moduleResolution: bundler
// @useCaseSensitiveFileNames: false

// @Filename: /src/components/ChargerHeader.tsx
export default function() {
  return null;
}

// @Filename: /src/components/index.ts
export { default } from "./ChargerHeader";

// @Filename: /src/screens/SomeScreen.tsx
export function SomeScreen() {
  return <ChargerHeader/*1*/
}
"#;
    let mut s = Session::new_for_test("autoImportDefaultPascalCaseReexportCaseInsensitive", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{"1"})
}

#[test]
fn auto_import_default_pascal_case_alias_case_insensitive() {
    let content = r#"// @jsx: react
// @module: esnext
// @moduleResolution: bundler
// @useCaseSensitiveFileNames: false

// @Filename: /src/components/ChargerHeader.tsx
function ChargerHeader() {
  return null;
}
export default ChargerHeader;

// @Filename: /src/screens/SomeScreen.tsx
export function SomeScreen() {
  return <ChargerHeader/*1*/
}
"#;
    let mut s = Session::new_for_test("autoImportDefaultPascalCaseAliasCaseInsensitive", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{"1"})
}
