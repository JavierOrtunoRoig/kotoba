use crate::db::{self, InsertOutcome, NewItem};
use crate::media::{self, MediaEntry};
use crate::models::{
    FieldMapping, ImportInspection, ImportMapping, ImportSummary, NoteTypeInfo,
};
use crate::text;
use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

const MAX_WARNINGS: usize = 20;

/// Progress callback: `(phase, current, total)`.
pub type ProgressFn<'a> = &'a mut dyn FnMut(&str, i64, i64);

pub fn import_file(
    path: &Path,
    conn: &mut Connection,
    media_dir: &Path,
    progress: ProgressFn<'_>,
    mapping: Option<&ImportMapping>,
) -> Result<ImportSummary> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();
    match extension.as_str() {
        "apkg" => import_apkg(path, conn, media_dir, progress, mapping),
        "txt" | "tsv" | "csv" => import_txt(
            path,
            conn,
            media_dir,
            progress,
            mapping.and_then(|m| m.txt.as_ref()),
            mapping.and_then(|m| m.media_folder.as_deref()),
        ),
        other => Err(anyhow!(
            "Formato no soportado: {other}. Usa un .apkg o un .txt exportado de Anki."
        )),
    }
}

/// Inspects a deck without importing it to offer field mapping.
pub fn inspect_file(path: &Path) -> Result<ImportInspection> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();
    match extension.as_str() {
        "apkg" => inspect_apkg(path),
        "txt" | "tsv" | "csv" => inspect_txt(path),
        other => Err(anyhow!(
            "Formato no soportado: {other}. Usa un .apkg o un .txt exportado de Anki."
        )),
    }
}

struct NoteRow {
    id: i64,
    mid: i64,
    flds: String,
    tags: String,
}

pub fn import_apkg(
    path: &Path,
    conn: &mut Connection,
    media_dir: &Path,
    progress: ProgressFn<'_>,
    mapping: Option<&ImportMapping>,
) -> Result<ImportSummary> {
    let file = File::open(path)
        .with_context(|| format!("No se pudo abrir {}", path.display()))?;
    let mut archive = ZipArchive::new(file).context("El archivo no es un .apkg válido")?;

    let db_bytes = read_collection(&mut archive)?;
    let tmp = tempfile::tempdir()?;
    let tmp_db = tmp.path().join("collection.sqlite");
    std::fs::write(&tmp_db, &db_bytes)?;

    let src = open_source(&tmp_db)?;
    let fields_by_mid = read_fields(&src).unwrap_or_default();
    let decks_by_note = read_note_decks(&src).unwrap_or_default();
    let notes = read_notes(&src)?;
    let media_entries = read_media_entries(&mut archive)?;

    let media_index: HashMap<String, usize> = media_entries
        .iter()
        .enumerate()
        .map(|(index, entry)| (entry.name.clone(), index))
        .collect();

    let mut summary = ImportSummary {
        notes: notes.len() as i64,
        deck_name: apkg_deck_name(&notes, &decks_by_note).to_string(),
        ..Default::default()
    };

    let total_notes = notes.len() as i64;
    progress("notes", 0, total_notes);

    let tx = conn.transaction()?;
    let mut audio_cache: HashMap<String, Option<String>> = HashMap::new();

    for (index, note) in notes.iter().enumerate() {
        if index % 25 == 0 {
            progress("notes", index as i64, total_notes);
        }
        let values: Vec<&str> = note.flds.split('\u{1f}').collect();
        let field_names = fields_by_mid.get(&note.mid).cloned().unwrap_or_default();
        let map = mapping
            .and_then(|mapping| {
                mapping
                    .note_types
                    .iter()
                    .find(|note_type| note_type.mid == note.mid)
            })
            .map(|note_type| note_type.mapping.clone())
            .unwrap_or_else(|| suggest_apkg_mapping(&field_names, values.len()));

        let reading_raw = mapped_value(&values, map.reading).unwrap_or_default();
        let kanji_raw = mapped_value(&values, map.kanji).unwrap_or_default();
        let meaning_raw = mapped_value(&values, map.meaning).unwrap_or_default();
        let gloss_raw = mapped_value(&values, map.gloss).unwrap_or_default();
        let audio_raw = mapped_value(&values, map.audio);
        let uid_raw = mapped_value(&values, map.uid).unwrap_or_default();

        let reading = text::clean_display(&reveal_cloze(reading_raw));
        let kanji_text = text::clean_display(&reveal_cloze(kanji_raw));
        let meaning = text::clean_display(&reveal_cloze(meaning_raw));

        let Some(meaning) = non_empty(meaning) else {
            push_warning(
                &mut summary,
                format!("Nota {} sin significado, se omite.", note.id),
            );
            continue;
        };
        let reading = if reading.is_empty() {
            kanji_text.clone()
        } else {
            reading
        };
        if reading.is_empty() {
            push_warning(
                &mut summary,
                format!("Nota {} sin lectura, se omite.", note.id),
            );
            continue;
        }

        let gloss = non_empty(text::clean_display(&reveal_cloze(gloss_raw)));
        let gloss = if gloss.as_deref() == Some(meaning.as_str()) {
            None
        } else {
            gloss
        };
        let kanji = non_empty(kanji_text);

        let deck_name = decks_by_note.get(&note.id).cloned().unwrap_or_default();
        let (mut lesson, mut category) = if deck_name.is_empty() {
            parse_uid(uid_raw)
        } else {
            parse_deck(&deck_name)
        };
        if let Some(value) = mapped_value(&values, map.lesson) {
            let value = text::clean_display(value);
            if !value.is_empty() {
                lesson = value;
            }
        }
        if let Some(value) = mapped_value(&values, map.category) {
            let value = text::clean_display(value);
            if !value.is_empty() {
                category = value;
            }
        }

        let uid = non_empty(uid_raw.to_string())
            .unwrap_or_else(|| format!("{deck_name}::{}", note.id));

        let is_pattern = reading.contains('〜')
            || reading.contains('+')
            || meaning.contains("...")
            || meaning.contains('…');
        let irregular = reading.contains('*');

        let audio_file = match audio_raw
            .and_then(sound_name)
            .and_then(|name| media_index.get(&name).map(|index| (name, *index)))
        {
            Some((name, index)) => {
                let entry = &media_entries[index];
                match ensure_audio(&mut archive, index, entry, media_dir, &mut audio_cache) {
                    Ok(Some((filename, is_new))) => {
                        if is_new {
                            summary.audio_extracted += 1;
                        } else {
                            summary.audio_reused += 1;
                        }
                        Some(filename)
                    }
                    Ok(None) => None,
                    Err(error) => {
                        push_warning(
                            &mut summary,
                            format!("No se pudo extraer el audio {name}: {error}"),
                        );
                        None
                    }
                }
            }
            None => None,
        };

        let mapped_tags = mapped_value(&values, map.tags).unwrap_or("").trim();
        let tags = if mapped_tags.is_empty() {
            note.tags.trim().to_string()
        } else {
            mapped_tags.to_string()
        };
        let dedup_key = format!(
            "{}|{}",
            text::normalize_key(&reading),
            text::normalize_key(&meaning)
        );
        let meaning_plain = text::normalize_key(&meaning);

        let new_item = NewItem {
            source_uid: uid.as_str(),
            kanji: kanji.as_deref(),
            reading: reading.as_str(),
            meaning: meaning.as_str(),
            meaning_plain: meaning_plain.as_str(),
            kanji_gloss: gloss.as_deref(),
            audio_file: audio_file.as_deref(),
            lesson: lesson.as_str(),
            category: category.as_str(),
            tags: tags.as_str(),
            is_pattern,
            irregular,
            dedup_key: dedup_key.as_str(),
        };

        match db::insert_item(&tx, &new_item)? {
            InsertOutcome::Inserted => summary.items_inserted += 1,
            InsertOutcome::Merged => summary.items_merged += 1,
            InsertOutcome::Skipped => {}
        }
        if is_pattern {
            summary.patterns += 1;
        }
        summary.sources += 1;
    }

    tx.commit()?;
    progress("done", total_notes, total_notes);
    Ok(summary)
}

