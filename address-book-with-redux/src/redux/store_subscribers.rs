use std::fmt::Debug;
use std::hash::Hash;
use r3bl_rs_utils::utils::style_dimmed;
use super::{Store, SubscriberFn, SubscriberManager};

/// More info on method chaining approaches in Rust:
/// <https://randompoison.github.io/posts/returning-self/>
impl<'a, S, A> SubscriberManager<'a, S, A> for Store<'a, S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  // Manage subscribers.
  fn add_subscriber_fn(
    &mut self,
    new_subscriber_fn: &'a SubscriberFn<S>,
  ) -> &mut Self {
    match self.subscriber_exists(new_subscriber_fn) {
      (true, _) => println!("{}", style_dimmed("Subscriber already exists")),
      (false, _) => self.subscriber_fns.push(new_subscriber_fn),
    }
    self
  }

  fn remove_subscriber_fn(
    &mut self,
    subscriber_fn_to_remove: &'a SubscriberFn<S>,
  ) -> &mut Self {
    match self.subscriber_exists(subscriber_fn_to_remove) {
      (true, index) => {
        self.subscriber_fns.remove(index.unwrap());
      }
      _ => {}
    }
    self
  }

  fn remove_all_subscribers(&mut self) -> &mut Self {
    self.subscriber_fns.clear();
    self
  }

  /// https://doc.rust-lang.org/std/primitive.pointer.html
  /// https://users.rust-lang.org/t/compare-function-pointers-for-equality/52339
  /// https://www.reddit.com/r/rust/comments/98xlh3/how_can_i_compare_two_function_pointers_to_see_if/
  fn subscriber_exists(
    &self,
    new_subscriber: &'a SubscriberFn<S>,
  ) -> (bool, Option<usize>) {
    let this = new_subscriber as *const SubscriberFn<S>;
    let mut index_if_found = 0 as usize;
    if self
      .subscriber_fns
      .iter()
      .enumerate()
      .any(|(index, other)| {
        index_if_found = index;
        this == *other as *const SubscriberFn<S>
      })
    {
      return (true, Some(index_if_found));
    }
    return (false, None);
  }
}

