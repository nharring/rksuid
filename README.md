# rksuid
 Rust implementation of Segment.io's ksuid

# Inspiration
Segment [published](https://segment.com/blog/a-brief-history-of-the-uuid/) a nifty UUID format and I thought it'd be fun to implement a compatible version in pure Rust as a way to
learn the language more thoroughly and keep myself sharp while looking for a coding gig.

# Current Features
- Creation of new ksuids via factory function with ability to specify timestamp and/or payload.
- Serialize to base62 encoded string
- Deserialize from base62 string to ksuid struct
- Benchmark tests
- Unit tests

# Planned Features
- Better perf
- CLI demonstrating basic usage and offering ability to "inspect" ksuids from serialized strings
