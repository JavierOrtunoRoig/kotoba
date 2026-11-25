---
status: accepted
---

# Word maturity states and lesson dominance

Kotoba used to call a word "mastered" once its review interval reached seven days, and a lesson "mastered" when a configurable share of its words crossed that line. We replace that binary with five derived maturity states — new, learning, relearning, young, mature — using Anki's boundaries (learning below one day, young from one to twenty days, mature at twenty-one days or more), and a lesson is now dominated when it has no new words and its share of mature words reaches the threshold (80% by default for new installs). The states remain a projection of the SRS rather than a second scheduler; the only persisted addition is `max_interval_minutes`, needed to tell relearning apart from learning.

## Considered Options

- **Keep seven days as the mature boundary.** Rejected: seven days is reached after only three successful reviews, and the idea asked for Anki-like levels.
- **Add a fifth "consolidated" state at thirty days.** Deferred until there is a user need for more granularity.
- **Fold relearning into learning.** Rejected: it hides words the user already knew and failed.

## Consequences

- Lesson dominance becomes a months-long goal: every word must have been reviewed at least once and most must survive roughly five reviews.
- Lesson stats respect `include_patterns`; otherwise pattern items, never shown by default, would block dominance forever.
- The lesson badge and both lesson achievements share one predicate. Achievements already unlocked stay unlocked.
- Existing settings keep their stored threshold; only new installs default to 80%.
- Migration backfills `max_interval_minutes` from the current interval, so a word that lapsed before this change appears as learning until its next lapse.
- Supersedes the lesson-mastery definition in D-027; the UI stops calling it "maestría" and uses the glossary terms.
