#[macro_use]
extern crate arrayref;
extern crate strum;
extern crate strum_macros;
pub mod rksuid {
    //! Module for creating, representing and transforming K-Sortable UIDs as described by Segment.io
    //!
    //! # Examples
    //! ```
    //! use ::rksuid::rksuid;
    //! use ::rksuid::rksuid::Ksuid;
    //!
    //! let ksuid: Ksuid = rksuid::new(None, None);
    //!
    //! let serialized: String = ksuid.serialize();
    //!
    //! let ksuid_2: Ksuid = rksuid::deserialize(&serialized);
    //! ```
    use base_encode::{from_str, to_string};
    extern crate time;
    use chrono::prelude::*;
    use time::Duration;
    use wyhash::wyrng;
    use rand::prelude::*;
    use rand::distributions::Standard;
    use rand_hc::Hc128Rng;
    use rand_pcg::Pcg64Mcg;
    use rand_xoshiro::{Xoshiro256StarStar, Xoshiro256PlusPlus};
    use hyper_thread_random::generate_hyper_thread_safe_random_u64;
    use rand_chacha::ChaCha8Rng;
    use rand_chacha::ChaCha12Rng;
    use strum_macros::{Display, EnumIter};


    /// Base62 Alphabet which preserves lexigraphic sorting
    pub const ALPHABET: &[u8; 62] =
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    /// K-Sortable Unique ID
    #[derive(Debug, PartialOrd, Ord, Clone, Copy, PartialEq, Eq)]
    pub struct Ksuid {
        /// 32 bit unsigned seconds since 2014-05-13T16:53:30Z
        pub timestamp: u32,
        /// 128 bits of payload, usually a rand\<u128\>
        pub payload: u128,
    }

    /// RNG Types supported for payload creation, ChaCha8 is the default
    #[derive(Debug, PartialOrd, Ord, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
    pub enum RngType{
        CORE,
        CHACHA8,
        CHACHA12,
        HYPERTHREAD,
        WYRNG,
        PCG64FAST,
        HC128,
        XOSHIRO256PLUSPLUS,
        XOSHIRO256STARSTAR,
    }

    /// Creates new Ksuid with optionally specified timestamp and payload
    ///
    /// # Examples
    ///
    /// ```
    /// use ::rksuid::rksuid;
    ///
    /// let ksuid = rksuid::new(None, None);
    /// ```
    pub fn new(timestamp: Option<u32>, payload: Option<u128>) -> Ksuid {
        let internal_timestamp = match timestamp {
            None => gen_timestamp(),
            Some(i) => i,
        };
        let internal_payload = match payload {
            None => gen_payload(None),
            Some(i) => i,
        };
        Ksuid {
            timestamp: internal_timestamp,
            payload: internal_payload,
        }
    }

    impl Ksuid {
        /// Serialize ksuid into base62 encoded string 27 characters long
        /// # Examples
        ///
        /// ```
        /// use ::rksuid::rksuid;
        ///
        /// let ksuid = rksuid::new(Some(107608047), Some(0xB5A1CD34B5F99D1154FB6853345C9735));
        /// println!("{}", ksuid.serialize());
        /// ```
        /// ```text
        /// 0ujtsYcgvSTl8PAuAdqWYSMnLOv
        /// ```
        pub fn serialize(&self) -> String {
            let mut merged_string: String;
            let all_bytes = self.get_bytes();
            merged_string = to_string(array_ref![all_bytes, 0, 20], 62, ALPHABET).unwrap();
            if merged_string.char_indices().count() < 27 {
                // Zero pad the left side of the string to get it to the required 27
                let num_zeros = 27 - merged_string.char_indices().count();
                let zero_str = String::from("0").repeat(num_zeros);
                merged_string = zero_str + merged_string.as_str();
            }
            return merged_string;
        }

        /// Return timestamp of Ksuid as DateTime<Utc>
        pub fn get_time(&self) -> DateTime<Utc> {
            to_std_epoch(self.timestamp)
        }

        /// Returns String of payload bytes hex encoded
        pub fn get_payload(&self) -> String {
            let payload_bytes = self.payload.to_be_bytes().to_vec();
            to_string(array_ref![payload_bytes, 0, 16], 16, b"0123456789ABCDEF").unwrap()
        }

        /// Get Vec<u8> of all 20 bytes of the Ksuid
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

        /// Get Vec<String> of lines in formatted output
        pub fn get_formatted_lines(&self) -> [String; 7] {
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
            let output: [String; 7] = [
                "REPRESENTATION:".to_string(),
                format!("\tString: {}", self.serialize()),
                format!("\tRaw: {}", all_bytes_str),
                "COMPONENTS:".to_string(),
                format!("\tTime: {}", self.get_time().to_rfc2822()),
                format!("\tTimestamp: {}", self.timestamp),
                format!("\tPayload: {}", self.get_payload()),
            ];
            return output;
        }

        /// Get 7 line formatted string with representation and components of Ksuid
        /// suitable for printing.
        /// # Examples
        /// ```
        /// use ::rksuid::rksuid;
        ///
        /// let ksuid = rksuid::deserialize("0ujtsYcgvSTl8PAuAdqWYSMnLOv");
        /// println!("{}", ksuid.get_formatted());
        /// ```
        /// ```text
        /// REPRESENTATION:
        ///     String: 0ujtsYcgvSTl8PAuAdqWYSMnLOv
        ///      Raw: 0669F7EFB5A1CD34B5F99D1154FB6853345C9735
        /// COMPONENTS:
        ///     Time: 2017-10-09 21:00:47 -0700 PDT
        ///     Timestamp: 107608047
        ///     Payload: B5A1CD34B5F99D1154FB6853345C9735
        /// ```
        pub fn get_formatted(&self) -> String {
            let mut formatted: String = String::new();
            for line in self.get_formatted_lines().iter() {
                formatted.push_str(&line);
                formatted.push_str("\n");
            }
            return formatted;
        }
    }

