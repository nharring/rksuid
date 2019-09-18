extern crate rksuid;

#[cfg(test)]
mod tests{
use ::rksuid::rksuid;
use chrono::prelude::*;
use rand::distributions::Standard;
use rand::prelude::*;
use std::{thread, time};


// Creation tests
#[test]
fn new() {
    let first = rksuid::new(None, None);
    thread::sleep(time::Duration::from_millis(2000));
    let second = rksuid::new(None, None);
    assert_ne!(first.timestamp, second.timestamp);
}
#[test]
fn new_with_timestamp() {
    let ksuid = rksuid::new(Some(85), None);
    assert_eq!(ksuid.timestamp, 85);
}
#[test]
fn new_with_payload() {
    let payload: u128 = StdRng::from_entropy().sample(Standard);
    let ksuid = rksuid::new(None, Some(payload));
    assert_eq!(payload, ksuid.payload);
}
#[test]
fn new_with_payload_and_timestamp() {
    let payload: u128 = StdRng::from_entropy().sample(Standard);
    let epoch_base = rksuid::gen_epoch();
    let timestamp = Utc::now().signed_duration_since(epoch_base).num_seconds() as u32;
    let ksuid = rksuid::new(Some(timestamp), Some(payload));
    assert_eq!(ksuid.payload, payload);
    assert_eq!(ksuid.timestamp, timestamp);
}
}
