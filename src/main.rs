use std::time::Duration;

use howlcore::{build_app, combat::resource::WorkStateData};

fn main() {
    let mut app = build_app();
    let _ = crossterm::terminal::enable_raw_mode();

    println!("Howlcore started. Press Space to advance one work round. Press Ctrl+C to quit.");

    loop {
        app.update();

        if app
            .world()
            .get_resource::<WorkStateData>()
            .is_some_and(|state| state.is_finished)
        {
            break;
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    let _ = crossterm::terminal::disable_raw_mode();
}
