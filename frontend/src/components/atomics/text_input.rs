use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub id: String,
    pub on_change: Callback<String>,
    pub input_type: Option<INPUTTYPE>,
    pub placeholder: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum INPUTTYPE {
    Email,
    Password,
}

fn get_input_type(props: &Props) -> String {
    match props.clone().input_type {
        Some(INPUTTYPE::Email) => "email".to_string(),
        Some(INPUTTYPE::Password) => "password".to_string(),
        None => "text".to_string(),
    }
}

fn get_placeholder(props: &Props) -> String {
    props.clone().placeholder.unwrap_or("".to_string())
}

#[function_component]
pub fn TextInput(props: &Props) -> Html {
    let state = use_state(String::new);
    let onchange = {
        let emit_on_change = props.on_change.clone();
        let state_change_handle = state.clone();
        Callback::from(move |event: Event| {
            let value = event
                .target()
                .unwrap()
                .unchecked_into::<HtmlInputElement>()
                .value();
            emit_on_change.emit(value.clone());
            state_change_handle.set(value);
        })
    };

    let input_type = get_input_type(props);
    let placeholder = get_placeholder(props);
    html! {
        <div>
            <label for={props.id.clone()} class="block mt-2 mb-1 text-sm font-medium text-gray-900 dark:text-white">{props.id.clone()}</label>
            <input type={input_type} onchange={onchange} id={props.id.clone()} class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-5002" placeholder={placeholder} required=true />
        </div>
    }
}
