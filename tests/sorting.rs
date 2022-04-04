extern crate rksuid;

#[cfg(test)]
mod tests {
    use rksuid::Ksuid;

    // Sorting tests
    #[test]
    fn gt_lt() {
        let first = Ksuid::new_with_timestamp(100);
        let second = Ksuid::new_with_timestamp(500);
        let third = Ksuid::new_with_timestamp(12321312);
        assert!(first < second);
        assert!(second < third);
        assert!(first < third);
    }
    #[test]
    fn sort_by_timestamp() {
        let first = Ksuid::new_with_timestamp(100);
        let second = Ksuid::new_with_timestamp(500);
        let third = Ksuid::new_with_timestamp(12321312);
        let mut ksuid_vec: Vec<Ksuid> = vec![second, third, first];
        ksuid_vec.sort();
        assert_eq!(ksuid_vec[0], first);
        assert_eq!(ksuid_vec[2], third);
    }
}
