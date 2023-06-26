use serde_migrate::{versioned, Versioned};

#[versioned]
#[derive(PartialEq, Debug)]
struct MyStruct {
    kept: u32,
    #[serde(skip)]
    skipped: Option<u32>,
}

#[test]
fn test_serde_attrs() {
    let orig = MyStruct {
        kept: 999,
        skipped: Some(555),
    };
    let json = serde_json::to_string_pretty(&Versioned(&orig)).unwrap();
    println!("{}", json);
    let decoded = serde_json::from_str::<Versioned<_>>(&json).unwrap().0;
    assert_eq!(MyStruct {
        kept: 999,
        skipped: None,
    }, decoded);
}