#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RustCodeContext {
    Production,
    Test,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceContexts {
    line_contexts: Vec<RustCodeContext>,
}

impl RustSourceContexts {
    pub(crate) fn analyse(source: &str) -> Self {
        let clean = sanitise_rust_structure(source);
        let tokens = lex_structure_tokens(&clean);
        let ranges = test_item_ranges(&tokens);
        let line_count = source.lines().count().max(1);
        let mut line_contexts = vec![RustCodeContext::Production; line_count + 1];
        for (start, end) in ranges {
            for context in line_contexts
                .iter_mut()
                .take(end.min(line_count) + 1)
                .skip(start)
            {
                *context = RustCodeContext::Test;
            }
        }
        Self { line_contexts }
    }

    pub(crate) fn context_for_line(&self, line_number: usize) -> RustCodeContext {
        self.line_contexts
            .get(line_number)
            .copied()
            .unwrap_or(RustCodeContext::Production)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Hash,
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semi,
    Ident(String),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    line: usize,
}

fn sanitise_rust_structure(source: &str) -> String {
    let chars = source.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < chars.len() {
        if starts_with(&chars, i, &['/', '/']) {
            i = consume_line_comment(&chars, i, &mut out);
        } else if starts_with(&chars, i, &['/', '*']) {
            i = consume_block_comment(&chars, i, &mut out);
        } else if let Some((prefix_len, hash_count)) = raw_string_start(&chars, i) {
            i = consume_raw_string(&chars, i, prefix_len, hash_count, &mut out);
        } else if starts_with(&chars, i, &['b', '"']) {
            i = consume_quoted(&chars, i, 2, '"', &mut out);
        } else if chars[i] == '"' {
            i = consume_quoted(&chars, i, 1, '"', &mut out);
        } else if starts_with(&chars, i, &['b', '\'']) && is_char_literal_at(&chars, i + 1) {
            i = consume_quoted(&chars, i, 2, '\'', &mut out);
        } else if chars[i] == '\'' && is_char_literal_at(&chars, i) {
            i = consume_quoted(&chars, i, 1, '\'', &mut out);
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn starts_with(chars: &[char], index: usize, needle: &[char]) -> bool {
    chars
        .get(index..index.saturating_add(needle.len()))
        .is_some_and(|slice| slice == needle)
}

fn push_masked(out: &mut String, ch: char) {
    if ch == '\n' {
        out.push('\n');
    } else {
        out.push(' ');
    }
}

fn consume_line_comment(chars: &[char], mut i: usize, out: &mut String) -> usize {
    while i < chars.len() {
        let ch = chars[i];
        push_masked(out, ch);
        i += 1;
        if ch == '\n' {
            break;
        }
    }
    i
}

fn consume_block_comment(chars: &[char], mut i: usize, out: &mut String) -> usize {
    let mut depth = 0usize;
    while i < chars.len() {
        if starts_with(chars, i, &['/', '*']) {
            push_masked(out, chars[i]);
            push_masked(out, chars[i + 1]);
            i += 2;
            depth += 1;
        } else if starts_with(chars, i, &['*', '/']) {
            push_masked(out, chars[i]);
            push_masked(out, chars[i + 1]);
            i += 2;
            depth = depth.saturating_sub(1);
            if depth == 0 {
                break;
            }
        } else {
            push_masked(out, chars[i]);
            i += 1;
        }
    }
    i
}

fn raw_string_start(chars: &[char], index: usize) -> Option<(usize, usize)> {
    let mut i = index;
    if chars.get(i) == Some(&'b') {
        i += 1;
    }
    if chars.get(i) != Some(&'r') {
        return None;
    }
    i += 1;
    let mut hashes = 0usize;
    while chars.get(i) == Some(&'#') {
        hashes += 1;
        i += 1;
    }
    if chars.get(i) == Some(&'"') {
        Some((i - index + 1, hashes))
    } else {
        None
    }
}

fn consume_raw_string(
    chars: &[char],
    mut i: usize,
    prefix_len: usize,
    hash_count: usize,
    out: &mut String,
) -> usize {
    for _ in 0..prefix_len {
        push_masked(out, chars[i]);
        i += 1;
    }

    while i < chars.len() {
        let ch = chars[i];
        push_masked(out, ch);
        i += 1;
        if ch == '"' {
            let mut matched = true;
            for offset in 0..hash_count {
                if chars.get(i + offset) != Some(&'#') {
                    matched = false;
                    break;
                }
            }
            if matched {
                for _ in 0..hash_count {
                    push_masked(out, chars[i]);
                    i += 1;
                }
                break;
            }
        }
    }
    i
}

fn consume_quoted(
    chars: &[char],
    mut i: usize,
    prefix_len: usize,
    quote: char,
    out: &mut String,
) -> usize {
    for _ in 0..prefix_len {
        push_masked(out, chars[i]);
        i += 1;
    }

    let mut escaped = false;
    while i < chars.len() {
        let ch = chars[i];
        push_masked(out, ch);
        i += 1;
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            break;
        }
    }
    i
}

fn is_char_literal_at(chars: &[char], quote_index: usize) -> bool {
    let content_index = quote_index + 1;
    let Some(&content) = chars.get(content_index) else {
        return false;
    };
    if content == '\n' {
        return false;
    }
    if content == '\\' {
        return closes_before_newline(chars, content_index + 1, '\'');
    }
    if is_ident_start(content) {
        return chars.get(content_index + 1) == Some(&'\'');
    }
    closes_before_newline(chars, content_index, '\'')
}

fn closes_before_newline(chars: &[char], mut i: usize, quote: char) -> bool {
    let mut escaped = false;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\n' {
            return false;
        }
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return true;
        }
        i += 1;
    }
    false
}

