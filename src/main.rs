use dioxus::prelude::*;
use imgfloat::data::Username;
use imgfloat::ui::App;

fn main() {
    #[cfg(feature = "server")]
    dioxus::LaunchBuilder::new()
        .with_cfg(dioxus::fullstack::ServeConfigBuilder::default())
        .with_context(imgfloat::create_server_state())
        .launch(App);

    #[cfg(feature = "web")]
    dioxus::LaunchBuilder::new()
        .with_context(imgfloat::create_client_state())
        .launch(App)
}
