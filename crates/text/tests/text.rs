use konekode_text::{char_at, is_ascii_text, last_index_of, slice};

#[test]
fn only_plain_ascii_counts_as_ascii() {
    assert!(is_ascii_text("int a{};"));
    assert!(!is_ascii_text("中"));
}

#[test]
fn negative_positions_count_from_the_end_like_the_browser() {
    assert_eq!(slice("hello", -3, 5), "llo");
    assert_eq!(slice("hello", -10, 5), "hello");
    assert_eq!(slice("hello", 1, -1), "ell");
    assert_eq!(slice("hello", -2, -1), "l");
}

#[test]
fn a_character_outside_the_text_is_empty() {
    assert_eq!(char_at("abc", -1), "");
    assert_eq!(char_at("abc", 3), "");
    assert_eq!(char_at("abc", 1), "b");
}

#[test]
fn searching_backwards_starts_from_where_the_browser_starts() {
    assert_eq!(last_index_of("abcabc", "c", 10), 5);
    assert_eq!(last_index_of("abcabc", "c", 4), 2);
    assert_eq!(last_index_of("abc", "c", -1), -1);
    assert_eq!(last_index_of("abc", "", 10), 3);
}
