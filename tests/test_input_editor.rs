use crossterm::event::{KeyCode, KeyEvent};
use rustic_calc::{
    input_editor::{InputEditor, Motion},
    tui_app::{App, Focus, InputEditMode},
};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::from(code)
}

#[test]
fn basic_insert_and_backspace() {
    let mut ed = InputEditor::new();
    ed.handle_key_event(key(KeyCode::Char('1')));
    ed.handle_key_event(key(KeyCode::Char('+')));
    ed.handle_key_event(key(KeyCode::Char('2')));
    assert_eq!(ed.input(), "1+2");
    assert_eq!(ed.cursor(), 3);

    ed.handle_key_event(key(KeyCode::Backspace));
    assert_eq!(ed.input(), "1+");
    assert_eq!(ed.cursor(), 2);
}

#[test]
fn normal_motions_are_reusable_for_navigation() {
    let mut ed = InputEditor::with_input("abc + def".to_string());
    ed.handle_key_event(key(KeyCode::Esc)); // -> Normal

    ed.apply_motion(Motion::LineStart);
    assert_eq!(ed.cursor(), 0);

    ed.apply_motion(Motion::WordForward);
    assert_eq!(ed.cursor(), 6);

    ed.apply_motion(Motion::WordBackward);
    assert_eq!(ed.cursor(), 0);

    ed.apply_motion(Motion::LineEnd);
    assert_eq!(ed.cursor(), 8);
}

#[test]
fn normal_mode_y_and_yy_do_not_copy_in_input_editor() {
    let mut ed = InputEditor::with_input("hello world".to_string());
    ed.handle_key_event(key(KeyCode::Esc)); // Normal
    ed.apply_motion(Motion::LineStart);

    ed.handle_key_event(key(KeyCode::Char('y')));
    ed.handle_key_event(key(KeyCode::Char('w')));
    assert_eq!(ed.register(), "");

    ed.apply_motion(Motion::LineEnd);
    ed.handle_key_event(key(KeyCode::Char('p')));
    assert_eq!(ed.input(), "hello world");

    ed.handle_key_event(key(KeyCode::Char('y')));
    ed.handle_key_event(key(KeyCode::Char('y')));
    assert_eq!(ed.register(), "");

    ed.handle_key_event(key(KeyCode::Char('P')));
    assert_eq!(ed.input(), "hello world");
}

#[test]
fn visual_mode_yank_and_paste_work_in_input_editor() {
    let mut ed = InputEditor::with_input("sum=1+2".to_string());
    ed.handle_key_event(key(KeyCode::Esc)); // Normal
    ed.handle_key_event(key(KeyCode::Char('0'))); // at 's'
    ed.handle_key_event(key(KeyCode::Char('v'))); // visual start
    ed.handle_key_event(key(KeyCode::Char('l'))); // select "su"
    ed.handle_key_event(key(KeyCode::Char('y'))); // yank visual selection

    assert_eq!(ed.register(), "su");

    ed.apply_motion(Motion::LineEnd);
    ed.handle_key_event(key(KeyCode::Char('p')));

    assert_eq!(ed.input(), "sum=1+2su");
}

#[test]
fn normal_mode_y_and_yy_do_not_copy_in_app() {
    let mut app = App::new();
    app.input = "hello world".to_string();
    app.character_index = app.input.chars().count();

    app.handle_key_event(key(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key(KeyCode::Char('0')));
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char('w')));
    app.handle_key_event(key(KeyCode::Char('$')));
    app.handle_key_event(key(KeyCode::Char('p')));

    assert_eq!(app.input, "hello world");
}

#[test]
fn normal_mode_yy_then_paste_before_is_no_op_in_app() {
    let mut app = App::new();
    app.input = "sum=1+2".to_string();
    app.character_index = app.input.chars().count();

    app.handle_key_event(key(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char('0')));
    app.handle_key_event(key(KeyCode::Char('P')));

    assert_eq!(app.input, "sum=1+2");
}

#[test]
fn normal_mode_v_enters_visual_mode_and_esc_exits_to_normal() {
    let mut app = App::new();
    app.input = "abcd".to_string();
    app.character_index = app.input.chars().count();

    app.handle_key_event(key(KeyCode::Esc)); // Insert -> Normal
    assert_eq!(app.input_edit_mode, InputEditMode::Normal);

    app.handle_key_event(key(KeyCode::Char('v'))); // Normal -> Visual
    assert_eq!(app.input_edit_mode, InputEditMode::Visual);

    app.handle_key_event(key(KeyCode::Esc)); // Visual -> Normal
    assert_eq!(app.input_edit_mode, InputEditMode::Normal);
    assert_eq!(app.focus, Focus::Input);
}

#[test]
fn visual_mode_yank_and_paste_work() {
    let mut app = App::new();
    app.input = "abcde".to_string();
    app.character_index = app.input.chars().count();

    app.handle_key_event(key(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key(KeyCode::Char('0'))); // cursor at 'a'
    app.handle_key_event(key(KeyCode::Char('v'))); // start visual selection
    app.handle_key_event(key(KeyCode::Char('l'))); // select "ab"
    app.handle_key_event(key(KeyCode::Char('y'))); // yank selection, back to normal

    assert_eq!(app.input_edit_mode, InputEditMode::Normal);

    app.handle_key_event(key(KeyCode::Char('$'))); // move to end
    app.handle_key_event(key(KeyCode::Char('p'))); // paste after

    assert_eq!(app.input, "abcdeab");
}

#[test]
fn visual_mode_delete_selection_works() {
    let mut app = App::new();
    app.input = "abcde".to_string();
    app.character_index = app.input.chars().count();

    app.handle_key_event(key(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key(KeyCode::Char('0'))); // cursor at 'a'
    app.handle_key_event(key(KeyCode::Char('l'))); // cursor at 'b'
    app.handle_key_event(key(KeyCode::Char('v'))); // start visual selection at 'b'
    app.handle_key_event(key(KeyCode::Char('l'))); // extend to 'c'
    app.handle_key_event(key(KeyCode::Char('d'))); // delete "bc"

    assert_eq!(app.input_edit_mode, InputEditMode::Normal);
    assert_eq!(app.input, "ade");
}

#[test]
fn normal_mode_navigation_and_delete_under_cursor_work() {
    let mut app = App::new();
    app.input = "12+34".to_string();
    app.character_index = 5;

    app.handle_key_event(key(KeyCode::Esc)); // Insert -> Normal

    app.handle_key_event(key(KeyCode::Char('0')));
    assert_eq!(app.character_index, 0);

    app.handle_key_event(key(KeyCode::Char('$')));
    assert_eq!(app.character_index, 4);

    app.handle_key_event(key(KeyCode::Char('h')));
    assert_eq!(app.character_index, 3);

    app.handle_key_event(key(KeyCode::Char('x')));
    assert_eq!(app.input, "12+4");
    assert_eq!(app.character_index, 3);
}

#[test]
fn normal_mode_word_motions_work() {
    let mut app = App::new();
    app.input = "abc + def_1".to_string();
    app.character_index = 0;

    app.handle_key_event(key(KeyCode::Esc)); // Insert -> Normal
    app.handle_key_event(key(KeyCode::Char('w')));
    assert_eq!(app.character_index, 6);

    app.handle_key_event(key(KeyCode::Char('b')));
    assert_eq!(app.character_index, 0);
}
