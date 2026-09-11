use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_inherited_properties10() {
    let content = r#"interface IFeedbackHandler {
  /*1*/handleAccept?(): void;
  handleReject?(): void;
}

abstract class AbstractFeedbackHandler implements IFeedbackHandler {}

class FeedbackHandler extends AbstractFeedbackHandler {
  /*2*/handleAccept(): void {
    console.log("Feedback accepted");
  }

  handleReject(): void {
    console.log("Feedback rejected");
  }
}

function foo(handler: IFeedbackHandler) {
  handler./*3*/handleAccept?.();
  handler.handleReject?.();
}"#;
    let mut s = Session::new_for_test("referencesForInheritedProperties10", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
