use serde_migrate::{versioned, Versioned};

#[versioned]
#[derive(PartialEq, Debug)]
struct A {
    pub a: u32,
}

#[versioned]
#[derive(PartialEq, Debug)]
struct A2 {
    pub a: u32,
    #[version(start = 2, end = 3)]
    pub b: String,
    #[version(start = 3)]
    pub c: u32,
}

impl a2_migrations::Migrate for A2 {
    fn to_v2(v:a2_migrations::A2V1) -> a2_migrations::A2V2 {
        a2_migrations::A2V2 {
            a: v.a,
            b: "321".to_string(),
        }
    }

    fn to_v3(v:a2_migrations::A2V2) -> a2_migrations::A2V3 {
        a2_migrations::A2V3 {
            a: v.a,
            c: v.b.parse().unwrap(),
        }
    }
}

#[test]
fn test_migration() {
    let orig = A {
        a: 123,
    };
    let json = serde_json::to_string_pretty(&Versioned(&orig)).unwrap();
    let decoded = serde_json::from_str::<Versioned<_>>(&json).unwrap().0;
    assert_eq!(orig, decoded);
    println!("{}", json);

    let decoded2: A2 = serde_json::from_str::<Versioned<_>>(&json).unwrap().0;
    assert_eq!(decoded2, A2 {
        a: orig.a,
        c: 321
    });
}
