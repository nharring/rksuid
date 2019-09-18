#[macro_use]
extern crate criterion;
extern crate rksuid;

use criterion::*;

use rand::prelude::thread_rng;
use rand::seq::SliceRandom;
use std::mem;
use std::convert::TryInto;

use rksuid::rksuid::{deserialize, Ksuid};

pub fn bench_new_ksuid_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("new");
    group.throughput(Throughput::Elements(1));
    group.bench_function("new", |b| b.iter(|| rksuid::rksuid::new(None, None)));
    group.bench_function("new-with-timestamp", |b| b.iter(|| rksuid::rksuid::new(Some(168582232), None)));
    group.bench_function("new-with-payload", |b| b.iter(|| rksuid::rksuid::new(None, Some(123456789))));
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
        ksuids.push(rksuid::rksuid::new(Some(i as u32), None));
    }
    ksuids.shuffle(&mut thread_rng());
    return ksuids;
}

pub fn bench_sort(c: &mut Criterion) {
    let element_count = vec![5,10,100,1000,5000,10000,50000,100000];
    let mut group = c.benchmark_group("sort");
    for n in element_count {
        let mut ksuids = build_ksuid_vec(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(BenchmarkId::from_parameter(n), |b| b.iter(|| ksuids.sort()));
    }
    group.finish();
}
pub fn bench_sort_unstable(c: &mut Criterion) {
    let element_count = vec![5,10,100,1000,5000,10000,50000,100000];
    let mut group = c.benchmark_group("sort_unstable");
    for n in element_count{
        let mut ksuids = build_ksuid_vec(n);
        group.throughput(Throughput::Elements(ksuids.len().try_into().unwrap()));
        group.bench_function(BenchmarkId::from_parameter(n), |b| b.iter(|| ksuids.sort_unstable()));
    }
    group.finish();
}

criterion_group!(benches, bench_new_ksuid_creation, bench_serialize, bench_deserialize, bench_sort, bench_sort_unstable);
criterion_main!(benches);
