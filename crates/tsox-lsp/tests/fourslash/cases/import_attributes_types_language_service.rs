use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_pattern_ambient_module_with_import_attributes() {
    let content = r#"// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "preserve", "moduleResolution": "bundler" } }

// @Filename: /types.d.ts
declare const outerAttributeName: unique symbol;
type OuterAttributeValue = "css";

declare module "*.asset" with { /*attributeName*/type: /*attributeValue*/"css" } {
	export interface CssAttributeValue {}
    export const shared: "css";
    export const cssOnly: "css-only";
}
declare module "*.asset" with { type: "text" } {
	export interface TextAttributeValue {}
    export const shared: "text";
    export const textOnly: "text-only";
}

// @Filename: /index.ts
import * as css from "./style.asset" with { type: "css" };
import * as text from "./copy.asset" with { type: "text" };
css./*css*/cssOnly;
text./*text*/textOnly;"#;
    let mut s = Session::new_for_test("completionsPatternAmbientModuleWithImportAttributes", content);
    // TODO: f.VerifyCompletions(t, "attributeName", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "attributeValue", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "css", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "text", &fourslash.CompletionsExpectedList{
}

#[test]
fn pattern_ambient_module_with_import_attributes_language_service() {
    let content = r#"// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "preserve", "moduleResolution": "bundler" } }

// @Filename: /types.d.ts
declare module "*.asset" with { type: "css" } {
    export interface CssPayload { kind: "css" }
    export const shared: CssPayload;
}
declare module "*.asset" with { type: "text" } {
    export interface TextPayload { kind: "text" }
    export const shared: TextPayload;
}

// @Filename: /index.ts
import * as css from /*cssModule*/"./style.asset" with { type: "css" };
import * as text from /*textModule*/"./copy.asset" with { type: "text" };
css./*cssUse*/shared;
text./*textUse*/shared;"#;
    let mut s = Session::new_for_test("patternAmbientModuleWithImportAttributesLanguageService", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineHover(t)
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "cssModule", "textModule", "cssUse", "textUse")
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "cssUse", "textUse")
    // TODO: f.VerifyBaselineFindAllReferences(t, "cssUse", "textUse")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "cssUse", "textUse")
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "cssUse", "textUse")
}
