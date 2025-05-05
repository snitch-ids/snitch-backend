use crate::components::token_list::TokenList;
use crate::services::backend::create_token;
use yew::prelude::*;

#[function_component]
pub fn Token() -> Html {
    let updated = use_state(|| false);
    let updated_handle = updated.clone();

    let create_token_callback = Callback::from(move |_| {
        let updated_handle = updated_handle.clone();
        wasm_bindgen_futures::spawn_local(async move {
            create_token().await;
            updated_handle.set(!*updated_handle);
        });
    });
    html!(
        <div class="card">
            <div class="card-content">
                <TokenList updated={*updated}/>
                <div class="grid w-36 pt-5">
                    <button class="button justify-items-end" onclick={create_token_callback}>
                        { "create token" }
                    </button>
                </div>
            </div>
        </div>
    )
}
