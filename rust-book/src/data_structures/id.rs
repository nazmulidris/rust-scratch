// UID for Node.
pub trait HasId {
  fn id(&self) -> usize;
}

#[derive(Debug)]
pub struct NodeId {
  index: usize,
}

impl HasId for NodeId {
  fn id(&self) -> usize {
    self.index
  }
}
