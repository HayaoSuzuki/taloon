//! Whole-input legacy grammar; names stay owned by the parent interpreter.
use crate::policy::LicenseExpression;

const MAX_BYTES: usize = 16 * 1024;
const MAX_LEAVES: usize = 64;
const MAX_DEPTH: usize = 32;

pub(super) fn parse(input: &str) -> Option<LicenseExpression> {
    if input.len() > MAX_BYTES {
        return None;
    }
    let mut parser = Parser {
        rest: input,
        leaves: 0,
    };
    let normalized = parser.disjunction(0)?;
    parser.rest.trim().is_empty().then_some(())?;
    LicenseExpression::parse(&normalized).ok()
}

struct Parser<'a> {
    rest: &'a str,
    leaves: usize,
}
impl Parser<'_> {
    fn disjunction(&mut self, depth: usize) -> Option<String> {
        let mut value = self.conjunction(depth)?;
        while self.operator("OR") {
            value.push_str(" OR ");
            value.push_str(&self.conjunction(depth)?);
        }
        Some(value)
    }

    fn conjunction(&mut self, depth: usize) -> Option<String> {
        let mut value = self.primary(depth)?;
        while self.operator("AND") {
            value.push_str(" AND ");
            value.push_str(&self.primary(depth)?);
        }
        Some(value)
    }

    fn primary(&mut self, depth: usize) -> Option<String> {
        self.rest = self.rest.trim_start();
        // Names containing parentheses are complete leaves, not structural groups.
        let mut unit = if let Some((length, id)) = name_prefix(self.rest, super::names()) {
            self.rest = &self.rest[length..];
            id.to_owned()
        } else if let Some(after) = self.rest.strip_prefix('(') {
            if depth == MAX_DEPTH {
                return None;
            }
            self.rest = after;
            let inner = self.disjunction(depth + 1)?;
            self.rest = self.rest.trim_start().strip_prefix(')')?;
            return Some(format!("({inner})"));
        } else {
            self.word()?.to_owned()
        };
        if self.operator("WITH") {
            unit.push_str(" WITH ");
            unit.push_str(self.word()?);
        }
        // This also rejects aliases for exceptions and non-Python references.
        LicenseExpression::parse(&unit).ok()?;
        self.leaves += 1;
        (self.leaves <= MAX_LEAVES).then_some(unit)
    }

    fn operator(&mut self, operator: &str) -> bool {
        self.rest = self.rest.trim_start();
        if let Some(after) = self.rest.strip_prefix(operator)
            && boundary(after)
        {
            self.rest = after;
            return true;
        }
        false
    }

    fn word(&mut self) -> Option<&str> {
        self.rest = self.rest.trim_start();
        let end = self
            .rest
            .find(|c: char| c.is_whitespace() || c == '(' || c == ')')
            .unwrap_or(self.rest.len());
        let (word, after) = self.rest.split_at(end);
        self.rest = after;
        (!word.is_empty()).then_some(word)
    }
}

fn boundary(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with(|c: char| c.is_whitespace() || c == '(' || c == ')')
}

fn name_prefix<'a>(
    input: &str,
    candidates: impl Iterator<Item = (&'a str, &'a str)>,
) -> Option<(usize, &'a str)> {
    super::longest_unique_name(candidates.filter_map(|(name, id)| {
        (input
            .get(..name.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(name))
            && boundary(&input[name.len()..]))
        .then_some((name.len(), id))
    }))
}

#[cfg(test)]
mod tests {
    use super::name_prefix;
    use rstest::rstest;

    #[rstest]
    #[case::unique(&[("Example License", "MIT")], Some((15, "MIT")))]
    #[case::same_identity(&[("Example License", "MIT"), ("example license", "MIT")], Some((15, "MIT")))]
    #[case::collision(&[("Example License", "MIT"), ("example license", "ISC")], None)]
    #[case::longest(&[("Example", "ISC"), ("Example License", "MIT")], Some((15, "MIT")))]
    #[case::short_conflict_before(&[("Example", "ISC"), ("Example", "MIT"), ("Example License", "BSD-2-Clause")], Some((15, "BSD-2-Clause")))]
    #[case::short_conflict_after(&[("Example License", "BSD-2-Clause"), ("Example", "ISC"), ("Example", "MIT")], Some((15, "BSD-2-Clause")))]
    #[case::long_conflict(&[("Example", "ISC"), ("Example License", "MIT"), ("example license", "BSD-2-Clause")], None)]
    fn resolves_longest_unique_complete_name(
        #[case] names: &[(&str, &str)],
        #[case] expected: Option<(usize, &str)>,
    ) {
        // Arrange
        let input = "Example License OR Apache 2.0";
        // Act
        let actual = name_prefix(input, names.iter().copied());
        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn prefix_resolution_visits_each_candidate_once() {
        // Arrange
        let names = [
            ("Example", "ISC"),
            ("Example License", "MIT"),
            ("Other", "ISC"),
        ];
        let visits = std::cell::Cell::new(0);
        let candidates = names.into_iter().inspect(|_| visits.set(visits.get() + 1));
        // Act
        let actual = name_prefix("Example License OR Apache 2.0", candidates);
        // Assert
        assert_eq!(actual, Some((15, "MIT")));
        assert_eq!(visits.get(), names.len());
    }
}
