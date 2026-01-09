use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
fn Outlook() -> Html {
    let image_url = "https://www.spc.noaa.gov/products/outlook/day1otlk_2000.gif";

    html! {
        <div>
            <img src={image_url} alt="Day 2 Outlook" />
        </div>
    }
}

#[component]
pub fn App() -> Html {
    html! {
        <body>
            <header>
                <strong>{"Meso"}</strong>
                {" Weather Dashboard"}
            </header>
            <main class="container">
                <div>
                    <p1>{"SPC Day 1 Convective Outlook for HSV:"}</p1>
                    <br/>
                    <p1 class={"risk-thunder"}>{"THUNDERSTORMS"}</p1>
                    <br/>
                    <br/>
                    <Outlook />
                </div>
            </main>
            <footer class="attribution">
                {"created by crhowell3 | v0.1.0"}
            </footer>
        </body>
    }
}
