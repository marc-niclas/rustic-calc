use rustic_calc::{
    io::{get_state_from_file, write_state_to_file},
    tui_app::App,
};

#[path = "common/state.rs"]
mod state;
#[path = "common/temp_home.rs"]
mod temp_home;
#[path = "common/with_home.rs"]
mod with_home;

use state::sample_state;
use temp_home::temp_home_dir;
use with_home::with_home;

#[test]
fn write_and_read_state_round_trip() {
    let home = temp_home_dir("roundtrip");

    with_home(&home, || {
        let state = sample_state();

        write_state_to_file(&state).expect("write_state_to_file should succeed");
        let loaded = get_state_from_file().expect("get_state_from_file should succeed");

        assert_eq!(loaded.history.len(), 1);
        assert_eq!(loaded.history[0].expression, "1+1");
        assert_eq!(loaded.history[0].result, Some(2.0));
        assert!(loaded.history[0].error.is_none());

        assert_eq!(loaded.variables.len(), 1);
        let x = loaded.variables.get("x").expect("x should exist");
        assert_eq!(x.expression, "2+3");
        assert_eq!(x.value, 5.0);

        assert_eq!(loaded.plot_data.as_ref().map(Vec::len), Some(2));
    });
}

#[test]
fn app_starts_from_saved_state_via_app_from() {
    let home = temp_home_dir("start-from-file");

    with_home(&home, || {
        let state = sample_state();
        write_state_to_file(&state).expect("state write should succeed");

        let loaded = get_state_from_file().expect("state read should succeed");
        let app = App::from(&loaded);

        assert_eq!(app.history.len(), 1);
        assert_eq!(app.history[0].expression, "1+1");
        assert_eq!(app.variables.get("x").map(|v| v.value), Some(5.0));
        assert_eq!(app.plot_data.as_ref().map(Vec::len), Some(2));
    });
}

#[test]
fn app_submit_message_saves_state_to_file() {
    let home = temp_home_dir("save-on-submit");

    with_home(&home, || {
        let mut app = App::new();
        app.input = "2+2".to_string();
        app.character_index = app.input.chars().count();

        app.submit_message();

        let loaded = get_state_from_file().expect("state should be saved after submit");
        assert_eq!(loaded.history.len(), 1);
        assert_eq!(loaded.history[0].expression, "2+2");
        assert_eq!(loaded.history[0].result, Some(4.0));
        assert!(loaded.history[0].error.is_none());
    });
}
