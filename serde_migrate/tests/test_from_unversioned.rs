use serde_migrate::{versioned, Versioned};
use serde::Serialize;

#[derive(PartialEq, Debug, Serialize)]
struct Aunversioned {
    pub a: u32,
}

#[versioned]
#[derive(PartialEq, Debug)]
struct A {
    pub a: u32,
    #[version(start = 2)]
    pub b: u32,
}

impl a_migrations::Migrate for A {
    fn to_v2(v: a_migrations::AV1) -> a_migrations::AV2 {
        a_migrations::AV2 {
            a: v.a,
            b: v.a * 2,
        }
    }
}

#[test]
fn test_from_unversioned() {
    let orig = Aunversioned {
        a: 123,
    };
    let json = serde_json::to_string_pretty(&Versioned(&orig)).unwrap();

    println!("Decoding json: {}", json);
    let decoded = serde_json::from_str::<Versioned<A>>(&json).unwrap().0;
    assert_eq!(decoded, A {
        a: 123,
        b: 246,
    });

    let bc = bincode::serialize(&Versioned(&orig)).unwrap();
    let decoded = bincode::deserialize::<Versioned<A>>(&bc).unwrap().0;
    assert_eq!(decoded, A {
        a: 123,
        b: 246,
    });
}
