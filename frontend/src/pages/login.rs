use serde::Serialize;
use std::ops::Deref;
use yew::prelude::*;
use yew::Callback;
use yewdux::prelude::*;

use crate::components::atomics::text_input::{TextInput, INPUTTYPE};
use crate::services::backend::authenticate;
use crate::stores::user_store::UserStore;
use crate::Route;
use yew_router::prelude::use_navigator;

#[derive(Serialize, Debug, Default, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[function_component]
pub fn LoginPage() -> Html {
    let navigator = use_navigator().unwrap();

    let (user_state, dispatch) = use_store::<UserStore>();
    let state = use_state(LoginRequest::default);

    let email_on_change = {
        let state = state.clone();
        Callback::from(move |value: String| {
            let mut state_handle = state.deref().clone();
            state_handle.email = value;
            state.set(state_handle);
        })
    };

    let password_on_change = {
        let state = state.clone();
        Callback::from(move |value: String| {
            let mut state_handle = state.deref().clone();
            state_handle.password = value;
            state.set(state_handle);
        })
    };
    let user_state_handle = user_state.clone();

    if user_state_handle.is_authenticated() {
        navigator.push(&Route::Messages);
    };

    let auth_error_message = match &user_state_handle.authentication_error {
        None => "".to_string(),
        Some(e) => e.to_string(),
    };

    let submit = Callback::from(move |_| {
        authenticate(state.deref().clone(), dispatch.clone());
    });
    html! {
        <div class="grid place-items-center">
            <div class="card-base my-10">
                <TextInput id={"email"} placeholder={Some("email")} input_type={Some(INPUTTYPE::Email)} on_change={email_on_change}/>
                <TextInput id={"password"} placeholder={Some("*********")} input_type={Some(INPUTTYPE::Password)} on_change={password_on_change}/>
                <button type="submit" onclick={submit} class="button my-3 px-10">{"Login"}</button>
                if user_state_handle.authentication_error.is_some() {
                    <div class="warning-message">
                        {auth_error_message}
                    </div>
                }
            </div>
        </div>
    }
}
