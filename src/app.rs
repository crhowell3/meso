use regex::Regex;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console::debug;
use yew::prelude::*;
use gloo_net::http::Request;
use std::fmt;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

const ARCGIS_BASE_URL: &str = "https://mapservices.weather.noaa.gov/vector/rest/services/outlooks/SPC_wx_outlks/MapServer/";

// These will be removed in the future in favor of configurability
// Currently hardcoded to Huntsville, AL
const LATITUDE: f64 = 34.7382;
const LONGITUDE: f64 = -86.6018;

#[derive(Debug, PartialEq, Copy, Clone)]
enum MapServer {
    Day1Outlook = 1,
    Day1Tornado = 3,
    Day1Hail = 5,
    Day1Wind = 7,
}

impl MapServer {
    fn get_common_name(&self) -> String {
        match self {
            Self::Day1Outlook => "categorical".to_string(),
            Self::Day1Tornado => "tornado".to_string(),
            Self::Day1Hail => "hail".to_string(),
            Self::Day1Wind => "wind".to_string(),
        }
    }

    fn get_dn(&self) -> i32 {
        match self {
            Self::Day1Outlook => 1,
            Self::Day1Tornado => 3,
            Self::Day1Hail => 5,
            Self::Day1Wind => 7,
        }
    }
}

impl fmt::Display for MapServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Properties, PartialEq)]
struct MapServerProps {
    map_server: MapServer,
}

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
}

async fn fetch_risk(map_server: MapServer) -> Result<i32, String> {
    let dn = map_server.get_dn();
    let url = format!("{ARCGIS_BASE_URL}/{dn}/query?f=json&geometry={LONGITUDE},{LATITUDE}&geometryType=esriGeometryPoint\
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

fn parse_temps(lines: &[&str]) -> Option<(i32, i32)> {
    let mut high = None;
    let mut low = None;

    for line in lines {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        web_sys::console::log_1(&JsValue::from_str(&format!("{:?}", parts).to_string()));
        if parts.first() == Some(&"TXN") {
            low = parts.get(1)?.parse::<i32>().ok();
            high = parts.get(2)?.parse::<i32>().ok();
        }
    }

    match (high, low) {
        (Some(h), Some(l)) => Some((h, l)),
        _ => None,
    }
}

async fn fetch_daycast() -> Result<(i32, i32), String> {
    let url = "https://blend.mdl.nws.noaa.gov/nbm-text-new?ele=NBS&sta=KHSV&cyc=Latest";
    let response = Request::get(&url).send().await.map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string())?;
    let lines: Vec<_> = response.lines().collect();
    let (hi, lo) = parse_temps(&lines).ok_or_else(|| "Temps not found")?;

    Ok((hi, lo))
}

#[component]
fn Outlook() -> Html {
    let state = use_context::<UseStateHandle<AppState>>().expect("AppState context not found");

    html! {
        <div>
            <img src={state.outlook_url.clone()} width="500" alt="Day 1 Outlook" />
        </div>
    }
}

#[component]
fn Climate() -> Html {
    let image_url = "https://www.cpc.ncep.noaa.gov/products/predictions/610day/610temp.new.gif";

    html! {
        <div>
            <img src={image_url} width="500" alt="6-10 Climate Outlook" />
        </div>
    }
}

#[component]
fn GetRisk(MapServerProps { map_server }: &MapServerProps) -> Html {
    let risk = use_state(|| None::<i32>);
    let ms = map_server.clone();
    {
        let risk = risk.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let result = fetch_risk(ms).await;
                if let Ok(r) = result {
                    risk.set(Some(r));
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
                        let risk_name = map_server.get_common_name();
                        let color = format!("{risk_name}-{}", r.to_string().to_lowercase());

                        if risk_name == "categorical" {
                            if *r == 0 {
                                html! { <p1 class={color}>{"NONE"}</p1> }
                            } else {
                                let caps = r.to_string().to_uppercase();
                                html! { <p1 class={color}>{format!("{caps}")}</p1> }
                            }
                        } else {
                            html! { <p1 class={color}>{format!("{r}%")}</p1> }
                        }
                    },
                    None => html! { "None" },
                }
            }
        </div>
    }
}

