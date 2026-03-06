use gloo_net::http::Request;
use serde::Deserialize;
use std::fmt;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

const ARCGIS_BASE_URL: &str =
    "https://mapservices.weather.noaa.gov/vector/rest/services/outlooks/SPC_wx_outlks/MapServer/";

// These will be removed in the future in favor of configurability
// Currently hardcoded to Huntsville, AL
const LATITUDE: f64 = 34.7382;
const LONGITUDE: f64 = -86.6018;

#[derive(Debug, PartialEq, Copy, Clone)]
enum MapServer {
    Outlook = 1,
    Tornado = 3,
    Hail = 5,
    Wind = 7,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Categories {
    NoThunder = 0,
    Thunderstorms = 2,
    Marginal = 3,
    Slight = 4,
    Enhanced = 5,
    Moderate = 6,
    High = 7,
}

impl Categories {
    fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::NoThunder),
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

impl fmt::Display for Categories {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoThunder => write!(f, "none"),
            Self::Thunderstorms => write!(f, "thunderstorms"),
            Self::Marginal => write!(f, "marginal"),
            Self::Slight => write!(f, "slight"),
            Self::Enhanced => write!(f, "enhanced"),
            Self::Moderate => write!(f, "moderate"),
            Self::High => write!(f, "high"),
        }
    }
}

impl MapServer {
    fn get_common_name(self) -> String {
        match self {
            Self::Outlook => "categorical".to_string(),
            Self::Tornado => "tornado".to_string(),
            Self::Hail => "hail".to_string(),
            Self::Wind => "wind".to_string(),
        }
    }

    fn get_dn(self) -> i32 {
        match self {
            Self::Outlook => 1,
            Self::Tornado => 3,
            Self::Hail => 5,
            Self::Wind => 7,
        }
    }
}

impl fmt::Display for MapServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
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
    let url = format!(
        "{ARCGIS_BASE_URL}/{dn}/query?f=json&geometry={LONGITUDE},{LATITUDE}&geometryType=esriGeometryPoint\
         &inSR=4326&spatialRel=esriSpatialRelIntersects&outFields=*"
    );

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
    let ms = *map_server;
    {
        let risk = risk.clone();
        use_effect_with((), move |()| {
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
                                html! { <p1 class={"categorical-none"}>{"NO THUNDER"}</p1> }
                            } else {
                                let category = Categories::from_i32(*r).unwrap_or(Categories::NoThunder);
                                let cat_name = category.to_string();
                                let color = format!("categorical-{cat_name}");
                                let caps = cat_name.to_uppercase();
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

#[derive(Clone, PartialEq)]
pub struct AppState {
    pub outlook_url: String,
}

impl AppState {
    pub fn new() -> Self {
        let outlook_url = "https://www.spc.noaa.gov/products/outlook/day1otlk.png";
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
            <button style="margin-right: 16px; width: 100px;" onmouseenter={change_outlook("https://www.spc.noaa.gov/products/outlook/day1otlk.png")}>{"Categorical"}</button>
            <button style="margin-right: 16px; width: 100px;" onmouseenter={change_outlook("https://www.spc.noaa.gov/products/outlook/day1probotlk_torn.png")}>{"Tornado"}</button>
            <button style="margin-right: 16px; width: 100px;" onmouseenter={change_outlook("https://www.spc.noaa.gov/products/outlook/day1probotlk_wind.png")}>{"Wind"}</button>
            <button style="width: 100px;" onmouseenter={change_outlook("https://www.spc.noaa.gov/products/outlook/day1probotlk_hail.png")}>{"Hail"}</button>
        </>
    }
}

#[component]
pub fn App() -> Html {
    let app_state = use_state(AppState::new);

    let reload = Callback::from(move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().reload();
        }
    });

    html! {
        <ContextProvider<UseStateHandle<AppState>> context={app_state}>
            <body>
                <header>
                    <strong>{"meso"}</strong>
                    {" | severe wx dashboard"}
                    <button onclick={reload}>{"Refresh"}</button>
                </header>
                <main class="container" style="align-items: center;">
                    <div class="status-row">
                        <section class="panel" style="width: 675px;">
                            <h2>{"Day 1 Categorical Outlook"}</h2>
                            <GetRisk map_server={MapServer::Outlook} />
                            <h2>{"Risks by Type"}</h2>
                            <div class="status-grid">
                                <div class="status-row">
                                    <div class="status-item" style="width: 150px;">
                                        <span class="label">{"Tornado"}</span>
                                        <span class="value"><GetRisk map_server={MapServer::Tornado} /></span>
                                    </div>
                                    <div class="status-item" style="width: 150px;">
                                        <span class="label">{"Wind"}</span>
                                        <span class="value"><GetRisk map_server={MapServer::Wind} /></span>
                                    </div>
                                    <div class="status-item" style="width: 150px;">
                                        <span class="label">{"Hail"}</span>
                                        <span class="value"><GetRisk map_server={MapServer::Hail} /></span>
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
