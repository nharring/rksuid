#[macro_use]
extern crate arrayref;
extern crate strum;
extern crate strum_macros;
pub mod rksuid {
    //! Module for creating, representing and transforming K-Sortable UIDs as described by Segment.io

    use base_encode::{from_str, to_string};
    use chrono::prelude::*;
    use rand::prelude::*;
    use rand::distributions::Standard;
    use rand_pcg::Pcg64Mcg;
    use strum_macros::{Display, EnumIter};


    /// Base62 Alphabet which preserves lexigraphic sorting
    pub const ALPHABET: &[u8; 62] =
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    ///  # Examples
    ///  ```
    ///  use ::rksuid::rksuid;
    ///  use ::rksuid::rksuid::Ksuid;
    ///  let ksuid: Ksuid = rksuid::new(None, None);
    ///  let serialized: String = ksuid.serialize();
    ///  let ksuid_2: Ksuid = rksuid::deserialize(&serialized);
    ///  assert_eq!(ksuid, ksuid_2);
    ///  ```
    /// K-Sortable Unique ID
    #[derive(Debug, PartialOrd, Ord, Clone, Copy, PartialEq, Eq)]
    pub struct Ksuid {
        /// 32 bit unsigned seconds since 2014-05-13T16:53:30Z
        pub timestamp: u32,
        /// 128 bits of payload, usually a rand\<u128\>
        pub payload: u128,
    }

    /// RNG Types supported for payload creation
    #[derive(Debug, PartialOrd, Ord, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
    pub enum RngType{
        PCG64FAST,
    }

    /// Creates new Ksuid with optionally specified timestamp and payload
    ///
    /// # Examples
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
        /// ```
        /// use ::rksuid::rksuid;
        ///
        /// let ksuid = rksuid::new(Some(107608047), Some(0xB5A1CD34B5F99D1154FB6853345C9735));
        /// let serialized = ksuid.serialize();
        /// assert_eq!(serialized, "0ujtsYcgvSTl8PAuAdqWYSMnLOv");
        ///
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
            merged_string
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
            self
                .timestamp
                .to_be_bytes()
                .to_vec()
                .into_iter()
                .chain(self.payload.to_be_bytes().to_vec().into_iter())
                .collect()
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
            output
        }

        /// Get 7 line formatted string with representation and components of Ksuid
        /// suitable for printing.
        /// # Examples
        /// ```
        /// use ::rksuid::rksuid;
        ///
        /// let ksuid = rksuid::deserialize("0ujtsYcgvSTl8PAuAdqWYSMnLOv");
        /// let formatted = ksuid.get_formatted();  // This binding is necessary for lifetime purposes
        /// let lines: Vec<&str> = formatted.lines().collect();
        /// assert_eq!(lines[5], "\tTimestamp: 107608047");
        ///
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
            formatted
        }
    }

    /// creates new ksuid from base62 encoded string serialized representation
    /// # Examples
    /// ```
    /// use ::rksuid::rksuid;
    ///
    /// let ksuid = rksuid::deserialize("0ujtsYcgvSTl8PAuAdqWYSMnLOv");
    /// println!("{}", ksuid.timestamp);
    /// assert_eq!(ksuid.timestamp, 107608047);
    /// ```
    /// ```text
    /// 107608047
    /// ```
    pub fn deserialize(text: &str) -> Ksuid {
        let unpadded = text.trim_start_matches('0');
        let bytes_from_str_be_parsed = from_str(unpadded, 62, ALPHABET);
        if let Some(bytes_from_str_be) = bytes_from_str_be_parsed {
            let timestamp_bytes: &[u8; 4] = array_ref![bytes_from_str_be, 0, 4];
            let payload_bytes: &[u8; 16] = array_ref![bytes_from_str_be, 4, 16];
            let timestamp: u32 = u32::from_be_bytes(*timestamp_bytes);
            let payload: u128 = u128::from_be_bytes(*payload_bytes);
            new(Some(timestamp), Some(payload))
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
            Some(_) => gen_payload_pcg64_fast(),
            None => gen_payload_pcg64_fast(),

        }
    }

    // Returns a fresh random u128 for use as payload
    fn gen_payload_pcg64_fast() -> u128 {
        let payload: u128 = Pcg64Mcg::from_entropy().sample(Standard);
        payload
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
        Utc.timestamp(1_400_000_000, 0)
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
        base_epoch.checked_add_signed(chrono::Duration::seconds(i64::from(timestamp))).expect("timestamp is convertible")
    }
}
