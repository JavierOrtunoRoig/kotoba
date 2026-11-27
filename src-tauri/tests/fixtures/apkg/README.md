# Anki package fixtures

These packages contain original sample notes exported by Anki 26.09.4 with media enabled,
legacy compatibility disabled, and scheduling excluded. Each package is exercised by the
Rust import tests.

| Fixture | Note type and sample |
|---|---|
| `basic.apkg` | Stock Basic with Front and Back for cat |
| `reversed.apkg` | Stock Basic (and reversed card) plus the Spelling field, with two cards for dog |
| `cloze.apkg` | Stock Cloze with multiple deletions, a hint, HTML and a nested deletion in Back Extra |
| `multi-deck.apkg` | Two Basic notes in `Japanese::Lesson 1::Food` and `Japanese::Lesson 2::Places::Outdoors` |
| `audio.apkg` | Basic with Japanese, Translation and Audio fields, with a generated silent WAV |
