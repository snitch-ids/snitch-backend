use yew::prelude::*;
use yewdux::prelude::*;

use crate::services::backend::logout;
use crate::stores::user_store::UserStore;

#[function_component]
pub fn Logout() -> Html {
    let (_, dispatch) = use_store::<UserStore>();

    logout(dispatch);

    html! {
        <div class="card">
            <div class="info-message">
                {"You are logged out now."}
            </div>
        </div>
    }
}
