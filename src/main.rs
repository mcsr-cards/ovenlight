use std::{collections::BTreeMap, env, sync::Arc};

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use image::{imageops::FilterType, DynamicImage, Rgba, RgbaImage};
use tract_onnx::prelude::*;

const MODEL: &[u8] = include_bytes!("../model/nsfw.onnx");
const SIZE: usize = 224;
const CLASSES: [&str; 5] = ["Drawing", "Hentai", "Neutral", "Porn", "Sexy"];

type Plan = TypedRunnableModel<TypedModel>;

struct App {
    plan: Arc<Plan>,
    tokens: Vec<String>,
}

fn load() -> TractResult<Arc<Plan>> {
    tract_onnx::onnx()
        .model_for_read(&mut &MODEL[..])?
        .with_input_fact(0, f32::fact([1, SIZE, SIZE, 3]).into())?
        .into_optimized()?
        .into_runnable()
        .map(Arc::new)
}

fn tensor(bytes: &[u8]) -> Option<Tensor> {
    let decoded = image::load_from_memory(bytes).ok()?.to_rgba8();
    let mut canvas = RgbaImage::from_pixel(decoded.width(), decoded.height(), Rgba([255; 4]));
    image::imageops::overlay(&mut canvas, &decoded, 0, 0);
    let rgb = DynamicImage::ImageRgba8(canvas)
        .resize_exact(SIZE as u32, SIZE as u32, FilterType::Triangle)
        .to_rgb8();
    let data: Vec<f32> = rgb.as_raw().iter().map(|&v| f32::from(v) / 255.0).collect();
    Tensor::from_shape(&[1, SIZE, SIZE, 3], &data).ok()
}

fn classify(plan: &Plan, input: Tensor) -> TractResult<BTreeMap<&'static str, f32>> {
    let out = plan.run(tvec!(input.into()))?;
    let scores = out[0].to_array_view::<f32>()?;
    Ok(CLASSES.into_iter().zip(scores.iter().copied()).collect())
}

async fn handle(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<BTreeMap<&'static str, f32>>, StatusCode> {
    if !app.tokens.is_empty() {
        let ok = headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .is_some_and(|v| app.tokens.iter().any(|t| t == v));
        if !ok {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    let plan = app.plan.clone();
    tokio::task::spawn_blocking(move || {
        let input = tensor(&body).ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
        classify(&plan, input)
            .map(Json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
}

#[tokio::main]
async fn main() {
    let app = Arc::new(App {
        plan: load().expect("model failed to load"),
        tokens: env::var("NSFW_TOKEN")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(String::from)
            .collect(),
    });
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = env::var("PORT").unwrap_or_else(|_| "3002".into());
    let router = Router::new()
        .route("/", post(handle))
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
        .with_state(app);
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .expect("bind failed");
    axum::serve(listener, router).await.expect("server error");
}
