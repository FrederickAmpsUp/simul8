#[cfg(not(target_arch = "wasm32"))]
use dialog::{Message, DialogBox};

#[cfg(not(target_arch = "wasm32"))]
pub fn show_error_dialog(message: &str) {
    Message::new(message)
        .title("simul8 error")
        .show()
        .expect("Failed to display error dialog")
}

#[cfg(target_arch = "wasm32")]
pub fn show_error_dialog(message: &str) {
    log::error!("{}", message); // TODO: window.alert()
}
