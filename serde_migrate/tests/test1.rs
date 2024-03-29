use serde_migrate::versioned;

#[versioned]
#[derive(PartialEq, Debug)]
struct MyStruct {
    #[version(end = 2)]
    old_field: u32,
    #[version(start = 2)]
    new_field: u32,
}

impl mystruct_migrations::Migrate for MyStruct {
    fn to_v2(v: mystruct_migrations::MyStructV1) -> mystruct_migrations::MyStructV2 {
       mystruct_migrations::MyStructV2 {
          new_field: v.old_field,
       }
    }
}

#[test]
fn noop() {
    
}