use log::debug;

pub fn add(left: u64, right: u64) -> u64 {
    debug!("Adding {} and {} now...", left, right);

    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