#[component]
fn GetTemp() -> Html {
    let temps = use_state(|| (None::<i32>, None::<i32>));

    {
        let temps = temps.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let result = fetch_daycast().await;
                if let Ok(r) = result {
                    temps.set((Some(r.0), Some(r.1)));
                } else {
                    web_sys::console::log_1(&JsValue::from_str(&format!("{:?}", result).to_string()));
                }
            });
            || ()
        });
    }

    html! {
        <div>
            {
                match &*temps {
                    (Some(h), Some(l)) => {
                        html! {
                            <>
                                <p style="font-size: 16pt;">{format!("{h}°F")}</p>
                                <p>{format!("{l}°F")}</p>
                            </>
                        }
                    }
                    _ => {
                        html! {
                            <>
                                <p style="font-size: 16pt;">{"-"}</p>
                                <p>{"-"}</p>
                            </>
                        }
                    }
                }
            }
        </div>
    }
}

#[derive(Clone, PartialEq)]
pub struct AppState {
    pub outlook_url: String,
}

impl AppState {
    pub fn new() -> Self {
        let outlook_url = "https://www.spc.noaa.gov/products/outlook/day1otlk.gif";
        Self {
            outlook_url: outlook_url.to_string(),
        }
    }
}

#[component]
pub fn OutlookButtons() -> Html {
    let state = use_context::<UseStateHandle<AppState>>().expect("AppsState not found");
    let change_outlook = |src: &'static str| {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(AppState {
                outlook_url: src.to_string(),
            });
        })
    };

    html! {
        <>
            <button style="margin-right: 16px; width: 100px;" onmouseenter={change_outlook("https://www.spc.noaa.gov/products/outlook/day1otlk.gif")}>{"Categorical"}</button>
            <button style="margin-right: 16px; width: 100px;" onmouseenter={change_outlook("https://www.spc.noaa.gov/products/outlook/day1probotlk_torn.gif")}>{"Tornado"}</button>
            <button style="margin-right: 16px; width: 100px;" onmouseenter={change_outlook("https://www.spc.noaa.gov/products/outlook/day1probotlk_wind.gif")}>{"Wind"}</button>
            <button style="width: 100px;" onmouseenter={change_outlook("https://www.spc.noaa.gov/products/outlook/day1probotlk_hail.gif")}>{"Hail"}</button>
        </>
    }
}

#[component]
pub fn App() -> Html {
    let app_state = use_state(AppState::new);

    html! {
        <ContextProvider<UseStateHandle<AppState>> context={app_state}>
            <body>
                <header>
                    <strong>{"Meso"}</strong>
                    {" | Weather Dashboard"}
                </header>
                <main class="container" style="align-items: center;">
                    <div class="status-row">
                        <section class="panel">
                            <h2>{"Daycast"}</h2>
                            <GetTemp />
                        </section>
                        <section class="panel">
                            <h2>{"Day 1 Categorical Outlook"}</h2>
                            <GetRisk map_server={MapServer::Day1Outlook} />
                            <h2>{"Risks by Type"}</h2>
                            <div class="status-grid">
                                <div class="status-row">
                                    <div class="status-item" style="width: 150px;">
                                        <span class="label">{"Tornado"}</span>
                                        <span class="value"><GetRisk map_server={MapServer::Day1Tornado} /></span>
                                    </div>
                                    <div class="status-item" style="width: 150px;">
                                        <span class="label">{"Wind"}</span>
                                        <span class="value"><GetRisk map_server={MapServer::Day1Wind} /></span>
                                    </div>
                                    <div class="status-item" style="width: 150px;">
                                        <span class="label">{"Hail"}</span>
                                        <span class="value"><GetRisk map_server={MapServer::Day1Hail} /></span>
                                    </div>
                                </div>
                            </div>
                        </section>
                    </div>
                    <section class="panel" style="width: 675px;">
                        <h2>{"SPC Outlook Map"}</h2>
                        <div class="status-item">
                            <OutlookButtons />
                            <br/>
                            <br/>
                            <Outlook />
                        </div>
                        <h2>{"6-10 Day Climate Outlook"}</h2>
                        <div class="status-item">
                            <Climate />
                        </div>
                    </section>
                </main>
                <footer class="attribution">
                    {"created by crhowell3 | v0.1.0"}
                </footer>
            </body>
        </ContextProvider<UseStateHandle<AppState>>>
    }
}
