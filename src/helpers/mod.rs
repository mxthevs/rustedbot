pub fn has_at_least_n_args(args: &str, n: usize) -> bool {
    args.split_whitespace().take(n).count() >= n
}
