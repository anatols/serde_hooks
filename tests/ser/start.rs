use std::cell::Cell;

use serde_hooks::ser;

#[test]
fn test_is_called() {
    struct Hooks {
        is_called: Cell<bool>,
    }
    impl ser::Hooks for Hooks {
        fn on_start(&self, _start: &mut ser::StartScope) {
            self.is_called.set(true);
        }
    }
    let hooks = Hooks {
        is_called: Cell::new(false),
    };

    serde_json::to_string(&ser::hook(&(), &hooks)).unwrap();
    assert!(hooks.is_called.get());
}

#[test]
fn test_is_human_readable() {
    struct Hooks {
        expect_human_readable: bool,
    }
    impl ser::Hooks for Hooks {
        fn on_start(&self, start: &mut ser::StartScope) {
            assert_eq!(start.is_format_human_readable(), self.expect_human_readable);
        }
    }

    serde_json::to_string(&ser::hook(
        &(),
        &Hooks {
            expect_human_readable: true,
        },
    ))
    .unwrap();

    bincode::serialize(&ser::hook(
        &(),
        &Hooks {
            expect_human_readable: false,
        },
    ))
    .unwrap();
}
