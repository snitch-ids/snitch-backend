use snitch_backend::backend_main;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    backend_main().await
}