fn read_collection(archive: &mut ZipArchive<File>) -> Result<Vec<u8>> {
    for name in ["collection.anki21b", "collection.anki21", "collection.anki2"] {
        let Ok(mut entry) = archive.by_name(name) else {
            continue;
        };
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        drop(entry);
        if media::is_zstd(&bytes) {
            bytes = zstd::stream::decode_all(&bytes[..])?;
        }
        if bytes.starts_with(b"SQLite format 3\0") {
            return Ok(bytes);
        }
    }
    Err(anyhow!(
        "El .apkg no contiene una base de datos compatible. Expórtalo de nuevo con una versión reciente de Anki."
    ))
}

fn open_source(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    // Anki uses the `unicase` collation on `fields.name` and `notetypes.name`.
    conn.create_collation("unicase", |left, right| {
        left.to_lowercase().cmp(&right.to_lowercase())
    })?;
    Ok(conn)
}

fn read_fields(src: &Connection) -> Result<HashMap<i64, Vec<String>>> {
    let mut stmt = src.prepare("SELECT ntid, name FROM fields ORDER BY ntid, ord")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut map: HashMap<i64, Vec<String>> = HashMap::new();
    for row in rows {
        let (mid, name) = row?;
        map.entry(mid).or_default().push(name);
    }
    Ok(map)
}

fn read_note_decks(src: &Connection) -> Result<HashMap<i64, String>> {
    let mut stmt = src.prepare(
        "SELECT c.nid, d.name FROM cards c JOIN decks d ON d.id = c.did ORDER BY c.id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (note_id, deck) = row?;
        map.entry(note_id).or_insert(deck);
    }
    Ok(map)
}

fn read_notes(src: &Connection) -> Result<Vec<NoteRow>> {
    let mut stmt = src.prepare("SELECT id, mid, flds, tags FROM notes ORDER BY id ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(NoteRow {
            id: row.get(0)?,
            mid: row.get(1)?,
            flds: row.get(2)?,
            tags: row.get(3)?,
        })
    })?;
    let mut notes = Vec::new();
    for row in rows {
        notes.push(row?);
    }
    Ok(notes)
}

fn apkg_deck_name<'a>(notes: &[NoteRow], decks_by_note: &'a HashMap<i64, String>) -> &'a str {
    notes
        .iter()
        .find_map(|note| decks_by_note.get(&note.id))
        .and_then(|name| name.split('\u{1f}').next())
        .unwrap_or("Anki")
}

fn read_media_entries(archive: &mut ZipArchive<File>) -> Result<Vec<MediaEntry>> {
    let Ok(mut entry) = archive.by_name("media") else {
        return Ok(Vec::new());
    };
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;
    drop(entry);
    media::parse_media_map(&bytes)
}

fn ensure_audio(
    archive: &mut ZipArchive<File>,
    index: usize,
    entry: &MediaEntry,
    media_dir: &Path,
    cache: &mut HashMap<String, Option<String>>,
) -> Result<Option<(String, bool)>> {
    if let Some(cached) = cache.get(&entry.name) {
        return Ok(cached.clone().map(|filename| (filename, false)));
    }

    let key = index.to_string();
    let mut file = archive
        .by_name(&key)
        .with_context(|| format!("Falta el archivo multimedia {key}"))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    drop(file);

    if media::is_zstd(&bytes) {
        bytes = zstd::stream::decode_all(&bytes[..])?;
    }

    let stored = media::store_bytes(&bytes, &entry.name, entry.sha1.as_deref(), media_dir);
    match stored {
        Ok(stored) => {
            cache.insert(
                entry.name.clone(),
                stored.as_ref().map(|(filename, _)| filename.clone()),
            );
            Ok(stored)
        }
        Err(error) => {
            // The hash does not match or the copy failed: we record it in the cache.
            cache.insert(entry.name.clone(), None);
            Err(error)
        }
    }
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn mapped_value<'a>(values: &[&'a str], index: Option<usize>) -> Option<&'a str> {
    index.and_then(|index| values.get(index).copied())
}

/// Suggests indices from the field names and the deck width.
fn suggest_mapping(fields: &[String], width: usize) -> FieldMapping {
    suggest_field_mapping(fields, width, false)
}

