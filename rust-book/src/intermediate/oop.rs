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

//! OOP: <https://doc.rust-lang.org/book/ch17-02-trait-objects.html>

pub fn run() {}

#[test]
fn test_vdom_react_prototype() {
  /// Virtual DOM struct.
  #[derive(Debug)]
  struct VirtualDOM {
    tag: String,
    value: Option<String>,
    children: Option<Children>,
  }
  type Children = Vec<VirtualDOM>;

  /// Virtual DOM builder.
  impl VirtualDOM {
    fn new(name: &str) -> VirtualDOM {
      VirtualDOM {
        tag: name.to_string(),
        value: None,
        children: None,
      }
    }

    fn value(mut self, value: &str) -> VirtualDOM {
      self.value = Some(value.to_string());
      self
    }

    fn add_child(mut self, child: VirtualDOM) -> VirtualDOM {
      if let Some(ref mut children) = self.children {
        children.push(child);
      } else {
        self.children = Some(vec![child]);
      }
      self
    }

    fn set_children(mut self, children: Children) -> VirtualDOM {
      self.children = Some(children);
      self
    }

    fn move_to_string(self) -> String {
      format!(
        "{} {} {}",
        self.tag.to_string(),
        self.value.unwrap().to_string(),
        self
          .children
          .unwrap()
          .into_iter()
          .map(|c| format!("{}", c.move_to_string()))
          .collect::<Vec<String>>()
          .join(" ")
      )
    }
  }

  /// `Render` trait & `Component` type.
  trait Render {
    fn render(&self) -> VirtualDOM;
  }
  type Component = Box<dyn Render>;

  /// `Screen` struct.
  struct Screen {
    pub components: Vec<Component>,
  }
  impl Screen {
    pub fn render_all(&self) -> Children {
      self
        .components
        .iter()
        .map(|component| component.render())
        .collect()
    }
  }

  /// `Select` component that implements `Render` trait.
  struct Select {
    pub name: String,
    pub options: Vec<String>,
  }
  impl Render for Select {
    fn render(&self) -> VirtualDOM {
      let options_to_vdom = self
        .options
        .iter()
        .map(|option| VirtualDOM::new("option").value(option))
        .collect::<Vec<VirtualDOM>>();
      VirtualDOM::new("select").set_children(options_to_vdom)
    }
  }

  /// `Text` component that implements `Render` trait.
  struct Text {
    text: String,
  }
  impl Render for Text {
    fn render(&self) -> VirtualDOM {
      VirtualDOM::new("text").value(&self.text)
    }
  }

  /// `Button` component that implements `Render` trait.
  struct Button {
    width: u32,
    height: u32,
    label: String,
  }
  impl Render for Button {
    fn render(&self) -> VirtualDOM {
      VirtualDOM::new("button")
        .add_child(VirtualDOM::new("width").value(&self.width.to_string()))
        .add_child(VirtualDOM::new("height").value(&self.height.to_string()))
        .add_child(VirtualDOM::new("label").value(&self.label.to_string()))
    }
  }

  // Create a screen, add some components, and then render.
  let screen = Screen {
    components: vec![
      Box::new(Select {
        name: String::from("select"),
        options: vec!["option1".to_string(), "option2".to_string()],
      }),
      Box::new(Text {
        text: String::from("text"),
      }),
      Box::new(Button {
        width: 100,
        height: 100,
        label: String::from("button"),
      }),
    ],
  };
  println!("{:#?}", screen.render_all());
}
