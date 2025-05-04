use crate::persistence::redis::NotificationSettings;
use actix::{Actor, Context, Handler, Message};
use chatterbox::message::Dispatcher;

pub(crate) struct NotificationManager {}

impl NotificationManager {
    pub(crate) fn new() -> NotificationManager {
        Self {}
    }
}

impl NotificationManager {
    pub(crate) fn try_notify(&self, notification_settings: NotificationSettings) -> bool {
        let dispatcher = Dispatcher::new(notification_settings.into());
        dispatcher.send_test_message().is_ok()
    }
}

#[derive(Message, Clone)]
#[rtype(result = "bool")]
pub(crate) struct TryNotify(pub NotificationSettings);

pub(crate) struct NotificationActor {
    pub(crate) notification_manager: NotificationManager,
}

impl Actor for NotificationActor {
    type Context = Context<Self>;
}

impl Handler<TryNotify> for NotificationActor {
    type Result = bool;

    fn handle(&mut self, msg: TryNotify, _: &mut Context<Self>) -> Self::Result {
        self.notification_manager.try_notify(msg.0)
    }
}
