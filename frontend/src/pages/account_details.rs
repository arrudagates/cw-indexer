use crate::app::fetch_api;
use common::AccountDetail;
use leptos::{component, create_resource, view, For, IntoView, SignalGet, SignalWith, Suspense};
use leptos_router::{use_params_map, A};

#[component]
pub fn AccountDetailsPage() -> impl IntoView {
    let params = use_params_map();
    let address = move || params.with(|p| p.get("address").cloned().unwrap_or_default());

    let account_resource = create_resource(address, |addr| async move {
        fetch_api::<AccountDetail>(&format!("/account/{}", addr)).await
    });

    view! {
        <Suspense fallback=move || view!{<p>"Loading account data..."</p>}>
            {move || account_resource.get().map(|res| match res {
                Some(detail) => view! {
                    <h1 class="title">"Account Details"</h1>
                    <p class="address-header">{detail.address}</p>

                    <h2 class="subtitle">"Token Balances"</h2>
                     <div class="table-container">
                        <table>
                            <thead>
                                <tr>
                                    <th>"Token Address"</th>
                                    <th>"Amount / Token ID"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || detail.token_balances.clone()
                                    key=|balance| balance.id
                                    let:balance
                                >
                                    <tr>
                                        <td><A href=format!("/account/{}", balance.token_address) class="link truncate">{balance.token_address}</A></td>
                                        <td>
                                            {
                                                if let Some(id) = balance.token_id {
                                                    format!("NFT ID: {}", id)
                                                } else {
                                                    balance.amount.to_string()
                                                }
                                            }
                                        </td>
                                    </tr>
                                </For>
                            </tbody>
                        </table>
                    </div>
                }.into_view(),
                None => view!{ <p class="error">"Error: Account not found."</p> }.into_view()
            })}
        </Suspense>
    }
}
