use serde_migrate::{versioned};

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
    fn to_v2(v: a_migrations::Av1) -> a_migrations::Av2 {
        a_migrations::Av2 {
            a: v.a + v.b,
        }
    }
}

impl awrap_migrations::Migrate for Awrap {
    fn to_v2(v:awrap_migrations::Awrapv1) -> awrap_migrations::Awrapv2 {
        awrap_migrations::Awrapv2 {
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
    let json = serde_json::to_string_pretty(&orig).unwrap();
    let decoded = serde_json::from_str(&json).unwrap();
    assert_eq!(orig, decoded);
    println!("{}", json);

    let decoded2 = serde_json::from_str(r#"{ "version": 1, "value": { "a": 121, "b": 2 } }"#).unwrap();

    assert_eq!(orig, decoded2);


    let bc = bincode::serialize(&orig).unwrap();
    println!("{:?}", bc);
    let decoded = bincode::deserialize(&bc).unwrap();

    assert_eq!(orig, decoded);

    let orig = Awrap {
        a: vec![A { a: 123 }, A { a: 456 }],
        c: 789,
    };
    let json = serde_json::to_string_pretty(&orig).unwrap();
    let decoded = serde_json::from_str(&json).unwrap();
    println!("{}", json);
    assert_eq!(orig, decoded);

    let bc = bincode::serialize(&orig).unwrap();
    println!("{:?}", bc);
    let decoded = bincode::deserialize(&bc).unwrap();
    assert_eq!(orig, decoded);
}
