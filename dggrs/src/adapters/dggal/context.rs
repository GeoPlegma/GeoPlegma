use once_cell::sync::Lazy;
use std::env;
use std::sync::Mutex;

use dggal_rust::dggal::DGGAL;
use dggal_rust::ecrt::Application;

pub static GLOBAL_APP: Lazy<Mutex<Application>> = Lazy::new(|| {
    let args = env::args().collect();
    let app = Application::new(&args);
    Mutex::new(app)
});

pub static GLOBAL_DGGAL: Lazy<Mutex<DGGAL>> = Lazy::new(|| {
    let app = GLOBAL_APP.lock().expect("Failed to lock GLOBAL_APP");
    let dggal = DGGAL::new(&*app);
    Mutex::new(dggal)
});
