extern crate criterion;
extern crate rksuid;
extern crate strum;
extern crate strum_macros;

use criterion::*;

use rand::prelude::thread_rng;
use rand::seq::SliceRandom;
use std::mem;
use std::convert::TryInto;

use rksuid::rksuid::{deserialize, Ksuid, RngType, new, gen_payload};
use strum::IntoEnumIterator;


pub fn bench_new_ksuid_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("new");
    group.throughput(Throughput::Elements(1));
    group.bench_function("new", |b| b.iter(|| new(None, None)));
    group.bench_function("new-with-timestamp", |b| b.iter(|| new(Some(168582232), None)));
    group.bench_function("new-with-payload", |b| b.iter(|| new(None, Some(123456789))));
    group.finish();
}
pub fn bench_serialize(c: &mut Criterion) {
    let ksuid = rksuid::rksuid::new(None, None);
    let mut group = c.benchmark_group("serialize");
    let example = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
    group.throughput(Throughput::Bytes(example.len().try_into().unwrap()));
    group.bench_function("Serialize", |b| b.iter(|| ksuid.serialize()));
    group.finish();
}
pub fn bench_deserialize(c: &mut Criterion) {
    let serialized = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
    let mut group = c.benchmark_group("deserialize");
    group.throughput(Throughput::Bytes(mem::size_of::<Ksuid>().try_into().unwrap()));
    group.bench_function("deserialize ", |b| b.iter(|| deserialize(&serialized)));
    group.finish();
}

fn build_ksuid_vec(n: i32) -> Vec<Ksuid> {
    let mut ksuids: Vec<Ksuid> = Vec::new();
    for i in 0..n {
        ksuids.push(new(Some(i as u32), None));
    }
    return ksuids;
}

pub fn bench_sort_unstable(c: &mut Criterion) {
    let element_count = vec![5,10,100,1000,5000,10000,50000,100000];
    let mut group = c.benchmark_group("sort");
    for n in element_count{
        let mut ksuids = build_ksuid_vec(n);
        group.throughput(Throughput::Elements(ksuids.len().try_into().unwrap()));
        ksuids.shuffle(&mut thread_rng());
        group.bench_with_input(BenchmarkId::new("Unstable", n), &n, |b, _n| b.iter(|| ksuids.sort_unstable()));
        ksuids.shuffle(&mut thread_rng());
        group.bench_with_input(BenchmarkId::new("Sort", n), &n, |b, _n| b.iter(|| ksuids.sort()));
    }
    group.finish();
}

pub fn bench_payload(c: &mut Criterion) {
    let mut group = c.benchmark_group("gen_payloads");
    group.throughput(Throughput::Elements(1));
    for e in RngType::iter() {
        group.bench_with_input(BenchmarkId::new("Rng", e), &e, |b, e| b.iter(|| gen_payload(Some(*e))));
    }
    group.finish();
}

criterion_group!(creation, bench_new_ksuid_creation);
criterion_group!(serde, bench_serialize, bench_deserialize);
criterion_group!(sorting, bench_sort_unstable);
criterion_group!(payload, bench_payload);
criterion_main!(creation, serde, sorting, payload);
