use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use gloo_net::http::Request;
use std::fmt;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

const SPC_DAY1_URL: &str = "https://mapservices.weather.noaa.gov/vector/rest/services/outlooks/SPC_wx_outlks/MapServer/1/query";
const SPC_DAY1_TOR_URL: &str = "https://mapservices.weather.noaa.gov/vector/rest/services/outlooks/SPC_wx_outlks/MapServer/3/query";
const SPC_DAY1_WIND_URL: &str = "https://mapservices.weather.noaa.gov/vector/rest/services/outlooks/SPC_wx_outlks/MapServer/7/query";
const SPC_DAY1_HAIL_URL: &str = "https://mapservices.weather.noaa.gov/vector/rest/services/outlooks/SPC_wx_outlks/MapServer/5/query";

const SPC_DAY2_URL: &str = "https://mapservices.weather.noaa.gov/vector/rest/services/outlooks/SPC_wx_outlks/MapServer/9/query";
const SPC_DAY2_TOR_URL: &str = "https://mapservices.weather.noaa.gov/vector/rest/services/outlooks/SPC_wx_outlks/MapServer/11/query";

const SPC_DAY3_URL: &str = "https://mapservices.weather.noaa.gov/vector/rest/services/outlooks/SPC_wx_outlks/MapServer/17/query";


#[derive(Debug, Deserialize)]
struct ArcGisResponse {
    features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
struct Feature {
    attributes: Attributes,
}

#[derive(Debug, Deserialize)]
struct Attributes {
    #[serde(rename = "dn")]
    dn: i32,
    #[serde(rename = "label2")]
    label2: String,
    #[serde(rename = "issue")]
    issue: Option<String>,
}

#[derive(Debug)]
enum RiskCategory {
    Thunderstorms,
    Marginal,
    Slight,
    Enhanced,
    Moderate,
    High,
}

impl RiskCategory {
    fn from_dn(dn: i32) -> Option<Self> {
        match dn {
            2 => Some(Self::Thunderstorms),
            3 => Some(Self::Marginal),
            4 => Some(Self::Slight),
            5 => Some(Self::Enhanced),
            6 => Some(Self::Moderate),
            7 => Some(Self::High),
            _ => None,
        }
    }
}

impl fmt::Display for RiskCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

async fn fetch_tor_risk() -> Result<i32, String> {
    let latitude = 34.7382;
    let longitude = -86.6018;

    let url = format!("{SPC_DAY1_TOR_URL}?f=json&geometry={longitude},{latitude}&geometryType=esriGeometryPoint\
         &inSR=4326&spatialRel=esriSpatialRelIntersects&outFields=*");

    let response: ArcGisResponse = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(feature) = response.features.first() {
        Ok(feature.attributes.dn)
    } else {
        Ok(0)
    }
}

async fn fetch_wind_risk() -> Result<i32, String> {
    let latitude = 34.7382;
    let longitude = -86.6018;

    let url = format!("{SPC_DAY1_WIND_URL}?f=json&geometry={longitude},{latitude}&geometryType=esriGeometryPoint\
         &inSR=4326&spatialRel=esriSpatialRelIntersects&outFields=*");

    let response: ArcGisResponse = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(feature) = response.features.first() {
        Ok(feature.attributes.dn)
    } else {
        Ok(0)
    }
}

async fn fetch_hail_risk() -> Result<i32, String> {
    let latitude = 34.7382;
    let longitude = -86.6018;

    let url = format!("{SPC_DAY1_HAIL_URL}?f=json&geometry={longitude},{latitude}&geometryType=esriGeometryPoint\
         &inSR=4326&spatialRel=esriSpatialRelIntersects&outFields=*");

    let response: ArcGisResponse = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(feature) = response.features.first() {
        Ok(feature.attributes.dn)
    } else {
        Ok(0)
    }
}

async fn fetch_cat_risk() -> Result<Option<RiskCategory>, String> {
    let latitude = 34.7382;
    let longitude = -86.6018;

    let url = format!("{SPC_DAY1_URL}?f=json&geometry={longitude},{latitude}&geometryType=esriGeometryPoint\
         &inSR=4326&spatialRel=esriSpatialRelIntersects&outFields=*");

    let response: ArcGisResponse = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(feature) = response.features.first() {
        Ok(RiskCategory::from_dn(feature.attributes.dn))
    } else {
        Ok(None)
    }
}

#[component]
fn Outlook() -> Html {
    let image_url = "https://www.spc.noaa.gov/products/outlook/day1otlk.gif";

    html! {
        <div>
            <img src={image_url} alt="Day 1 Outlook" />
        </div>
    }
}

#[component]
fn CategoricalRisk() -> Html {
    let risk = use_state(|| None::<RiskCategory>);
    {
        let risk = risk.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let result = fetch_cat_risk().await;
                if let Ok(r) = result {
                    risk.set(r);
                }
            });
            || ()
        });
    }

    html! {
        <div>
            {
                match &*risk {
                    Some(r) => {
                        let caps = r.to_string().to_uppercase();
                        let color = format!("categorical-risk-{}", r.to_string().to_lowercase());
                        html! { <p1 class={color}>{format!("{caps}")}</p1> }
                    },
                    None => html! { "None" },
                }
            }
        </div>
    }
}

