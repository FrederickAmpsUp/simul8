use dialog::Message;
use dialog::DialogBox;

pub fn show_error_dialog(message: &str) {
    Message::new(message)
        .title("simul8 error")
        .show()
        .expect("Failed to display error dialog")
}
