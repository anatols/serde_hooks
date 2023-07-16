use serde::{Serialize, Serializer};

// struct we can't change
#[derive(Serialize)]
struct Employee {
    name: String,
    department_id: i32,
    //... 150 more fields
}

#[derive(Serialize)]
struct Compensation {
    salary: f64,
    has_stock_options: bool,
}

#[derive(Serialize)]
struct EmployeePayload {
    #[serde(flatten)]
    employee: Employee,
    // need to add this one field
    department_name: String,
    char_field: char,
    compensation: Compensation,
}

#[test]
fn test_payload() {
    use crate::ser;

    let payload = EmployeePayload {
        employee: Employee {
            name: "John Doe".into(),
            department_id: 10,
        },
        department_name: "Sales".into(),
        char_field: 'c',
        compensation: Compensation {
            salary: 1_000_000.99,
            has_stock_options: true,
        },
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
            map.rename_key("department_name", "renamed_department_name");
            map.rename_key("renamed_department_name", "renamed_again_department_name");
        }

        fn on_struct(&self, st: &mut ser::StructScope) {
            println!(
                "==== STRUCT {} at {}, len={:?}",
                st.struct_name(),
                st.path().to_string(),
                st.struct_len()
            );

            st.rename_field("has_stock_options", "is_the_boss".to_string());
        }

        fn on_map_key<S: Serializer>(&self, map_key: &mut ser::ValueScope<S>) {
            println!(
                "==== MAP KEY at {}, {:?}",
                map_key.path().to_string(),
                map_key.value()
            );
            // map_key.replace("IT");
        }

        fn on_value<S: Serializer>(&self, value: &mut ser::ValueScope<S>) {
            println!(
                "==== VALUE at {}, {:?}",
                value.path().to_string(),
                value.value()
            );
            // value.replace("IT");
        }
    }

    println!("{}", serde_json::to_string_pretty(&payload).unwrap());

    println!(
        "{}",
        serde_json::to_string_pretty(&ser::hook(
            &payload,
            &Hks {
                data: "BLAH".into()
            }
        ))
        .unwrap()
    );
}
