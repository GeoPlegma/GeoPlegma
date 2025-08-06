use once_cell::sync::Lazy;
use std::env;
use std::sync::Mutex;

#[cfg(not(target_arch = "wasm32"))]
use dggal_rust::dggal::DGGAL;
#[cfg(not(target_arch = "wasm32"))]
use dggal_rust::ecrt::Application;

#[cfg(not(target_arch = "wasm32"))]
pub static GLOBAL_APP: Lazy<Mutex<Application>> = Lazy::new(|| {
    let args = env::args().collect();
    let app = Application::new(&args);
    Mutex::new(app)
});

#[cfg(not(target_arch = "wasm32"))]
pub static GLOBAL_DGGAL: Lazy<Mutex<DGGAL>> = Lazy::new(|| {
    let app = GLOBAL_APP.lock().expect("Failed to lock GLOBAL_APP");
    let dggal = DGGAL::new(&*app);
    Mutex::new(dggal)
});
