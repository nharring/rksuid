extern crate criterion;
extern crate rksuid;
extern crate strum;
extern crate strum_macros;

use criterion::black_box;
use criterion::*;
use ksuid::Ksuid;
use rand::prelude::thread_rng;
use rand::seq::SliceRandom;
use std::convert::TryInto;
use std::mem;

use rksuid::rksuid::{deserialize, new, Ksuid as OtherKsuid};

pub fn bench_new_ksuid_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("new");
    group.throughput(Throughput::Elements(1));
    group.bench_function("new", |b| b.iter(|| new(None, None)));
    group.bench_function("new-with-timestamp", |b| {
        b.iter(|| new(Some(168582232), None))
    });
    group.bench_function("new-with-payload", |b| {
        b.iter(|| new(None, Some(123456789)))
    });
    group.bench_function("new-with-timestamp-and-payload", |b| {
        b.iter(|| new(Some(black_box(168582232)), Some(black_box(123456789))))
    });
    group.finish();
}

pub fn bench_old_ksuid_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("old-new");
    group.throughput(Throughput::Elements(1));
    group.bench_function("old-new", |b| b.iter(|| ksuid::Ksuid::generate()));
    group.bench_function("old-new-with-payload", |b| {
        b.iter(|| ksuid::Ksuid::with_payload([0; 16]))
    });
    group.bench_function("old-new-with-timestamp-and-payload", |b| {
        b.iter(|| ksuid::Ksuid::new(black_box(168582232), black_box([0; 16])))
    });
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

pub fn bench_old_serialize(c: &mut Criterion) {
    let ksuid = ksuid::Ksuid::generate();
    let mut group = c.benchmark_group("old-serialize");
    let example = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
    group.throughput(Throughput::Bytes(example.len().try_into().unwrap()));
    group.bench_function("Old-Serialize", |b| b.iter(|| ksuid.to_base62()));
    group.finish();
}
pub fn bench_deserialize(c: &mut Criterion) {
    let serialized = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
    let mut group = c.benchmark_group("deserialize");
    group.throughput(Throughput::Bytes(
        mem::size_of::<Ksuid>().try_into().unwrap(),
    ));
    group.bench_function("deserialize ", |b| b.iter(|| deserialize(&serialized)));
    group.finish();
}
pub fn bench_old_deserialize(c: &mut Criterion) {
    let serialized = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
    let mut group = c.benchmark_group("old-deserialize");
    group.throughput(Throughput::Bytes(
        mem::size_of::<Ksuid>().try_into().unwrap(),
    ));
    group.bench_function("old-deserialize ", |b| {
        b.iter(|| ksuid::Ksuid::from_base62(&serialized))
    });
    group.finish();
}

fn build_ksuid_vec(n: i32) -> Vec<OtherKsuid> {
    let mut ksuids: Vec<OtherKsuid> = Vec::new();
    for i in 0..n {
        ksuids.push(new(Some(i as u32), None));
    }
    return ksuids;
}

fn build_old_ksuid_vec(n: i32) -> Vec<Ksuid> {
    let mut ksuids: Vec<Ksuid> = Vec::new();
    for _ in 0..n {
        ksuids.push(ksuid::Ksuid::generate());
    }
    return ksuids;
}

pub fn bench_sort_unstable(c: &mut Criterion) {
    let element_count = vec![5, 10, 100, 1000, 5000, 10000, 50000, 100000];
    let mut group = c.benchmark_group("sort");
    for n in element_count {
        let mut ksuids = build_ksuid_vec(n);
        group.throughput(Throughput::Elements(ksuids.len().try_into().unwrap()));
        ksuids.shuffle(&mut thread_rng());
        group.bench_with_input(BenchmarkId::new("Unstable", n), &n, |b, _n| {
            b.iter(|| ksuids.sort_unstable())
        });
        ksuids.shuffle(&mut thread_rng());
        group.bench_with_input(BenchmarkId::new("Sort", n), &n, |b, _n| {
            b.iter(|| ksuids.sort())
        });
        let mut old_ksuids = build_old_ksuid_vec(n);
        old_ksuids.shuffle(&mut thread_rng());
        group.bench_with_input(BenchmarkId::new("Old-Unstable", n), &n, |b, _n| {
            b.iter(|| old_ksuids.sort_unstable())
        });
        old_ksuids.shuffle(&mut thread_rng());
        group.bench_with_input(BenchmarkId::new("Old-Sort", n), &n, |b, _n| {
            b.iter(|| old_ksuids.sort())
        });
    }
    group.finish();
}

criterion_group!(creation, bench_new_ksuid_creation);
criterion_group!(old_creation, bench_old_ksuid_creation);
criterion_group!(
    serde,
    bench_serialize,
    bench_deserialize,
    bench_old_deserialize,
    bench_old_serialize
);
criterion_group!(sorting, bench_sort_unstable);
criterion_main!(creation, serde, sorting, old_creation);
