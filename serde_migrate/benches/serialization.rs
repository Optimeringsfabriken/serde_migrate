use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_migrate::{Versioned, VersionSerializer};
use serde_migrate_macros::versioned;

#[versioned]
#[derive(Clone, PartialEq, Debug)]
struct Bv {
    pub v: u32,
    pub c: String,
    pub v2: u64,
}

#[versioned]
#[derive(PartialEq, Debug)]
struct Av {
    pub a: u32,
    pub b: Vec<Bv>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct B {
    pub v: u32,
    pub c: String,
    pub v2: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct A {
    pub a: u32,
    pub b: Vec<B>,
}

fn serialize<T: Serialize>(v: T) -> Vec<u8> {
    bincode::serialize(&v).unwrap()
}

fn deserialize<T: DeserializeOwned>(data: &[u8]) -> T {
    bincode::deserialize::<T>(data).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let n = 20;
    let versioned_data = Av {
        a: n as u32,
        b: vec![Bv { v: n as u32, c: "Hello World".to_string(), v2: 43 }; 100],
    };
    let data = A {
        a: n as u32,
        b: vec![B { v: n as u32, c: "Hello World".to_string(), v2: 43 }; 100],
    };

    let s_versioned_data = serialize(Versioned(&versioned_data));
    let s_data = serialize(&versioned_data);
    assert_eq!(versioned_data, deserialize::<Versioned<Av>>(black_box(&s_versioned_data)).0);

    c.bench_function("serialize version info (empty)", |b| b.iter(|| {
        let mut s = VersionSerializer::default();
        black_box(&data).serialize(&mut s).unwrap();
    }));
    c.bench_function("serialize version info", |b| b.iter(|| {
        let mut s = VersionSerializer::default();
        black_box(&versioned_data).serialize(&mut s).unwrap();
    }));
    c.bench_function("serialize versioned", |b| b.iter(|| serialize(black_box(Versioned(&versioned_data)))));
    c.bench_function("serialize unversioned", |b| b.iter(|| serialize(black_box(&versioned_data))));
    c.bench_function("serialize baseline", |b| b.iter(|| serialize(black_box(&data))));

    c.bench_function("deserialize versioned", |b| b.iter(|| deserialize::<Versioned<Av>>(black_box(&s_versioned_data))));
    c.bench_function("deserialize unversioned", |b| b.iter(|| deserialize::<Av>(black_box(&s_data))));
    c.bench_function("deserialize baseline", |b| b.iter(|| deserialize::<A>(black_box(&s_data))));
}

criterion_group!(serialization, criterion_benchmark);
criterion_main!(serialization);