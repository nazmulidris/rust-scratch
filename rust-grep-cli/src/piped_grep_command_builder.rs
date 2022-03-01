#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipedGrepOptions {
  pub search: String,
  pub case_sensitive: bool,
}

const REQUIRED_ARGS_COUNT: usize = 2;

pub struct PipedGrepOptionsBuilder;

impl PipedGrepOptionsBuilder {
  pub fn parse(args: Vec<String>) -> Result<PipedGrepOptions, String> {
    if args.len() < REQUIRED_ARGS_COUNT {
      return Err(format!(
        "Expected at least {} arguments, got {}.",
        REQUIRED_ARGS_COUNT,
        args.len()
      ));
    }

    let mut args = args.iter();
    args.next(); // Skip the first argument.

    let options = PipedGrepOptions {
      search: match args.next() {
        Some(arg) => arg.clone(),
        None => String::new(),
      },
      case_sensitive: args.next().is_some(), // If 3rd arg exists, then true.
    };

    Ok(options)
  }
}

#[test]
fn test_can_build_options_with_2_args() {
  let options = PipedGrepOptionsBuilder::parse(vec![
    String::from("program"),
    String::from("search-string"),
    String::from("case-sensitive"),
  ]);
  match options {
    Ok(options) => {
      assert_eq!(options.search, "search-string");
      assert_eq!(options.case_sensitive, true);
    }
    Err(error) => panic!("{}", error),
  }
}

#[test]
fn test_can_build_options_with_1_args() {
  let options = PipedGrepOptionsBuilder::parse(vec![String::from("search-string")]);
  if let Ok(options) = options {
    assert_eq!(options.search, "search-string");
    assert_eq!(options.case_sensitive, false);
  }
}

#[test]
#[should_panic]
fn test_can_not_build_options_with_0_args() {
  let options = PipedGrepOptionsBuilder::parse(vec![]);
  if let Err(error) = options {
    panic!("{}", error)
  }
}
