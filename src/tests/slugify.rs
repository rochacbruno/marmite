use super::*;

#[test]
fn test_latin_text() {
    assert_eq!(slugify("Simple Text"), "simple-text");
}

#[test]
fn test_accented_latin() {
    assert_eq!(slugify("Comunicação"), "comunicacao");
}

#[test]
fn test_chinese_produces_nonempty_slug() {
    let slug = slugify("你好");
    assert!(!slug.is_empty());
}

#[test]
fn test_chinese_distinct_tags_produce_distinct_slugs() {
    let slug1 = slugify("你好");
    let slug2 = slugify("中文");
    assert_ne!(slug1, slug2);
    assert!(!slug1.is_empty());
    assert!(!slug2.is_empty());
}

#[test]
fn test_japanese_produces_nonempty_slug() {
    let slug = slugify("日本語");
    assert!(!slug.is_empty());
}

#[test]
fn test_korean_produces_nonempty_slug() {
    let slug = slugify("한국어");
    assert!(!slug.is_empty());
}

#[test]
fn test_empty_string() {
    assert_eq!(slugify(""), "");
}

#[test]
fn test_mixed_latin_and_cjk() {
    let result = slugify("hello 你好");
    assert!(!result.is_empty());
}

#[test]
fn test_slugify_is_idempotent() {
    let slug = slugify("你好");
    assert_eq!(slugify(&slug), slug);
}

#[test]
fn test_latin_unchanged_behavior() {
    assert_eq!(
        slugify("Text with Special Characters!@#"),
        "text-with-special-characters"
    );
    assert_eq!(
        slugify("Text    with    multiple    spaces"),
        "text-with-multiple-spaces"
    );
    assert_eq!(slugify("Text_with_underscores"), "text-with-underscores");
    assert_eq!(slugify("Text with numbers 123"), "text-with-numbers-123");
}

#[test]
fn test_unicode_fallback_with_spaces() {
    let result = unicode_slugify_for_test("你好 世界");
    assert_eq!(result, "你好-世界");
}

#[test]
fn test_unicode_fallback_trims_hyphens() {
    let result = unicode_slugify_for_test("  你好  ");
    assert_eq!(result, "你好");
}

#[test]
fn test_unicode_fallback_collapses_separators() {
    let result = unicode_slugify_for_test("你好   世界");
    assert_eq!(result, "你好-世界");
}

#[test]
fn test_unicode_fallback_strips_punctuation() {
    let result = unicode_slugify_for_test("你好！世界");
    assert!(!result.contains('！'));
}
