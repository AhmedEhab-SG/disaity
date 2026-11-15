mod free_games;
pub use free_games::FreeGamesSubscription;

use disaity_core::{AsSubscription, Client, Feature, SubscriptionModule};

#[derive(Default)]
pub struct Subscriptions(Vec<Box<dyn Feature>>);

impl Subscriptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, sub: impl SubscriptionModule) -> Self {
        self.0.push(Box::new(AsSubscription(sub)));
        self
    }

    pub fn all() -> Self {
        Self::new().with(FreeGamesSubscription)
    }
}

impl IntoIterator for Subscriptions {
    type Item = Box<dyn Feature>;
    type IntoIter = std::vec::IntoIter<Box<dyn Feature>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub trait SubscriptionsExt {
    fn with_subscription(self, sub: impl SubscriptionModule) -> Self;

    fn with_subscriptions(self, subs: impl IntoIterator<Item = Box<dyn Feature>>) -> Self;
}

impl SubscriptionsExt for Client {
    fn with_subscription(self, sub: impl SubscriptionModule) -> Self {
        self.with_feature(AsSubscription(sub))
    }

    fn with_subscriptions(mut self, subs: impl IntoIterator<Item = Box<dyn Feature>>) -> Self {
        self.features.extend(subs);
        self
    }
}
