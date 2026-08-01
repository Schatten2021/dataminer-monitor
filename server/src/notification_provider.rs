use crate::notification::Notification;
use crate::Component;

/// [`Component`] that can send out notifications over a specific channel.
pub trait NotificationProvider: Component {
    /// Notify via the specific channel about the [`Notification`]
    fn notify(&self, notification: Notification);
}