use anyhow::Result;
use axum::{
    Json,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    routing::get,
};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::env::BaseEnv;

#[derive(OpenApi)]
#[openapi()]
struct Doc;

pub struct HttpServer;

impl HttpServer {
    pub fn new(env: &BaseEnv) -> OpenApiRouter {
        let doc = Self::create_doc(env);

        OpenApiRouter::with_openapi(doc)
    }

    fn create_doc(env: &BaseEnv) -> utoipa::openapi::OpenApi {
        let mut doc = Doc::openapi();

        doc.info.title = env.openapi.info.title.clone();
        doc.info.version = env.openapi.info.version.clone();

        if let Some(description) = &env.openapi.info.description {
            doc.info.description = Some(description.clone());
        }

        doc.servers = Some(
            env.openapi
                .server_urls
                .split(',')
                .map(str::trim)
                .filter(|url| !url.is_empty())
                .map(utoipa::openapi::Server::new)
                .collect(),
        );

        doc
    }
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn start_http_server(env: &BaseEnv, router: OpenApiRouter) -> Result<()> {
    let listener = TcpListener::bind(&env.http.address).await?;
    let (router, doc) = router.split_for_parts();

    let custom_html = format!(
        r#"<!doctype html>
<html>
<head>
    <title>{} - Sevria API Docs</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
    <link rel="icon" href="https://static.sevria.com/images/favicon.svg"/>
</head>
<body>
    <script id="api-reference" type="application/json">$spec</script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"#,
        env.openapi.info.title
    );

    let mut router = router
        .route("/", get(health_check))
        .merge(Scalar::with_url("/docs", doc.clone()).custom_html(custom_html))
        .route("/openapi.json", get(async move || Json(doc)));

    if let Some(allowed_origins) = &env.cors.allowed_origins {
        router = router.layer(
            CorsLayer::new()
                .allow_origin(allowed_origins.parse::<HeaderValue>().unwrap())
                .allow_methods([
                    Method::OPTIONS,
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_headers([ACCEPT, AUTHORIZATION, CONTENT_TYPE]),
        );
    }

    log::info!("Running HTTP server on {}", &env.http.address);

    axum::serve(listener, router).await?;

    Ok(())
}
