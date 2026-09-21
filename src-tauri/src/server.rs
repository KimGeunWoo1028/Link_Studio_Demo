use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::cert::{self, CertBundle};
use crate::db::{self, Camera, Project};
use crate::net::{
    install_lan_inbound_rules, lan_ipv4s, list_lan_interfaces, new_session_id, primary_lan_ipv4,
    recommended_ipv4,
};
use crate::signaling::{CameraSlot, Hub, HubError, Outgoing, PgmState, Role};

const HTTP_PORT: u16 = 8787;
const HTTPS_PORT: u16 = 8443;

#[derive(Clone)]
pub struct AppState {
    pub hub: Arc<Mutex<Hub>>,
    pub db: Arc<Mutex<Connection>>,
    pub sockets: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>>,
    pub http_port: u16,
    pub https_port: u16,
    pub ca_pem: PathBuf,
    pub dist_dir: PathBuf,
    pub cert_dir: PathBuf,
    pub tls: Option<RustlsConfig>,
}

#[derive(Clone)]
pub struct StudioConfig {
    pub data_dir: PathBuf,
    pub dist_dir: PathBuf,
}

#[derive(Serialize)]
pub struct RuntimeInfo {
    pub http_base: String,
    pub https_base: String,
    pub lan_ips: Vec<String>,
    #[serde(default)]
    pub interfaces: Vec<crate::net::LanInterface>,
    pub recommended_ipv4: Option<String>,
    pub bind_http: String,
    pub bind_https: String,
    pub ca_url: String,
    pub ios_profile_url: String,
    pub android_ca_url: String,
    pub http_port: u16,
    pub https_port: u16,
    pub active_session_id: Option<String>,
    pub last_project_id: Option<String>,
}

pub async fn run_studio(config: StudioConfig) -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    std::fs::create_dir_all(&config.data_dir)?;
    let db_path = config.data_dir.join("link-studio.sqlite");
    let conn = db::open(&db_path)?;
    let cert_dir = config.data_dir.join("certs");
    let mut ips = lan_ipv4s();
    ips.push("127.0.0.1".parse()?);
    let certs = crate::cert::ensure_tls(&cert_dir, &[], &ips)?;
    let tls = load_rustls(&certs).await?;

    let mut hub = Hub::default();
    for session_id in db::list_session_ids(&conn)? {
        hub.open_session(session_id);
    }
    if let Ok(Some(session_id)) = db::setting(&conn, "active_session_id") {
        hub.open_session(session_id);
    }

    let state = AppState {
        hub: Arc::new(Mutex::new(hub)),
        db: Arc::new(Mutex::new(conn)),
        sockets: Arc::new(Mutex::new(HashMap::new())),
        http_port: HTTP_PORT,
        https_port: HTTPS_PORT,
        ca_pem: certs.ca_pem.clone(),
        dist_dir: config.dist_dir,
        cert_dir,
        tls: Some(tls.clone()),
    };

    let http_app = router(state.clone());
    let https_app = router(state.clone());

    let http_addr = SocketAddr::from(([0, 0, 0, 0], HTTP_PORT));
    let https_addr = SocketAddr::from(([0, 0, 0, 0], HTTPS_PORT));

    tracing::info!("HTTP listening on {http_addr}");
    tracing::info!("HTTPS listening on {https_addr}");
    if let Some(ip) = primary_lan_ipv4() {
        tracing::info!("Camera URL base https://{ip}:{HTTPS_PORT}");
    }

    let http = {
        let app = http_app;
        async move {
            let listener = tokio::net::TcpListener::bind(http_addr).await?;
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await?;
            anyhow::Ok(())
        }
    };

    let tls = match state.tls.clone() {
        Some(config) => config,
        None => load_rustls(&certs).await?,
    };
    let https = async move {
        axum_server::bind_rustls(https_addr, tls)
            .serve(https_app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
        anyhow::Ok(())
    };

    tokio::try_join!(http, https)?;
    Ok(())
}

async fn load_rustls(certs: &CertBundle) -> anyhow::Result<RustlsConfig> {
    Ok(RustlsConfig::from_pem_file(&certs.cert_pem, &certs.key_pem).await?)
}

