use std::collections::HashMap;
use std::fmt::Formatter;

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
/// The state that an element has.
pub struct State {
    /// whether the element is currently online.
    pub online: bool,
    /// The attributes of the element.
    pub attributes: HashMap<String, AttributeValue>
}
impl State {
    /// creates a new offline state
    #[must_use]
    pub fn new() -> Self {
        Self {
            online: false,
            attributes: HashMap::new(),
        }
    }
    /// creates a new state.
    #[must_use]
    pub fn init(online: bool, attributes: HashMap<String, AttributeValue>) -> Self {
        Self { online, attributes }
    }
    /// creates a new state with the given online status.
    #[must_use]
    pub fn with_online(online: bool) -> Self {
        Self {
            online,
            attributes: HashMap::new(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
/// values that an attribute can have.
pub enum AttributeValue {
    #[default]
    /// Primarily meant for marker attributes.
    /// Supposed to mark "This attribute exists".
    Unit,
    /// A Boolean value.
    Boolean(bool),
    /// Some sort of count.
    Count(usize),
    /// A date. Useful for keeping track when an element was seen.
    Date(chrono::DateTime<chrono::Utc>),
    /// A percentage of some kind.
    Percentage(f32),
    /// A list of [`AttributeValue`]s.
    List(Vec<AttributeValue>),
    /// Some sort of number
    Number(i128),
    /// Some string.
    String(String),
    /// An enum variant.
    Enum {
        /// The identifier of the variant.
        variant: String,
        /// The actual value of the variant.
        value: Box<AttributeValue>,
    },
    /// A map mapping one [`AttributeValue`] to another.
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
