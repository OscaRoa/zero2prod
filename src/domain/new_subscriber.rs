use crate::domain::EmailAddress;
use crate::domain::subscriber_name::SubscriberName;

#[derive(Debug)]
pub struct NewSubscriber {
    pub email: EmailAddress,
    pub name: SubscriberName,
}
