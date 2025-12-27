mod api;
mod apl;
mod combat;
mod stats;
mod types;

use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = api::create_router();

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Tunnel Fight server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
