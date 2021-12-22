use x_lang_ast::shared::{Kind, KindName};

pub fn to_kind_str(kind: &Kind) -> Option<&str> {
    match kind {
        Kind::Some(kind_name) => match kind_name {
            KindName::Number => Some("number"),
            KindName::Boolean => Some("boolean"),
            KindName::String => Some("string"),
            KindName::Void => Some("void"),
        },
        Kind::Infer => Some("infer"),
        Kind::None => None,
    }
}
