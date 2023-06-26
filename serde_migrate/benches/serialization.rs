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

fn serialize_bincode<T: Serialize>(v: T) -> Vec<u8> {
    bincode::serialize(&v).unwrap()
}

fn deserialize_bincode<T: DeserializeOwned>(data: &[u8]) -> T {
    bincode::deserialize::<T>(data).unwrap()
}

fn serialize_postcard<T: Serialize>(v: T) -> Vec<u8> {
    postcard::to_stdvec(&v).unwrap()
}

fn deserialize_postcard<T: DeserializeOwned>(data: &[u8]) -> T {
    postcard::from_bytes::<T>(data).unwrap()
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

    let s_versioned_data = serialize_bincode(Versioned(&versioned_data));
    let s_data = serialize_bincode(&versioned_data);
    assert_eq!(versioned_data, deserialize_bincode::<Versioned<Av>>(black_box(&s_versioned_data)).0);

    let p_versioned_data = serialize_postcard(Versioned(&versioned_data));
    let p_data = serialize_postcard(&versioned_data);
    assert_eq!(versioned_data, deserialize_postcard::<Versioned<Av>>(black_box(&p_versioned_data)).0);

    c.bench_function("serialize version info (empty)", |b| b.iter(|| {
        let mut s = VersionSerializer::default();
        black_box(&data).serialize(&mut s).unwrap();
    }));
    c.bench_function("serialize version info", |b| b.iter(|| {
        let mut s = VersionSerializer::default();
        black_box(&versioned_data).serialize(&mut s).unwrap();
    }));


    {
        let mut g = c.benchmark_group("serialization (bincode)");
        g.bench_function("versioned", |b| b.iter(|| serialize_bincode(black_box(Versioned(&versioned_data)))));
        g.bench_function("unversioned", |b| b.iter(|| serialize_bincode(black_box(&versioned_data))));
        g.bench_function("baseline", |b| b.iter(|| serialize_bincode(black_box(&data))));
    }

    {
        let mut g = c.benchmark_group("serialization (postcard)");
        g.bench_function("versioned", |b| b.iter(|| serialize_postcard(black_box(Versioned(&versioned_data)))));
        g.bench_function("unversioned", |b| b.iter(|| serialize_postcard(black_box(&versioned_data))));
        g.bench_function("baseline", |b| b.iter(|| serialize_postcard(black_box(&data))));
    }

    {
        let mut g = c.benchmark_group("deserialization (bincode)");
        g.bench_function("versioned", |b| b.iter(|| deserialize_bincode::<Versioned<Av>>(black_box(&s_versioned_data))));
        g.bench_function("unversioned", |b| b.iter(|| deserialize_bincode::<Av>(black_box(&s_data))));
        g.bench_function("baseline", |b| b.iter(|| deserialize_bincode::<A>(black_box(&s_data))));
    }

    {
        let mut g = c.benchmark_group("deserialization (postcard)");
        g.bench_function("versioned", |b| b.iter(|| deserialize_postcard::<Versioned<Av>>(black_box(&p_versioned_data))));
        g.bench_function("unversioned", |b| b.iter(|| deserialize_postcard::<Av>(black_box(&p_data))));
        g.bench_function("baseline", |b| b.iter(|| deserialize_postcard::<A>(black_box(&p_data))));
    }
}

criterion_group!(serialization, criterion_benchmark);
criterion_main!(serialization);