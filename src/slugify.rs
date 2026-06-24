/// Unicode-aware slugify wrapper.
///
/// Falls back to preserving non-Latin alphanumeric characters
/// when the `slug` crate strips everything (e.g. CJK text).
pub fn slugify<S: AsRef<str>>(text: S) -> String {
    let text = text.as_ref();
    let result = slug::slugify(text);
    if !result.is_empty() {
        return result;
    }
    unicode_slugify(text)
}

#[cfg(test)]
pub(crate) fn unicode_slugify_for_test(text: &str) -> String {
    unicode_slugify(text)
}

fn unicode_slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut prev_was_sep = true;

    for c in text.chars() {
        if c.is_alphanumeric() {
            for lc in c.to_lowercase() {
                slug.push(lc);
            }
            prev_was_sep = false;
        } else if (c.is_whitespace() || c == '_' || c == '-') && !prev_was_sep && !slug.is_empty() {
            slug.push('-');
            prev_was_sep = true;
        }
    }

    if slug.ends_with('-') {
        slug.pop();
    }

    slug
}

#[cfg(test)]
#[path = "tests/slugify.rs"]
mod tests;
