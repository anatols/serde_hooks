/// Case convention for case-renaming actions.
///
/// During serialization, if you use serde's `#[derive(Serialize)]` and `#[serde(rename=...)]` or
/// `#[serde(rename_all=...)]`, those will be applied at compile time. This means any case
/// conversions you perform at runtime will need to operate on the results of serde's renames.
///
/// This can lead to unsolvable corner cases. For example, imagine you've got
/// an enum renamed like this:
/// ```
/// # use serde::Serialize;
/// #[derive(Serialize)]
/// #[serde(rename_all = "UPPERCASE")]
/// enum Reason {
///     JustInCase,
/// }
/// ```
/// At runtime, the `JustInCase` variant will be serialized as `"JUSTINCASE"`. Trying to change
/// its case convention to snake case will yield `"justincase"` instead of maybe expected
/// `"just_in_case"`. This happens because at runtime there is no way to figure out word
/// boundaries after serde has transformed everything to uppercase.
#[derive(Copy, Clone)]
pub enum Case {
    /// `lowercase`
    Lower,
    /// `UPPERCASE`
    Upper,
    /// `PascalCase`
    Pascal,
    /// `camelCase`
    Camel,
    /// `snake_case`
    Snake,
    /// `SCREAMING_SNAKE_CASE`
    ScreamingSnake,
    /// `kebab-case`
    Kebab,
    /// `SCREAMING-KEBAB-CASE`
    ScreamingKebab,
}

impl From<&str> for Case {
    /// Convert from a string literal to [`Case`].
    ///
    /// This function accepts the same case convention identifiers, as `#[serde rename_all=...]`:
    /// `"lowercase"`,
    /// `"UPPERCASE"`,
    /// `"PascalCase"`,
    /// `"camelCase"`,
    /// `"snake_case"`,
    /// `"SCREAMING_SNAKE_CASE"`,
    /// `"kebab-case"`,
    /// `"SCREAMING-KEBAB-CASE"`.
    ///
    /// Panics on unknown identifiers.
    fn from(value: &str) -> Self {
        match value {
            "lowercase" => Self::Lower,
            "UPPERCASE" => Self::Upper,
            "PascalCase" => Self::Pascal,
            "camelCase" => Self::Camel,
            "snake_case" => Self::Snake,
            "SCREAMING_SNAKE_CASE" => Self::ScreamingSnake,
            "kebab-case" => Self::Kebab,
            "SCREAMING-KEBAB-CASE" => Self::ScreamingKebab,
            _ => panic!("unsupported case convention '{value}'"),
        }
    }
}

impl Case {
    pub(crate) fn string_to_case(key: &str, to_case: Case) -> String {
        use convert_case::Casing;
        match to_case {
            Case::Lower => key.to_case(convert_case::Case::Lower),
            Case::Upper => key.to_case(convert_case::Case::Upper),
            Case::Pascal => key.to_case(convert_case::Case::Pascal),
            Case::Camel => key.to_case(convert_case::Case::Camel),
            Case::Snake => key.to_case(convert_case::Case::Snake),
            Case::ScreamingSnake => key.to_case(convert_case::Case::ScreamingSnake),
            Case::Kebab => key.to_case(convert_case::Case::Kebab),
            Case::ScreamingKebab => key.to_case(convert_case::Case::UpperKebab),
        }
    }
}
