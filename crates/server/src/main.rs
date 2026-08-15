mod admin;
mod api;
mod auth;
mod calc;
mod db;
mod error;
mod line;
mod products;
mod snapshot;
mod weather;

use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, services::{ServeDir, ServeFile}, trace::TraceLayer};

#[derive(Clone)]
pub struct Config {
    pub line_secret: Option<String>,
    pub line_token: Option<String>,
    pub line_add_friend_url: Option<String>,
    pub web_dir: PathBuf,
}

#[derive(Clone)]
pub struct AppState {
    pub db: db::Db,
    pub http: reqwest::Client,
    pub cfg: Arc<Config>,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sqlx=warn,tower_http=info".into()))
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/teedet.db".into());
    if let Some(dir) = db_url.strip_prefix("sqlite://").and_then(|p| std::path::Path::new(p).parent().map(|d| d.to_path_buf())) {
        let _ = std::fs::create_dir_all(dir);
    }
    let db = db::connect(&db_url).await.expect("database");

    let cfg = Config {
        line_secret: std::env::var("LINE_CHANNEL_SECRET").ok().filter(|s| !s.is_empty()),
        line_token: std::env::var("LINE_CHANNEL_ACCESS_TOKEN").ok().filter(|s| !s.is_empty()),
        line_add_friend_url: std::env::var("LINE_ADD_FRIEND_URL").ok().filter(|s| !s.is_empty()),
        web_dir: PathBuf::from(std::env::var("WEB_DIR").unwrap_or_else(|_| "web/dist".into())),
    };
    let state = AppState {
        db,
        http: reqwest::Client::builder().user_agent("teedet-pla/0.1").timeout(std::time::Duration::from_secs(20)).build().expect("http client"),
        cfg: Arc::new(cfg),
    };
    auth::ensure_bootstrap_admin(&state).await.expect("bootstrap admin");
    products::seed_if_empty(&state).await.expect("seed feed products");
    line::spawn_scheduler(state.clone());

    let api = Router::new()
        // สาธารณะ
        .route("/health", get(|| async { "ok" }))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/species", get(calc::species))
        .route("/rules", get(calc::rules))
        .route("/calc/recommend", post(calc::calc_recommend))
        .route("/calc/simulate", post(calc::calc_simulate))
        .route("/calc/growth", post(calc::calc_growth))
        .route("/calc/health", post(calc::calc_health))
        .route("/calc/mix", post(calc::calc_mix))
        .route("/nutrition/{code}", get(calc::nutrition_stages))
        .route("/nutrition-ingredients", get(calc::nutrition_ingredients))
        .route("/feed-products", get(products::list).post(products::create))
        .route("/feed-products/{id}", axum::routing::patch(products::update).delete(products::remove))
        .route("/weather", get(weather::get_weather))
        .route("/weather/forecast", get(weather::get_forecast))
        .route("/prices", get(api::list_prices))
        .route("/disease-reports", get(api::list_disease_reports))
        .route("/line/webhook", post(line::webhook))
        // ต้องล็อกอิน
        .route("/auth/logout", post(auth::logout))
        .route("/auth/pin", post(auth::change_pin))
        .route("/me", get(auth::me))
        .route("/users", post(auth::create_user))
        .route("/farms", get(api::list_farms).post(api::create_farm))
        .route("/farms/{id}", get(api::get_farm).patch(api::update_farm))
        .route("/farms/{id}/today", get(snapshot::farm_today))
        .route("/farms/{id}/ponds", post(api::create_pond))
        .route("/farms/{id}/crops", get(api::list_crops))
        .route("/farms/{id}/stock", get(api::stock_summary).post(api::create_stock_move))
        .route("/ponds/{id}", axum::routing::patch(api::update_pond))
        .route("/ponds/{id}/crops", post(api::create_crop))
        .route("/ponds/{id}/water", get(api::list_water).post(api::create_water))
        .route("/crops/{id}", axum::routing::patch(api::update_crop))
        .route("/crops/{id}/close", post(api::close_crop))
        .route("/crops/{id}/today", get(snapshot::crop_today))
        .route("/crops/{id}/logs", get(api::list_logs).post(api::upsert_log))
        .route("/crops/{id}/weighings", get(api::list_weighings).post(api::create_weighing))
        .route("/crops/{id}/expenses", get(api::list_expenses).post(api::create_expense))
        .route("/crops/{id}/harvests", get(api::list_harvests).post(api::create_harvest))
        .route("/crops/{id}/treatments", get(api::list_treatments).post(api::create_treatment))
        .route("/crops/{id}/export.csv", get(api::export_crop_csv))
        .route("/prices", post(api::create_price))
        .route("/disease-reports", post(api::create_disease_report))
        .route("/sync", post(api::sync))
        .route("/line/link-code", post(line::link_code))
        .route("/line/unlink", post(line::unlink))
        .route("/announcements", get(admin::list_announcements).post(admin::create_announcement))
        .route("/benchmark", get(admin::benchmark))
        // หลังบ้าน
        .route("/admin/farms", get(admin::farms_overview))
        .route("/admin/farms/{id}", get(admin::farm_detail))
        .route("/admin/rules", get(admin::get_rules).put(admin::put_rules))
        .route("/admin/species", get(admin::get_species))
        .route("/admin/species/{code}", axum::routing::put(admin::put_species))
        .route("/admin/users", get(admin::list_users))
        .route("/admin/audit", get(admin::audit_list))
        .route("/admin/line/morning", post(line::trigger_morning))
        .with_state(state.clone());

    let cors = CorsLayer::new()
        .allow_origin(
            std::env::var("CORS_ORIGIN")
                .ok()
                .and_then(|o| o.parse::<HeaderValue>().ok())
                .map(tower_http::cors::AllowOrigin::exact)
                .unwrap_or_else(tower_http::cors::AllowOrigin::any),
        )
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let web_dir = state.cfg.web_dir.clone();
    let index = web_dir.join("index.html");
    let spa = ServeDir::new(&web_dir).not_found_service(ServeFile::new(index));

    let app = Router::new()
        .nest("/api", api)
        .fallback_service(spa)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8787);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("teedet-server listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("server");
}
