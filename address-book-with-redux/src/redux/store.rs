use std::fmt::Debug;
use std::hash::Hash;

pub type ReducerFn<S, A> = dyn Fn(&S, &A) -> S;
pub type SubscriberFn<S> = dyn Fn(&S);
// Equivalent.
// pub type ReducerFn<S, A> = fn(&S, &A) -> S;
// pub type SubscriberFn<S> = fn(&S);

pub struct Store<S, A> {
  pub state: S,
  pub reducer: Box<ReducerFn<S, A>>,
  pub subscribers: Vec<Box<SubscriberFn<S>>>,
}

impl<S, A> Store<S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  pub fn new(reducer: Box<ReducerFn<S, A>>) -> Self {
    Store {
      state: S::default(),
      reducer,
      subscribers: Vec::new(),
    }
  }

  pub fn dispatch_action(
    &mut self,
    action: &A,
  ) {
    self.state = (self.reducer)(&self.state, &action);
    self.subscribers.iter_mut().for_each(|subscriber| {
      (subscriber)(&self.state);
    });
  }

  pub fn add_subscriber(
    &mut self,
    subscriber: Box<SubscriberFn<S>>,
  ) {
    self.subscribers.push(subscriber);
  }
}
