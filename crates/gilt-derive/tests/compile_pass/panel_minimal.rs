//! Panel derive — minimal valid input must compile.

use gilt::Panel;

#[derive(Panel)]
#[panel(title = "Server")]
struct Server {
    #[field(label = "Host")]
    host: String,
    #[field(label = "Port")]
    port: u16,
}

fn main() {
    let _panel = Server { host: "web-01".into(), port: 443 }.to_panel();
}
