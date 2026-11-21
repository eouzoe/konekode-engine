use konekode_typing::{Feedback, Next, create_drill, press};

fn after(drill: &konekode_typing::Drill, key: &str) -> konekode_typing::Drill {
    press(drill, key).resolve(drill)
}

#[test]
fn a_new_line_skips_the_indent_and_clears_the_warning() {
    let start = after(&create_drill("a\n  b"), "i");
    let typed = after(&start, "a");
    assert_eq!(typed.position, 1);
    let next_line = after(&typed, "\n");
    assert_eq!(next_line.position, 4);
    assert_eq!(next_line.feedback, Feedback::None);
    let wrong = after(&typed, "x");
    assert_eq!(wrong.feedback, Feedback::WrongKey);
    let fixed = after(&wrong, "\n");
    assert_eq!(fixed.position, 4);
    assert_eq!(fixed.feedback, Feedback::None);
    assert!(matches!(press(&fixed, "b"), Next::New(_)));
}
