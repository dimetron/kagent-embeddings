use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};

use fastembed::TextEmbedding;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

// Request/Response types
#[derive(Deserialize)]
struct EmbeddingRequest {
    texts: Vec<String>,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Deserialize)]
struct EmbeddingQueryRequest {
    text: String,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Serialize)]
struct EmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
    model: String,
    dimensions: usize,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    models: Vec<String>,
}

// Application state
struct AppState {
    models: HashMap<String, Arc<Mutex<TextEmbedding>>>,
}

impl AppState {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut models = HashMap::new();

        // Initialize default model (all-MiniLM-L6-v2)
        println!("Loading AllMiniLmL6V2 model...");

        // Set-up sentence embeddings model
        let model = tokio::task::spawn_blocking(|| {
            TextEmbedding::try_new(Default::default())
        }).await??;
        models.insert("all-minilm-l6-v2".to_string(), Arc::new(Mutex::new(model)));

        println!("Models loaded successfully!");

        Ok(AppState { models })
    }

    fn get_available_models(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }
}

// Handlers
async fn health(state: axum::extract::State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        models: state.get_available_models(),
    })
}

async fn embeddings_post(
    state: axum::extract::State<Arc<AppState>>,
    Json(request): Json<EmbeddingRequest>,
) -> Result<Json<EmbeddingResponse>, (StatusCode, Json<ErrorResponse>)> {
    let model_name = request.model.unwrap_or_else(|| "all-minilm-l6-v2".to_string());

    let model = state.models.get(&model_name).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Model '{}' not available", model_name),
            }),
        )
    })?;

    if request.texts.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No texts provided".to_string(),
            }),
        ));
    }

    let embeddings = {
        let model_guard = model.lock().await;
        model_guard.embed(request.texts, None).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Embedding generation failed: {}", e),
                }),
            )
        })?
    };

    let dimensions = if !embeddings.is_empty() {
        embeddings[0].len()
    } else {
        0
    };

    Ok(Json(EmbeddingResponse {
        embeddings,
        model: model_name,
        dimensions,
    }))
}

async fn embeddings_get(
    state: axum::extract::State<Arc<AppState>>,
    Query(params): Query<EmbeddingQueryRequest>,
) -> Result<Json<EmbeddingResponse>, (StatusCode, Json<ErrorResponse>)> {
    let request = EmbeddingRequest {
        texts: vec![params.text],
        model: params.model,
    };

    embeddings_post(state, Json(request)).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Initializing embeddings service...");

    // Initialize application state
    let state = Arc::new(AppState::new().await?);

    // Build the application router
    let app = Router::new()
        .route("/health", get(health))
        .route("/embeddings", post(embeddings_post))
        .route("/embeddings", get(embeddings_get))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive()) // Enable CORS for all routes
        )
        .with_state(state);

    // Start the server
    let port = std::env::var("PORT").unwrap_or_else(|_| "9000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 Embeddings service starting on {}", addr);
    println!("📋 Available endpoints:");
    println!("  GET  /health - Service health check");
    println!("  POST /embeddings - Generate embeddings (JSON body)");
    println!("  GET  /embeddings?text=your_text - Generate single embedding");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}


