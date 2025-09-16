use reqwest::blocking::get;

fn main() {
    let resp = get("https://www.google.com");
    println!("{:?}", resp);
}