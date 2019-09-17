#![feature(test)]

#[macro_use]
extern crate arrayref;

extern crate test;

pub mod rksuid {
    use base_encode::{from_str, to_string};
    use rand::distributions::Standard;
    use rand::prelude::*;
    extern crate time;
    use chrono::prelude::*;
    use time::Duration;

    pub const ALPHABET: &[u8; 62] =
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    #[derive(Debug, PartialOrd, Ord, Clone, Copy, PartialEq, Eq)]
    pub struct Ksuid {
        pub timestamp: u32,
        pub payload: u128,
    }

    // Creates new ksuid with optionally specified timestamp and payload
    pub fn new(timestamp: Option<u32>, payload: Option<u128>) -> Ksuid {
        let internal_timestamp = match timestamp {
            None => gen_timestamp(),
            Some(i) => i,
        };
        let internal_payload = match payload {
            None => gen_payload(),
            Some(i) => i,
        };
        Ksuid {
            timestamp: internal_timestamp,
            payload: internal_payload,
        }
    }

    impl Ksuid {
        // Serialize ksuid into base62 encoded string
        pub fn serialize(&self) -> String {
            let mut merged_string: String;
            let all_bytes = self.get_bytes();
            merged_string = to_string(array_ref![all_bytes, 0, 20], 62, ALPHABET).unwrap();
            if merged_string.char_indices().count() < 27 {
                // We will zero pad the left side of the string to get it to the required 27
                let num_zeros = 27 - merged_string.char_indices().count();
                let zero_str = String::from("0").repeat(num_zeros);
                merged_string = zero_str + merged_string.as_str();
            }
            return merged_string;
        }

        pub fn get_time(&self) -> DateTime<Utc> {
            to_std_epoch(self.timestamp)
        }

        pub fn get_payload(&self) -> String {
            let payload_bytes = self.payload.to_be_bytes().to_vec();
            to_string(array_ref![payload_bytes, 0, 16], 16, b"0123456789ABCDEF").unwrap()
        }

        fn get_bytes(&self) -> Vec<u8> {
            let all_bytes = self.timestamp.to_be_bytes().to_vec().into_iter().chain(self.payload.to_be_bytes().to_vec().into_iter()).collect();
            return all_bytes;
        }

        pub fn get_formatted_lines(&self) -> Vec<String> {
            // REPRESENTATION:
            //   String: Base62, 0 padded to 27 chars
            //      Raw: Hex of raw big endian 20 bytes
            // COMPONENTS:
            //        Time: RFC 2822
            //   Timestamp: Seconds since Ksuid epoch
            //     Payload: Hex of u128
            let all_bytes = self.get_bytes();
            let all_bytes_str = to_string(array_ref![all_bytes, 0, 16], 16, b"0123456789ABCDEF").unwrap();
            let ksuid_time = self.get_time();
            let payload_str = self.get_payload();
            let mut output: Vec<String> = Vec::new();
            output.push("REPRESENTATION:".to_string());
            output.push(format!("\tString: {}", self.serialize()));
            output.push(format!("\tRaw: {}", all_bytes_str));
            output.push("COMPONENTS:".to_string());
            output.push(format!("\tTime: {}", ksuid_time.to_rfc2822()));
            output.push(format!("\tTimestamp: {}", self.timestamp));
            output.push(format!("\tPayload: {}", payload_str));
            return output;
        }

        pub fn get_formatted(&self) -> String {
            let mut formatted: String = String::new();
            for line in self.get_formatted_lines() {
                formatted.push_str(&line);
                formatted.push_str("\n");
            }
            return formatted;
        }
    }

