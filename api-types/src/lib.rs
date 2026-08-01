use std::collections::HashMap;
use std::fmt::Debug;

macro_rules! api_type {
    (
        $(#[$ty_attrs:meta])*
        struct $name:ident {$(
            $(#[$field_attr:meta])*
            $field_name:ident: $field_ty:ty
        ),* $(,)?}
    ) => {
        $(#[$ty_attrs])*
        #[derive(Clone, Debug, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
        pub struct $name {$(
            $(#[$field_attr])*
            pub $field_name: $field_ty,
        )*}
        impl $crate::ApiType for $name {}
    };
    ($(#[$ty_attrs:meta])*
    struct $name:ident($(#[$inner_attrs:meta])*$inner:ty)) => {
        $(#[$ty_attrs])*
        #[derive(Clone, Debug, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
        pub struct $name($(#[$inner_attrs])* pub $inner);
        impl $crate::ApiType for $name {}
    };

    (
        $(#[$ty_attrs:meta])*
        enum $name:ident {$(
            $(#[$variant_attr:meta])*
            $variant_name:ident$(($(#[$inner_attr:meta])*$inner:ty))?
        ),* $(,)?}
    ) => {
        $(#[$ty_attrs])*
        #[derive(Clone, Debug, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
        pub enum $name {$(
            $(#[$variant_attr])*
            $variant_name$(($(#[$inner_attr])* $inner))?,
        )*}
        impl $crate::ApiType for $name {}
    };
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, ::serde::Serialize)]
#[expect(private_bounds, reason="this is supposed to be private to prevent accidentally sending the\
 wrong type that might not be understood by the client.")]
pub enum ApiResponse<V: ApiType=(), E: ApiType=()> {
    Ok(V),
    ServerError(ServerError),
    ClientError(E),
}
trait ApiType {}
impl ApiType for () {}
impl ApiType for String {}
impl ApiType for &'static str {}

api_type!(struct ServerError {
    id: String,
    message: String,
});
api_type!(
#[derive(Default)]
enum AttributeValue {
    #[default]
    Unit,
    Boolean(bool),
    Count(usize),
    Date(chrono::DateTime<chrono::Utc>),
    Percentage(f32),
    List(Vec<AttributeValue>),
    Number(i128),
    String(String),
    Enum(EnumAttributeValue),
    Map(Vec<(AttributeValue, AttributeValue)>)
});
api_type!(struct EnumAttributeValue  {
        variant: String,
        value: Box<AttributeValue>,
    });
#[cfg(feature = "server-support")]
impl From<server::AttributeValue> for AttributeValue {
    fn from(value: server::AttributeValue) -> Self {
        match value {
            server::AttributeValue::Unit => Self::Unit,
            server::AttributeValue::Boolean(v) => Self::Boolean(v),
            server::AttributeValue::Count(v) => Self::Count(v),
            server::AttributeValue::Date(v) => Self::Date(v),
            server::AttributeValue::Percentage(v) => Self::Percentage(v),
            server::AttributeValue::List(v) => Self::List(v.into_iter().map(Into::into).collect()),
            server::AttributeValue::Number(v) => Self::Number(v),
            server::AttributeValue::String(v) => Self::String(v),
            server::AttributeValue::Enum { variant, value } => Self::Enum(EnumAttributeValue { variant, value: Box::new((*value).into()) }),
            server::AttributeValue::Map(v) => Self::Map(v.into_iter()
                .map(|(a, b)| (a.into(), b.into()))
                .collect()),
        }
    }
}

api_type!(struct State {
    online: bool,
    attributes: HashMap<String, AttributeValue>,
});
#[cfg(feature = "server-support")]
impl From<server::State> for State {
    fn from(value: server::State) -> Self {
        Self {
            online: value.online,
            attributes: value.attributes.into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect()
        }
    }
}
api_type!(struct States(HashMap<String, State>));
#[cfg(feature = "server-support")]
impl From<HashMap<String, server::State>> for States {
    fn from(value: HashMap<String, server::State>) -> Self {
        States(value.into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect())
    }
}
pub mod websocket {
    use crate::AttributeValue;

    api_type!(struct Message {
        element_id: String,
        component_id: String,
        reason: MessageReason
    });
    api_type!(enum MessageReason {
        OnlineStatus(OnlineStatusChange),
        Attribute(AttributeMessage),
    });
    api_type!(enum OnlineStatusChange {
        Delete,
        Create(bool),
        Change(bool),
    });
    api_type!(struct AttributeMessage {
        attribute_id: String,
        change: AttributeChange,
    });
    api_type!(enum AttributeChange {
        Create(AttributeValue),
        Change(AttributeValue),
        Delete,
    });
    #[cfg(feature = "server-support")]
    impl From<server::Notification> for Message {
        fn from(value: server::Notification) -> Self {
            Self {
                element_id: value.element_id,
                component_id: value.component_id,
                reason: MessageReason::from(value.reason)
            }
        }
    }
    #[cfg(feature = "server-support")]
    impl From<server::NotificationReason> for MessageReason {
        fn from(value: server::NotificationReason) -> Self {
            match value {
                server::NotificationReason::OnlineStatusChanged(new) => Self::OnlineStatus(OnlineStatusChange::Change(new)),
                server::NotificationReason::NewElement(state) => Self::OnlineStatus(OnlineStatusChange::Create(state)),
                server::NotificationReason::AttributeCreated(id, val) => Self::Attribute(AttributeMessage {
                    attribute_id: id,
                    change: AttributeChange::Create(val.into()),
                }),
                server::NotificationReason::AttributeChanged(id, _, new) => Self::Attribute(AttributeMessage {
                    attribute_id: id,
                    change: AttributeChange::Change(new.into()),
                }),
                server::NotificationReason::DeleteAttribute(id, _) => Self::Attribute(AttributeMessage {
                    attribute_id: id,
                    change: AttributeChange::Delete,
                })
            }
        }
    }
}
