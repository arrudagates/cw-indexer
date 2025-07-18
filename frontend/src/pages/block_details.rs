use crate::app::fetch_api;
use common::{Block, Transaction};
use leptos::{component, create_resource, view, For, IntoView, SignalGet, SignalWith, Suspense};
use leptos_router::{use_params_map, A};

#[component]
pub fn BlockDetailsPage() -> impl IntoView {
    let params = use_params_map();
    let hash = move || params.with(|p| p.get("hash").cloned().unwrap_or_default());

    let block_resource = create_resource(hash, |h| async move {
        fetch_api::<Block>(&format!("/block/{}", h)).await
    });

    let transactions_resource = create_resource(hash, |h| async move {
        fetch_api::<Vec<Transaction>>(&format!("/block/{}/transactions", h)).await
    });

    view! {
        <Suspense fallback=move || view!{<p>"Loading block data..."</p>}>
            {move || block_resource.get().map(|res| match res {
                Some(block) => view! {
                    <h1 class="title">"Block Details"</h1>
                    <div class="detail-grid">
                        <span>"Hash:"</span>      <span>{block.hash.clone()}</span>
                        <span>"Number:"</span>    <span>{block.number}</span>
                        <span>"Timestamp:"</span> <span>{block.timestamp.to_string()}</span>
                        <span>"Miner:"</span>     <span><A href=format!("/account/{}", block.miner) class="link">{block.miner.clone()}</A></span>
                        <span>"Transactions:"</span> <span>{block.tx_count}</span>
                        <span>"Gas Used:"</span>  <span>{block.gas_used.to_string()}</span>
                        <span>"Gas Limit:"</span> <span>{block.gas_limit.to_string()}</span>

                        <h2 class="subtitle">"Transactions"</h2>
                    <Suspense fallback=move || view!{<p>"Loading transactions..."</p>}>
                        {move || transactions_resource.get().map(|tx_res| match tx_res {
                            Some(transactions) => view! {
                                <div class="table-container">
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>"Tx Hash"</th>
                                                <th>"From"</th>
                                                <th>"To"</th>
                                                <th>"Value"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For
                                                each=move || transactions.clone()
                                                key=|tx| tx.hash.clone()
                                                let:tx
                                            >
                                                <tr>
                                                    <td><A href=format!("/tx/{}", tx.hash) class="link truncate">{tx.hash.clone()}</A></td>
                                                    <td><A href=format!("/account/{}", tx.from_address) class="link truncate">{tx.from_address.clone()}</A></td>
                                                    <td>{
                                                        if let Some(to) = tx.to_address {
                                                            view! { <A href=format!("/account/{}", to) class="link truncate">{to}</A> }.into_view()
                                                        } else {
                                                            view! { <span class="tag">"Contract Creation"</span> }.into_view()
                                                        }
                                                    }</td>
                                                    <td>{tx.value.to_string()}</td>
                                                </tr>
                                            </For>
                                        </tbody>
                                    </table>
                                </div>
                            }.into_view(),
                            None => view! { <p class="error">"Could not load transactions for this block."</p> }.into_view()
                        })}
                    </Suspense>
                    </div>
                }.into_view(),
                None => view! { <p class="error">"Error: Block not found."</p> }.into_view()
            })}
        </Suspense>
    }
}