fn router(state: AppState) -> Router {
    let assets = ServeDir::new(state.dist_dir.join("assets"));

    Router::new()
        .route("/api/health", get(health))
        .route("/api/runtime", get(runtime))
        .route("/api/projects", get(list_projects).post(create_project))
        .route("/api/projects/{id}", get(get_project))
        .route("/api/projects/{id}/cameras", get(list_cameras))
        .route("/api/cameras/{id}", patch(rename_camera))
        .route("/api/projects/{id}/session", post(start_session))
        .route("/api/sessions/{id}", get(get_session))
        .route("/api/settings", get(get_settings).put(put_settings))
        .route("/api/network/firewall", post(install_firewall))
        .route("/ca.pem", get(download_ca_pem))
        .route("/ca.crt", get(download_ca_crt))
        .route("/ios/link-studio-ca.mobileconfig", get(download_ios_profile))
        .route("/install-ca", get(install_ca_page))
        .route("/ws", get(ws_upgrade))
        .nest_service("/assets", assets)
        .fallback(get(spa_index))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn spa_index(State(state): State<AppState>) -> Response {
    let path = state.dist_dir.join("index.html");
    match tokio::fs::read(&path).await {
        Ok(bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "frontend bundle missing").into_response(),
    }
}

async fn health() -> &'static str {
    "ok"
}

fn runtime_from(state: &AppState) -> RuntimeInfo {
    let interfaces = list_lan_interfaces();
    let lan_ips = interfaces
        .iter()
        .filter(|iface| iface.selectable)
        .map(|iface| iface.ipv4.clone())
        .collect::<Vec<_>>();
    let recommended = recommended_ipv4(&interfaces);
    let host = recommended
        .clone()
        .or_else(|| lan_ips.first().cloned())
        .unwrap_or_else(|| "127.0.0.1".into());
    let db = state.db.lock().expect("db lock");
    RuntimeInfo {
        http_base: format!("http://{host}:{}", state.http_port),
        https_base: format!("https://{host}:{}", state.https_port),
        ca_url: format!("http://{host}:{}/ca.crt", state.http_port),
        ios_profile_url: format!(
            "http://{host}:{}/ios/link-studio-ca.mobileconfig",
            state.http_port
        ),
        android_ca_url: format!("http://{host}:{}/ca.crt", state.http_port),
        lan_ips,
        interfaces,
        recommended_ipv4: recommended,
        bind_http: format!("0.0.0.0:{}", state.http_port),
        bind_https: format!("0.0.0.0:{}", state.https_port),
        http_port: state.http_port,
        https_port: state.https_port,
        active_session_id: db::setting(&db, "active_session_id").ok().flatten(),
        last_project_id: db::setting(&db, "last_project_id").ok().flatten(),
    }
}

async fn runtime(State(state): State<AppState>) -> Json<RuntimeInfo> {
    let _ = refresh_server_cert(&state).await;
    Json(runtime_from(&state))
}

async fn refresh_server_cert(state: &AppState) -> anyhow::Result<()> {
    let mut ips = lan_ipv4s();
    ips.push("127.0.0.1".parse()?);
    let cert_pem_path = state.cert_dir.join("server.pem");
    let key_pem_path = state.cert_dir.join("server-key.pem");
    if let Ok(existing) = std::fs::read_to_string(&cert_pem_path) {
        if crate::cert::server_cert_covers_ips(&existing, &ips) {
            return Ok(());
        }
    }
    crate::cert::ensure_tls(&state.cert_dir, &[], &ips)?;
    if let Some(tls) = &state.tls {
        tls.reload_from_pem_file(&cert_pem_path, &key_pem_path).await?;
    }
    Ok(())
}

async fn download_ca_pem(State(state): State<AppState>) -> impl IntoResponse {
    match std::fs::read(&state.ca_pem) {
        Ok(bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/x-pem-file"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"link-studio-ca.pem\"",
                ),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "CA not ready").into_response(),
    }
}

async fn download_ca_crt(State(state): State<AppState>) -> impl IntoResponse {
    match std::fs::read_to_string(&state.ca_pem).and_then(|pem| {
        cert::ca_certificate_der(&pem).map_err(|err| std::io::Error::other(err.to_string()))
    }) {
        Ok(der) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/x-x509-ca-cert"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"link-studio-ca.crt\"",
                ),
            ],
            der,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "CA not ready").into_response(),
    }
}

