use serde_migrate::{versioned, Versioned};

#[versioned]
#[derive(PartialEq, Debug)]
struct MyStruct {
    v: Deep1,
}

#[versioned]
#[derive(PartialEq, Debug)]
struct Deep1 {
    v: Deep2,
}

#[versioned]
#[derive(PartialEq, Debug)]
struct Deep2 {
    v: Deep3,
}

#[versioned]
#[derive(PartialEq, Debug)]
struct Deep3 {
    #[version(end = 2)]
    older_field: u32,
    #[version(end = 3)]
    old_field: u32,
    #[version(start = 3)]
    new_field: u32,
}

impl deep3_migrations::Migrate for Deep3 {
    fn to_v2(v: deep3_migrations::Deep3V1) -> deep3_migrations::Deep3V2 {
        deep3_migrations::Deep3V2 {
           old_field: v.old_field,
        }
     }

    fn to_v3(v: deep3_migrations::Deep3V2) -> deep3_migrations::Deep3V3 {
       deep3_migrations::Deep3V3 {
          new_field: v.old_field,
       }
    }
}

#[test]
fn test_deep() {
    let orig = MyStruct {
        v: Deep1 {
            v: Deep2 {
                v: Deep3 {
                    new_field: 999,
                }
            }
        }
    };
    let json = serde_json::to_string_pretty(&Versioned(&orig)).unwrap();
    println!("{}", json);
    let decoded = serde_json::from_str::<Versioned<_>>(&json).unwrap().0;
    assert_eq!(orig, decoded);

    let decoded = serde_json::from_str::<Versioned<_>>(r#"{ "versions": { "test_deep::MyStruct": 1, "test_deep::Deep1": 1, "test_deep::Deep2": 1, "test_deep::Deep3": 2 }, "value": { "v": { "v": { "v": { "old_field": 999, "older_field": 555 }}}}}"#).unwrap().0;
    assert_eq!(orig, decoded);
}