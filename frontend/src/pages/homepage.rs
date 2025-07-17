use crate::app::fetch_api;
use common::Block;
use leptos::{component, create_resource, view, For, IntoView, SignalGet, Suspense};
use leptos_router::A;

#[component]
pub fn HomePage() -> impl IntoView {
    let blocks_resource = create_resource(
        || (),
        |()| async move { fetch_api::<Vec<Block>>("/blocks").await },
    );

    view! {
        <h1 class="title">"Latest Blocks"</h1>
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            {move || blocks_resource.get().map(|res| match res {
                Some(blocks) => view! {
                    <div class="table-container">
                        <table>
                            <thead>
                                <tr>
                                    <th>"Block"</th>
                                    <th>"Hash"</th>
                                    <th>"Miner"</th>
                                    <th>"Transactions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || blocks.clone()
                                    key=|block| block.hash.clone()
                                    let:block
                                >
                                    <tr>
                                        <td><A href=format!("/block/{}", block.hash) class="link">{block.number}</A></td>
                                        <td><A href=format!("/block/{}", block.hash) class="link truncate">{block.hash.clone()}</A></td>
                                        <td><A href=format!("/account/{}", block.miner) class="link truncate">{block.miner.clone()}</A></td>
                                        <td>{block.tx_count}</td>
                                    </tr>
                                </For>
                            </tbody>
                        </table>
                    </div>
                }.into_view(),
                None => view! { <p class="error">"Error: Could not fetch recent blocks."</p> }.into_view(),
            })}
        </Suspense>
    }
}