#[component]
fn TornadoRisk() -> Html {
    let percentage = use_state(|| None::<i32>);
    {
        let percentage = percentage.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let result = fetch_tor_risk().await;
                if let Ok(p) = result {
                    percentage.set(Some(p));
                }
            });
            || ()
        });
    }

    html! {
        <div>
            {
                match &*percentage {
                    Some(p) => {
                        let color = format!("tornado-{p}");
                        html! { <p1 class={color}>{format!("{p}%")}</p1> }
                    },
                    None => html! { "None" },
                }
            }
        </div>
    }
}

#[component]
fn WindRisk() -> Html {
    let percentage = use_state(|| None::<i32>);
    {
        let percentage = percentage.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let result = fetch_wind_risk().await;
                if let Ok(p) = result {
                    percentage.set(Some(p));
                }
            });
            || ()
        });
    }

    html! {
        <div>
            {
                match &*percentage {
                    Some(p) => {
                        let color = format!("tornado-{p}");
                        html! { <p1 class={color}>{format!("{p}%")}</p1> }
                    },
                    None => html! { "None" },
                }
            }
        </div>
    }
}

#[component]
fn HailRisk() -> Html {
    let percentage = use_state(|| None::<i32>);
    {
        let percentage = percentage.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let result = fetch_hail_risk().await;
                if let Ok(p) = result {
                    percentage.set(Some(p));
                }
            });
            || ()
        });
    }

    html! {
        <div>
            {
                match &*percentage {
                    Some(p) => {
                        let color = format!("tornado-{p}");
                        html! { <p1 class={color}>{format!("{p}%")}</p1> }
                    },
                    None => html! { "None" },
                }
            }
        </div>
    }
}

#[component]
pub fn App() -> Html {
    html! {
        <body>
            <header>
                <strong>{"Meso"}</strong>
                {" | Weather Dashboard"}
            </header>
            <main class="container">
                <section class="panel">
                    <h2>{"Day 1 Categorical Outlook"}</h2>
                    <CategoricalRisk />
                    <h2>{"Risks by Type"}</h2>
                    <div class="status-grid">
                        <div class="status-item">
                            <span class="label">{"Tornado"}</span>
                            <span class="value"><TornadoRisk /></span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Wind"}</span>
                            <span class="value"><WindRisk /></span>
                        </div>
                        <div class="status-item">
                            <span class="label">{"Hail"}</span>
                            <span class="value"><HailRisk /></span>
                        </div>
                    </div>
                </section>
                <section class="panel">
                    <h2>{"SPC Outlook Map"}</h2>
                    <div class="status-item">
                        <button>{"Categorical"}</button>
                        <button>{"Tornado"}</button>
                        <button>{"Wind"}</button>
                        <button>{"Hail"}</button>
                        <br/>
                        <br/>
                        <Outlook />
                    </div>
                </section>
            </main>
            <footer class="attribution">
                {"created by crhowell3 | v0.1.0"}
            </footer>
        </body>
    }
}
