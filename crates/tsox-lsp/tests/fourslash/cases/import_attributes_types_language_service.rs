use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "attributeName", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "attributeValue", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "css", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "text", &fourslash.CompletionsExpectedList{
}

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "cssModule", "textModule", "cssUse", "textUse")
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "cssUse", "textUse")
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "cssUse", "textUse")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "cssUse", "textUse")
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "cssUse", "textUse")
}