    /// creates new ksuid from base62 encoded string serialized representation
    /// # Examples
    /// ```
    /// use ::rksuid::rksuid;
    ///
    /// let ksuid = rksuid::deserialize("0ujtsYcgvSTl8PAuAdqWYSMnLOv");
    /// println!("{}", ksuid.timestamp);
    /// ```
    /// ```text
    /// 107608047
    /// ```
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
    // Returns now as u32 seconds since the unix epoch + 14e8 (May 13, 2014)
    fn gen_timestamp() -> u32 {
        Utc::now().signed_duration_since(gen_epoch()).num_seconds() as u32
    }

    /// Returns a pseudo-random u128 for use as payload of a new Ksuid
    /// Optionally accepts an RngType instead of default ChaCha8
    pub fn gen_payload(rng: Option<RngType>) -> u128 {
        match rng {
            Some(rng) if rng == RngType::CORE => return gen_payload_core(),
            Some(rng) if rng == RngType::CHACHA8 => return gen_payload_chacha8(),
            Some(rng) if rng == RngType::CHACHA12 => return gen_payload_chacha12(),
            Some(rng) if rng == RngType::HYPERTHREAD => return gen_payload_hyperthread(),
            Some(rng) if rng == RngType::WYRNG => return gen_payload_wyrng(),
            Some(rng) if rng == RngType::PCG64FAST => return gen_payload_pcg64_fast(),
            Some(rng) if rng == RngType::HC128 => return gen_payload_hc128(),
            Some(rng) if rng == RngType::XOSHIRO256PLUSPLUS => return gen_payload_xoshiro256plusplus(),
            Some(rng) if rng == RngType::XOSHIRO256STARSTAR => return gen_payload_xoshiro256starstar(),
            Some(_) => return gen_payload_chacha8(),
            None => return gen_payload_chacha8(),
        }
    }

    // Returns a fresh random u128 for use as payload
    fn gen_payload_core() -> u128 {
        let payload: u128 = StdRng::from_entropy().sample(Standard);
        return payload;
    }
    // Some additional payload generators for benchmarking
    // Some from the rand crate family
    fn gen_payload_chacha8() -> u128 {
        let payload: u128 = ChaCha8Rng::from_entropy().sample(Standard);
        return payload;
    }
    fn gen_payload_chacha12() -> u128 {
        let payload: u128 = ChaCha12Rng::from_entropy().sample(Standard);
        return payload;
    }
    fn gen_payload_pcg64_fast() -> u128 {
        let payload: u128 = Pcg64Mcg::from_entropy().sample(Standard);
        return payload;
    }
    fn gen_payload_hc128() -> u128 {
        let payload: u128 = Hc128Rng::from_entropy().sample(Standard);
        return payload;
    }
    fn gen_payload_xoshiro256plusplus() -> u128 {
        let payload: u128 = Xoshiro256PlusPlus::from_entropy().sample(Standard);
        return payload;
    }
    fn gen_payload_xoshiro256starstar() -> u128 {
        let payload: u128 = Xoshiro256StarStar::from_entropy().sample(Standard);
        return payload;
    }
    // Some more esoteric generators
    fn gen_payload_hyperthread() -> u128 {
        let first = generate_hyper_thread_safe_random_u64();
        let second = generate_hyper_thread_safe_random_u64();
        let byte_vec: Vec<u8> = first.to_le_bytes().to_vec().into_iter().chain(second.to_le_bytes().to_vec().into_iter()).collect();
        u128::from_ne_bytes(*array_ref![byte_vec, 0, 16])
    }
    fn gen_payload_wyrng() -> u128 {
        let mut seed = 8675309;
        let first = wyrng(&mut seed);
        let second = wyrng(&mut seed);
        let byte_vec: Vec<u8> = first.to_le_bytes().to_vec().into_iter().chain(second.to_le_bytes().to_vec().into_iter()).collect();
        u128::from_ne_bytes(*array_ref![byte_vec, 0, 16])
    }


    /// Returns a Chrono::DateTime<Utc> representing the adjusted epoch
    /// # Examples
    /// ```
    /// use rksuid::rksuid::gen_epoch;
    /// let ksuid_epoch = gen_epoch();
    /// println!("{:?}", ksuid_epoch);
    /// ```
    /// ```text
    /// 2014-05-13T16:53:20Z
    /// ```
    ///
    pub fn gen_epoch() -> DateTime<Utc> {
        Utc.timestamp(1400000000, 0)
    }

    /// Convert a u32 timestamp from a Ksuid.timestamp into DateTime<Utc>
    /// # Examples
    /// ```
    /// use rksuid::rksuid::to_std_epoch;
    ///
    /// let some_day = to_std_epoch(10);
    /// println!("{:?}", some_day);
    /// ```
    /// ```text
    /// 2014-05-13T16:53:30Z
    /// ```
    pub fn to_std_epoch(timestamp: u32) -> DateTime<Utc> {
        let base_epoch = gen_epoch();
        base_epoch + Duration::seconds(timestamp as i64)
    }
}
