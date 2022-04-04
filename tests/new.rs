#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use rand::distributions::Standard;
    use rand::prelude::*;
    use rksuid::{gen_epoch, gen_payload, Ksuid, RngType};
    use std::any::{Any, TypeId};
    use std::{thread, time};
    use strum::IntoEnumIterator;

    // Creation tests
    #[test]
    fn test_new() {
        let first = Ksuid::new();
        thread::sleep(time::Duration::from_millis(2000));
        let second = Ksuid::new();
        assert_ne!(first.timestamp, second.timestamp);
    }
    #[test]
    fn new_with_timestamp() {
        let ksuid = Ksuid::new_with_timestamp(85);
        assert_eq!(ksuid.timestamp, 85);
    }
    #[test]
    fn new_with_payload() {
        let payload: u128 = StdRng::from_entropy().sample(Standard);
        let ksuid = Ksuid::new_with_payload(payload);
        assert_eq!(payload, ksuid.payload);
    }
    #[test]
    fn new_with_payload_and_timestamp() {
        let payload: u128 = StdRng::from_entropy().sample(Standard);
        let epoch_base = gen_epoch();
        let timestamp = Utc::now().signed_duration_since(epoch_base).num_seconds() as u32;
        let ksuid = Ksuid::new_with_timestamp_and_payload(timestamp, payload);
        assert_eq!(ksuid.payload, payload);
        assert_eq!(ksuid.timestamp, timestamp);
    }

    #[test]
    fn test_payload_variants() {
        for e in RngType::iter() {
            let payload: u128 = gen_payload(Some(e));
            assert_eq!(payload.type_id(), TypeId::of::<u128>());
        }
    }
}
