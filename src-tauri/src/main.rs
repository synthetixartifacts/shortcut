// Hide console window in release builds (production)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Catch panics and display them
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC: {}", panic_info);
        // Keep console open
        println!("Press Enter to exit...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }));

    println!("Starting ShortCut...");
    shortcut_lib::run();
    println!("ShortCut finished");
}