fn suggest_apkg_mapping(fields: &[String], width: usize) -> FieldMapping {
    suggest_field_mapping(fields, width, true)
}

fn suggest_field_mapping(fields: &[String], width: usize, anki: bool) -> FieldMapping {
    let lower: Vec<String> = fields
        .iter()
        .map(|field| field.trim().to_lowercase())
        .collect();
    let lookup = |names: &[&str]| {
        lower
            .iter()
            .position(|field| names.iter().any(|name| field == name))
    };
    let anki_field = |names: &[&str]| if anki { lookup(names) } else { None };

    let reading = lookup(&["kana", "reading", "lectura"])
        .or_else(|| anki_field(&["front", "japanese", "text"]))
        .or((width > 0).then_some(0));
    let meaning = lookup(&["español", "espanol", "meaning", "significado", "definition"])
        .or_else(|| anki_field(&["back", "translation"]))
        .or_else(|| anki_field(&["text"]).and_then(|_| lookup(&["back extra", "extra"])))
        .or(match width {
            2 if anki => Some(if reading == Some(1) { 0 } else { 1 }),
            3 => Some(2),
            4.. => Some(3),
            _ => None,
        });
    let audio = lookup(&["sonido", "audio", "sound"]).or((width > 5).then_some(5));
    let kanji = lookup(&["kanjis", "kanji", "word"])
        .or_else(|| anki_field(&["spelling"]))
        .or_else(|| {
            let already_mapped = [meaning, reading, audio].contains(&Some(1));
            (width > 1 && (!anki || !already_mapped)).then_some(1)
        });
    let gloss = lookup(&["significado", "kanji meaning", "gloss"])
        .filter(|index| Some(*index) != meaning)
        .or(if width > 4 && Some(4) != meaning {
            Some(4)
        } else {
            None
        });
    let uid = lookup(&["uid", "guid"]).or(if width > 7 { Some(7) } else { None });

    FieldMapping {
        reading,
        kanji,
        meaning,
        gloss,
        audio,
        uid,
        tags: None,
        lesson: None,
        category: None,
    }
}

fn preview(value: &str) -> String {
    let clean = text::clean_display(value);
    let mut out: String = clean.chars().take(80).collect();
    if clean.chars().count() > 80 {
        out.push('…');
    }
    out
}

/// Reveals Anki's text deletions, including nested deletions, and discards hints.
fn reveal_cloze(input: &str) -> std::borrow::Cow<'_, str> {
    let Some(first) = input.find("{{c") else {
        return input.into();
    };
    let mut output = String::with_capacity(input.len());
    output.push_str(&input[..first]);
    let mut rest = &input[first..];
    let mut hints = Vec::new();
    let mut unclosed = (0, 0);

    while !rest.is_empty() {
        if let Some(after_c) = rest.strip_prefix("{{c") {
            let length = after_c
                .find(|c: char| !c.is_ascii_digit() && c != ',')
                .unwrap_or(after_c.len());
            let ordinals = &after_c[..length];
            if ordinals.bytes().any(|b| b.is_ascii_digit()) {
                if let Some(after_open) = after_c[length..].strip_prefix("::") {
                    if hints.is_empty() {
                        unclosed = (input.len() - rest.len(), output.len());
                    }
                    hints.push(hints.last().copied().unwrap_or(false));
                    rest = after_open;
                    continue;
                }
            }
        }
        if let Some(in_hint) = hints.last_mut() {
            if let Some(after_close) = rest.strip_prefix("}}") {
                hints.pop();
                rest = after_close;
                continue;
            }
            if let Some(after_separator) = rest.strip_prefix("::") {
                *in_hint = true;
                rest = after_separator;
                continue;
            }
        }
        let character = rest.chars().next().expect("non-empty cloze input");
        if !hints.last().copied().unwrap_or(false) {
            output.push(character);
        }
        rest = &rest[character.len_utf8()..];
    }
    if !hints.is_empty() {
        output.truncate(unclosed.1);
        output.push_str(&input[unclosed.0..]);
    }
    output.into()
}

fn read_note_type_names(src: &Connection) -> Result<HashMap<i64, String>> {
    let mut stmt = src.prepare("SELECT id, name FROM notetypes")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (mid, name) = row?;
        map.insert(mid, name);
    }
    Ok(map)
}

fn inspect_txt(path: &Path) -> Result<ImportInspection> {
    let table = parse_txt_table(path)?;
    let sample = table
        .lines
        .iter()
        .take(3)
        .map(|line| {
            let values: Vec<&str> = line.split(table.separator).collect();
            table
                .columns
                .iter()
                .enumerate()
                .map(|(index, _)| preview(values.get(index).copied().unwrap_or_default()))
                .collect()
        })
        .collect();

    Ok(ImportInspection {
        kind: "txt".to_string(),
        deck_name: txt_deck_name(path),
        total_notes: table.lines.len() as i64,
        suggested: Some(suggest_mapping(&table.columns, table.columns.len())),
        columns: table.columns,
        sample,
        note_types: Vec::new(),
        media_folder: sibling_media_folder(path),
    })
}