fn lex_structure_tokens(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars = source.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    let mut line = 1usize;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\n' {
            line += 1;
            i += 1;
        } else if ch.is_whitespace() {
            i += 1;
        } else if is_ident_start(ch) {
            let start = i;
            i += 1;
            while i < chars.len() && is_ident_continue(chars[i]) {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Ident(chars[start..i].iter().collect()),
                line,
            });
        } else {
            let kind = match ch {
                '#' => TokenKind::Hash,
                '[' => TokenKind::OpenBracket,
                ']' => TokenKind::CloseBracket,
                '(' => TokenKind::OpenParen,
                ')' => TokenKind::CloseParen,
                '{' => TokenKind::OpenBrace,
                '}' => TokenKind::CloseBrace,
                ';' => TokenKind::Semi,
                _ => TokenKind::Other,
            };
            tokens.push(Token { kind, line });
            i += 1;
        }
    }
    tokens
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn test_item_ranges(tokens: &[Token]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut pending_test_attr_start = None;
    let mut i = 0usize;
    while i < tokens.len() {
        if let Some(attr_end) = attr_end(tokens, i) {
            if attr_is_cfg_test(&tokens[i..=attr_end]) {
                pending_test_attr_start = Some(pending_test_attr_start.unwrap_or(tokens[i].line));
            }
            i = attr_end + 1;
            continue;
        }

        if let Some(start_line) = pending_test_attr_start.take() {
            let end = item_end(tokens, i);
            ranges.push((start_line, tokens[end].line));
            i = end + 1;
            continue;
        }

        i += 1;
    }
    ranges
}

