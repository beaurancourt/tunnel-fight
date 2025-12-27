mod api;
mod apl;
mod combat;
mod stats;
mod types;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = api::create_router();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Tunnel Fight server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
