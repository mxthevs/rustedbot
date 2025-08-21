pub fn has_at_least_one_arg(args: &str) -> bool {
    !args.is_empty()
        && !args
            .split(' ')
            .map(|param| param.trim())
            .collect::<Vec<&str>>()
            .is_empty()
}

pub fn has_more_than_one_arg(args: &str) -> bool {
    !args.is_empty()
        && args
            .split(' ')
            .map(|param| param.trim())
            .collect::<Vec<&str>>()
            .len()
            > 1
}

pub fn has_at_least_four_args(args: &str) -> bool {
    !args.is_empty()
        && args
            .split(' ')
            .map(|param| param.trim())
            .collect::<Vec<&str>>()
            .len()
            > 3
}
