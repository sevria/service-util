use std::sync::Arc;

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
use utoipa::openapi::Server;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::config::Config;

#[derive(OpenApi)]
#[openapi()]
struct Doc;

pub struct HttpServer {
    config: Arc<Config>,
    pub router: OpenApiRouter,
}

impl HttpServer {
    pub fn new(config: Arc<Config>) -> Self {
        let doc = Self::create_doc(&*config);
        let router = OpenApiRouter::with_openapi(doc);

        HttpServer { config, router }
    }

    fn create_doc(config: &Config) -> utoipa::openapi::OpenApi {
        let mut openapi = Doc::openapi();

        openapi.info.title = config.openapi.info.title.clone();
        openapi.info.version = config.openapi.info.version.clone();

        if let Some(description) = &config.openapi.info.description {
            openapi.info.description = Some(description.clone());
        }

        openapi.servers = Some(
            config
                .openapi
                .server_urls
                .split(',')
                .map(str::trim)
                .filter(|url| !url.is_empty())
                .map(Server::new)
                .collect(),
        );

        openapi
    }

    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.http.address).await?;
        let (router, doc) = self.router.clone().split_for_parts();

        let mut router = router
            .route("/", get(health_check))
            .merge(Scalar::with_url("/docs", doc.clone()))
            .route("/openapi.json", get(async move || Json(doc)));

        if let Some(allowed_origins) = &self.config.cors.allowed_origins {
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

        log::info!("Running HTTP server on {}", &self.config.http.address);

        axum::serve(listener, router).await?;

        Ok(())
    }
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
