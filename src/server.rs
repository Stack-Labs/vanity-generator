use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use crate::GrindArgs;
use sha2::Digest;
use rand::Rng;

#[derive(Clone)]
struct AppState {
    token_program_id: Pubkey,
}

#[derive(Deserialize)]
struct GenerateRequest {
    base: String,
}

#[derive(Serialize)]
struct GenerateResponse {
    address: String,
    seed: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn health_check() -> &'static str {
    "ok"
}

fn grind_with_result(args: GrindArgs) -> (String, Pubkey) {
    let mut seed = [0u8; 16];
    let mut found = false;
    let mut address = Pubkey::default();

    // Run the grind function with a closure to capture the result
    let base_sha = sha2::Sha256::new().chain_update(args.base);
    let prefix = args.prefix.as_deref().unwrap_or("");
    let suffix = args.suffix.as_deref().unwrap_or("");
    
    let timer = std::time::Instant::now();
    let mut count = 0_u64;

    while !found {
        let mut seed_iter = rand::thread_rng().sample_iter(&rand::distributions::Alphanumeric).take(16);
        seed = std::array::from_fn(|_| seed_iter.next().unwrap());

        let pubkey_bytes: [u8; 32] = base_sha
            .clone()
            .chain_update(seed)
            .chain_update(args.owner)
            .finalize()
            .into();
        let pubkey = fd_bs58::encode_32(pubkey_bytes);
        let out_str_target_check = if args.case_insensitive {
            pubkey.to_ascii_lowercase()
        } else {
            pubkey.clone()
        };

        count += 1;

        if out_str_target_check.starts_with(prefix) && out_str_target_check.ends_with(suffix) {
            address = Pubkey::new_from_array(pubkey_bytes);
            found = true;
        }
    }

    (std::str::from_utf8(&seed).unwrap().to_string(), address)
}

async fn generate_vanity_address(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, Json<ErrorResponse>> {
    // Validate base address
    let base = match Pubkey::try_from(req.base.as_str()) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(Json(ErrorResponse {
            error: "Invalid base address".to_string(),
        })),
    };

    // Create GrindArgs for the vanity generator
    let args = GrindArgs {
        base,
        owner: state.token_program_id,
        prefix: None,
        suffix: Some("Loop".to_string()),
        case_insensitive: false,
        logfile: None,
        num_cpus: 0,
    };

    // Run the grind function
    let (seed, address) = grind_with_result(args);

    Ok(Json(GenerateResponse {
        address: address.to_string(),
        seed,
    }))
}

pub async fn start_server() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create app state
    let state = Arc::new(AppState {
        token_program_id: Pubkey::try_from("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/generate", post(generate_vanity_address))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    // Run server
    let addr = "0.0.0.0:3001";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
