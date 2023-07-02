use serde::Serialize;

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
    };

    struct Hks {
        data: String,
    }
    impl ser::Hooks for Hks {
        // fn handle_something(&mut self) {
        //     println!("handle_something {}", self.data);
        //     self.data.push('A');
        // }
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
