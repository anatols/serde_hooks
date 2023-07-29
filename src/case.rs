//TODO document
#[derive(Copy, Clone)]
pub enum Case {
    Lower,
    Upper,
    Pascal,
    Camel,
    Snake,
    ScreamingSnake,
    Kebab,
    ScreamingKebab,
}

impl From<&str> for Case {
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