async fn download_ios_profile(State(state): State<AppState>) -> impl IntoResponse {
    match std::fs::read_to_string(&state.ca_pem)
        .ok()
        .and_then(|pem| cert::ios_root_ca_mobileconfig(&pem).ok())
    {
        Some(body) => (
            StatusCode::OK,
            [
                (
                    header::CONTENT_TYPE,
                    "application/x-apple-aspen-config; charset=utf-8",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"link-studio-ca.mobileconfig\"",
                ),
            ],
            body,
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "CA profile not ready").into_response(),
    }
}

async fn install_ca_page(State(state): State<AppState>) -> impl IntoResponse {
    let runtime = runtime_from(&state);
    Html(format!(
        "<!doctype html><meta charset=utf-8><meta name=viewport content=\"width=device-width, initial-scale=1\">
        <title>Install Link Studio CA</title>
        <body style='font-family:sans-serif;background:#111;color:#eee;padding:24px;line-height:1.5'>
        <h1>Install Link Studio Local CA</h1>
        <p>Do this on HTTP (port {http}), not on the HTTPS camera URL. Safari cannot install a CA until this profile is trusted.</p>
        <h2>iPhone / iPad (Safari)</h2>
        <p><a href=\"/ios/link-studio-ca.mobileconfig\" style='color:#8cf'>Download iOS configuration profile</a></p>
        <p>Then Settings → Profile Downloaded → Install. After that: Settings → General → About → Certificate Trust Settings → enable full trust for Link Studio Local CA.</p>
        <h2>Android</h2>
        <p><a href=\"/ca.crt\" style='color:#8cf'>Download CA certificate</a></p>
        <p>Settings → Security → Encryption &amp; credentials → Install a certificate → CA certificate.</p>
        <p style='color:#888'>HTTP profile URL: {ios}<br>Android CA URL: {android}</p>
        </body>",
        http = runtime.http_port,
        ios = runtime.ios_profile_url,
        android = runtime.android_ca_url
    ))
}

#[derive(Deserialize)]
struct CreateProjectBody {
    name: String,
    #[serde(default)]
    description: String,
}

async fn list_projects(State(state): State<AppState>) -> Result<Json<Vec<Project>>, ApiError> {
    let db = state.db.lock().map_err(|_| ApiError::lock())?;
    Ok(Json(db::list_projects(&db)?))
}

async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectBody>,
) -> Result<(StatusCode, Json<Project>), ApiError> {
    let db = state.db.lock().map_err(|_| ApiError::lock())?;
    let project = db::create_project(&db, &body.name, &body.description).map_err(map_db)?;
    Ok((StatusCode::CREATED, Json(project)))
}

async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, ApiError> {
    let db = state.db.lock().map_err(|_| ApiError::lock())?;
    db::get_project(&db, &id)?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Project not found."))
}

