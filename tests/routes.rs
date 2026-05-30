use axum::body::Body;
use axum::extract::Json as AxumJson;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router as AxumRouter};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use srvcs_csc::{api::Deps, health, router, telemetry};
use tower::ServiceExt;

const DEAD_URL: &str = "http://127.0.0.1:1";

// --- Computing mocks for every srvcs primitive this family composes over.
//
// Each reads its operands from the request body and returns the *real* answer,
// so the orchestration is genuinely exercised rather than fed a canned value.
// csc only calls `srvcs-sin` and `srvcs-floatdivide`; the rest are provided for
// completeness of the family's contract.

/// `srvcs-floatadd`: reads `{a, b}` -> `{"result": a + b}` (as f64).
#[allow(dead_code)]
async fn spawn_floatadd() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let a = body.get("a").and_then(Value::as_f64).unwrap_or(0.0);
            let b = body.get("b").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": a + b }))
        }),
    );
    serve(app).await
}

/// `srvcs-floatsubtract`: reads `{a, b}` -> `{"result": a - b}` (as f64).
#[allow(dead_code)]
async fn spawn_floatsubtract() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let a = body.get("a").and_then(Value::as_f64).unwrap_or(0.0);
            let b = body.get("b").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": a - b }))
        }),
    );
    serve(app).await
}

/// `srvcs-floatmultiply`: reads `{a, b}` -> `{"result": a * b}` (as f64).
#[allow(dead_code)]
async fn spawn_floatmultiply() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let a = body.get("a").and_then(Value::as_f64).unwrap_or(0.0);
            let b = body.get("b").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": a * b }))
        }),
    );
    serve(app).await
}

/// `srvcs-floatdivide`: reads `{a, b}` -> `{"result": a / b}` (as f64).
async fn spawn_floatdivide() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let a = body.get("a").and_then(Value::as_f64).unwrap_or(0.0);
            let b = body.get("b").and_then(Value::as_f64).unwrap_or(1.0);
            Json(json!({ "result": a / b }))
        }),
    );
    serve(app).await
}

/// `srvcs-sqrt`: reads `{value}` -> `{"result": value.sqrt()}` (as f64).
#[allow(dead_code)]
async fn spawn_sqrt() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let value = body.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": value.sqrt() }))
        }),
    );
    serve(app).await
}

/// `srvcs-sin`: reads `{value}` -> `{"result": value.sin()}` (as f64).
async fn spawn_sin() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let value = body.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": value.sin() }))
        }),
    );
    serve(app).await
}

/// `srvcs-cos`: reads `{value}` -> `{"result": value.cos()}` (as f64).
#[allow(dead_code)]
async fn spawn_cos() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let value = body.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": value.cos() }))
        }),
    );
    serve(app).await
}

/// `srvcs-tan`: reads `{value}` -> `{"result": value.tan()}` (as f64).
#[allow(dead_code)]
async fn spawn_tan() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let value = body.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": value.tan() }))
        }),
    );
    serve(app).await
}

/// `srvcs-pi`: returns `{"result": PI}` for any body.
#[allow(dead_code)]
async fn spawn_pi() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|| async move { Json(json!({ "result": std::f64::consts::PI })) }),
    );
    serve(app).await
}

/// Spawn a mock returning a fixed status + body (used for error-path tests).
async fn spawn_fixed(status: StatusCode, body: Value) -> String {
    let app = AxumRouter::new().route(
        "/",
        post(move || {
            let body = body.clone();
            async move { (status, Json(body)) }
        }),
    );
    serve(app).await
}

async fn serve(app: AxumRouter) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn app(sin_url: &str, floatdivide_url: &str) -> axum::Router {
    router(
        telemetry::metrics_handle_for_tests(),
        Deps {
            sin_url: sin_url.to_string(),
            floatdivide_url: floatdivide_url.to_string(),
        },
    )
}

async fn csc(sin_url: &str, floatdivide_url: &str, value: Value) -> (StatusCode, Value) {
    let res = app(sin_url, floatdivide_url)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "value": value }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn status_of(uri: &str) -> StatusCode {
    app(DEAD_URL, DEAD_URL)
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

fn result_f64(body: &Value) -> f64 {
    body["result"].as_f64().expect("result is a JSON number")
}

// --- Standard endpoints. ---

#[tokio::test]
async fn healthz_ok() {
    assert_eq!(status_of("/healthz").await, StatusCode::OK);
}

#[tokio::test]
async fn readyz_reflects_state() {
    health::set_ready(true);
    assert_eq!(status_of("/readyz").await, StatusCode::OK);
}

#[tokio::test]
async fn metrics_ok() {
    assert_eq!(status_of("/metrics").await, StatusCode::OK);
}

#[tokio::test]
async fn openapi_ok() {
    assert_eq!(status_of("/openapi.json").await, StatusCode::OK);
}

#[tokio::test]
async fn generates_request_id_when_absent() {
    let res = app(DEAD_URL, DEAD_URL)
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        res.headers().contains_key("x-request-id"),
        "response must carry a generated x-request-id"
    );
}

