use serde::{Serialize, Serializer};

// struct we can't change
#[derive(Serialize)]
struct Employee {
    name: String,
    department_id: i32,
    //... 150 more fields
}

#[derive(Serialize)]
struct EmployeePayload {
    #[serde(flatten)]
    employee: Employee,
    // need to add this one field
    department_name: String,
    char_field: char,
}

#[test]
fn test_payload() {
    use crate::ser;

    let employee = Employee {
        name: "John Doe".into(),
        department_id: 10,
    };

    let payload = EmployeePayload {
        employee,
        department_name: "Sales".into(),
        char_field: 'c',
    };

    struct Hks {
        data: String,
    }
    impl ser::Hooks for Hks {
        fn start(&self) {
            println!("==== START")
        }

        fn end(&self) {
            println!("==== END")
        }

        fn on_map(&self, map: &mut ser::MapScope) {
            println!(
                "==== MAP at {}, len={:?}",
                map.path().to_string(),
                map.map_len()
            );
            // map.skip_entry("department_id");
            // map.retain_entry("department_id")
            //     .insert_entry("department_name", "foo");
        }

        fn on_value<S: Serializer>(&self, value: &mut ser::ValueScope<S>) {
            println!(
                "==== VALUE at {}, {:?}",
                value.path().to_string(),
                value.value()
            );
            value.replace("IT");
        }
    }

    println!("{}", serde_json::to_string(&payload).unwrap());

    println!(
        "{}",
        serde_json::to_string(&ser::hook(
            &payload,
            Hks {
                data: "BLAH".into()
            }
        ))
        .unwrap()
    );
}
