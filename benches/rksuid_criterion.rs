#[macro_use]
extern crate criterion;
extern crate rksuid;

//use criterion::black_box;
use criterion::Criterion;

use rand::prelude::thread_rng;
 use rand::seq::SliceRandom;

use rksuid::rksuid::{deserialize, Ksuid};

pub fn bench_new_ksuid_creation(c: &mut Criterion) {
    c.bench_function("New ksuid", |b| b.iter(|| rksuid::rksuid::new(None, None)));
}
pub fn bench_new_ksuid_fixed_timestamp(c: &mut Criterion) {
    c.bench_function("New ksuid fixed timestamp", |b| b.iter(|| rksuid::rksuid::new(Some(168582232), None)));
}
pub fn bench_new_ksuid_fixed_payload(c: &mut Criterion) {
    c.bench_function("New ksuid fixed everything", |b| b.iter(|| rksuid::rksuid::new(None, Some(123456789))));
}
pub fn bench_serialize(c: &mut Criterion) {
    let ksuid = rksuid::rksuid::new(None, None);
    c.bench_function("Serialize", |b| b.iter(|| ksuid.serialize()));
}
pub fn bench_deserialize(c: &mut Criterion) {
    let serialized = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
    c.bench_function("deserialize ", |b| b.iter(|| deserialize(&serialized)));
}

fn build_ksuid_vec(n: i32) -> Vec<Ksuid> {
    let mut ksuids: Vec<Ksuid> = Vec::new();
    for i in 0..n {
        ksuids.push(rksuid::rksuid::new(Some(i as u32), None));
    }
    ksuids.shuffle(&mut thread_rng());
    return ksuids;
}

pub fn bench_sort(c: &mut Criterion) {
    let mut ksuids = build_ksuid_vec(500);
    c.bench_function("sort 500", |b| b.iter(|| ksuids.sort()));
}
pub fn bench_sort_unstable(c: &mut Criterion) {
    let mut ksuids = build_ksuid_vec(500);
    c.bench_function("sort_unstable 500", |b| b.iter(|| ksuids.sort_unstable()));
}

criterion_group!(benches, bench_new_ksuid_creation, bench_new_ksuid_fixed_timestamp, bench_new_ksuid_fixed_payload, bench_serialize, bench_deserialize, bench_sort, bench_sort_unstable);
criterion_main!(benches);