async fn list_cameras(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Camera>>, ApiError> {
    let db = state.db.lock().map_err(|_| ApiError::lock())?;
    Ok(Json(db::list_cameras(&db, &id)?))
}

#[derive(Deserialize)]
struct RenameBody {
    name: String,
}

async fn rename_camera(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<RenameBody>,
) -> Result<Json<Camera>, ApiError> {
    let db = state.db.lock().map_err(|_| ApiError::lock())?;
    db::rename_camera(&db, &id, &body.name)
        .map_err(map_db)?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Camera not found."))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionResponse {
    session_id: String,
    camera_url: String,
    program_url: String,
    program_http_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionCamera {
    role: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionLookup {
    session_id: String,
    project_id: String,
    cameras: Vec<SessionCamera>,
}

async fn start_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SessionResponse>, ApiError> {
    let session_id = new_session_id();
    {
        let db = state.db.lock().map_err(|_| ApiError::lock())?;
        db::create_session(&db, &id, &session_id).map_err(map_db)?;
    }
    state
        .hub
        .lock()
        .map_err(|_| ApiError::lock())?
        .open_session(session_id.clone());
    let runtime = runtime_from(&state);
    Ok(Json(SessionResponse {
        session_id: session_id.clone(),
        camera_url: format!("{}/camera/{}", runtime.https_base, session_id),
        program_url: format!("{}/program/{}", runtime.https_base, session_id),
        program_http_url: format!("http://127.0.0.1:{}/program/{}", HTTP_PORT, session_id),
    }))
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SessionLookup>, ApiError> {
    let db = state.db.lock().map_err(|_| ApiError::lock())?;
    let row = db::get_session(&db, &id)?
        .ok_or_else(|| ApiError::not_found("Session not found."))?;
    let cameras = db::list_cameras(&db, &row.project_id)?
        .into_iter()
        .map(|camera| SessionCamera {
            role: camera.role,
            name: camera.name,
        })
        .collect();
    Ok(Json(SessionLookup {
        session_id: row.id,
        project_id: row.project_id,
        cameras,
    }))
}

#[derive(Serialize, Deserialize, Default)]
struct SettingsBody {
    caption_text: Option<String>,
    pip_enabled: Option<String>,
    selected_lan_ipv4: Option<String>,
}

async fn get_settings(State(state): State<AppState>) -> Result<Json<HashMap<String, String>>, ApiError> {
    let db = state.db.lock().map_err(|_| ApiError::lock())?;
    let mut map = HashMap::new();
    for key in [
        "last_project_id",
        "active_session_id",
        "caption_text",
        "pip_enabled",
        "selected_lan_ipv4",
    ] {
        if let Some(value) = db::setting(&db, key)? {
            map.insert(key.to_string(), value);
        }
    }
    Ok(Json(map))
}

async fn put_settings(
    State(state): State<AppState>,
    Json(body): Json<SettingsBody>,
) -> Result<StatusCode, ApiError> {
    let db = state.db.lock().map_err(|_| ApiError::lock())?;
    if let Some(text) = body.caption_text {
        db::set_setting(&db, "caption_text", &text)?;
    }
    if let Some(pip) = body.pip_enabled {
        db::set_setting(&db, "pip_enabled", &pip)?;
    }
    if let Some(ip) = body.selected_lan_ipv4 {
        db::set_setting(&db, "selected_lan_ipv4", &ip)?;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct FirewallConsent {
    confirm: bool,
}

async fn install_firewall(Json(body): Json<FirewallConsent>) -> Result<Json<serde_json::Value>, ApiError> {
    if !body.confirm {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: "Confirm adding Windows Firewall inbound rules for TCP 8787 and 8443.".into(),
        });
    }
    match install_lan_inbound_rules() {
        Ok(message) => Ok(Json(serde_json::json!({ "ok": true, "message": message }))),
        Err(message) => Err(ApiError {
            status: StatusCode::FORBIDDEN,
            message,
        }),
    }
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    tracing::info!("websocket from {addr}");
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
enum ClientMsg {
    Register {
        #[serde(alias = "session_id")]
        session_id: String,
        role: Role,
        slot: Option<CameraSlot>,
        #[serde(alias = "client_id")]
        client_id: Option<String>,
    },
    Offer {
        to: String,
        sdp: String,
    },
    Answer {
        to: String,
        sdp: String,
    },
    Ice {
        to: String,
        candidate: String,
        #[serde(alias = "sdp_mid")]
        sdp_mid: Option<String>,
        #[serde(alias = "sdp_m_line_index")]
        sdp_m_line_index: Option<u16>,
    },
    PgmState {
        pgm: PgmState,
    },
    CameraOrientation {
        orientation: String,
    },
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let runtime = runtime_from(&state);
    let welcome = serde_json::json!({
        "type": "welcome",
        "httpBase": runtime.http_base,
        "httpsBase": runtime.https_base,
        "lanIps": runtime.lan_ips,
        "caUrl": runtime.ca_url,
    });
    if socket
        .send(Message::Text(welcome.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let mut bound_id: Option<String> = None;

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMsg>(&text) {
                            Ok(msg) => {
                                if let Err(err) = handle_client_msg(&state, &tx, &mut bound_id, msg) {
                                    let payload = serde_json::json!({
                                        "type": "error",
                                        "code": err.code(),
                                        "message": err.message(),
                                    });
                                    let _ = tx.send(Message::Text(payload.to_string().into()));
                                }
                            }
                            Err(err) => {
                                let payload = serde_json::json!({
                                    "type": "error",
                                    "code": "bad-message",
                                    "message": err.to_string(),
                                });
                                let _ = tx.send(Message::Text(payload.to_string().into()));
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            outgoing = rx.recv() => {
                match outgoing {
                    Some(msg) => {
                        if socket.send(msg).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }

    if let Some(id) = bound_id {
        state.sockets.lock().ok().map(|mut map| map.remove(&id));
        if let Ok(mut hub) = state.hub.lock() {
            dispatch(&state, hub.disconnect(&id));
        }
    }
}

fn handle_client_msg(
    state: &AppState,
    tx: &mpsc::UnboundedSender<Message>,
    bound_id: &mut Option<String>,
    msg: ClientMsg,
) -> Result<(), HubError> {
    match msg {
        ClientMsg::Register {
            session_id,
            role,
            slot,
            client_id,
        } => {
            let id = client_id
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let result = {
                let mut hub = state.hub.lock().map_err(|_| HubError::NotAllowed)?;
                hub.register(&session_id, id.clone(), role, slot)?
            };
            if let Ok(mut sockets) = state.sockets.lock() {
                sockets.insert(id.clone(), tx.clone());
            }
            *bound_id = Some(id);
            dispatch(state, result.outgoing);
            Ok(())
        }
        ClientMsg::Offer { to, sdp } => {
            let from = bound_id.clone().ok_or(HubError::NotAllowed)?;
            let hub = state.hub.lock().map_err(|_| HubError::NotAllowed)?;
            dispatch(state, vec![hub.relay_offer(&from, &to, sdp)?]);
            Ok(())
        }
        ClientMsg::Answer { to, sdp } => {
            let from = bound_id.clone().ok_or(HubError::NotAllowed)?;
            let hub = state.hub.lock().map_err(|_| HubError::NotAllowed)?;
            dispatch(state, vec![hub.relay_answer(&from, &to, sdp)?]);
            Ok(())
        }
        ClientMsg::Ice {
            to,
            candidate,
            sdp_mid,
            sdp_m_line_index,
        } => {
            let from = bound_id.clone().ok_or(HubError::NotAllowed)?;
            let hub = state.hub.lock().map_err(|_| HubError::NotAllowed)?;
            dispatch(
                state,
                vec![hub.relay_ice(&from, &to, candidate, sdp_mid, sdp_m_line_index)?],
            );
            Ok(())
        }
        ClientMsg::PgmState { pgm } => {
            let from = bound_id.clone().ok_or(HubError::NotAllowed)?;
            let mut hub = state.hub.lock().map_err(|_| HubError::NotAllowed)?;
            dispatch(state, hub.set_pgm(&from, pgm)?);
            Ok(())
        }
        ClientMsg::CameraOrientation { orientation } => {
            let from = bound_id.clone().ok_or(HubError::NotAllowed)?;
            let hub = state.hub.lock().map_err(|_| HubError::NotAllowed)?;
            dispatch(state, hub.relay_camera_orientation(&from, orientation)?);
            Ok(())
        }
    }
}

fn dispatch(state: &AppState, outgoing: Vec<Outgoing>) {
    let sockets = match state.sockets.lock() {
        Ok(s) => s,
        Err(_) => return,
    };
    for item in outgoing {
        match item {
            Outgoing::To { client_id, payload } => {
                if let Ok(text) = serde_json::to_string(&payload) {
                    if let Some(tx) = sockets.get(&client_id) {
                        let _ = tx.send(Message::Text(text.into()));
                    }
                }
            }
            Outgoing::Close { client_id, reason } => {
                if let Some(tx) = sockets.get(&client_id) {
                    let _ = tx.send(Message::Close(Some(axum::extract::ws::CloseFrame {
                        code: axum::extract::ws::close_code::NORMAL,
                        reason: reason.into(),
                    })));
                }
            }
        }
    }
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn lock() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Internal lock error.".into(),
        }
    }

    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
}

fn map_db(err: rusqlite::Error) -> ApiError {
    let message = err.to_string();
    let status = if message.contains("required") || message.contains("Invalid") {
        StatusCode::BAD_REQUEST
    } else if matches!(err, rusqlite::Error::QueryReturnedNoRows) {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    ApiError { status, message }
}

impl From<rusqlite::Error> for ApiError {
    fn from(err: rusqlite::Error) -> Self {
        map_db(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.message });
        (self.status, Json(body)).into_response()
    }
}

// Silence unused import warnings if ConnectInfo-only camera gate is added later.
#[allow(dead_code)]
fn camera_https_notice(headers: &HeaderMap, uri: &Uri, runtime: &RuntimeInfo) -> Option<Response<Body>> {
    let path = uri.path();
    if !path.starts_with("/camera") {
        return None;
    }
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if host.starts_with("127.0.0.1") || host.starts_with("localhost") {
        return None;
    }
    Some(
        Html(format!(
            "<!doctype html><meta charset=utf-8><title>HTTPS required</title>
            <body style='font-family:sans-serif;background:#111;color:#eee;padding:24px'>
            <h1>Camera needs HTTPS</h1>
            <p>Browsers only allow camera access in a secure context. Open:</p>
            <p><a href='{0}' style='color:#8cf'>{0}</a></p>
            <p>Install the local CA first: <a href='{1}' style='color:#8cf'>{1}</a></p>
            </body>",
            runtime.https_base.clone() + path,
            runtime.ca_url
        ))
        .into_response(),
    )
}

#[cfg(test)]
mod ca_http_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, HeaderMap};
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use uuid::Uuid;

    fn test_app() -> Router {
        let dir = std::env::temp_dir().join(format!("link-studio-http-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let certs = cert::ensure_tls(&dir, &[], &["127.0.0.1".parse().unwrap()]).unwrap();
        let state = AppState {
            hub: Arc::new(Mutex::new(Hub::default())),
            db: Arc::new(Mutex::new(crate::db::open_in_memory().unwrap())),
            sockets: Arc::new(Mutex::new(HashMap::new())),
            http_port: HTTP_PORT,
            https_port: HTTPS_PORT,
            ca_pem: certs.ca_pem,
            dist_dir: dir.clone(),
            cert_dir: dir,
            tls: None,
        };
        router(state)
    }

    async fn get(path: &str) -> (StatusCode, HeaderMap, Vec<u8>) {
        let response = test_app()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        (status, headers, body.to_vec())
    }

    #[tokio::test]
    async fn ca_pem_is_attachment_pem() {
        let (status, headers, body) = get("/ca.pem").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "application/x-pem-file"
        );
        assert!(headers
            .get(header::CONTENT_DISPOSITION)
            .unwrap()
            .to_str()
            .unwrap()
            .contains("link-studio-ca.pem"));
        let text = String::from_utf8(body).unwrap();
        assert!(text.contains("BEGIN CERTIFICATE"));
        assert!(!text.contains("PRIVATE KEY"));
    }

    #[tokio::test]
    async fn android_ca_crt_is_der_ca_cert() {
        let (status, headers, body) = get("/ca.crt").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "application/x-x509-ca-cert"
        );
        assert!(headers
            .get(header::CONTENT_DISPOSITION)
            .unwrap()
            .to_str()
            .unwrap()
            .contains("link-studio-ca.crt"));
        assert_eq!(body[0], 0x30);
    }

    #[tokio::test]
    async fn ios_profile_is_apple_config() {
        let (status, headers, body) = get("/ios/link-studio-ca.mobileconfig").await;
        assert_eq!(status, StatusCode::OK);
        let content_type = headers.get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
        assert!(content_type.starts_with("application/x-apple-aspen-config"));
        assert!(headers
            .get(header::CONTENT_DISPOSITION)
            .unwrap()
            .to_str()
            .unwrap()
            .contains("link-studio-ca.mobileconfig"));
        let text = String::from_utf8(body).unwrap();
        assert!(text.contains("com.apple.security.root"));
        assert!(text.contains("PayloadType"));
        assert!(!text.contains("PRIVATE KEY"));
    }
}

#[cfg(test)]
mod session_contract_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use uuid::Uuid;

    fn test_app() -> Router {
        let dir = std::env::temp_dir().join(format!("link-studio-session-{}", Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(
            dir.join("index.html"),
            r#"<!doctype html><html><body><div id="root"></div><script type="module" src="/assets/app.js"></script></body></html>"#,
        )
        .unwrap();
        std::fs::write(dir.join("assets").join("app.js"), "window.__LINK_STUDIO_ASSET = true;").unwrap();
        let certs = cert::ensure_tls(&dir, &[], &["127.0.0.1".parse().unwrap()]).unwrap();
        let state = AppState {
            hub: Arc::new(Mutex::new(Hub::default())),
            db: Arc::new(Mutex::new(crate::db::open_in_memory().unwrap())),
            sockets: Arc::new(Mutex::new(HashMap::new())),
            http_port: HTTP_PORT,
            https_port: HTTPS_PORT,
            ca_pem: certs.ca_pem,
            dist_dir: dir.clone(),
            cert_dir: dir,
            tls: None,
        };
        router(state)
    }

    #[test]
    fn register_accepts_frontend_camel_case_session_id() {
        let msg = serde_json::from_str::<ClientMsg>(
            r#"{"type":"register","sessionId":"abc123","role":"director"}"#,
        )
        .expect("frontend register payload must deserialize");
        match msg {
            ClientMsg::Register { session_id, role, .. } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(role, Role::Director);
            }
            _ => panic!("expected register"),
        }
    }

    #[test]
    fn register_still_accepts_snake_case_session_id() {
        let msg = serde_json::from_str::<ClientMsg>(
            r#"{"type":"register","session_id":"snake","role":"camera","slot":"cam1"}"#,
        )
        .unwrap();
        match msg {
            ClientMsg::Register { session_id, .. } => assert_eq!(session_id, "snake"),
            _ => panic!("expected register"),
        }
    }

    #[test]
    fn ice_accepts_camel_case_sdp_fields() {
        let msg = serde_json::from_str::<ClientMsg>(
            r#"{"type":"ice","to":"x","candidate":"cand","sdpMid":"0","sdpMLineIndex":0}"#,
        )
        .unwrap();
        match msg {
            ClientMsg::Ice {
                sdp_mid,
                sdp_m_line_index,
                ..
            } => {
                assert_eq!(sdp_mid.as_deref(), Some("0"));
                assert_eq!(sdp_m_line_index, Some(0));
            }
            _ => panic!("expected ice"),
        }
    }

    #[tokio::test]
    async fn start_session_persists_and_returns_camel_case_session_id() {
        let app = test_app();
        let created = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/projects")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"name":"Demo","description":""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);
        let created_body = created.into_body().collect().await.unwrap().to_bytes();
        let project: serde_json::Value = serde_json::from_slice(&created_body).unwrap();
        let project_id = project["id"].as_str().unwrap();

        let started = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/projects/{project_id}/session"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(started.status(), StatusCode::OK);
        let session_body = started.into_body().collect().await.unwrap().to_bytes();
        let session: serde_json::Value = serde_json::from_slice(&session_body).unwrap();
        let session_id = session["sessionId"]
            .as_str()
            .expect("sessionId camelCase field");
        assert!(!session_id.is_empty());
        assert!(session.get("session_id").is_none());
        assert!(session["cameraUrl"]
            .as_str()
            .unwrap()
            .ends_with(&format!("/camera/{session_id}")));
        assert!(session["programUrl"]
            .as_str()
            .unwrap()
            .ends_with(&format!("/program/{session_id}")));

        let lookup = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/sessions/{session_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(lookup.status(), StatusCode::OK);
        let lookup_body = lookup.into_body().collect().await.unwrap().to_bytes();
        let found: serde_json::Value = serde_json::from_slice(&lookup_body).unwrap();
        assert_eq!(found["sessionId"], session_id);
        assert_eq!(found["projectId"], project_id);
        assert_eq!(found["cameras"][0]["role"], "cam1");
        assert_eq!(found["cameras"][0]["name"], "Camera 1");
        assert_eq!(found["cameras"][1]["role"], "cam2");
        assert_eq!(found["cameras"][1]["name"], "Camera 2");

        let missing = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/sessions/does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);

        let camera_page = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/camera/{session_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(camera_page.status(), StatusCode::OK);
        let html = String::from_utf8(
            camera_page
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("/assets/app.js"));

        let program_page = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/program/{session_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(program_page.status(), StatusCode::OK);

        let asset = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/assets/app.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(asset.status(), StatusCode::OK);
        let js = String::from_utf8(asset.into_body().collect().await.unwrap().to_bytes().to_vec())
            .unwrap();
        assert!(js.contains("window.__LINK_STUDIO_ASSET"));
        assert!(!js.contains("<html"));

        let missing_asset = app
            .oneshot(
                Request::builder()
                    .uri("/assets/missing.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing_asset.status(), StatusCode::NOT_FOUND);
        let missing_body = String::from_utf8(
            missing_asset
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(!missing_body.contains("<!doctype html"));
        assert!(!missing_body.contains("<div id=\"root\""));
    }

    #[tokio::test]
    async fn runtime_reports_wildcard_binds_and_does_not_prefer_stale_ips() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/runtime")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let runtime: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(runtime["bind_http"], "0.0.0.0:8787");
        assert_eq!(runtime["bind_https"], "0.0.0.0:8443");
        assert!(runtime["interfaces"].is_array());
    }
}
