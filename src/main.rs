use askama_axum::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use chrono::{DateTime, Local};
use listenfd::ListenFd;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tower_http::services::ServeDir;

#[derive(Serialize, Deserialize, Debug, Default)]
enum OnlineState {
    Online,
    #[default]
    Offline,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetDeviceResponse {
    remotecontrol_id: Option<String>,
    device_id: Option<String>,
    userid: Option<String>,
    alias: Option<String>,
    groupid: Option<String>,
    description: Option<String>,
    // pessimistic default to show offline
    #[serde(default)]
    online_state: OnlineState,
    policy_id: Option<String>,
    assigned_to: Option<bool>,
    supported_features: Option<String>,
    last_seen: Option<DateTime<Local>>,
    teamviewer_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetAllDevicesResponse {
    // technically optional but I doubt this is ever omitted
    #[serde(default)]
    devices: Vec<GetDeviceResponse>,
}

#[derive(Clone)]
struct AppState {
    client: Client,
    teamviewer_token: String,
}

enum AppError {
    RequestError(reqwest::Error),
    ApiResponseError(reqwest::Error),
}

impl From<reqwest::Error> for AppError {
    fn from(inner: reqwest::Error) -> Self {
        if inner.is_decode() {
            AppError::ApiResponseError(inner)
        } else {
            AppError::RequestError(inner)
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::RequestError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
            Self::ApiResponseError(e) => {
                // TODO use serde_json and provide better error message?
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}

mod filters {
    // This filter requires a `usize` input when called in templates
    pub fn debug(stuff: impl std::fmt::Debug) -> ::askama::Result<String> {
        Ok(format!("{:?}", stuff))
    }
}

#[derive(Template)] // this will generate the code...
#[template(path = "index.html")] // using the template in this path, relative
                                 // to the `templates` dir in the crate root
struct IndexTemplate {
    now: String,
    devices: Vec<GetDeviceResponse>,
}

async fn root(State(state): State<AppState>) -> IndexTemplate {
    let resp = state
        .client
        .get("https://webapi.teamviewer.com/api/v1/devices")
        .bearer_auth(state.teamviewer_token)
        .send()
        .await
        .unwrap();
    let content = resp.json::<GetAllDevicesResponse>().await.unwrap();
    IndexTemplate {
        now: Local::now().to_rfc3339().into(),
        devices: content.devices,
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let state = AppState {
        client: reqwest::Client::new(),
        teamviewer_token: env::var("TEAMVIEWER_TOKEN").expect("Teamviewer token must be provided"),
    };

    // build our application with a single route
    let app = Router::new()
        .route("/", get(root))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let server_builder = if let Ok(Some(listener)) = ListenFd::from_env().take_tcp_listener(0) {
        eprintln!("Using socket");
        axum::Server::from_tcp(listener).unwrap()
    } else {
        eprintln!("Using :3000");
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
    };

    server_builder.serve(app.into_make_service()).await.unwrap()
}
