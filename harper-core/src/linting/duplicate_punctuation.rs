use std::collections::HashSet;

use crate::{
    Document, Punctuation, Span, TokenKind,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Flags clusters of punctuation that should be collapsed to a single mark
/// (e.g. `!!`, `?!?`, `//`, `.,`, `; :`, etc.).
#[derive(Debug, Default)]
pub struct DuplicatePunctuation;

impl DuplicatePunctuation {
    /// Punctuation kinds we’re willing to condense.
    fn is_candidate(kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Punctuation(
                Punctuation::Comma
                    | Punctuation::Semicolon
                    | Punctuation::Colon
                    | Punctuation::ForwardSlash
                    | Punctuation::Bang
                    | Punctuation::Question
                    | Punctuation::Period
                    | Punctuation::Ampersand
            )
        )
    }

    /// Map a candidate punctuation token to its canonical char.
    fn char_of(kind: &TokenKind) -> char {
        match kind {
            TokenKind::Punctuation(Punctuation::Comma) => ',',
            TokenKind::Punctuation(Punctuation::Semicolon) => ';',
            TokenKind::Punctuation(Punctuation::Colon) => ':',
            TokenKind::Punctuation(Punctuation::ForwardSlash) => '/',
            TokenKind::Punctuation(Punctuation::Bang) => '!',
            TokenKind::Punctuation(Punctuation::Question) => '?',
            TokenKind::Punctuation(Punctuation::Period) => '.',
            TokenKind::Punctuation(Punctuation::Ampersand) => '&',
            _ => unreachable!("`char_of` called on non-candidate punctuation"),
        }
    }
}

impl Linter for DuplicatePunctuation {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let toks = document.get_tokens();
        let mut lints = Vec::new();
        let mut i = 0;

        while i < toks.len() {
            // Start of a potential cluster
            if !Self::is_candidate(&toks[i].kind) {
                i += 1;
                continue;
            }

            let start = i;
            let mut end = i;
            let mut uniq = HashSet::<char>::new();

            // Consume the cluster (allowing spaces/newlines in between)
            while i < toks.len() {
                match &toks[i].kind {
                    k if Self::is_candidate(k) => {
                        uniq.insert(Self::char_of(k));
                        end = i;
                        i += 1;
                    }
                    TokenKind::Space(_) | TokenKind::Newline(_) => {
                        end = i;
                        i += 1;
                    }
                    _ => break,
                }
            }

            // How many candidate tokens were there?
            let count = (start..=end)
                .filter(|idx| Self::is_candidate(&toks[*idx].kind))
                .count();

            if count >= 2 {
                let span = Span::new(toks[start].span.start, toks[end].span.end);

                // One suggestion per distinct glyph in the cluster
                let suggestions = uniq
                    .into_iter()
                    .map(|c| Suggestion::ReplaceWith(vec![c]))
                    .collect::<Vec<_>>();

                lints.push(Lint {
                    span,
                    lint_kind: LintKind::Formatting,
                    suggestions,
                    message: "Condense this punctuation cluster to a single mark.".into(),
                    priority: 63,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Detects consecutive or mixed punctuation marks that should be reduced \
         to a single comma, semicolon, colon, slash, question mark, \
         exclamation mark, period, or ampersand."
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

    #[test]
    fn flags_double_ampersand() {
        assert_lint_count("This && that.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_period_comma_cluster() {
        assert_lint_count("Oops., excuse me.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_colon_comma_cluster() {
        assert_lint_count("Delay:, we must wait.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_semicolon_colon_cluster() {
        assert_lint_count("Choices;: A or B.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_comma_period_cluster() {
        assert_lint_count("Hold on,. actually…", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_question_period_cluster() {
        assert_lint_count("Really?.", DuplicatePunctuation, 1);
    }

    #[test]
    fn flags_bang_period_cluster() {
        assert_lint_count("Stop!.", DuplicatePunctuation, 1);
    }
}
