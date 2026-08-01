use std::collections::HashMap;
use std::fmt::Formatter;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct State {
    pub online: bool,
    pub attributes: HashMap<String, AttributeValue>
}
impl State {
    pub fn new() -> Self {
        Self {
            online: false,
            attributes: HashMap::new(),
        }
    }
    pub fn init(online: bool, attributes: HashMap<String, AttributeValue>) -> Self {
        Self { online, attributes }
    }
    pub fn with_online(online: bool) -> Self {
        Self {
            online,
            attributes: HashMap::new(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AttributeValue {
    #[default]
    Unit,
    Boolean(bool),
    Count(usize),
    Date(chrono::DateTime<chrono::Utc>),
    Percentage(f32),
    List(Vec<AttributeValue>),
    Number(i128),
    String(String),
    Enum {
        variant: String,
        value: Box<AttributeValue>,
    },
    Map(Vec<(AttributeValue, AttributeValue)>)
}
impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::Unit => write!(f, "()"),
            AttributeValue::Boolean(bool) => bool.fmt(f),
            AttributeValue::Count(num) => num.fmt(f),
            AttributeValue::Date(date) => date.format("%d.%m.%Y %H:%M:%S%.3f").fmt(f),
            AttributeValue::Percentage(v) => write!(f, "{:.2}", v * 100.0),
            AttributeValue::List(list) => write!(f, "[{}]", list.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>().join(", ")),
            AttributeValue::Number(num) => num.fmt(f),
            AttributeValue::String(str) => str.fmt(f),
            AttributeValue::Enum { variant, value } => write!(f, "{variant}: {value}"),
            AttributeValue::Map(map) => write!(f, "{{{}}}", map.iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>()
                .join(", ")
            )
        }
    }
}
