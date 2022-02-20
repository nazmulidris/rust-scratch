//! Integration tests for the `tree_memory_arena` module.
/// Rust book: https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory
use rust_book_lib::tree_memory_arena::{Arena, HasId, Uid};

#[test]
fn test_tree_memory_arena() {
  // Can create an arena.
  let mut arena = Arena::new();
  let node_1_value = 42;
  let node_2_value = 100;

  // Can insert a node - node_1.
  {
    let node_1_id = arena.new_node(node_1_value, None);
    assert_eq!(node_1_id.get_id(), 0);
  }

  // Can find node_1 by id.
  {
    let node_1_id = Uid::new(0);
    assert!(arena.get_node_ref_strong(&node_1_id).is_some());

    let node_1_ref = dbg!(arena.get_node_ref_strong(&node_1_id).unwrap());
    let node_1_ref_weak = arena.get_node_ref_weak(&node_1_id).unwrap();
    assert_eq!(node_1_ref.read().unwrap().data, node_1_value);
    assert_eq!(node_1_ref_weak.upgrade().unwrap().read().unwrap().data, 42);
  }

  // Can't find node by id that doesn't exist.
  {
    let node_id_dne = Uid::new(200);
    assert!(arena.get_node_ref_strong(&node_id_dne).is_none());
  }

  // Can add child to node_1.
  {
    let node_1_id = Uid::new(0);
    let node_2_id = arena.new_node(node_2_value, Some(&node_1_id));
    let node_2_ref = dbg!(arena.get_node_ref_strong(&node_2_id).unwrap());
    let node_2_ref_weak = arena.get_node_ref_weak(&node_2_id).unwrap();
    assert_eq!(node_2_ref.read().unwrap().data, node_2_value);
    assert_eq!(
      node_2_ref_weak.upgrade().unwrap().read().unwrap().data,
      node_2_value
    );
  }

  // Can dfs tree walk.
  {
    let node_1_id = Uid::new(0);
    let node_2_id = Uid::new(1);

    let node_list = dbg!(arena.tree_walk_dfs(&node_1_id).unwrap());

    assert_eq!(node_list.len(), 2);
    assert_eq!(node_list, vec![node_1_id.get_copy_of_id(), node_2_id.get_copy_of_id()]);
  }
}
