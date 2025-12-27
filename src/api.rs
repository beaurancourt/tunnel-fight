use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use crate::combat::CombatSimulator;
use crate::stats::{SimulationResult, StatsCollector};
use crate::types::Encounter;

#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    pub encounter_yaml: String,
    #[serde(default = "default_sample_count")]
    pub sample_count: usize,
    pub seed: Option<u64>,
}

fn default_sample_count() -> usize {
    5
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/simulate", post(simulate))
        .layer(cors)
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn simulate(Json(request): Json<SimulateRequest>) -> impl IntoResponse {
    // Parse the encounter YAML
    let encounter: Encounter = match serde_yaml::from_str(&request.encounter_yaml) {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Invalid YAML: {}", e) })),
            )
                .into_response();
        }
    };

    // Create RNG
    let mut rng = match request.seed {
        Some(seed) => ChaCha8Rng::seed_from_u64(seed),
        None => ChaCha8Rng::from_entropy(),
    };

    // Calculate totals for stats (using expected values for dice-based HP)
    let side1_count = encounter.side1.len();
    let side2_count = encounter.side2.len();
    let side1_total_hp: i32 = encounter.side1.iter().map(|a| a.hp.expected_value() as i32).sum();
    let side2_total_hp: i32 = encounter.side2.iter().map(|a| a.hp.expected_value() as i32).sum();

    let mut collector = StatsCollector::new(side1_count, side2_count, side1_total_hp, side2_total_hp);

    // Run simulations
    let iterations = encounter.iterations;
    for _ in 0..iterations {
        let mut sim = CombatSimulator::new(&encounter, 100, &mut rng);
        let result = sim.run(&mut rng);
        collector.add_result(result);
    }

    let stats = collector.compute_stats();
    let sample_combats = collector.get_sample_combats(request.sample_count);

    let result = SimulationResult {
        stats,
        sample_combats,
    };

    (StatusCode::OK, Json(result)).into_response()
}
