use web_sys::console::log_1;
use yew::prelude::*;

use yew::{function_component, html, use_effect_with, use_state, Html};

use crate::components::message::MessageCard;
use crate::services::backend::request_messages;

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub hostname: String,
}

#[function_component]
pub fn MessageList(props: &Props) -> Html {
    log::info!("Fetching data for {}", &props.hostname);
    let hostname = props.hostname.clone();

    let messages = use_state(std::vec::Vec::new);
    {
        let messages = messages.clone();
        use_effect_with(props.hostname.clone(), move |_| {
            let messages = messages.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_messages = request_messages(&hostname).await.unwrap();
                messages.set(fetched_messages);
            });
            || ()
        });
    }

    let message_cards = messages.iter().rev().map(|message| {
        html! {
            <MessageCard message={message.clone()}/>
        }
    });

    html! {
        <>
            <div class="relative overflow-x-auto">
                <table class="w-full text-sm text-left text-gray-500 dark:text-gray-400">
                    <thead class="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400">
                        <tr>
                            <th scope="col" class="px-6 py-3">
                                {"Time"}
                            </th>
                            <th scope="col" class="px-6 py-3">
                                {"Title"}
                            </th>
                            <th scope="col" class="px-6 py-3">
                                {"Details"}
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        { for message_cards }
                    </tbody>
                </table>
            </div>
        </>
    }
}
