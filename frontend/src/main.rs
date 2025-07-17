#![allow(clippy::uninlined_format_args)]
#![allow(clippy::too_many_lines)]
#![allow(non_snake_case)]

use crate::app::App;
use leptos::{mount_to_body, view};

mod app;
mod pages;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <App/> }
    });
}
