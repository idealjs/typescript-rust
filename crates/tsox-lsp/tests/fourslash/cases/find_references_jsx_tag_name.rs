use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