    // creates new ksuid from base62 encoded string serialized representation
    pub fn deserialize(text: &str) -> Ksuid {
        let unpadded = text.trim_start_matches("0");
        let bytes_from_str_be_parsed = from_str(unpadded, 62, ALPHABET);
        if let Some(bytes_from_str_be) = bytes_from_str_be_parsed {
            let timestamp_bytes: &[u8; 4] = array_ref![bytes_from_str_be, 0, 4];
            let payload_bytes: &[u8; 16] = array_ref![bytes_from_str_be, 4, 16];
            let timestamp: u32 = u32::from_be_bytes(*timestamp_bytes);
            let payload: u128 = u128::from_be_bytes(*payload_bytes);
            let ksuid = new(Some(timestamp), Some(payload));
            return ksuid;
        } else {
            panic!();
        }
    }

    // Returns a fresh random u128 for use as payload
    fn gen_payload() -> u128 {
        let payload: u128 = StdRng::from_entropy().sample(Standard);
        return payload;
    }
    // Returns now as u32 seconds since the unix epoch + 14e8 (May 13, 2014)
    fn gen_timestamp() -> u32 {
        Utc::now().signed_duration_since(gen_epoch()).num_seconds() as u32
    }

    // Returns a Chrono::DateTime representing the adjusted epoch
    pub fn gen_epoch() -> DateTime<Utc> {
        Utc.timestamp(1400000000, 0)
    }

