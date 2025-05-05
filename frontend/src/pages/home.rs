use yew::prelude::*;

const GIT_HASH: &str = env!("GIT_HASH");

const ASSETS_BASE_PATH: &str = if cfg!(debug_assertions) {
    "img/" // Development path
} else {
    "" // Production path (root)
};

#[function_component]
pub fn Home() -> Html {
    html! {
       <section>
        <div class="grid max-w-screen-xl px-4 py-8 mx-auto lg:gap-8 xl:gap-0 lg:py-16 lg:grid-cols-12">
            <div class="mr-auto place-self-center lg:col-span-7">
                <h1 class="max-w-2xl mb-4 text-4xl font-extrabold tracking-tight leading-none md:text-5xl xl:text-6xl dark:text-white">{"Snitch"}</h1>
                <p class="max-w-2xl mb-6 font-light text-gray-500 lg:mb-8 md:text-lg lg:text-xl dark:text-gray-400">{"Make sure nobody fumbles with your files."}</p>
                <a href="https://github.com/snitch-id/snitch#installation" class="inline-flex items-center justify-center px-5 py-3 mr-3 text-base font-medium text-center dark:text-white rounded-lg bg-primary-700 hover:bg-gray-600 focus:ring-4 focus:ring-primary-300 dark:focus:ring-primary-900">
                    <svg class="w-5 h-5 ml-2 -ml-2" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M10.293 3.293a1 1 0 011.414 0l6 6a1 1 0 010 1.414l-6 6a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-4.293-4.293a1 1 0 010-1.414z" clip-rule="evenodd"></path></svg>
                    {"Get started"}
                </a>
                <a href="/register" class="inline-flex items-center justify-center px-5 py-3 text-base font-medium text-center text-gray-900 border border-gray-300 rounded-lg hover:bg-gray-100 focus:ring-4 focus:ring-gray-100 dark:text-white dark:border-gray-700 dark:hover:bg-gray-700 dark:focus:ring-gray-800">
                    {"Register"}
                </a>
            </div>
            <div class="hidden lg:mt-0 col-span-3 lg:flex dark:drop-shadow-[0_2px_45px_rgba(156,255,138,0.4)]">
                <img src={format!("{}lock.svg", ASSETS_BASE_PATH)} alt="lock"/>
            </div>
        </div>
        <footer>
            <div class="mx-auto w-full container p-4 sm:p-6">
                <hr class="my-6 border-gray-200 sm:mx-auto dark:border-gray-700 lg:my-8" />
                <div class="sm:flex sm:items-center sm:justify-between">
                    <div class="text-gray-500">
                        {"build: "} <a href="https://github.com/snitch-ids/snitch-frontend/commit/"{GIT_HASH}>{&GIT_HASH[0..8]}</a>
                    </div>
                </div>
            </div>
        </footer>
    </section>
    }
}
