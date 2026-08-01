use api_types::AttributeValue;
use std::collections::HashMap;
use api_types::websocket::{AttributeChange, Message, MessageReason, OnlineStatusChange};

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub online: bool,
    pub attributes: HashMap<String, AttributeValue>,
}
impl From<api_types::State> for Element {
    fn from(value: api_types::State) -> Self {
        Self {
            online: value.online,
            attributes: value.attributes,
        }
    }
}
pub struct AppState(pub HashMap<String, Element>);
impl From<api_types::States> for AppState {
    fn from(value: api_types::States) -> Self {
        Self(value.0.into_iter()
            .map(|(a, b)| (a, b.into()))
            .collect())
    }
}
impl AppState {
    pub fn handle(&mut self, message: Message) {
        if matches!(message.reason, MessageReason::OnlineStatus(OnlineStatusChange::Delete)) {
            self.0.remove(&message.element_id);
            return;
        }
        #[expect(clippy::single_match_else, reason="is just more readable this way.")]
        let element = match self.0.get_mut(&message.element_id) {
            Some(e) => e,
            None => {
                self.0.insert(message.element_id.clone(), Element {
                    online: false,
                    attributes: HashMap::new(),
                });
                self.0.get_mut(&message.element_id).unwrap()
            }
        };
        element.handle(message.reason);
    }
}
impl Element {
    pub fn handle(&mut self, msg: MessageReason) {
        match msg {
            MessageReason::OnlineStatus(change) => match change {
                OnlineStatusChange::Delete => {}
                OnlineStatusChange::Create(state) |
                OnlineStatusChange::Change(state) => self.online = state,
            }
            MessageReason::Attribute(change) => {
                match change.change {
                    AttributeChange::Create(val) |
                    AttributeChange::Change(val) => self.attributes.insert(change.attribute_id, val),
                    AttributeChange::Delete => self.attributes.remove(&change.attribute_id)
                };
            }
        }
    }
}