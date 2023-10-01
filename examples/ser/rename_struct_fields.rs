use serde::Serialize;
use serde_hooks::ser;

#[allow(dead_code)]
fn rename_struct_fields() {
    #[derive(Serialize)]
    struct SpamSubscription {
        email: String,
        victim_name: String,
        channels: SubscriptionChannels,
    }

    #[derive(Serialize)]
    struct SubscriptionChannels {
        email: bool,
        sms: bool,
    }

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, path: &serde_hooks::Path, st: &mut serde_hooks::ser::StructScope) {
            st.rename_all_fields_case("camelCase"); // all struct fields to camelCase by default
            if path == "channels" {
                st.rename_field("sms", "SMS"); // special treatment of an abbreviation
            }
        }
    }

    let payload = SpamSubscription {
        email: "foo@example.com".into(),
        victim_name: "Joseph Average".into(),
        channels: SubscriptionChannels {
            email: true,
            sms: false,
        },
    };

    let without_hooks = serde_json::to_string(&payload).unwrap();
    assert_eq!(
        without_hooks,
        r#"{"email":"foo@example.com","victim_name":"Joseph Average","channels":{"email":true,"sms":false}}"#
    );

    let with_hooks = serde_json::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(
        with_hooks,
        r#"{"email":"foo@example.com","victimName":"Joseph Average","channels":{"email":true,"SMS":false}}"#
    );
}

#[test]
fn test_rename_struct_fields() {
    rename_struct_fields();
}
