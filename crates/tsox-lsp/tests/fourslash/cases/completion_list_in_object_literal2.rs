use tsox_lsp::fourslash::Session;


#[test]
fn completion_list_in_object_literal2() {
    let content = r#"interface TelemetryService {
    publicLog(eventName: string, data: any): any;
};
class SearchResult {
    count() { return 5; }
    isEmpty() { return true; }
    fileCount(): string { return ""; }
}
class Foo {
    public telemetryService: TelemetryService;   // If telemetry service is of type 'any' (i.e. uncomment below line), the drop-down list works
    public telemetryService2;
    private test() {
        var onComplete = (searchResult: SearchResult) => {
            var hasResults = !searchResult.isEmpty();  // Drop-down list on searchResult fine here
            // No drop-down list available on searchResult members within object literal below
            this.telemetryService.publicLog('searchResultsShown', { count: searchResult./*1*/count(), fileCount: searchResult.fileCount() });
            this.telemetryService2.publicLog('searchResultsShown', { count: searchResult./*2*/count(), fileCount: searchResult.fileCount() });
        };
    }
}"#;
    let _s = Session::new_for_test("completionListInObjectLiteral2", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
