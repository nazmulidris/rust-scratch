use seshat::unicode::Segmentation;
use seshat::unicode::Ucd;

pub fn print_graphemes() {
  println!("🦀 is {}!", '🦀'.na());
  println!("📦 is {}!", '📦'.na());
  println!("🦜 is {}!", '🦜'.na());
  println!("Multiple code points: 🙏🏽");
  println!("Multiple code points: 💇🏽‍♂️");
}

pub fn print_cluster_breaks() {
  let s = "Hi + 📦 + 🙏🏽 + 👨🏾‍🤝‍👨🏿";
  let breaks = s.break_graphemes();
  for (idx, str) in breaks.enumerate() {
    println!("{}: '{}'", idx, str);
  }
}
