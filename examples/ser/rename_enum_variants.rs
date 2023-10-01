use serde::Serialize;
use serde_hooks::ser;

#[allow(dead_code)]
fn rename_enum_variants() {
    #[derive(Serialize)]
    #[serde(rename_all = "snake_case")]
    enum Possibilities {
        ABird,
        APlane,
        Superman,
    }

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_enum_variant(&self, _path: &serde_hooks::Path, ev: &mut ser::EnumVariantScope) {
            ev.rename_variant_case("SCREAMING-KEBAB-CASE"); // rename all variants by default

            if ev.variant_name() == "superman" {
                ev.rename_variant("SUPERMAN!!!"); // special treatment for the hero
            }
        }
    }

    // without hooks: #[serde(rename_all = "snake_case")] in effect
    assert_eq!(
        serde_json::to_string(&Possibilities::ABird).unwrap(),
        r#""a_bird""#
    );

    assert_eq!(
        serde_json::to_string(&Possibilities::Superman).unwrap(),
        r#""superman""#
    );

    // with hooks
    assert_eq!(
        serde_json::to_string(&ser::hook(&Possibilities::ABird, &Hooks)).unwrap(),
        r#""A-BIRD""#
    );

    assert_eq!(
        serde_json::to_string(&ser::hook(&Possibilities::Superman, &Hooks)).unwrap(),
        r#""SUPERMAN!!!""#
    );
}

#[test]
fn test_rename_enum_variants() {
    rename_enum_variants();
}
