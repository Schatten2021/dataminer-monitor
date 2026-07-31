use crate::notification::Notification;
use crate::Component;

pub trait NotificationProvider: Component {
    fn notify(&self, notification: Notification);
}