fn inspect_apkg(path: &Path) -> Result<ImportInspection> {
    let file = File::open(path)
        .with_context(|| format!("No se pudo abrir {}", path.display()))?;
    let mut archive = ZipArchive::new(file).context("El archivo no es un .apkg válido")?;
    let db_bytes = read_collection(&mut archive)?;
    let tmp = tempfile::tempdir()?;
    let tmp_db = tmp.path().join("collection.sqlite");
    std::fs::write(&tmp_db, &db_bytes)?;
    let src = open_source(&tmp_db)?;

    let fields_by_mid = read_fields(&src).unwrap_or_default();
    let names_by_mid = read_note_type_names(&src).unwrap_or_default();
    let decks_by_note = read_note_decks(&src).unwrap_or_default();
    let notes = read_notes(&src)?;

    let mut order: Vec<i64> = Vec::new();
    let mut counts: HashMap<i64, i64> = HashMap::new();
    let mut samples: HashMap<i64, Vec<Vec<String>>> = HashMap::new();

    for note in &notes {
        let values: Vec<&str> = note.flds.split('\u{1f}').collect();
        if !counts.contains_key(&note.mid) {
            order.push(note.mid);
        }
        *counts.entry(note.mid).or_insert(0) += 1;
        let sample = samples.entry(note.mid).or_default();
        if sample.len() < 2 {
            sample.push(
                values
                    .iter()
                    .map(|value| preview(&reveal_cloze(value)))
                    .collect(),
            );
        }
    }

    let mut note_types = Vec::with_capacity(order.len());
    for mid in order {
        let sample = samples.remove(&mid).unwrap_or_default();
        let mut fields = fields_by_mid.get(&mid).cloned().unwrap_or_default();
        let width = sample.first().map(|row| row.len()).unwrap_or(0);
        while fields.len() < width {
            fields.push(format!("Campo {}", fields.len() + 1));
        }
        note_types.push(NoteTypeInfo {
            mid,
            name: names_by_mid.get(&mid).cloned().unwrap_or_default(),
            fields: fields.clone(),
            count: counts.get(&mid).copied().unwrap_or(0),
            sample,
            suggested: suggest_apkg_mapping(&fields, fields.len()),
        });
    }

    Ok(ImportInspection {
        kind: "apkg".to_string(),
        deck_name: apkg_deck_name(&notes, &decks_by_note).to_string(),
        total_notes: notes.len() as i64,
        columns: Vec::new(),
        sample: Vec::new(),
        suggested: None,
        note_types,
        media_folder: None,
    })
}

fn sound_name(value: &str) -> Option<String> {
    let start = value.find("[sound:")? + "[sound:".len();
    let rest = &value[start..];
    let end = rest.find(']')?;
    let name = rest[..end].trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn parse_deck(name: &str) -> (String, String) {
    let parts: Vec<&str> = name.split('\u{1f}').map(str::trim).collect();
    match parts.len() {
        0 => (String::new(), String::new()),
        1 => (String::new(), parts[0].to_string()),
        2 => (parts[1].to_string(), parts[1].to_string()),
        _ => (parts[1].to_string(), parts[2..].join(" / ")),
    }
}

fn parse_uid(uid: &str) -> (String, String) {
    let parts: Vec<&str> = uid.split("::").collect();
    if parts.len() >= 3 {
        let mut category = parts[2..].join(" / ");
        if let Some((base, suffix)) = category.rsplit_once('_') {
            if !base.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
                category = base.to_string();
            }
        }
        (parts[1].to_string(), category)
    } else {
        (String::new(), uid.to_string())
    }
}

fn push_warning(summary: &mut ImportSummary, message: String) {
    if summary.warnings.len() < MAX_WARNINGS {
        summary.warnings.push(message);
    }
}

struct TxtTable {
    separator: char,
    tags_column: Option<usize>,
    columns: Vec<String>,
    lines: Vec<String>,
}

fn parse_txt_table(path: &Path) -> Result<TxtTable> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("No se pudo leer {}", path.display()))?;

    let mut separator = '\t';
    let mut tags_column: Option<usize> = None;
    let mut columns: Vec<String> = Vec::new();
    let mut lines = Vec::new();

    for line in content.lines() {
        if let Some(header) = line.strip_prefix('#') {
            if let Some(value) = header.strip_prefix("separator:") {
                separator = match value.trim() {
                    "comma" => ',',
                    "semicolon" => ';',
                    "space" => ' ',
                    _ => '\t',
                };
            } else if let Some(value) = header.strip_prefix("tags column:") {
                tags_column = value
                    .trim()
                    .parse::<usize>()
                    .ok()
                    .map(|n| n.saturating_sub(1));
            } else if let Some(value) = header.strip_prefix("columns:") {
                let mut names: Vec<String> = value
                    .split(separator)
                    .map(|name| name.trim().to_string())
                    .collect();
                if names.len() <= 1 && value.contains('\t') {
                    names = value.split('\t').map(|name| name.trim().to_string()).collect();
                }
                columns = names;
            }
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        lines.push(line.to_string());
    }

    let width = lines
        .iter()
        .take(50)
        .map(|line| line.split(separator).count())
        .max()
        .unwrap_or(columns.len());
    while columns.len() < width {
        columns.push(format!("Columna {}", columns.len() + 1));
    }

    Ok(TxtTable {
        separator,
        tags_column,
        columns,
        lines,
    })
}

fn txt_deck_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Anki")
        .to_string()
}

/// Audio folder next to the `.txt`, with the same name as the file.
fn sibling_media_folder(path: &Path) -> Option<String> {
    let folder = path.parent()?.join(path.file_stem()?);
    folder
        .is_dir()
        .then(|| folder.to_string_lossy().into_owned())
}