    pub fn to_std_epoch(timestamp: u32) -> DateTime<Utc> {
        let base_epoch = gen_epoch();
        base_epoch + Duration::seconds(timestamp as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rksuid::Ksuid;
    use chrono::prelude::*;
    use rand::distributions::Standard;
    use rand::prelude::*;
    use std::{thread, time};
    use test::Bencher;
    use rand::thread_rng;
    use rand::seq::SliceRandom;

    #[test]
    fn test_new_with_timestamp() {
        let ksuid = rksuid::new(Some(85), None);
        assert_eq!(ksuid.timestamp, 85);
    }
    // Creation tests
    #[test]
    fn test_new() {
        let first = rksuid::new(None, None);
        thread::sleep(time::Duration::from_millis(2000));
        let second = rksuid::new(None, None);
        assert_ne!(first.timestamp, second.timestamp);
    }
    #[test]
    fn test_new_with_payload() {
        let payload: u128 = StdRng::from_entropy().sample(Standard);
        let ksuid = rksuid::new(None, Some(payload));
        assert_eq!(payload, ksuid.payload);
    }
    #[test]
    fn test_new_with_payload_and_timestamp() {
        let payload: u128 = StdRng::from_entropy().sample(Standard);
        let epoch_base = rksuid::gen_epoch();
        let timestamp = Utc::now().signed_duration_since(epoch_base).num_seconds() as u32;
        let ksuid = rksuid::new(Some(timestamp), Some(payload));
        assert_eq!(ksuid.payload, payload);
        assert_eq!(ksuid.timestamp, timestamp);
    }
    // Test get_time
    #[test]
    fn test_get_time() {
        // Friday, July 14, 2017 2:40:00 AM
        let timestamp: DateTime<Utc> = Utc.timestamp(1500000000, 0);
        let epoch_offset = timestamp.signed_duration_since(rksuid::gen_epoch()).num_seconds();
        // Sanity check the expected timestamp for the ksuid
        assert_eq!(100000000, epoch_offset);
        let ksuid = rksuid::new(Some(epoch_offset as u32), None);
        let ksuid_time = ksuid.get_time();
        assert_eq!(timestamp, ksuid_time);
    }
    // SerDe tests
    #[test]
    fn test_serialize_with_random_data_returns_right_length() {
        let ksuid = rksuid::new(None, None);
        let serialized = ksuid.serialize();
        assert_eq!(serialized.char_indices().count(), 27);
    }
    #[test]
    fn test_serialize_deserialize() {
        let ksuid = rksuid::new(None, None);
        let serialized = ksuid.serialize();
        let ksuid2 = rksuid::deserialize(&serialized);
        assert_eq!(ksuid, ksuid2);
    }
    #[test]
    fn test_get_formatted_lines() {
        let ksuid = rksuid::deserialize("0ujtsYcgvSTl8PAuAdqWYSMnLOv");
        let formatted = ksuid.get_formatted_lines();
        assert!(!formatted.is_empty());
        let timestamp_line = "\tTimestamp: 107608047";
        assert_eq!(formatted[5], timestamp_line);
        let raw_line = "\tRaw: 669F7EFB5A1CD34B5F99D1154FB6853";
        assert_eq!(formatted[2], raw_line);
    }
    // Sorting tests
    #[test]
    fn test_ge_le() {
        let first = rksuid::new(Some(100), None);
        let second = rksuid::new(Some(500), None);
        let third = rksuid::new(Some(12321312), None);
        assert!(first < second);
        assert!(second < third);
        assert!(first < third);
    }
    #[test]
    fn test_sort_by_timestamp() {
        let first = rksuid::new(Some(100), None);
        let second = rksuid::new(Some(500), None);
        let third = rksuid::new(Some(12321312), None);
        let mut ksuid_vec: Vec<Ksuid> = vec![second, third, first];
        ksuid_vec.sort();
        assert_eq!(ksuid_vec[0], first);
        assert_eq!(ksuid_vec[2], third);
    }
    #[bench]
    fn bench_new_ksuid_creation(b: &mut Bencher) {
        b.iter(|| rksuid::new(None, None));
    }
    #[bench]
    fn bench_new_ksuid_fixed_timestamp(b: &mut Bencher) {
        b.iter(|| rksuid::new(Some(168582232), None));
    }
    #[bench]
    fn bench_new_ksuid_fixed_payload(b: &mut Bencher) {
        b.iter(|| rksuid::new(None, Some(123456789)));
    }
    #[bench]
    fn bench_serialize(b: &mut Bencher) {
        let ksuid = rksuid::new(None, None);
        b.iter(|| ksuid.serialize());
    }

    #[bench]
    fn bench_deserialize(b: &mut Bencher) {
        let ksuid = rksuid::new(None, None).serialize();
        b.iter(|| rksuid::deserialize(&ksuid));
    }

    fn build_ksuid_vec(n: i32) -> Vec<Ksuid> {
        let mut ksuids: Vec<Ksuid> = Vec::new();
        for i in 0..n {
            ksuids.push(rksuid::new(Some(i as u32), None));
        }
        ksuids.shuffle(&mut thread_rng());
        return ksuids;
    }

    #[bench]
    fn bench_sort(b: &mut Bencher) {
        let mut ksuids = build_ksuid_vec(500);
        b.iter(|| ksuids.sort());
    }

    #[bench]
    fn bench_sort_unstable(b: &mut Bencher) {
        let mut ksuids = build_ksuid_vec(500);
        b.iter(|| ksuids.sort_unstable());
    }

    fn build_segment_ksuid() -> Ksuid {
        rksuid::new(Some(107608047), Some(0xB5A1CD34B5F99D1154FB6853345C9735))
    }
    // Compat tests with ksuids generated by the segment go code
    #[test]
    fn test_serialize_compat_with_segment_ksuid() {
        // From github.com/segmentio/ksuid#inspect-the-components-of-a-ksuid
        // REPRESENTATION:
        //   String: 0ujtsYcgvSTl8PAuAdqWYSMnLOv
        //      Raw: 0669F7EFB5A1CD34B5F99D1154FB6853345C9735
        // COMPONENTS:
        //        Time: 2017-10-09 21:00:47 -0700 PDT
        //   Timestamp: 107608047
        //     Payload: B5A1CD34B5F99D1154FB6853345C9735
        let segment_serialized = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
        let ksuid_known_good = build_segment_ksuid();
        let test_serialized = ksuid_known_good.serialize();
        assert_eq!(segment_serialized, test_serialized);
    }

    #[test]
    fn test_deserialize_compat_with_segment_ksuid() {
        let segment_serialized = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
        let ksuid = rksuid::deserialize(segment_serialized);
        let known_good = build_segment_ksuid();
        assert_eq!(ksuid, known_good);
    }
}
