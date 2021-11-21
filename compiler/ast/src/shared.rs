// 关键字
const KEYWORDS: [&str; 17] = [
    "fn", "var", "return", "true", "false", "if", "else", "loop", "break", "continue",
    "pub", "import", "as", //
    // reserve
    "class", "this", "extends", "super",
];

fn array_index_of_str(arr: &[&str], value: &str) -> isize {
    for (i, v) in arr.iter().enumerate() {
        if *v == value {
            return i as isize;
        }
    }
    -1
}

// 判断是否为关键字
pub fn is_keyword_str(str: &str) -> bool {
    array_index_of_str(&KEYWORDS, str) >= 0
}

#[derive(Debug, PartialEq, Eq, Serialize, Copy, Clone)]
pub enum KindName {
    Number,
    Boolean,
    Void,
}

impl Into<Kind> for KindName {
    fn into(self) -> Kind {
        Kind::Some(self)
    }
}

impl KindName {
    // 通过字符串创建 KindName，无效类型将会抛错
    pub fn from(kind_str: &str, allow_void: bool) -> Option<Self> {
        match kind_str.as_bytes() {
            b"num" => Some(KindName::Number),
            b"bool" => Some(KindName::Boolean),
            b"void" => {
                if !allow_void {
                    None
                } else {
                    Some(KindName::Void)
                }
            }
            _ => None,
        }
    }

    // 返回类型名称字符串
    pub fn to_string(&self) -> String {
        match self {
            KindName::Number => "num".to_string(),
            KindName::Boolean => "bool".to_string(),
            KindName::Void => "void".to_string(),
        }
    }
}

#[derive(Debug, Eq, Serialize, Copy, Clone)]
pub enum Kind {
    Some(KindName),
    Infer, // 推断的类型
    None,  // 无类型 或者 void
}

impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        if self.is_exact() && other.is_exact() {
            self.read_kind_name() == other.read_kind_name()
        } else {
            false
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Kind {
    pub fn create(kind_str: &str) -> Self {
        KindName::from(kind_str, true).expect("Invalid kind string").into()
    }

    // 类型是否是精确的
    pub fn is_exact(&self) -> bool {
        if let Kind::Some(_) = self {
            true
        } else {
            false
        }
    }

    // 读取 KindName
    pub fn read_kind_name(&self) -> Option<&KindName> {
        if let Kind::Some(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn read_return_kind_name(&self) -> &KindName {
        match self {
            Kind::Some(v) => v,
            Kind::Infer => panic!("Return type can not be infer type"),
            Kind::None => &KindName::Void,
        }
    }

    // 返回类型字符串，非精确的类型返回 ""
    pub fn to_string(&self) -> String {
        if let Kind::Some(v) = self {
            v.to_string()
        } else {
            String::from("")
        }
    }
}
