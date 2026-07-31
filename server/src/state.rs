use std::collections::HashMap;

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