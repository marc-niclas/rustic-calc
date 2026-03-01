use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rustic_calc::tui_app::{App, Focus, InputEditMode};

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn app_starts_with_empty_state() {
    let app = App::new();
    assert_eq!(app.input, "");
    assert_eq!(app.character_index, 0);
    assert!(app.history.is_empty());
    assert_eq!(app.focus, Focus::Input);
    assert_eq!(app.input_edit_mode, InputEditMode::Insert);
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
        Some("Unknown variables: a, s, d, f")
    );
}

#[test]
fn up_arrow_recalls_last_expression_in_insert_mode() {
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

#[test]
fn esc_in_insert_switches_to_normal_mode() {
    let mut app = App::new();

    app.handle_key_event(key_event(KeyCode::Esc));

    assert_eq!(app.focus, Focus::Input);
    assert_eq!(app.input_edit_mode, InputEditMode::Normal);
}

#[test]
fn esc_in_normal_switches_focus_to_history_and_selects_first() {
    let mut app = App::new();
    app.input = "x=2".to_string();
    app.submit_message();
    app.input = "y=3".to_string();
    app.submit_message();

    app.handle_key_event(key_event(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key_event(KeyCode::Esc)); // Normal -> History
    app.handle_key_event(key_event(KeyCode::Right)); // History -> Variables

    assert_eq!(app.focus, Focus::Variables);
    assert_eq!(app.variables_state.selected(), Some(0));
}

#[test]
fn pressing_i_while_not_in_input_mode_re_enters_input_insert_mode() {
    let mut app = App::new();
    app.input = "x=2".to_string();
    app.submit_message();

    app.handle_key_event(key_event(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key_event(KeyCode::Esc)); // Normal -> Variables
    assert_eq!(app.focus, Focus::History);

    app.handle_key_event(key_event(KeyCode::Char('i')));

    assert_eq!(app.focus, Focus::Input);
    assert_eq!(app.input_edit_mode, InputEditMode::Insert);
}

#[test]
fn enter_on_history_populates_input_from_selected_item() {
    let mut app = App::new();
    app.input = "1+1".to_string();
    app.submit_message();
    app.input = "2+2".to_string();
    app.submit_message();

    app.handle_key_event(key_event(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key_event(KeyCode::Esc)); // Normal -> Variables
    app.handle_key_event(key_event(KeyCode::Left)); // Variables -> History
    app.handle_key_event(key_event(KeyCode::Enter)); // Populate input

    assert_eq!(app.input, "2+2");
    assert_eq!(app.character_index, 3);
    assert_eq!(app.focus, Focus::Input);
    assert_eq!(app.input_edit_mode, InputEditMode::Insert);
}

#[test]
fn enter_on_variables_populates_input_from_selected_variable_expression() {
    let mut app = App::new();
    app.input = "x=2".to_string();
    app.submit_message();
    app.input = "y=3".to_string();
    app.submit_message();

    app.handle_key_event(key_event(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key_event(KeyCode::Esc)); // Normal -> History
    app.handle_key_event(key_event(KeyCode::Right)); // History -> Variables
    app.handle_key_event(key_event(KeyCode::Enter)); // Populate input from selected variable

    assert_eq!(app.input, "x=2");
    assert_eq!(app.character_index, 3);
    assert_eq!(app.focus, Focus::Input);
    assert_eq!(app.input_edit_mode, InputEditMode::Insert);
}

#[test]
fn normal_mode_y_and_yy_no_longer_copy() {
    let mut app = App::new();
    app.input = "abc".to_string();
    app.character_index = app.input.chars().count();

    app.handle_key_event(key_event(KeyCode::Esc)); // Insert -> Normal

    app.handle_key_event(key_event(KeyCode::Char('y')));
    app.handle_key_event(key_event(KeyCode::Char('p')));
    assert_eq!(app.input, "abc");

    app.handle_key_event(key_event(KeyCode::Char('y')));
    app.handle_key_event(key_event(KeyCode::Char('y')));
    app.handle_key_event(key_event(KeyCode::Char('p')));
    assert_eq!(app.input, "abc");
}

#[test]
fn visual_mode_v_toggles_and_esc_returns_to_normal() {
    let mut app = App::new();
    app.input = "abcd".to_string();
    app.character_index = app.input.chars().count();

    app.handle_key_event(key_event(KeyCode::Esc)); // Insert -> Normal
    assert_eq!(app.input_edit_mode, InputEditMode::Normal);

    app.handle_key_event(key_event(KeyCode::Char('v'))); // Normal -> Visual
    assert_eq!(app.input_edit_mode, InputEditMode::Visual);

    app.handle_key_event(key_event(KeyCode::Char('v'))); // Visual -> Normal
    assert_eq!(app.input_edit_mode, InputEditMode::Normal);

    app.handle_key_event(key_event(KeyCode::Char('v'))); // Normal -> Visual
    app.handle_key_event(key_event(KeyCode::Esc)); // Visual -> Normal
    assert_eq!(app.input_edit_mode, InputEditMode::Normal);
}

#[test]
fn visual_mode_yank_then_paste_works() {
    let mut app = App::new();
    app.input = "abcde".to_string();
    app.character_index = app.input.chars().count();

    app.handle_key_event(key_event(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key_event(KeyCode::Char('0'))); // at 'a'
    app.handle_key_event(key_event(KeyCode::Char('v'))); // start visual
    app.handle_key_event(key_event(KeyCode::Char('l'))); // select "ab"
    app.handle_key_event(key_event(KeyCode::Char('y'))); // yank selection, back to normal
    app.handle_key_event(key_event(KeyCode::Char('$'))); // end
    app.handle_key_event(key_event(KeyCode::Char('p'))); // paste after cursor

    assert_eq!(app.input, "abcdeab");
    assert_eq!(app.input_edit_mode, InputEditMode::Normal);
}

#[test]
fn visual_mode_delete_selection_works() {
    let mut app = App::new();
    app.input = "abcde".to_string();
    app.character_index = app.input.chars().count();

    app.handle_key_event(key_event(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key_event(KeyCode::Char('0'))); // at 'a'
    app.handle_key_event(key_event(KeyCode::Char('l'))); // at 'b'
    app.handle_key_event(key_event(KeyCode::Char('v'))); // anchor at 'b'
    app.handle_key_event(key_event(KeyCode::Char('l'))); // select "bc"
    app.handle_key_event(key_event(KeyCode::Char('d'))); // delete selection

    assert_eq!(app.input, "ade");
    assert_eq!(app.input_edit_mode, InputEditMode::Normal);
}

#[test]
fn save_variable() {
    let mut app = App::new();
    app.input = "x=2".to_string();
    app.character_index = 3;

    app.submit_message();

    assert_eq!(app.input, "");
    assert_eq!(app.character_index, 0);
    assert_eq!(app.history.len(), 0);
    assert_eq!(
        app.variables.get("x").unwrap().expression,
        "x=2".to_string()
    );
    assert_eq!(app.variables.get("x").unwrap().value, 2.0);
}

#[test]
fn plot_expression() {
    let mut app = App::new();
    app.input = "7x+1".to_string();
    app.character_index = 4;

    app.submit_message();

    assert_eq!(app.input, "");
    assert_eq!(app.character_index, 0);
    assert_eq!(app.history.len(), 1);
    let plot_data = app.plot_data.unwrap();
    assert_eq!(plot_data.len(), 10);
    println!("{plot_data:?}");
    assert_eq!(plot_data[0], (0.0, 1.0));
    assert_eq!(plot_data[9], (9.0, 64.0));
}
