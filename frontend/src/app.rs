use crate::pages::{AccountDetailsPage, BlockDetailsPage, HomePage, TransactionDetailsPage};
use gloo_net::http::Request;
use leptos::{component, view, IntoView};
use leptos_router::{Outlet, Redirect, Route, Router, Routes, TrailingSlash, A};
use serde::Deserialize;

const API_BASE_URL: &str = "http://127.0.0.1:3000/api";

pub(crate) async fn fetch_api<T: for<'de> Deserialize<'de>>(path: &str) -> Option<T> {
    let url = format!("{}{}", API_BASE_URL, path);
    Request::get(&url).send().await.ok()?.json::<T>().await.ok()
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <nav class="navbar">
                <A href="/" class="nav-brand">"CloudWalk Indexer"</A>
            </nav>
            <main class="container">
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/block/:hash" view=BlockDetailsPage/>
                    <Route path="/tx/:hash" view=TransactionDetailsPage/>
                    <Route path="/account/:address" view=AccountDetailsPage/>
                </Routes>
            </main>
        </Router>
    }
}
