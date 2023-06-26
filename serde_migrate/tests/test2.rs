use serde_migrate::{versioned, Versioned};

#[versioned]
#[derive(PartialEq, Debug)]
struct Awrap {
    pub a: Vec<A>,
    #[version(start = 2)]
    pub c: u32,
}

#[versioned]
#[derive(PartialEq, Debug)]
struct A {
    pub a: u32,
    #[version(end = 2)]
    pub b: u32,
}

impl a_migrations::Migrate for A {
    fn to_v2(v: a_migrations::AV1) -> a_migrations::AV2 {
        a_migrations::AV2 {
            a: v.a + v.b,
        }
    }
}

impl awrap_migrations::Migrate for Awrap {
    fn to_v2(v:awrap_migrations::AwrapV1) -> awrap_migrations::AwrapV2 {
        awrap_migrations::AwrapV2 {
            a: v.a,
            c: 0,
        }
    }
}

#[test]
fn test_roundtrip() {
    println!("Hello, world!");
    let orig = A {
        a: 123,
    };
    let json = serde_json::to_string_pretty(&Versioned(&orig)).unwrap();
    dbg!(&json);
    let decoded: Versioned<A> = serde_json::from_str(&json).unwrap();
    assert_eq!(orig, decoded.0);
    println!("{}", json);

    let decoded2 = serde_json::from_str::<Versioned<A>>(r#"{ "versions": { "test2::A": 1 }, "value": { "a": 121, "b": 2 } }"#).unwrap().0;

    assert_eq!(orig, decoded2);


    let bc = bincode::serialize(&Versioned(&orig)).unwrap();
    println!("{:?}", bc);
    let decoded = bincode::deserialize::<Versioned<_>>(&bc).unwrap().0;

    assert_eq!(orig, decoded);

    let orig = Awrap {
        a: vec![A { a: 123 }, A { a: 456 }],
        c: 789,
    };
    let json = serde_json::to_string_pretty(&orig).unwrap();
    println!("Decoding json: {}", json);
    let decoded = serde_json::from_str(&json).unwrap();
    println!("{}", json);
    assert_eq!(orig, decoded);

    let bc = bincode::serialize(&orig).unwrap();
    println!("{:?}", bc);
    let decoded = bincode::deserialize(&bc).unwrap();
    assert_eq!(orig, decoded);
}
