extern crate rksuid;

#[cfg(test)]
mod tests {
    use ::rksuid::rksuid;
    use ::rksuid::rksuid::Ksuid;
    use chrono::prelude::*;
    use rand::distributions::Standard;
    use rand::prelude::*;
    use std::{thread, time};

    // Sorting tests
    #[test]
    fn gt_lt() {
        let first = rksuid::new(Some(100), None);
        let second = rksuid::new(Some(500), None);
        let third = rksuid::new(Some(12321312), None);
        assert!(first < second);
        assert!(second < third);
        assert!(first < third);
    }
    #[test]
    fn sort_by_timestamp() {
        let first = rksuid::new(Some(100), None);
        let second = rksuid::new(Some(500), None);
        let third = rksuid::new(Some(12321312), None);
        let mut ksuid_vec: Vec<Ksuid> = vec![second, third, first];
        ksuid_vec.sort();
        assert_eq!(ksuid_vec[0], first);
        assert_eq!(ksuid_vec[2], third);
    }
}
