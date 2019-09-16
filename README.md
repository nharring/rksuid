# rksuid
 Rust implementation of Segment.io's ksuid

 I have no association with Segment, just a fan of the format.

# Inspiration
Segment [published](https://segment.com/blog/a-brief-history-of-the-uuid/) a nifty UUID format and I thought it'd be fun to implement a compatible version in pure Rust as a way to learn the language more thoroughly and keep myself sharp while looking for a new coding gig. 

# Usage
Cargo.toml:
```
rksuid = "0.1.0"
```

```rust
use rksuid;

// Generate new Ksuid with current timestamp and random payload
let ksuid = rksuid::new(None, None);
// Serialize to a base-62 encoded string
let serialized = ksuid.serialize();

// Deserialize a base-62 encoded string into a Ksuid
let ksuid = rksuid::deserialize("1QtFxXJfPVU6NOwPOsHsaihkm8U");
println!("{:?}", ksuid);
```
```rust
Ksuid { timestamp: 168582232, payload: 312509952699879867963141934813379438280 }
```



# Current Features
- Ability to specify timestamp and/or payload when creating new Ksuids.
- Serialize to base62 encoded string and deserialize back to identical struct
- No unsafe code

# Planned Features
- Pretty printer
- CLI demonstrating basic usage and offering ability to "inspect" ksuids from serialized strings

# Benchmark results
From my desktop with i9-9900k @ 3.6Ghz w/ 32G, sorts are on a 500 element vector:
```
test tests::bench_new_ksuid_creation        ... bench:         233 ns/iter (+/- 8)
test tests::bench_new_ksuid_fixed_payload   ... bench:          35 ns/iter (+/- 0)
test tests::bench_new_ksuid_fixed_timestamp ... bench:         196 ns/iter (+/- 12)
test tests::bench_serialize                 ... bench:       1,438 ns/iter (+/- 65)
test tests::bench_deserialize               ... bench:       1,360 ns/iter (+/- 2)
test tests::bench_sort                      ... bench:         400 ns/iter (+/- 8)
test tests::bench_sort_unstable             ... bench:         330 ns/iter (+/- 9)
```
