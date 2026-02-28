use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rustic_calc::tui_app::App;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn app_starts_with_empty_state() {
    let app = App::new();
    assert_eq!(app.input, "");
    assert_eq!(app.character_index, 0);
    assert!(app.history.is_empty());
}

#[test]
fn enter_char_and_cursor_movement_work() {
    let mut app = App::new();
    app.enter_char('1');
    app.enter_char('+');
    app.enter_char('2');
    assert_eq!(app.input, "1+2");
    assert_eq!(app.character_index, 3);

    app.move_cursor_left();
    app.move_cursor_left();
    assert_eq!(app.character_index, 1);
    app.enter_char('0');
    assert_eq!(app.input, "10+2");
    assert_eq!(app.character_index, 2);
}

#[test]
fn delete_char_removes_character_before_cursor() {
    let mut app = App::new();
    app.input = "12+3".to_string();
    app.character_index = 2;

    app.delete_char();

    assert_eq!(app.input, "1+3");
    assert_eq!(app.character_index, 1);
}

#[test]
fn submit_message_records_success_and_clears_input() {
    let mut app = App::new();
    app.input = "2+2".to_string();
    app.character_index = 3;

    app.submit_message();

    assert_eq!(app.input, "");
    assert_eq!(app.character_index, 0);
    assert_eq!(app.history.len(), 1);
    assert_eq!(app.history[0].expression, "2+2");
    assert_eq!(app.history[0].result, Some(4.0));
    assert_eq!(app.history[0].error, None);
}

#[test]
fn submit_message_records_error_and_clears_input() {
    let mut app = App::new();
    app.input = "asdf".to_string();
    app.character_index = 4;

    app.submit_message();

    assert_eq!(app.input, "");
    assert_eq!(app.character_index, 0);
    assert_eq!(app.history.len(), 1);
    assert_eq!(app.history[0].expression, "asdf");
    assert_eq!(app.history[0].result, None);
    assert_eq!(
        app.history[0].error.as_deref(),
        Some("Expression could not be parsed")
    );
}

#[test]
fn up_arrow_recalls_last_expression() {
    let mut app = App::new();
    app.input = "1+1".to_string();
    app.submit_message();

    app.handle_key_event(key_event(KeyCode::Up));

    assert_eq!(app.input, "1+1");
    assert_eq!(app.character_index, 3);
}

#[test]
fn ctrl_c_returns_quit_signal() {
    let mut app = App::new();
    let quit = app.handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(quit);
}
