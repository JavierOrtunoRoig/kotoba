# kotoba

Desktop app for studying Japanese with spaced repetition that appears on its own, without requiring long sessions. This glossary fixes the vocabulary of review, maturity and progress.

## Review

**Review**:
The user's answer to a word that is due for practice.
_Avoid_: test, quiz, card

**Review interval**:
Time from one review until that word becomes available again.
_Avoid_: due date, deadline

**New word**:
A word that has never been reviewed yet.
_Avoid_: pending word, unseen word

## Word maturity

**Maturity**:
State of a word according to its current interval: new, learning, relearning, young or mature.
_Avoid_: mastery, level

**Learning word**:
A word that has been reviewed but has not yet reached a one-day interval, and had never reached it before.
_Avoid_: in-progress word, weak word

**Relearning word**:
A word that had already reached at least a one-day interval and, after a lapse, is below one day again.
_Avoid_: failed word, forgotten word

**Young word**:
A word whose current interval is between one and twenty days.
_Avoid_: intermediate word

**Mature word**:
A word whose current interval is twenty-one days or more.
_Avoid_: mastered word, consolidated word

## Lesson dominance

**Lesson**:
A grouping of words imported from the deck, such as a chapter or a level.
_Avoid_: deck, category

**Pattern**:
An item that represents a grammatical structure instead of a single word. It only enters reviews if the patterns setting is enabled.
_Avoid_: rule, grammar

**Dominance**:
Percentage of mature words over the total number of lesson words that enter reviews.
_Avoid_: mastery, progress

**Dominance threshold**:
Minimum percentage of mature words a lesson must reach to be considered dominated.
_Avoid_: mastery threshold

**Dominated lesson**:
A lesson with no new words whose dominance reaches the configured threshold.
_Avoid_: mastered lesson, completed lesson
