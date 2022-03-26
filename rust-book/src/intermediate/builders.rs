/*
 Copyright 2022 Nazmul Idris

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

      https://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
*/

//! Builders: <https://doc.rust-lang.org/1.0.0/style/ownership/builders.html>

pub fn run() {}

/// [std::thread::Builder](https://doc.rust-lang.org/src/std/thread/mod.rs.html#263-268)
#[test]
fn test_consuming_builder() {

  struct Builder {
    proc_exec: Option<String>,
    cwd: Option<String>,
    args: Option<Vec<String>>,
  }

  impl Builder {
    fn new() -> Builder {
      Builder {
        proc_exec: None,
        cwd: None,
        args: None,
      }
    }

    fn name(mut self, name: &str) -> Builder {
      self.proc_exec = Some(name.to_string());
      self
    }

    fn size(mut self, size: &str) -> Builder {
      self.cwd = Some(size.to_string());
      self
    }

    fn arg(mut self, arg: &str) -> Builder {
      if let Some(ref mut args) = self.args {
        args.push(arg.to_string());
      } else {
        self.args = Some(vec![arg.to_string()]);
      }
      self
    }

    fn build(self) -> String {
      format!(
        "{} {} {}",
        self.proc_exec.unwrap(),
        self.cwd.unwrap(),
        self.args.unwrap().join(" ")
      )
    }
  }

  let mut builder = Builder::new().name("foo").size("42");
  assert_eq!(builder.proc_exec, Some("foo".to_string()));
  assert_eq!(builder.cwd, Some("42".to_string()));
  assert_eq!(builder.args, None);

  builder = builder.arg("bar");
  assert_eq!(builder.args, Some(vec!["bar".to_string()]));

  let result = builder.build();
  assert_eq!(result, "foo 42 bar");
}

#[test]
fn test_non_consuming_builder_without_clone() {
  pub struct Command {
    program: String,
    args: Vec<String>,
    cwd: Option<String>,
    // etc
  }

  impl Command {
    pub fn new(program: String) -> Command {
      Command {
        program: program,
        args: Vec::new(),
        cwd: None,
      }
    }

    /// Add an argument to pass to the program.
    pub fn arg<'a>(&'a mut self, arg: String) -> &'a mut Command {
      self.args.push(arg);
      self
    }

    /// Set the working directory for the child process.
    pub fn cwd<'a>(&'a mut self, dir: String) -> &'a mut Command {
      self.cwd = Some(dir);
      self
    }

    /// Turns all the args into a String.
    pub fn build(&self) -> String {
      format!(
        "Do something with {}, {}, {}",
        self.program,
        self.args.join(", "),
        if let Some(path) = &self.cwd {
          path.clone()
        } else {
          "".to_string()
        }
      )
    }
  }

  let command1 = Command::new("ls".to_string())
    .arg("foo=bar".to_string())
    .arg("baz=qux".to_string())
    .cwd("/tmp".to_string())
    .build();
  assert_eq!(
    command1,
    "Do something with ls, foo=bar, baz=qux, /tmp".to_string()
  );

  let mut command2 = Command::new("ls".to_string());
  command2.arg("foo=bar".to_string());
  assert_eq!(command2.program, "ls".to_string());
  assert_eq!(command2.args, vec!["foo=bar".to_string()]);
}

#[test]
fn test_non_consuming_builder_with_clone() {
  #[derive(Clone, Debug)]
  pub struct CommandBuilder {
    req_proc_exec: String,
    opt_args: Option<Vec<String>>,
    opt_cwd: Option<String>,
  }

  impl CommandBuilder {
    pub fn new(program: &str) -> CommandBuilder {
      CommandBuilder {
        req_proc_exec: program.to_string(),
        opt_args: None,
        opt_cwd: None,
      }
    }

    /// Add an argument to pass to the program.
    pub fn arg<'a>(&'a mut self, arg: &str) -> &'a mut CommandBuilder {
      if self.opt_args.is_none() {
        self.opt_args = Some(Vec::new());
      }
      self.opt_args.as_mut().unwrap().push(arg.to_string());
      self
    }

    /// Set the working directory for the child process.
    pub fn cwd<'a>(&'a mut self, dir: &str) -> &'a mut CommandBuilder {
      self.opt_cwd = Some(dir.to_string());
      self
    }

    /// Return all the config options.
    pub fn build(&self) -> CommandBuilder {
      self.clone()
    }
  }

  let command = CommandBuilder::new("ls")
    .arg("foo=bar")
    .arg("baz=qux")
    .cwd("/tmp")
    .build();
  assert!(command.opt_args.is_some());
  assert_eq!(command.opt_args.unwrap(), vec!["foo=bar", "baz=qux"]);
  assert_eq!(command.opt_cwd, Some("/tmp".to_string()));
  assert_eq!(command.req_proc_exec, "ls".to_string());
}

#[test]
fn test_non_consuming_builder_with_clone_2() {
  pub struct Command {
    req_proc_exec: String,
    opt_args: Vec<String>,
    opt_cwd: Option<String>,
  }

  impl Command {
    pub fn new(program: String) -> Command {
      Command {
        req_proc_exec: program,
        opt_args: Vec::new(),
        opt_cwd: None,
      }
    }

    /// Add an argument to pass to the program.
    pub fn arg<'a>(&'a mut self, arg: String) -> &'a mut Command {
      self.opt_args.push(arg);
      self
    }

    /// Set the working directory for the child process.
    pub fn cwd<'a>(&'a mut self, dir: String) -> &'a mut Command {
      self.opt_cwd = Some(dir);
      self
    }

    /// Executes the command as a child process, which is returned.
    pub fn build(&self) -> Command {
      Command {
        req_proc_exec: self.req_proc_exec.clone(),
        opt_args: self.opt_args.clone(),
        opt_cwd: self.opt_cwd.clone(),
      }
    }
  }

  let command = Command::new("ls".to_string())
    .arg("foo=bar".to_string())
    .arg("baz=qux".to_string())
    .cwd("/tmp".to_string())
    .build();

  assert!(command.opt_args.len() == 2);
}
