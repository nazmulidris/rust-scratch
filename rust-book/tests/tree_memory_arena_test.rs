//! Integration tests for the `tree_memory_arena` module.
/// Rust book: https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory
use rust_book_lib::{
  tree_memory_arena::{Arena, HasId, Id, Uid},
  utils::{style_primary, style_prompt},
};

#[test]
fn test_can_add_nodes_to_tree() {
  // Can create an arena.
  let mut arena = Arena::new();
  let node_1_value = 42;
  let node_2_value = 100;

  // Can insert a node - node_1.
  {
    let node_1_id = arena.add_new_node(node_1_value, None);
    assert_eq!(node_1_id.get_id(), 0);
  }

  // Can find node_1 by id.
  {
    let node_1_id = Uid::new(0);
    assert!(arena.get_arc_to_node(&node_1_id).is_some());

    let node_1_ref = dbg!(arena.get_arc_to_node(&node_1_id).unwrap());
    let node_1_ref_weak = arena.get_weak_ref_to_node(&node_1_id).unwrap();
    assert_eq!(node_1_ref.read().unwrap().payload, node_1_value);
    assert_eq!(
      node_1_ref_weak.upgrade().unwrap().read().unwrap().payload,
      42
    );
  }

  // Can't find node by id that doesn't exist.
  {
    let node_id_dne = Uid::new(200);
    assert!(arena.get_arc_to_node(&node_id_dne).is_none());
  }

  // Can add child to node_1.
  {
    let node_1_id = Uid::new(0);
    let node_2_id = arena.add_new_node(node_2_value, Some(&node_1_id));
    let node_2_ref = dbg!(arena.get_arc_to_node(&node_2_id).unwrap());
    let node_2_ref_weak = arena.get_weak_ref_to_node(&node_2_id).unwrap();
    assert_eq!(node_2_ref.read().unwrap().payload, node_2_value);
    assert_eq!(
      node_2_ref_weak.upgrade().unwrap().read().unwrap().payload,
      node_2_value
    );
  }

  // Can dfs tree walk.
  {
    let node_1_id = Uid::new(0);
    let node_2_id = Uid::new(1);

    let node_list = dbg!(arena.tree_walk_dfs(&node_1_id).unwrap());

    assert_eq!(node_list.len(), 2);
    assert_eq!(node_list, vec![node_1_id.get_uid(), node_2_id.get_uid()]);
  }
}

#[test]
fn test_can_walk_tree_and_delete_nodes_from_tree() {
  let mut arena = Arena::<String>::new();

  // root
  //   +- child1
  //   |    +- gc1
  //   |    +- gc2
  //   +- child2

  let root = arena.add_new_node("root".to_string(), None);
  let child1 = arena.add_new_node("child1".to_string(), Some(&root));
  let gc_1_id = arena.add_new_node("gc1".to_string(), Some(&child1));
  let gc_2_id = arena.add_new_node("gc2".to_string(), Some(&child1));
  let child_2_id = arena.add_new_node("child2".to_string(), Some(&root));
  println!("{}, {:#?}", style_primary("arena"), arena);

  // Test that the data is correct for each node.
  assert_node_data_is_eq(&arena, &root, "root");
  assert_node_data_is_eq(&arena, &child1.get_uid(), "child1");
  assert_node_data_is_eq(&arena, &gc_1_id.get_uid(), "gc1");
  assert_node_data_is_eq(&arena, &gc_2_id.get_uid(), "gc2");
  assert_node_data_is_eq(&arena, &child_2_id.get_uid(), "child2");

  assert_eq!(arena.get_children_of(&root).unwrap().len(), 2);
  assert_eq!(arena.get_parent_of(&root).is_none(), true);

  assert_eq!(arena.get_children_of(&child1).unwrap().len(), 2);
  assert_eq!(
    arena.get_parent_of(&child1).unwrap().get_id(),
    root.get_id()
  );

  // Test that tree walking works correctly for nodes.
  assert_eq!(arena.tree_walk_dfs(&root).unwrap().len(), 5);

  let child1_and_descendants = arena.tree_walk_dfs(&child1.get_uid()).unwrap();
  assert_eq!(child1_and_descendants.len(), 3);
  assert!(child1_and_descendants.contains(&child1.get_uid()));
  assert!(child1_and_descendants.contains(&gc_1_id.get_uid()));
  assert!(child1_and_descendants.contains(&gc_2_id.get_uid()));

  assert_eq!(arena.tree_walk_dfs(&child_2_id.get_uid()).unwrap().len(), 1);
  assert!(arena
    .tree_walk_dfs(&child_2_id.get_uid())
    .unwrap()
    .contains(&child_2_id.get_uid()));

  // Test that node deletion works correclty.
  {
    println!(
      "{} {:?}",
      style_primary("root -before- ==>"),
      arena.tree_walk_dfs(&root).unwrap()
    );
    let deletion_list = arena.delete_node(&child1.get_uid());
    assert_eq!(deletion_list.as_ref().unwrap().len(), 3);
    assert!(deletion_list.as_ref().unwrap().contains(&gc_1_id.get_uid()));
    assert!(deletion_list.as_ref().unwrap().contains(&gc_2_id.get_uid()));
    assert!(deletion_list.as_ref().unwrap().contains(&child1.get_uid()));
    println!(
      "{} {:?}",
      style_prompt("root -after- <=="),
      arena.tree_walk_dfs(&root).unwrap()
    );
    assert_eq!(dbg!(arena.tree_walk_dfs(&root).unwrap()).len(), 2);
  }

  // Helper functions.
  fn assert_node_data_is_eq(
    arena: &Arena<String>,
    node_id: &Id,
    expected_name: &str,
  ) {
    let child_ref = arena.get_arc_to_node(node_id).unwrap();
    assert_eq!(child_ref.read().unwrap().payload, expected_name.to_string());
  }
}

#[test]
fn test_can_search_nodes_in_tree_with_filter_lambda() {
  let mut arena = Arena::<String>::new();

  // root
  //   +- child1
  //   |    +- gc1
  //   |    +- gc2
  //   +- child2

  let root = arena.add_new_node("root".to_string(), None);
  let child1 = arena.add_new_node("child1".to_string(), Some(&root));
  let _gc_1_id = arena.add_new_node("gc1".to_string(), Some(&child1));
  let _gc_2_id = arena.add_new_node("gc2".to_string(), Some(&child1));
  let _child_2_id = arena.add_new_node("child2".to_string(), Some(&root));
  println!("{}, {:#?}", style_primary("arena"), &arena);
  println!(
    "{}, {:#?}",
    style_primary("root"),
    arena.get_arc_to_node(&root)
  );

  // Search entire arena for root.get_id().
  {
    let filter_id = root.get_id();
    let result = &arena.filter_all_nodes_by(&move |id, _node_ref| {
      if id == filter_id {
        true
      } else {
        false
      }
    });
    assert_eq!(result.as_ref().unwrap().len(), 1);
  }

  // Search entire arena for node that contains payload "gc1".
  {
    let result = &arena.filter_all_nodes_by(&move |_id, node_ref| {
      if node_ref.payload == "gc1" {
        true
      } else {
        false
      }
    });
    assert_eq!(result.as_ref().unwrap().len(), 1);
  }
}
