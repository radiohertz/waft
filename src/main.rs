use axum::Router;

pub mod v1;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await?;
    let router = Router::new().nest("/v1", v1::router());
    println!("Serving on https://127.0.0.1:5000");
    axum::serve(listener, router).await?;
    Ok(())
}
