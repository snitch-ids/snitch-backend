use crate::services::backend::MessageBackend;
use yew::{function_component, html, Html, Properties};

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub message: MessageBackend,
}

#[function_component(MessageCard)]
pub fn message_card(props: &Props) -> Html {
    let message = &props.message;
    html!(
        <tr class="bg-white border-b dark:bg-gray-800 dark:border-gray-700">
            <td class="px-6 py-4">
                { &message.timestamp }
            </td>
            <td scope="row" class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white">
                { &message.title }
            </td>
            <td scope="row" class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white" >
                { &message.body }
            </td>
        </tr>
    )
}
