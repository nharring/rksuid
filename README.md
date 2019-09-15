# rksuid
 Rust implementation of Segment.io's ksuid

# Inspiration
Segment [published](https://segment.com/blog/a-brief-history-of-the-uuid/) a nifty UUID format and I thought it'd be fun to implement a compatible version in pure Rust as a way to learn the language more thoroughly and keep myself sharp while looking for a new coding gig.

# Usage
Cargo.toml:
```
rksuid = "0.1.0"
```

```rust
// Generate new Ksuid with current timestamp and random payload
let ksuid = ksuid::new(None, None);
// Serialize to a base-62 encoded string
let serialized = ksuid.serialize();

// Deserialize a base-62 encoded string into a Ksuid
let ksuid = ksuid::deserialize("1QtFxXJfPVU6NOwPOsHsaihkm8U");
println!("{:?}", ksuid);
```
```rust
Ksuid { timestamp: 168582232, payload: 312509952699879867963141934813379438280 }
```



# Current Features
- Creation of new ksuids via factory function with ability to specify timestamp and/or payload.
- Serialize to base62 encoded string
- Deserialize from base62 string to ksuid struct
- Benchmark tests
- Unit tests

# Planned Features
- Better perf
- CLI demonstrating basic usage and offering ability to "inspect" ksuids from serialized strings
