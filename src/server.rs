use anyhow::Context;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tower_http::{
    request_id::{
        MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
    },
    trace::TraceLayer,
};
use tracing::{debug, info, info_span, Span};
use crate::json_or_form::JsonOrForm;
use crate::errors::ServerError;
use crate::config::Config;

#[derive(Debug)]
pub struct Server;

struct ServerStateImpl {
    cfg: Config,
    reqwest: reqwest::Client,
}

type ServerState = Arc<ServerStateImpl>;

impl Server {
    pub async fn run() -> Result<(), anyhow::Error> {
        let cfg = Config::load().unwrap();

        let client = reqwest::Client::builder().build()?;
        let shared_state = Arc::new(ServerStateImpl {
            cfg: cfg.clone(),
            reqwest: client,
        });

        let router = Router::new()
            .route("/health", get(|| async { StatusCode::NO_CONTENT }))
            .route("/", post(post_index))
            .layer(
                tower::ServiceBuilder::new()
                    .layer(SetRequestIdLayer::x_request_id(StringRequestId))
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(|request: &axum::http::Request<_>| {
                                info_span!(
                                  "request",

                                  trace_id = request
                                    .extensions()
                                    .get::<RequestId>()
                                    .map(|x| x.header_value().to_str().unwrap_or("")),

                                  method = ?request.method(),
                                  uri = %request.uri(),
                                )
                            })
                            .on_request(|request: &axum::http::Request<_>, _span: &Span| {
                                debug!(
                                  request = ?request,
                                  "started processing request"
                                );
                            })
                            .on_response(|response: &Response, latency: Duration, _span: &Span| {
                                debug!(
                                  response = ?response,
                                  latency = format_args!("{} us", latency.as_micros()),
                                  "finished processing request"
                                );
                            })
                    )
                    .layer(PropagateRequestIdLayer::x_request_id())
            )
            .with_state(shared_state);

        info!("listening on {}", cfg.addr);

        let listener = tokio::net::TcpListener::bind(cfg.addr).await?;
        axum::serve(listener, router).await?;

        Ok(())
    }
}

#[derive(Clone)]
struct StringRequestId;

impl MakeRequestId for StringRequestId {
    fn make_request_id<B>(
        &mut self,
        _request: &Request<B>,
    ) -> Option<RequestId> {
        let id = ulid::Ulid::new().to_string();

        let Ok(value) = HeaderValue::from_str(&id) else {
            return None;
        };

        Some(RequestId::new(value))
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct SubmitBody {
    email: Option<String>,
    subject: String,
    message: String,
    source: String,
}

// #[axum::debug_handler]
async fn post_index(
    State(state): State<ServerState>,
    JsonOrForm(body): JsonOrForm<SubmitBody>,
) -> Result<Response<Body>, ServerError> {
    debug!(?body);

    let title = format!("[Feedback] {}", body.subject);
    let message = format!(
        "Source: {}\nEmail: {}\n\n{}",
        body.source,
        body.email.unwrap_or("anonymous".to_string()),
        body.message
    );

    let mut body = json!({
        "body": message,
        "title": title,
    });

    let mut req = state.reqwest.post(&state.cfg.apprise.url)
        .header("Content-Type", "application/json");

    for (k, v) in state.cfg.apprise.headers.iter() {
        req = req.header(k, v);
    }

    if let Some(urls) = state.cfg.apprise.stateless_urls.as_ref() {
        body = body
            .as_object_mut()
            .unwrap()
            .insert(
                "urls".to_string(),
                urls.to_string().into()
            )
            .unwrap();
    }

    req
        .body(body.to_string())
        .send()
        .await
        .context("failed to post")?;

    Ok(StatusCode::CREATED.into_response())
}