fn attr_end(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches!(tokens.get(start)?.kind, TokenKind::Hash) {
        return None;
    }
    let mut i = start + 1;
    if matches!(tokens.get(i).map(|t| &t.kind), Some(TokenKind::Other)) {
        i += 1;
    }
    if !matches!(tokens.get(i).map(|t| &t.kind), Some(TokenKind::OpenBracket)) {
        return None;
    }

    let mut depth = 0usize;
    while i < tokens.len() {
        match tokens[i].kind {
            TokenKind::OpenBracket => depth += 1,
            TokenKind::CloseBracket => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn attr_is_cfg_test(tokens: &[Token]) -> bool {
    let Some(cfg_index) = tokens
        .iter()
        .position(|t| matches!(&t.kind, TokenKind::Ident(ident) if ident == "cfg"))
    else {
        return false;
    };
    let Some(open_index) = tokens[cfg_index + 1..]
        .iter()
        .position(|t| matches!(t.kind, TokenKind::OpenParen))
        .map(|offset| cfg_index + 1 + offset)
    else {
        return false;
    };
    let close_index = matching_paren(tokens, open_index).unwrap_or(tokens.len() - 1);
    cfg_expr_contains_positive_test(&tokens[open_index + 1..close_index])
}

fn matching_paren(tokens: &[Token], open_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        match token.kind {
            TokenKind::OpenParen => depth += 1,
            TokenKind::CloseParen => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn cfg_expr_contains_positive_test(tokens: &[Token]) -> bool {
    if matches!(
        tokens.first().map(|t| &t.kind),
        Some(TokenKind::Ident(ident)) if ident == "not"
    ) {
        return false;
    }

    tokens
        .iter()
        .enumerate()
        .any(|(index, token)| match &token.kind {
            TokenKind::Ident(ident) if ident == "test" => {
                !matches!(previous_ident(tokens, index).as_deref(), Some("not"))
            }
            _ => false,
        })
}

fn previous_ident(tokens: &[Token], before: usize) -> Option<String> {
    tokens[..before].iter().rev().find_map(|token| {
        if let TokenKind::Ident(ident) = &token.kind {
            Some(ident.clone())
        } else {
            None
        }
    })
}

fn item_end(tokens: &[Token], start: usize) -> usize {
    let mut i = start;
    let mut brace_depth = 0usize;
    let mut saw_open_brace = false;
    while i < tokens.len() {
        match tokens[i].kind {
            TokenKind::OpenBrace => {
                brace_depth += 1;
                saw_open_brace = true;
            }
            TokenKind::CloseBrace => {
                if saw_open_brace {
                    brace_depth = brace_depth.saturating_sub(1);
                    if brace_depth == 0 {
                        return i;
                    }
                }
            }
            TokenKind::Semi if !saw_open_brace => return i,
            _ => {}
        }
        i += 1;
    }
    tokens.len().saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_of(source: &str, needle: &str) -> usize {
        source
            .lines()
            .position(|line| line.contains(needle))
            .map_or_else(
                || panic!("missing line containing {needle:?}"),
                |index| index + 1,
            )
    }

    #[test]
    fn classifies_blocks_rs_inline_cfg_test_without_swallowing_production() {
        let source = include_str!("../../wildside-core/src/blocks.rs");
        let contexts = RustSourceContexts::analyse(source);

        assert_eq!(
            contexts.context_for_line(267),
            RustCodeContext::Test,
            "early #[cfg(test)] helper is test-only"
        );
        assert_eq!(
            contexts.context_for_line(1692),
            RustCodeContext::Production,
            "production citation after early gated helper must not be swallowed"
        );
        assert_eq!(
            contexts.context_for_line(4675),
            RustCodeContext::Production,
            "late production citation before mod tests remains production"
        );
        assert_eq!(
            contexts.context_for_line(5700),
            RustCodeContext::Test,
            "citation inside #[cfg(test)] mod tests is test-only"
        );
    }

    #[test]
    fn ignores_braces_in_comments_and_literals() {
        let source = r####"
#[cfg(test)]
fn test_only() {
    let char_brace = '{';
    let string_brace = "}";
    let raw_brace = r###"{ /* still a string */ }"###;
}

/* { */
fn production() {
    let char_brace = '{';
    // }
}
"####;
        let contexts = RustSourceContexts::analyse(source);

        assert_eq!(
            contexts.context_for_line(line_of(source, "char_brace = '{';")),
            RustCodeContext::Test
        );
        assert_eq!(
            contexts.context_for_line(line_of(source, "fn production")),
            RustCodeContext::Production
        );
        assert_eq!(
            contexts.context_for_line(line_of(source, "// }")),
            RustCodeContext::Production
        );
    }

    #[test]
    fn sanitiser_masks_comment_and_literal_structure() {
        let source = r####"fn production() {
    let char_brace = '{';
    let byte_char_brace = b'}';
    let string_brace = "}";
    let byte_string_brace = b"{";
    let raw_brace = r###"{ /* still a string */ }"###;
}
// #[cfg(test)] fn fake_line() { }
/* #[cfg(test)] fn fake_block() { /* nested } */ } */
fn after() {}
"####;

        let clean = sanitise_rust_structure(source);

        assert_eq!(clean.matches('\n').count(), source.matches('\n').count());
        assert!(!clean.contains("cfg"));
        assert_eq!(clean.matches('{').count(), 2);
        assert_eq!(clean.matches('}').count(), 2);
        assert!(clean.contains("fn production() {"));
        assert!(clean.contains("fn after() {}"));
    }

    #[test]
    fn sanitiser_consumes_block_comments_exactly() {
        assert_eq!(
            sanitise_rust_structure("/*{}*/fn after() {}"),
            "      fn after() {}"
        );
        let nested_comment = "/* outer /* nested } */ done */";
        assert_eq!(
            sanitise_rust_structure(&format!("{nested_comment}fn after() {{}}")),
            format!("{}fn after() {{}}", " ".repeat(nested_comment.len()))
        );
    }

    #[test]
    fn sanitiser_masks_raw_strings_exactly() {
        assert_eq!(sanitise_rust_structure(r##"r#"{ }"#"##), "        ");
        assert_eq!(sanitise_rust_structure(r##"br#"{ }"#"##), "         ");

        let source = r####"fn production() {
    let raw = r###"{ #[cfg(test)] fn fake() {} }"###;
    let byte_raw = br#"{ }"#;
}
"####;
        let clean = sanitise_rust_structure(source);

        assert_eq!(clean.matches('\n').count(), source.matches('\n').count());
        assert!(!clean.contains("cfg"));
        assert_eq!(clean.matches('{').count(), 1);
        assert_eq!(clean.matches('}').count(), 1);
    }

    #[test]
    fn sanitiser_masks_line_comment_at_eof() {
        assert_eq!(sanitise_rust_structure("// }"), "    ");
    }

    #[test]
    fn sanitiser_masks_byte_and_plain_char_literals_but_keeps_lifetimes() {
        assert_eq!(sanitise_rust_structure("b'{'"), "    ");
        assert_eq!(sanitise_rust_structure("x b'{'"), "x     ");
        assert_eq!(sanitise_rust_structure("'{'"), "   ");
        assert_eq!(sanitise_rust_structure("'a'"), "   ");
        assert_eq!(sanitise_rust_structure("'\\n'"), "    ");
        assert_eq!(sanitise_rust_structure("'a"), "'a");

        let source = "fn f<'a>() { let byte = b'{'; let plain = '{'; }\n";
        let clean = sanitise_rust_structure(source);

        assert!(clean.contains("fn f<'a>() {"));
        assert!(clean.contains("let byte =     ;"));
        assert!(clean.contains("let plain =    ;"));
        assert_eq!(clean.matches('{').count(), 1);
        assert_eq!(clean.matches('}').count(), 1);
    }

    #[test]
    fn cfg_test_range_includes_only_the_gated_item() {
        let source = "\
#[cfg(test)]
fn test_only() {
}
fn production() {
}
";
        let contexts = RustSourceContexts::analyse(source);

        assert_eq!(
            contexts.context_for_line(line_of(source, "fn test_only")),
            RustCodeContext::Test
        );
        assert_eq!(
            contexts.context_for_line(line_of(source, "}")),
            RustCodeContext::Test
        );
        assert_eq!(
            contexts.context_for_line(line_of(source, "fn production")),
            RustCodeContext::Production
        );
    }

    #[test]
    fn cfg_test_supports_any_and_semicolon_items_without_cfg_attr_false_positive() {
        let source = r#"
#[cfg(any(test, feature = "probe"))]
const TEST_ONLY: u8 = 1;
#[cfg_attr(test, allow(dead_code))]
const PRODUCTION: u8 = 2;
#[cfg(feature = "test-support")]
const FEATURE_ONLY: u8 = 3;
#[cfg(all(feature = "probe", test))]
const ALL_TEST_ONLY: u8 = 4;
"#;
        let contexts = RustSourceContexts::analyse(source);

        assert_eq!(
            contexts.context_for_line(line_of(source, "TEST_ONLY")),
            RustCodeContext::Test
        );
        assert_eq!(
            contexts.context_for_line(line_of(source, "PRODUCTION")),
            RustCodeContext::Production
        );
        assert_eq!(
            contexts.context_for_line(line_of(source, "FEATURE_ONLY")),
            RustCodeContext::Production
        );
        assert_eq!(
            contexts.context_for_line(line_of(source, "ALL_TEST_ONLY")),
            RustCodeContext::Test
        );
    }

    #[test]
    fn does_not_treat_cfg_not_test_as_test_only() {
        let source = r#"
#[cfg(not(test))]
fn production_only() {
    let label = "loc_1234";
}
"#;
        let contexts = RustSourceContexts::analyse(source);

        assert_eq!(
            contexts.context_for_line(line_of(source, "label")),
            RustCodeContext::Production
        );
    }
}
