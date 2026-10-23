use axum::{
    extract::{DefaultBodyLimit, Json},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use pension_calculator::{calculate, Input};

async fn index() -> impl IntoResponse {
    Html(include_str!("../frontend/index.html"))
}

async fn defaults() -> Json<Input> {
    Json(Input::default())
}

async fn compute(Json(input): Json<Input>) -> Response {
    match calculate(&input) {
        Ok(result) => Json(result).into_response(),
        Err(error) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": error})),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(index))
        .route(
            "/favicon.svg",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "image/svg+xml")],
                    include_str!("../frontend/favicon.svg"),
                )
            }),
        )
        .route(
            "/favicon-32.png",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "image/png")],
                    include_bytes!("../frontend/favicon-32.png").as_slice(),
                )
            }),
        )
        .route(
            "/favicon.ico",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "image/x-icon")],
                    include_bytes!("../frontend/favicon.ico").as_slice(),
                )
            }),
        )
        .route("/api/defaults", get(defaults))
        .route("/api/calculate", post(compute))
        .route("/health", get(|| async { "ok" }))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(axum::middleware::map_response(
            |mut response: Response| async move {
                response
                    .headers_mut()
                    .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
                response.headers_mut().insert(
                    header::X_CONTENT_TYPE_OPTIONS,
                    HeaderValue::from_static("nosniff"),
                );
                response
                    .headers_mut()
                    .insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
                response.headers_mut().insert(
                    header::REFERRER_POLICY,
                    HeaderValue::from_static("no-referrer"),
                );
                response
            },
        ));
    let address = std::env::var("PENSION_BIND").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let listener = tokio::net::TcpListener::bind(&address).await?;
    eprintln!("养老保险情景计算器：http://{address} （不保存输入、不写访问日志）");
    axum::serve(listener, app).await?;
    Ok(())
}
