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

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(fut: F) where F: futures::Future<Output: Send> + Send + 'static{
    smol::spawn(fut).detach();
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(fut: F) where F: futures::Future<Output = ()> + Send + 'static {
    wasm_bindgen_futures::spawn_local(fut);
}

use std::sync::{Arc, Mutex};

pub struct OverwriteSlot<T> {
    inner: Arc<Mutex<Option<T>>>,
}

impl<T> Clone for OverwriteSlot<T> {
    fn clone(&self) -> Self {
        OverwriteSlot {
            inner: Arc::clone(&self.inner),
        }
    }
}


impl<T> OverwriteSlot<T> {
    pub fn new() -> (Self, Self) {
        let v = Self {
            inner: Arc::new(Mutex::new(None)),
        };

        (v.clone(), v)
    }

    pub fn write(&self, value: T) {
        let mut slot = self.inner.lock().unwrap();
        *slot = Some(value); // overwrite
    }

    pub fn try_read(&self) -> Option<T> {
        let mut slot = self.inner.lock().unwrap();
        slot.take() // consume
    }
}

pub fn color32_to_hsva(color: egui::Color32) -> egui::epaint::Hsva {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    egui::epaint::Hsva::from_srgba_unmultiplied(
        [r, g, b, a]
    )
}

pub fn hsva_to_color32(hsva: egui::epaint::Hsva) -> egui::Color32 {
    let [r, g, b, a] = hsva.to_srgba_unmultiplied();
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

