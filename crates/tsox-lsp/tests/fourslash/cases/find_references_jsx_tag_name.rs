use tsox_lsp::fourslash::Session;


#[test]
fn find_references_jsx_tag_name() {
    let content = r#"// @Filename: index.tsx
import { /*1*/SubmissionComp } from "./RedditSubmission"
function displaySubreddit(subreddit: string) {
    let components = submissions
        .map((value, index) => <SubmissionComp key={ index } elementPosition= { index } {...value.data} />);
}
// @Filename: RedditSubmission.ts
export const /*2*/SubmissionComp = (submission: SubmissionProps) =>
    <div style={{ fontFamily: "sans-serif" }}></div>;"#;
    let _s = Session::new_for_test("findReferencesJSXTagName", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
