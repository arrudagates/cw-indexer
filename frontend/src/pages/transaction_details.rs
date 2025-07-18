use crate::app::fetch_api;
use common::TransactionDetail;
use leptos::{
    component, create_resource, view, CollectView, For, IntoView, SignalGet, SignalWith, Suspense,
};
use leptos_router::{use_params_map, A};

#[component]
pub fn TransactionDetailsPage() -> impl IntoView {
    let params = use_params_map();
    let hash = move || params.with(|p| p.get("hash").cloned().unwrap_or_default());

    let tx_resource = create_resource(hash, |h| async move {
        fetch_api::<TransactionDetail>(&format!("/tx/{}", h)).await
    });

    view! {
        <Suspense fallback=move || view!{<p>"Loading transaction data..."</p>}>
            {move || tx_resource.get().map(|res| match res {
                Some(detail) => {
                    let tx = detail.transaction;
                    let logs = detail.logs;
                    let token_transfers = detail.token_transfers;

                    view! {
                        <h1 class="title">"Transaction Details"</h1>
                        <div class="detail-grid">
                            <span>"Hash:"</span>        <span>{tx.hash.clone()}</span>
                            <span>"Block:"</span>       <span><A href=format!("/block/{}", tx.block_hash) class="link">{tx.block_number}</A></span>
                            <span>"From:"</span>        <span><A href=format!("/account/{}", tx.from_address) class="link">{tx.from_address.clone()}</A></span>
                            <span>"To:"</span>          <span>{
                                if let Some(to) = tx.to_address {
                                    view! { <A href=format!("/account/{}", to) class="link">{to}</A> }.into_view()
                                } else {
                                    view! { "Contract Creation" }.into_view()
                                }
                            }</span>
                            <span>"Value:"</span>       <span>{tx.value.to_string()}</span>
                            <span>"Gas Used:"</span>    <span>{tx.gas_used.map(|g| g.to_string()).unwrap_or_default()}</span>
                            <span>"Gas Price:"</span>   <span>{tx.gas_price.map(|g| g.to_string()).unwrap_or_default()}</span>
                        </div>

                        <h2 class="subtitle">"Token Transfers"</h2>
                        { if token_transfers.is_empty() {
                            view! { <p>"No token transfers in this transaction."</p> }.into_view()
                        } else {
                            view! {
                                <div class="table-container">
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>"Token"</th>
                                                <th>"From"</th>
                                                <th>"To"</th>
                                                <th>"Amount / ID"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For
                                                each=move || token_transfers.clone()
                                                key=|transfer| transfer.id
                                                let:transfer
                                            >
                                                <tr>
                                                    <td><A href=format!("/account/{}", transfer.token_address) class="link truncate">{transfer.token_address}</A></td>
                                                    <td><A href=format!("/account/{}", transfer.from_address) class="link truncate">{transfer.from_address}</A></td>
                                                    <td><A href=format!("/account/{}", transfer.to_address) class="link truncate">{transfer.to_address}</A></td>
                                                    <td>
                                                        {
                                                            if let Some(id) = transfer.token_id {
                                                                format!("NFT ID: {}", id)
                                                            } else {
                                                                transfer.value.unwrap_or_default().to_string()
                                                            }
                                                        }
                                                    </td>
                                                </tr>
                                            </For>
                                        </tbody>
                                    </table>
                                </div>
                            }.into_view()
                        }}

                        <h2 class="subtitle">"Logs"</h2>
                        { if logs.is_empty() {
                           view! { <p>"No logs in this transaction."</p> }.into_view()
                        } else {
                            logs.into_iter().enumerate().map(|(i, log)| view! {
                                <div class="log-entry">
                                    <div class="log-header">
                                        <span class="log-index">{i}</span>
                                        <span class="log-address"><A href=format!("/account/{}", log.address) class="link">{log.address}</A></span>
                                    </div>
                                    <div class="log-topics">
                                        <p><strong>"Topics"</strong></p>
                                        {
                                            let topics = vec![log.topic0, log.topic1, log.topic2, log.topic3];
                                            topics.into_iter().flatten().enumerate().map(|(ti, topic)| view!{
                                               <div class="log-data-item">
                                                   <span>{ti}:</span><span>{topic}</span>
                                               </div>
                                            }).collect_view()
                                        }
                                    </div>
                                    <div class="log-data">
                                        <p><strong>"Data"</strong></p>
                                        <pre>{log.data}</pre>
                                    </div>
                                </div>
                            }).collect_view()
                        }}
                    }.into_view()
                },
                None => view!{ <p class="error">"Error: Transaction not found."</p> }.into_view()
            })}
        </Suspense>
    }
}
