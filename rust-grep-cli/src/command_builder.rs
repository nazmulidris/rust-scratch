pub struct CommandBuilder {
  command: String,
  args: Vec<String>,
}

impl CommandBuilder {
  pub fn new(command: String) -> CommandBuilder {
    CommandBuilder {
      command,
      args: Vec::new(),
    }
  }

  pub fn add_arg(
    &mut self,
    arg: String,
  ) -> &mut CommandBuilder {
    self.args.push(arg);
    self
  }
}
