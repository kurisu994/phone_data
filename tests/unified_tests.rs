mod test_suite;

#[cfg(test)]
mod tests {
    use super::test_suite::*;

    #[test]
    fn run_unified_test_suite() {
        run_all_tests();
    }
}