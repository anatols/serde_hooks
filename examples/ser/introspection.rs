use std::cell::RefCell;

use serde::Serialize;
use serde_hooks::ser;

#[allow(dead_code)]
fn introspection() {
    #[derive(Serialize)]
    struct Wheels {
        left_front: bool,
        left_rear: bool,
        right_front: bool,
        right_rear: bool,
    }

    #[derive(Serialize)]
    struct Car {
        model: String,
        wheels: Wheels,
    }

    #[derive(Serialize)]
    struct Human {
        name: String,
        siblings: Vec<String>,
    }

    #[derive(Default)]
    struct PathIntrospection {
        paths: RefCell<Vec<String>>,
    }

    impl ser::Hooks for PathIntrospection {
        fn on_start(&self, _start: &mut ser::StartScope) {
            self.paths.borrow_mut().clear();
        }

        fn on_value<S: serde::Serializer>(
            &self,
            path: &serde_hooks::Path,
            _value: &mut ser::ValueScope<S>,
        ) {
            if !path.is_root() {
                self.paths.borrow_mut().push(path.to_string())
            }
        }
    }

    let path_introspection = PathIntrospection::default();

    // Serializing one type with PathIntrospection hooks
    ser::invoke_hooks(
        &Car {
            model: "Cybervan".into(),
            wheels: Wheels {
                left_front: true,
                left_rear: false,
                right_front: true,
                right_rear: false,
            },
        },
        &path_introspection,
    )
    .unwrap();
    let paths = path_introspection.paths.borrow().clone();
    assert_eq!(
        paths,
        vec![
            "model",
            "wheels",
            "wheels.left_front",
            "wheels.left_rear",
            "wheels.right_front",
            "wheels.right_rear"
        ]
    );

    // Reusing the same PathIntrospection instance with a different unrelated type
    ser::invoke_hooks(
        &Human {
            name: "Mary".into(),
            siblings: vec!["April".into(), "June".into(), "July".into()],
        },
        &path_introspection,
    )
    .unwrap();
    let paths = path_introspection.paths.borrow().clone();
    assert_eq!(
        paths,
        vec![
            "name",
            "siblings",
            "siblings[0]",
            "siblings[1]",
            "siblings[2]"
        ]
    );
}

#[test]
fn test_introspection() {
    introspection();
}
