use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

use r3bl_rs_utils::utils::{print_header, print_header2, style_dimmed};

pub type ReducerFn<S, A> = dyn Fn(&S, &A) -> S;
pub type SubscriberFn<S> = dyn Fn(&S);
// Equivalent.
// pub type ReducerFn<S, A> = fn(&S, &A) -> S;
// pub type SubscriberFn<S> = fn(&S);

pub struct Store<'a, S, A> {
  pub state: S,
  pub reducer: &'a ReducerFn<S, A>,
  pub subscribers: Vec<&'a SubscriberFn<S>>,
}

impl<'a, S, A> Store<'a, S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  pub fn new(reducer: &'a ReducerFn<S, A>) -> Self {
    Store {
      state: S::default(),
      reducer,
      subscribers: Vec::<&SubscriberFn<S>>::new(),
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

  /// https://doc.rust-lang.org/std/primitive.pointer.html
  /// https://users.rust-lang.org/t/compare-function-pointers-for-equality/52339
  /// https://www.reddit.com/r/rust/comments/98xlh3/how_can_i_compare_two_function_pointers_to_see_if/
  pub fn add_subscriber(
    &mut self,
    new_subscriber: &'a SubscriberFn<S>,
  ) {
    let new_subscriber_raw_ptr = new_subscriber as *const SubscriberFn<S>;
    if self.subscribers.iter().any(|existing_subscriber| {
      let existing_subscriber_raw_ptr = *existing_subscriber as *const SubscriberFn<S>;
      let new_and_existing_subscriber_same =
        existing_subscriber_raw_ptr == new_subscriber_raw_ptr;
      new_and_existing_subscriber_same
    }) {
      // Don't add new_scriber if it's already in the list.
      println!("{}", style_dimmed("Subscriber already exists"));
      return;
    }
    self.subscribers.push(new_subscriber);
  }
}