/// Media folder chosen in the dialog, or the sibling one by default.
fn resolve_media_folder(path: &Path, chosen: Option<&str>) -> PathBuf {
    if let Some(folder) = chosen.map(str::trim).filter(|value| !value.is_empty()) {
        return PathBuf::from(folder);
    }
    path.parent()
        .map(|parent| parent.join(path.file_stem().unwrap_or_default()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Reads an audio file from the media folder and copies it under its SHA-1 name.
fn audio_from_folder(
    folder: &Path,
    name: &str,
    media_dir: &Path,
    cache: &mut HashMap<String, Option<String>>,
) -> Result<Option<(String, bool)>> {
    if let Some(cached) = cache.get(name) {
        return Ok(cached.clone().map(|filename| (filename, false)));
    }

    let safe_name = Path::new(name)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| *value == name)
        .ok_or_else(|| anyhow!("nombre de audio no válido: {name}"))?;

    let source = folder.join(safe_name);
    let bytes = std::fs::read(&source)
        .with_context(|| format!("no se encontró {}", source.display()))?;

    let stored = media::store_bytes(&bytes, name, None, media_dir);
    match stored {
        Ok(stored) => {
            cache.insert(
                name.to_string(),
                stored.as_ref().map(|(filename, _)| filename.clone()),
            );
            Ok(stored)
        }
        Err(error) => {
            cache.insert(name.to_string(), None);
            Err(error)
        }
    }
}

pub fn import_txt(
    path: &Path,
    conn: &mut Connection,
    media_dir: &Path,
    progress: ProgressFn<'_>,
    mapping: Option<&FieldMapping>,
    media_folder: Option<&str>,
) -> Result<ImportSummary> {
    let table = parse_txt_table(path)?;
    let deck_name = txt_deck_name(path);
    let map = mapping
        .cloned()
        .unwrap_or_else(|| suggest_mapping(&table.columns, table.columns.len()));

    let mut summary = ImportSummary {
        notes: table.lines.len() as i64,
        deck_name: deck_name.clone(),
        ..Default::default()
    };

    let audio_folder = map.audio.and_then(|_| {
        let folder = resolve_media_folder(path, media_folder);
        if folder.is_dir() {
            Some(folder)
        } else {
            push_warning(
                &mut summary,
                format!(
                    "No se encontró la carpeta de medios {}; los audios se omiten.",
                    folder.display()
                ),
            );
            None
        }
    });
    let mut audio_cache: HashMap<String, Option<String>> = HashMap::new();

    let total_lines = table.lines.len() as i64;
    progress("notes", 0, total_lines);

    let tx = conn.transaction()?;
    for (line_index, line) in table.lines.iter().enumerate() {
        if line_index % 25 == 0 {
            progress("notes", line_index as i64, total_lines);
        }
        let values: Vec<&str> = line.split(table.separator).collect();
        if values.is_empty() {
            continue;
        }

        let reading_raw = mapped_value(&values, map.reading).unwrap_or_default();
        let kanji_raw = mapped_value(&values, map.kanji).unwrap_or_default();
        let meaning_raw = mapped_value(&values, map.meaning).unwrap_or_default();
        let gloss_raw = mapped_value(&values, map.gloss);
        let uid_raw = mapped_value(&values, map.uid);

        let reading = text::clean_display(reading_raw);
        let kanji_text = text::clean_display(kanji_raw);
        let meaning = text::clean_display(meaning_raw);
        let Some(meaning) = non_empty(meaning) else {
            continue;
        };
        let reading = if reading.is_empty() {
            kanji_text.clone()
        } else {
            reading
        };
        if reading.is_empty() {
            continue;
        }
        let gloss = gloss_raw
            .map(text::clean_display)
            .and_then(non_empty);
        let kanji = non_empty(kanji_text);

        let audio_file = match audio_folder
            .as_deref()
            .zip(mapped_value(&values, map.audio).and_then(sound_name))
        {
            Some((folder, name)) => {
                match audio_from_folder(folder, &name, media_dir, &mut audio_cache) {
                    Ok(Some((filename, is_new))) => {
                        if is_new {
                            summary.audio_extracted += 1;
                        } else {
                            summary.audio_reused += 1;
                        }
                        Some(filename)
                    }
                    Ok(None) => None,
                    Err(error) => {
                        summary.audio_missing += 1;
                        push_warning(
                            &mut summary,
                            format!("No se pudo usar el audio {name}: {error}"),
                        );
                        None
                    }
                }
            }
            None => None,
        };

        let uid = uid_raw
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("{deck_name}::txt::{line_index}"));

        let lesson = mapped_value(&values, map.lesson)
            .map(text::clean_display)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| deck_name.clone());
        let category = mapped_value(&values, map.category)
            .map(text::clean_display)
            .filter(|value| !value.is_empty())
            .unwrap_or_default();

        let is_pattern = reading.contains('〜')
            || reading.contains('+')
            || meaning.contains("...")
            || meaning.contains('…');
        let irregular = reading.contains('*');
        let dedup_key = format!(
            "{}|{}",
            text::normalize_key(&reading),
            text::normalize_key(&meaning)
        );
        let meaning_plain = text::normalize_key(&meaning);
        let mapped_tags = mapped_value(&values, map.tags)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let tags = mapped_tags
            .map(str::to_string)
            .or_else(|| {
                table
                    .tags_column
                    .and_then(|index| values.get(index))
                    .map(|value| value.trim().to_string())
            })
            .unwrap_or_default();

        let new_item = NewItem {
            source_uid: uid.as_str(),
            kanji: kanji.as_deref(),
            reading: reading.as_str(),
            meaning: meaning.as_str(),
            meaning_plain: meaning_plain.as_str(),
            kanji_gloss: gloss.as_deref(),
            audio_file: audio_file.as_deref(),
            lesson: lesson.as_str(),
            category: category.as_str(),
            tags: tags.as_str(),
            is_pattern,
            irregular,
            dedup_key: dedup_key.as_str(),
        };

        match db::insert_item(&tx, &new_item)? {
            InsertOutcome::Inserted => summary.items_inserted += 1,
            InsertOutcome::Merged => summary.items_merged += 1,
            InsertOutcome::Skipped => {}
        }
        if is_pattern {
            summary.patterns += 1;
        }
        summary.sources += 1;
    }
    tx.commit()?;
    progress("done", total_lines, total_lines);

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn import_apkg_fixture(name: &str, expected_notes: i64) -> Vec<crate::models::Item> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/apkg")
            .join(format!("{name}.apkg"));
        let inspection = inspect_file(&path).expect("inspect fixture");
        assert_eq!(inspection.total_notes, expected_notes);
        let mapping = ImportMapping {
            note_types: inspection
                .note_types
                .into_iter()
                .map(|note_type| crate::models::NoteTypeMapping {
                    mid: note_type.mid,
                    mapping: note_type.suggested,
                })
                .collect(),
            ..Default::default()
        };
        let mut conn = Connection::open_in_memory().expect("db");
        db::migrate(&mut conn).expect("migrate");
        let media = tempfile::tempdir().expect("media directory");
        let mut noop = |_: &str, _: i64, _: i64| {};

        let first = import_file(&path, &mut conn, media.path(), &mut noop, Some(&mapping))
            .expect("import fixture");
        assert!(first.warnings.is_empty(), "{:?}", first.warnings);
        assert_eq!(first.items_inserted, expected_notes);
        let items = db::all_items(&conn).expect("imported items");
        assert_eq!(items.len() as i64, expected_notes);

        let second = import_file(&path, &mut conn, media.path(), &mut noop, None)
            .expect("reimport fixture");
        assert!(second.warnings.is_empty(), "{:?}", second.warnings);
        assert_eq!(second.sources, expected_notes);
        assert_eq!(second.items_inserted, 0);
        // A changed source UID with the same content would be counted as a merge.
        assert_eq!(second.items_merged, 0);
        assert_eq!(db::count_items(&conn).unwrap(), expected_notes);

        items
    }

    #[test]
    fn imports_cloze_as_one_item_with_revealed_text() {
        let items = import_apkg_fixture("cloze", 1);
        assert_eq!(items[0].reading, "あおい そら");
        assert_eq!(items[0].meaning, "blue sky");
        assert_eq!(items[0].kanji, None);
    }

    #[test]
    fn imports_basic_front_and_back() {
        let items = import_apkg_fixture("basic", 1);
        assert_eq!(items[0].reading, "ねこ");
        assert_eq!(items[0].meaning, "cat");
        assert_eq!(items[0].kanji, None);
    }

    #[test]
    fn imports_reversed_cards_as_one_item_with_optional_spelling() {
        let items = import_apkg_fixture("reversed", 1);
        assert_eq!(items[0].reading, "いぬ");
        assert_eq!(items[0].meaning, "dog");
        assert_eq!(items[0].kanji.as_deref(), Some("犬"));
    }

    #[test]
    fn imports_each_note_with_its_own_deck_metadata() {
        let items = import_apkg_fixture("multi-deck", 2);
        let apple = items.iter().find(|item| item.meaning == "apple").unwrap();
        assert_eq!(apple.lesson, "Lesson 1");
        assert_eq!(apple.category, "Food");
        let mountain = items.iter().find(|item| item.meaning == "mountain").unwrap();
        assert_eq!(mountain.lesson, "Lesson 2");
        assert_eq!(mountain.category, "Places / Outdoors");
    }

    #[test]
    fn imports_custom_fields_with_dedicated_audio() {
        let items = import_apkg_fixture("audio", 1);
        assert_eq!(items[0].reading, "みず");
        assert_eq!(items[0].meaning, "water");
        assert_eq!(items[0].kanji, None);
        assert!(items[0].has_audio);
    }

    #[test]
    fn parses_deck_paths() {
        assert_eq!(
            parse_deck("Genki 1\u{1f}L00\u{1f}01 Saludos"),
            ("L00".to_string(), "01 Saludos".to_string())
        );
        assert_eq!(
            parse_deck("Genki 1\u{1f}L04\u{1f}01 Vocabulario"),
            ("L04".to_string(), "01 Vocabulario".to_string())
        );
    }

    #[test]
    fn parses_uid_paths() {
        assert_eq!(
            parse_uid("Genki 1::L00::01 Saludos_0"),
            ("L00".to_string(), "01 Saludos".to_string())
        );
    }

    #[test]
    fn extracts_sound_names() {
        assert_eq!(
            sound_name("[sound:01_Saludos_001_Buenos_dias_jap.mp3]"),
            Some("01_Saludos_001_Buenos_dias_jap.mp3".to_string())
        );
        assert_eq!(sound_name("sin audio"), None);
    }

    #[test]
    fn suggests_mapping_from_names_and_positions() {
        let empty = suggest_mapping(&[], 8);
        assert_eq!(empty.reading, Some(0));
        assert_eq!(empty.kanji, Some(1));
        assert_eq!(empty.meaning, Some(3));
        assert_eq!(empty.gloss, Some(4));
        assert_eq!(empty.audio, Some(5));
        assert_eq!(empty.uid, Some(7));

        let short = suggest_mapping(&[], 3);
        assert_eq!(short.meaning, Some(2));
        assert_eq!(short.gloss, None);
        assert_eq!(short.audio, None);

        let fields = vec![
            "Kana".to_string(),
            "Kanji".to_string(),
            "Significado".to_string(),
            "Sonido".to_string(),
            "UID".to_string(),
        ];
        let named = suggest_mapping(&fields, fields.len());
        assert_eq!(named.reading, Some(0));
        assert_eq!(named.kanji, Some(1));
        assert_eq!(named.meaning, Some(2));
        assert_eq!(named.audio, Some(3));
        assert_eq!(named.uid, Some(4));
    }

    #[test]
    fn inspects_and_imports_txt_with_custom_mapping() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("custom.txt");
        std::fs::write(
            &path,
            "#separator:semicolon\n#columns:Significado;Lectura;Kanji\nhola;こんにちは;今日は\nadiós;さようなら;左様なら\n",
        )
        .expect("txt");

        let inspection = inspect_file(&path).expect("inspect");
        assert_eq!(inspection.kind, "txt");
        assert_eq!(
            inspection.columns,
            vec!["Significado", "Lectura", "Kanji"]
        );
        let suggested = inspection.suggested.expect("suggested");
        assert_eq!(suggested.meaning, Some(0));
        assert_eq!(suggested.reading, Some(1));
        assert_eq!(suggested.kanji, Some(2));
        assert_eq!(inspection.sample[0], vec!["hola", "こんにちは", "今日は"]);

        let mut conn = Connection::open_in_memory().expect("db");
        db::migrate(&mut conn).expect("migrate");
        let mut noop = |_: &str, _: i64, _: i64| {};
        let mapping = ImportMapping {
            txt: Some(suggested),
            note_types: Vec::new(),
            media_folder: None,
        };
        import_file(&path, &mut conn, dir.path(), &mut noop, Some(&mapping))
            .expect("import");

        let items = db::all_items(&conn).expect("items");
        let item = items
            .iter()
            .find(|item| item.reading == "こんにちは")
            .expect("item");
        assert_eq!(item.meaning, "hola");
        assert_eq!(item.kanji.as_deref(), Some("今日は"));
    }

    fn import_txt_with_audio(
        path: &Path,
        media_dir: &Path,
        mapping: &ImportMapping,
    ) -> (ImportSummary, Vec<crate::models::Item>) {
        let mut conn = Connection::open_in_memory().expect("db");
        db::migrate(&mut conn).expect("migrate");
        let mut noop = |_: &str, _: i64, _: i64| {};
        let summary =
            import_file(path, &mut conn, media_dir, &mut noop, Some(mapping)).expect("import");
        let items = db::all_items(&conn).expect("items");
        (summary, items)
    }

    fn audio_mapping(reading: usize, meaning: usize, audio: usize) -> ImportMapping {
        ImportMapping {
            txt: Some(FieldMapping {
                reading: Some(reading),
                meaning: Some(meaning),
                audio: Some(audio),
                ..Default::default()
            }),
            note_types: Vec::new(),
            media_folder: None,
        }
    }

    #[test]
    fn imports_txt_audio_from_sibling_media_folder() {
        let dir = tempfile::tempdir().expect("tempdir");
        let media_folder = dir.path().join("deck");
        std::fs::create_dir(&media_folder).expect("media folder");
        std::fs::write(media_folder.join("hola.mp3"), b"fake-mp3-bytes").expect("mp3");
        let media_dir = dir.path().join("store");
        std::fs::create_dir(&media_dir).expect("media dir");

        let path = dir.path().join("deck.txt");
        std::fs::write(
            &path,
            "#separator:tab\n#columns:lectura\tsignificado\tsonido\nこんにちは\thola\t<div>[sound:hola.mp3]</div>\n",
        )
        .expect("txt");

        let (summary, items) = import_txt_with_audio(&path, &media_dir, &audio_mapping(0, 1, 2));

        assert_eq!(summary.audio_extracted, 1);
        assert_eq!(summary.audio_reused, 0);
        assert_eq!(summary.audio_missing, 0);
        assert!(summary.warnings.is_empty(), "{:?}", summary.warnings);
        assert_eq!(items.len(), 1);
        assert!(items[0].has_audio);
        let stored: Vec<_> = std::fs::read_dir(&media_dir)
            .expect("read media dir")
            .filter_map(Result::ok)
            .collect();
        assert_eq!(stored.len(), 1);
    }

    #[test]
    fn warns_about_missing_txt_audio_without_aborting() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(dir.path().join("deck")).expect("media folder");
        let media_dir = dir.path().join("store");
        std::fs::create_dir(&media_dir).expect("media dir");

        let path = dir.path().join("deck.txt");
        std::fs::write(
            &path,
            "#separator:tab\n#columns:lectura\tsignificado\tsonido\nこんにちは\thola\t[sound:missing.mp3]\n",
        )
        .expect("txt");

        let (summary, items) = import_txt_with_audio(&path, &media_dir, &audio_mapping(0, 1, 2));

        assert_eq!(summary.audio_missing, 1);
        assert_eq!(summary.audio_extracted, 0);
        assert_eq!(items.len(), 1, "la importación sigue adelante");
        assert!(!items[0].has_audio);
        assert!(
            summary
                .warnings
                .iter()
                .any(|warning| warning.contains("missing.mp3")),
            "{:?}",
            summary.warnings
        );
    }

    #[test]
    fn deduplicates_repeated_txt_audio_references() {
        let dir = tempfile::tempdir().expect("tempdir");
        let media_folder = dir.path().join("deck");
        std::fs::create_dir(&media_folder).expect("media folder");
        std::fs::write(media_folder.join("hola.mp3"), b"fake-mp3-bytes").expect("mp3");
        let media_dir = dir.path().join("store");
        std::fs::create_dir(&media_dir).expect("media dir");

        let path = dir.path().join("deck.txt");
        std::fs::write(
            &path,
            "#separator:tab\n#columns:lectura\tsignificado\tsonido\nこんにちは\thola\t[sound:hola.mp3]\nさようなら\tadiós\t[sound:hola.mp3]\n",
        )
        .expect("txt");

        let (summary, items) = import_txt_with_audio(&path, &media_dir, &audio_mapping(0, 1, 2));

        assert_eq!(summary.audio_extracted, 1);
        assert_eq!(summary.audio_reused, 1);
        assert_eq!(summary.audio_missing, 0);
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.has_audio));
        let stored: Vec<_> = std::fs::read_dir(&media_dir)
            .expect("read media dir")
            .filter_map(Result::ok)
            .collect();
        assert_eq!(stored.len(), 1, "mismo hash, un solo archivo");
    }

    #[test]
    fn uses_media_folder_chosen_in_mapping() {
        let dir = tempfile::tempdir().expect("tempdir");
        let chosen = dir.path().join("audio_extra");
        std::fs::create_dir(&chosen).expect("chosen folder");
        std::fs::write(chosen.join("hola.mp3"), b"chosen-bytes").expect("mp3");
        let media_dir = dir.path().join("store");
        std::fs::create_dir(&media_dir).expect("media dir");

        let path = dir.path().join("deck.txt");
        std::fs::write(
            &path,
            "#separator:tab\n#columns:lectura\tsignificado\tsonido\nこんにちは\thola\t[sound:hola.mp3]\n",
        )
        .expect("txt");

        let mut mapping = audio_mapping(0, 1, 2);
        mapping.media_folder = Some(chosen.to_string_lossy().into_owned());

        let (summary, items) = import_txt_with_audio(&path, &media_dir, &mapping);

        assert_eq!(summary.audio_extracted, 1);
        assert_eq!(summary.audio_missing, 0);
        assert!(items[0].has_audio);
    }

    #[test]
    #[ignore = "requiere el mazo real en el escritorio"]
    fn inspects_real_deck_note_types() {
        let path = Path::new(r"D:\Users\Javier\Desktop\Genki 1.apkg");
        if !path.exists() {
            eprintln!("mazo no encontrado, se omite");
            return;
        }
        let inspection = inspect_file(path).expect("inspect");
        assert_eq!(inspection.kind, "apkg");
        assert!(!inspection.note_types.is_empty());
        let note_type = &inspection.note_types[0];
        assert!(!note_type.fields.is_empty());
        assert!(note_type.fields.iter().any(|field| field == "kana"));
        assert!(note_type.suggested.reading.is_some());
        assert!(note_type.suggested.meaning.is_some());
    }

    #[test]
    #[ignore = "requiere el mazo real en el escritorio"]
    fn imports_real_genki_deck() {
        let path = Path::new(r"D:\Users\Javier\Desktop\Genki 1.apkg");
        if !path.exists() {
            eprintln!("mazo no encontrado, se omite");
            return;
        }
        let mut conn = Connection::open_in_memory().expect("db");
        db::migrate(&mut conn).expect("migrate");
        let dir = tempfile::tempdir().expect("tempdir");
        let mut noop = |_: &str, _: i64, _: i64| {};
        let summary = import_apkg(path, &mut conn, dir.path(), &mut noop, None).expect("import");
        println!("{summary:#?}");
        assert_eq!(summary.notes, 1076);
        assert!(summary.items_inserted >= 1000);
        assert!(summary.audio_extracted >= 700);
        assert_eq!(summary.sources, 1076);
    }

    #[test]
    #[ignore = "requiere el mazo real en el escritorio"]
    fn reports_import_progress() {
        let path = Path::new(r"D:\Users\Javier\Desktop\Genki 1.apkg");
        if !path.exists() {
            eprintln!("mazo no encontrado, se omite");
            return;
        }

        let mut conn = Connection::open_in_memory().expect("db");
        db::migrate(&mut conn).expect("migrate");
        let dir = tempfile::tempdir().expect("tempdir");

        let mut events: Vec<(String, i64, i64)> = Vec::new();
        {
            let mut progress = |phase: &str, current: i64, total: i64| {
                events.push((phase.to_string(), current, total));
            };
            import_apkg(path, &mut conn, dir.path(), &mut progress, None).expect("import");
        }

        assert!(events.len() > 5, "se esperaban varios eventos de progreso");
        let (phase, current, total) = events.last().expect("last event");
        assert_eq!(phase.as_str(), "done");
        assert_eq!(*current, 1076);
        assert_eq!(*total, 1076);
        assert!(events.iter().any(|(_, current, _)| *current > 0));
    }

    fn primary_japanese(item: &crate::models::Item) -> Option<String> {
        if let Some(kanji) = item.kanji.as_deref() {
            let cleaned = text::clean_display(kanji);
            let without = text::remove_parentheticals(&cleaned);
            for token in without.split_whitespace() {
                let token = token.trim_matches(|c: char| matches!(c, '。' | '.' | ',' | ';' | '、'));
                if text::has_japanese(token) {
                    return Some(token.to_string());
                }
            }
        }

        let reading = text::clean_display(&item.reading).replace('*', "");
        let without = text::remove_parentheticals(&reading);
        for token in without.split(|c: char| {
            c == '／' || c == '/' || c == '+' || c.is_whitespace()
        }) {
            let token = token.trim_matches(|c: char| {
                matches!(c, '。' | '.' | ',' | ';' | '、' | '〜')
            });
            if text::has_japanese(token) {
                return Some(token.to_string());
            }
        }
        None
    }

    fn meaning_candidates(item: &crate::models::Item) -> Vec<String> {
        let cleaned = text::clean_display(&item.meaning);
        let without = text::remove_parentheticals(&cleaned);
        let mut out: Vec<String> = Vec::new();

        for source in [&cleaned, &without] {
            for part in source.split(';') {
                for sub in part.split(" / ") {
                    let sub = sub.replace("...", "").replace('…', "");
                    let sub = sub.trim_matches(|c: char| {
                        matches!(c, '。' | '.' | ',' | ';' | ':' | '、' | '—' | '-')
                            || c.is_whitespace()
                    });
                    if !sub.is_empty() && !out.iter().any(|value| value.as_str() == sub) {
                        out.push(sub.to_string());
                    }
                }
            }
        }

        out
    }

    #[test]
    #[ignore = "requiere el mazo real en el escritorio"]
    fn normalizes_real_deck_answers() {
        let path = Path::new(r"D:\Users\Javier\Desktop\Genki 1.apkg");
        if !path.exists() {
            eprintln!("mazo no encontrado, se omite");
            return;
        }

        let mut conn = Connection::open_in_memory().expect("db");
        db::migrate(&mut conn).expect("migrate");
        let dir = tempfile::tempdir().expect("tempdir");
        let mut noop = |_: &str, _: i64, _: i64| {};
        import_apkg(path, &mut conn, dir.path(), &mut noop, None).expect("import");

        let items = db::all_items(&conn).expect("items");
        assert!(items.len() >= 1000, "se esperaban ~1.012 ítems");

        let mut failures = Vec::new();
        let mut checked = 0usize;

        for item in &items {
            if let Some(answer) = primary_japanese(item) {
                checked += 1;
                if !text::check_es_to_jp(&answer, item) {
                    failures.push(format!(
                        "ES->JP rechazó {answer:?} para {:?} / {:?}",
                        item.reading, item.meaning
                    ));
                }

                let has_kanji = answer.chars().any(|c| {
                    let value = c as u32;
                    (0x3400..=0x4DBF).contains(&value) || (0x4E00..=0x9FFF).contains(&value)
                });
                let kana_only = answer.chars().all(|c| {
                    let value = c as u32;
                    (0x3040..=0x309F).contains(&value) || (0x30A0..=0x30FF).contains(&value)
                });
                if !has_kanji && kana_only {
                    if let Some(romaji) = text::romaji_variants(&answer).into_iter().next() {
                        checked += 1;
                        if !text::check_es_to_jp(&romaji, item) {
                            failures.push(format!(
                                "ES->JP rechazó romaji {romaji:?} para {:?} / {:?}",
                                item.reading, item.meaning
                            ));
                        }
                    }
                }
            }

            for answer in meaning_candidates(item) {
                checked += 1;
                if !text::check_jp_to_es(&answer, item) {
                    failures.push(format!(
                        "JP->ES rechazó {answer:?} para {:?} / {:?}",
                        item.reading, item.meaning
                    ));
                }
            }

            assert!(
                !text::check_es_to_jp("zzzz", item),
                "aceptó basura para {:?} / {:?}",
                item.reading,
                item.meaning
            );
        }

        if !failures.is_empty() {
            panic!(
                "{} fallos de {} comprobaciones:\n{}",
                failures.len(),
                checked,
                failures.join("\n")
            );
        }

        println!("{checked} respuestas reales validadas");
    }
}
