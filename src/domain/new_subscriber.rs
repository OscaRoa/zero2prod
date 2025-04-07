use crate::domain::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;

#[derive(Debug)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
