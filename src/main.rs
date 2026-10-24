mod engine; mod scenes; mod texture; mod screen; mod animation; mod font; mod audio; mod ball; mod settings;

// TODO: Un-Vibe Code Scenes
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = engine::TerminalGuard::new()?;
    let ctx = engine::Context::new()?;
    let first = scenes::splash::Splash::new(&ctx)?;
    engine::run(ctx, Box::new(first))
}