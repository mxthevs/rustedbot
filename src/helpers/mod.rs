pub fn has_at_least_one_arg(args: &str) -> bool {
    args != ""
        && args
            .split(' ')
            .map(|param| param.trim())
            .collect::<Vec<&str>>()
            .len()
            > 0
}

pub fn has_more_than_one_arg(args: &str) -> bool {
    args != ""
        && args
            .split(' ')
            .map(|param| param.trim())
            .collect::<Vec<&str>>()
            .len()
            > 1
}
