use serde_migrate::versioned;

#[versioned]
#[derive(PartialEq, Debug)]
struct MyStruct<T> {
    #[version(end = 2)]
    old_field: T,
    #[version(start = 2)]
    new_field: T,
}

impl<T> mystruct_migrations::Migrate<T> for MyStruct<T> {
    fn to_v2(v: mystruct_migrations::MyStructV1<T>) -> mystruct_migrations::MyStructV2<T> {
       mystruct_migrations::MyStructV2 {
          new_field: v.old_field,
       }
    }
}

#[test]
fn noop() {
    
}