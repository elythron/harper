use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, Punctuation, TokenKind};

#[derive(Debug, Default)]
pub struct DuplicatePunctuation;

impl Linter for DuplicatePunctuation {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let toks = document.get_tokens();
        let mut lints = Vec::new();
        let mut i = 0;

        while i < toks.len() {
            if let TokenKind::Punctuation(p0) = toks[i].kind {
                if is_interest(p0) {
                    let start = i;
                    let mut end = i;
                    let mut kinds = vec![p0];
                    let mut j = i + 1;
                    while j < toks.len() {
                        match toks[j].kind {
                            TokenKind::Space(_) | TokenKind::Newline(_) => {
                                j += 1;
                                continue;
                            }
                            TokenKind::Punctuation(p) if in_same_group(p0, p) => {
                                kinds.push(p);
                                end = j;
                                j += 1;
                                continue;
                            }
                            _ => break,
                        }
                    }
                    if kinds.len() > 1 {
                        let span = toks[start].span.expanded_to_include(toks[end].span.end - 1);
                        lints.push(Lint {
                            span,
                            lint_kind: LintKind::Punctuation,
                            suggestions: build_suggestions(&kinds),
                            message: "Remove duplicate or mixed punctuation.".to_owned(),
                            priority: 63,
                        });
                    }
                    i = end + 1;
                    continue;
                }
            }
            i += 1;
        }
        lints
    }

    fn description(&self) -> &str {
        "Flags duplicate or mixed punctuation sequences."
    }
}

fn is_interest(p: Punctuation) -> bool {
    matches!(
        p,
        Punctuation::Comma
            | Punctuation::Semicolon
            | Punctuation::Colon
            | Punctuation::Bang
            | Punctuation::Question
            | Punctuation::ForwardSlash
            | Punctuation::Ampersand
    )
}

fn in_same_group(first: Punctuation, next: Punctuation) -> bool {
    match first {
        Punctuation::Bang | Punctuation::Question => {
            matches!(next, Punctuation::Bang | Punctuation::Question)
        }
        _ => first == next,
    }
}

fn build_suggestions(seq: &[Punctuation]) -> Vec<Suggestion> {
    use Punctuation::*;
    let mut opts = match seq[0] {
        Bang | Question => vec![Bang, Question],
        other => vec![other],
    };
    opts.dedup();
    opts.into_iter()
        .map(|p| Suggestion::ReplaceWith(vec![punct_char(p)]))
        .collect()
}

fn punct_char(p: Punctuation) -> char {
    use Punctuation::*;
    match p {
        Comma => ',',
        Semicolon => ';',
        Colon => ':',
        Bang => '!',
        Question => '?',
        ForwardSlash => '/',
        Ampersand => '&',
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_lint_count;

    use super::DuplicatePunctuation;

    #[test]
    fn flags_double_comma() {
        assert_lint_count("Wait,, what happened?", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_double_semicolon() {
        assert_lint_count("He hesitated;; then spoke.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_double_colon() {
        assert_lint_count("Choices:: A or B.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_double_bang() {
        assert_lint_count("Stop!!", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_double_question() {
        assert_lint_count("Really??", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_mixed_qbang_pair() {
        assert_lint_count("What?!", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_triple_bang() {
        assert_lint_count("No!!! Absolutely not.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_q_bang_bang() {
        assert_lint_count("Really?!!", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_double_slash() {
        assert_lint_count("This // is a typo.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_triple_slash() {
        assert_lint_count("Path error: ///tmp.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_triple_question() {
        assert_lint_count("Why???", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_quadruple_bang() {
        assert_lint_count("Stop!!!!", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_question_bang_question() {
        assert_lint_count("You did what?!?", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_bang_question_bang() {
        assert_lint_count("No way!?!", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_question_bang_bang_question() {
        assert_lint_count("Seriously?!!?", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_slash_run_inside_sentence() {
        assert_lint_count("Comment // still visible.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_with_intervening_whitespace() {
        assert_lint_count("Why?! ?", DuplicatePunctuation, 1);
    }
}
