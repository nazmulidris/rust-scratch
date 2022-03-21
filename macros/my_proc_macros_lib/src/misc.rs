use proc_macro::TokenStream;

pub fn make_answer_macro() -> TokenStream {
  let token_stream_str = "fn answer() -> u32 { 42 }";
  let token_stream = token_stream_str.parse().unwrap();
  eprintln!("{} => \n{:#?}", token_stream_str, token_stream);
  token_stream
}