#[tokio::test]
async fn index_reports_identity() {
    let res = app(DEAD_URL, DEAD_URL)
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["service"], "srvcs-csc");
    assert_eq!(body["concern"], "trigonometry: cosecant");
    assert_eq!(
        body["depends_on"],
        json!(["srvcs-sin", "srvcs-floatdivide"])
    );
}

// --- Correctness cases, against the computing mocks. ---

#[tokio::test]
async fn csc_half_pi_is_one() {
    let (s, d) = (spawn_sin().await, spawn_floatdivide().await);
    // 1.5707963267948966 is pi/2; csc(pi/2) = 1 / sin(pi/2) = 1 / 1 = 1.0.
    let (status, body) = csc(&s, &d, json!(std::f64::consts::FRAC_PI_2)).await;
    assert_eq!(status, StatusCode::OK);
    assert!((body["value"].as_f64().unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    // sin(pi/2) = 1; 1 / 1 = 1.0
    assert!((result_f64(&body) - 1.0).abs() < 1e-9);
}

#[tokio::test]
async fn csc_pi_sixth_is_two() {
    let (s, d) = (spawn_sin().await, spawn_floatdivide().await);
    // sin(pi/6) = 0.5; csc = 1 / 0.5 = 2.0
    let (status, body) = csc(&s, &d, json!(std::f64::consts::FRAC_PI_6)).await;
    assert_eq!(status, StatusCode::OK);
    assert!((result_f64(&body) - 2.0).abs() < 1e-9);
}

#[tokio::test]
async fn csc_negative_half_pi_is_negative_one() {
    let (s, d) = (spawn_sin().await, spawn_floatdivide().await);
    // sin(-pi/2) = -1; csc = 1 / -1 = -1.0
    let (status, body) = csc(&s, &d, json!(-std::f64::consts::FRAC_PI_2)).await;
    assert_eq!(status, StatusCode::OK);
    assert!((result_f64(&body) - (-1.0)).abs() < 1e-9);
}

#[tokio::test]
async fn csc_pi_quarter_is_sqrt_two() {
    let (s, d) = (spawn_sin().await, spawn_floatdivide().await);
    // sin(pi/4) = 1/sqrt(2); csc = sqrt(2) ~= 1.4142135623730951
    let (status, body) = csc(&s, &d, json!(std::f64::consts::FRAC_PI_4)).await;
    assert_eq!(status, StatusCode::OK);
    assert!((result_f64(&body) - std::f64::consts::SQRT_2).abs() < 1e-9);
}

// --- Error / edge cases. ---

#[tokio::test]
async fn degrades_when_sin_unreachable() {
    let d = spawn_floatdivide().await;
    let (status, body) = csc(DEAD_URL, &d, json!(1.0)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["dependency"], "srvcs-sin");
}

#[tokio::test]
async fn degrades_when_floatdivide_unreachable() {
    // sin is reachable so the pipeline reaches the floatdivide call, which then
    // degrades.
    let s = spawn_sin().await;
    let (status, body) = csc(&s, DEAD_URL, json!(1.0)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["dependency"], "srvcs-floatdivide");
}

#[tokio::test]
async fn forwards_422_from_sin() {
    let d = spawn_floatdivide().await;
    let s = spawn_fixed(
        StatusCode::UNPROCESSABLE_ENTITY,
        json!({ "error": "value is not a number" }),
    )
    .await;
    let (status, body) = csc(&s, &d, json!("nope")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "value is not a number");
}

#[tokio::test]
async fn forwards_422_from_floatdivide() {
    let s = spawn_sin().await;
    let d = spawn_fixed(
        StatusCode::UNPROCESSABLE_ENTITY,
        json!({ "error": "bad operand" }),
    )
    .await;
    let (status, _) = csc(&s, &d, json!(1.0)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn malformed_sin_result_is_500() {
    let d = spawn_floatdivide().await;
    let s = spawn_fixed(StatusCode::OK, json!({ "result": "not-a-number" })).await;
    let (status, body) = csc(&s, &d, json!(1.0)).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["dependency"], "srvcs-sin");
}

#[tokio::test]
async fn malformed_floatdivide_result_is_500() {
    let s = spawn_sin().await;
    let d = spawn_fixed(StatusCode::OK, json!({ "result": "not-a-number" })).await;
    let (status, body) = csc(&s, &d, json!(1.0)).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["dependency"], "srvcs-floatdivide");
}
