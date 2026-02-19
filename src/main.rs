use aivim_tui::App;
use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    
    let mut app = if args.len() > 1 {
        let file_path = PathBuf::from(&args[1]);
        App::with_file(file_path)?
    } else {
        App::new()
    };

    app.run()?;
    
    Ok(())
}
