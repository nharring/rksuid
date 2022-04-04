#[macro_use]
extern crate arrayref;
extern crate strum;
extern crate strum_macros;

/// Module for creating, representing and transforming K-Sortable UIDs as described by Segment.io
use base_encode::{from_str, to_string};
use chrono::prelude::*;
use lazy_static::lazy_static;
use rand::distributions::Standard;
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;
use strum_macros::{Display, EnumIter};

/// Base62 Alphabet which preserves lexigraphic sorting
pub const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

lazy_static! {
    static ref BASE_EPOCH: DateTime<Utc> = Utc.timestamp(1_400_000_000, 0);
}
///  # Examples
///  ```
///  use rksuid::{deserialize, Ksuid};
///  let ksuid: Ksuid = Ksuid::new();
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
pub enum RngType {
    PCG64FAST,
}

impl Ksuid {
    /// Creates new Ksuid with current timestamp and random payload
    ///
    /// # Examples
    /// ```
    /// use ::rksuid::Ksuid;
    ///
    /// let ksuid = Ksuid::new();
    /// ```
    pub fn new() -> Self {
        Self {
            timestamp: gen_timestamp(),
            payload: gen_payload(None),
        }
    }

    /// Creates new Ksuid with specified timestamp and random payload
    pub fn new_with_timestamp(timestamp: u32) -> Self {
        Self {
            timestamp,
            payload: gen_payload(None),
        }
    }

    /// Creates new Ksuid with current timestamp and specified payload
    pub fn new_with_payload(payload: u128) -> Self {
        Self {
            payload,
            timestamp: gen_timestamp(),
        }
    }

    /// Creates new Ksuid with specified timestamp and paylod
    pub fn new_with_timestamp_and_payload(timestamp: u32, payload: u128) -> Self {
        Self { timestamp, payload }
    }

    /// Creates new Ksuid from the supplied byte array in native byte order
    pub fn from_native_bytes(b: Vec<u8>) -> Self {
        let id = b.as_slice();
        Self {
            timestamp: u32::from_ne_bytes(*array_ref![id, 0, 4]),
            payload: u128::from_ne_bytes(*array_ref![id, 4, 16]),
        }
    }

    /// Creates new Ksuid from the supplied byte array in network byte order
    pub fn from_network_bytes(b: Vec<u8>) -> Self {
        let id = b.as_slice();
        Self {
            timestamp: u32::from_be_bytes(*array_ref![id, 0, 4]),
            payload: u128::from_be_bytes(*array_ref![id, 4, 16]),
        }
    }

    /// Serialize ksuid into base62 encoded string 27 characters long
    /// # Examples
    /// ```
    /// use rksuid::Ksuid;
    ///
    /// let ksuid = Ksuid::new_with_timestamp_and_payload(107608047, 0xB5A1CD34B5F99D1154FB6853345C9735);
    /// let serialized = ksuid.serialize();
    /// assert_eq!(serialized, "0ujtsYcgvSTl8PAuAdqWYSMnLOv");
    ///
    /// ```
    /// ```text
    /// 0ujtsYcgvSTl8PAuAdqWYSMnLOv
    /// ```
    pub fn serialize(&self) -> String {
        
        let all_bytes = self.get_network_bytes();
        let mut merged_string: String = to_string(array_ref![all_bytes, 0, 20], 62, ALPHABET).unwrap();
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

    /// Get Vec<u8> of all 20 bytes of the Ksuid in Network byte order
    pub fn get_network_bytes(&self) -> Vec<u8> {
        self.timestamp
            .to_be_bytes()
            .iter()
            .copied()
            .chain(self.payload.to_be_bytes().iter().copied())
            .collect()
    }

    /// Get Vec<u8> of all 20 bytes of the Ksuid in native byte order
    pub fn get_bytes(&self) -> Vec<u8> {
        self.timestamp
            .to_ne_bytes()
            .iter()
            .copied()
            .chain(self.payload.to_ne_bytes().iter().copied())
            .collect()
    }

    /// Get Vec<u8> of the 16 bytes in the payload in Network byte order
    pub fn payload_network_bytes(&self) -> Vec<u8> {
        self.payload.to_be_bytes().to_vec()
    }

    /// Get Vec<u8> of the 16 bytes in teh payload in native byte order
    pub fn payload_bytes(&self) -> Vec<u8> {
        self.payload.to_ne_bytes().to_vec()
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
        let all_bytes = self.get_network_bytes();
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
    /// use rksuid;
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
            formatted.push_str(line);
            formatted.push('\n');
        }
        formatted
    }
}

/// creates new ksuid from base62 encoded string serialized representation
/// # Examples
/// ```
/// use rksuid;
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
        Ksuid::from_network_bytes(bytes_from_str_be)
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
/// use rksuid::gen_epoch;
/// let ksuid_epoch = gen_epoch();
/// println!("{:?}", ksuid_epoch);
/// ```
/// ```text
/// 2014-05-13T16:53:20Z
/// ```
///
pub fn gen_epoch() -> DateTime<Utc> {
    *BASE_EPOCH
}

/// Convert a u32 timestamp from a Ksuid.timestamp into DateTime<Utc>
/// # Examples
/// ```
/// use rksuid::to_std_epoch;
///
/// let some_day = to_std_epoch(10);
/// println!("{:?}", some_day);
/// ```
/// ```text
/// 2014-05-13T16:53:30Z
/// ```
pub fn to_std_epoch(timestamp: u32) -> DateTime<Utc> {
    let base_epoch = gen_epoch();
    base_epoch
        .checked_add_signed(chrono::Duration::seconds(i64::from(timestamp)))
        .expect("timestamp is convertible")
}
