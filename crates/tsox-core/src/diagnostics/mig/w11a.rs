pub fn stringify_args(args: &[String]) -> Vec<String> {
    if args.is_empty() {
        return Vec::new();
    }
    args.to_vec()
}
