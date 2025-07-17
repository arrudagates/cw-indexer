use crate::pages::{AccountDetailsPage, BlockDetailsPage, HomePage, TransactionDetailsPage};
use gloo_net::http::Request;
use leptos::{component, view, IntoView};
use leptos_router::{Outlet, Redirect, Route, Router, Routes, A};
use serde::Deserialize;

const API_BASE_URL: &str = "http://127.0.0.1:3000/api";

pub(crate) async fn fetch_api<T: for<'de> Deserialize<'de>>(path: &str) -> Option<T> {
    let url = format!("{}{}", API_BASE_URL, path);
    Request::get(&url).send().await.ok()?.json::<T>().await.ok()
}

#[component]
fn AppLayout() -> impl IntoView {
    view! {
        <nav class="navbar">
            <A href="/cw-indexer" class="nav-brand">"CloudWalk Indexer"</A>
        </nav>
        <main class="container">
            <Outlet/>
        </main>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=|| view! { <Redirect path="/cw-indexer"/> }/>

                <Route path="/cw-indexer" view=AppLayout>
                    <Route path="" view=HomePage/>
                    <Route path="block/:hash" view=BlockDetailsPage/>
                    <Route path="tx/:hash" view=TransactionDetailsPage/>
                    <Route path="account/:address" view=AccountDetailsPage/>
                </Route>
            </Routes>
        </Router>
    }
}
