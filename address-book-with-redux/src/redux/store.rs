use std::{fmt::Debug};
use std::hash::Hash;


use r3bl_rs_utils::utils::{style_dimmed};

pub type ReducerFn<S, A> = dyn Fn(&S, &A) -> S;
pub type SubscriberFn<S> = dyn Fn(&S);
// Equivalent.
// pub type ReducerFn<S, A> = fn(&S, &A) -> S;
// pub type SubscriberFn<S> = fn(&S);

pub struct Store<'a, S, A> {
  pub state: S,
  pub reducer_fn: &'a ReducerFn<S, A>,
  pub subscriber_fns: Vec<&'a SubscriberFn<S>>,
}

impl<'a, S, A> Store<'a, S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  pub fn new(reducer: &'a ReducerFn<S, A>) -> Self {
    Store {
      state: S::default(),
      reducer_fn: reducer,
      subscriber_fns: Vec::<&SubscriberFn<S>>::new(),
    }
  }

  pub fn dispatch_action(
    &mut self,
    action: &A,
  ) {
    self.state = (self.reducer_fn)(&self.state, &action);
    self.subscriber_fns.iter_mut().for_each(|subscriber| {
      (subscriber)(&self.state);
    });
  }

  pub fn add_subscriber_fn(
    &mut self,
    new_subscriber: &'a SubscriberFn<S>,
  ) {
    match self.subscriber_exists(new_subscriber) {
      (true, _) => println!("{}", style_dimmed("Subscriber already exists")),
      (false, _) => self.subscriber_fns.push(new_subscriber),
    }
  }

  pub fn remove_subscriber_fn(
    &mut self,
    subscriber_to_remove: &'a SubscriberFn<S>,
  ) {
    match self.subscriber_exists(subscriber_to_remove) {
      (true, index) => {
        self.subscriber_fns.remove(index.unwrap());
      }
      _ => {}
    }
  }

  pub fn remove_all_subscribers(&mut self) {
    self.subscriber_fns.clear();
  }

  /// https://doc.rust-lang.org/std/primitive.pointer.html
  /// https://users.rust-lang.org/t/compare-function-pointers-for-equality/52339
  /// https://www.reddit.com/r/rust/comments/98xlh3/how_can_i_compare_two_function_pointers_to_see_if/
  fn subscriber_exists(
    &self,
    new_subscriber: &'a SubscriberFn<S>,
  ) -> (bool, Option<usize>) {
    let new_subscriber_raw_ptr = new_subscriber as *const SubscriberFn<S>;
    let mut return_index = 0 as usize;
    if self
      .subscriber_fns
      .iter()
      .enumerate()
      .any(|(index, existing_subscriber)| {
        return_index = index;
        let existing_subscriber_raw_ptr = *existing_subscriber as *const SubscriberFn<S>;
        let new_and_existing_subscriber_same =
          existing_subscriber_raw_ptr == new_subscriber_raw_ptr;
        new_and_existing_subscriber_same
      })
    {
      return (true, Some(return_index));
    }
    return (false, None);
  }
}
