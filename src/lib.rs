#[macro_use]
extern crate arrayref;

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
            let all_bytes = self
                .timestamp
                .to_be_bytes()
                .to_vec()
                .into_iter()
                .chain(self.payload.to_be_bytes().to_vec().into_iter())
                .collect();
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
            let all_bytes_str =
                to_string(array_ref![all_bytes, 0, 16], 16, b"0123456789ABCDEF").unwrap();
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
