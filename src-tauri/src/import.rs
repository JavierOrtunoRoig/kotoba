use crate::db::{self, InsertOutcome, NewItem};
use crate::media::{self, MediaEntry};
use crate::models::ImportSummary;
use crate::text;
use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

const MAX_WARNINGS: usize = 20;

pub fn import_file(path: &Path, conn: &mut Connection, media_dir: &Path) -> Result<ImportSummary> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();
    match extension.as_str() {
        "apkg" => import_apkg(path, conn, media_dir),
        "txt" | "tsv" | "csv" => import_txt(path, conn),
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
) -> Result<ImportSummary> {
    let file = File::open(path)
        .with_context(|| format!("No se pudo abrir {}", path.display()))?;
    let mut archive = ZipArchive::new(file).context("El archivo no es un .apkg válido")?;

    let db_bytes = read_collection(&mut archive)?;
    let tmp = tempfile::tempdir()?;
    let tmp_db = tmp.path().join("collection.sqlite");
    std::fs::write(&tmp_db, &db_bytes)?;

    let src = Connection::open(&tmp_db)?;
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
        deck_name: decks_by_note
            .values()
            .next()
            .and_then(|name| name.split('\u{1f}').next())
            .unwrap_or("Anki")
            .to_string(),
        ..Default::default()
    };

    let tx = conn.transaction()?;
    let mut audio_cache: HashMap<String, Option<String>> = HashMap::new();

    for note in &notes {
        let values: Vec<&str> = note.flds.split('\u{1f}').collect();
        let lower_names: Vec<String> = fields_by_mid
            .get(&note.mid)
            .map(|names| names.iter().map(|n| n.trim().to_lowercase()).collect())
            .unwrap_or_default();

        let reading_raw =
            field(&values, &lower_names, &["kana", "reading", "lectura"], Some(0));
        let kanji_raw = field(&values, &lower_names, &["kanjis", "kanji", "word"], Some(1));
        let meaning_raw = field(
            &values,
            &lower_names,
            &["español", "espanol", "meaning"],
            Some(3),
        )
        .or_else(|| field(&values, &lower_names, &["significado", "definition"], Some(3)));
        let gloss_raw = field(
            &values,
            &lower_names,
            &["significado", "kanji meaning", "gloss"],
            Some(4),
        );
        let audio_raw = field(&values, &lower_names, &["sonido", "audio", "sound"], Some(5));
        let uid_raw = field(&values, &lower_names, &["uid", "guid"], Some(7));

        let reading = text::clean_display(reading_raw.unwrap_or_default());
        let kanji_text = text::clean_display(kanji_raw.unwrap_or_default());
        let meaning = text::clean_display(meaning_raw.unwrap_or_default());

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

        let gloss = non_empty(text::clean_display(gloss_raw.unwrap_or_default()));
        let gloss = if gloss.as_deref() == Some(meaning.as_str()) {
            None
        } else {
            gloss
        };
        let kanji = non_empty(kanji_text);

        let deck_name = decks_by_note.get(&note.id).cloned().unwrap_or_default();
        let (lesson, category) = if deck_name.is_empty() {
            parse_uid(uid_raw.unwrap_or_default())
        } else {
            parse_deck(&deck_name)
        };

        let uid = non_empty(uid_raw.unwrap_or_default().to_string())
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

        let tags = note.tags.trim().to_string();
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
        "SELECT c.nid, d.name FROM cards c JOIN decks d ON d.id = c.did",
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
    let mut stmt = src.prepare("SELECT id, mid, flds, tags FROM notes")?;
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

    let sha = hex::encode(Sha1::digest(&bytes));
    if let Some(expected) = &entry.sha1 {
        if expected != &sha {
            // El hash no coincide: seguimos, pero lo dejamos anotado en la caché.
            cache.insert(entry.name.clone(), None);
            return Err(anyhow!("sha1 no coincide para {}", entry.name));
        }
    }

    let extension = Path::new(&entry.name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("bin");
    let filename = format!("{sha}.{extension}");
    let target = media_dir.join(&filename);
    let is_new = !target.exists();
    if is_new {
        std::fs::write(&target, &bytes)?;
    }
    cache.insert(entry.name.clone(), Some(filename.clone()));
    Ok(Some((filename, is_new)))
}

fn field<'a>(
    values: &[&'a str],
    lower_names: &[String],
    candidates: &[&str],
    fallback: Option<usize>,
) -> Option<&'a str> {
    for candidate in candidates {
        if let Some(index) = lower_names.iter().position(|name| name == candidate) {
            if let Some(value) = values.get(index).copied() {
                return Some(value);
            }
        }
    }
    fallback.and_then(|index| values.get(index).copied())
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
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

pub fn import_txt(path: &Path, conn: &mut Connection) -> Result<ImportSummary> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("No se pudo leer {}", path.display()))?;

    let mut separator = '\t';
    let mut tags_column: Option<usize> = None;
    let mut data_lines = Vec::new();

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
                tags_column = value.trim().parse::<usize>().ok().map(|n| n.saturating_sub(1));
            }
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        data_lines.push(line);
    }

    let deck_name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Anki")
        .to_string();

    let mut summary = ImportSummary {
        notes: data_lines.len() as i64,
        deck_name: deck_name.clone(),
        ..Default::default()
    };

    let tx = conn.transaction()?;
    for (line_index, line) in data_lines.iter().enumerate() {
        let values: Vec<&str> = line.split(separator).collect();
        if values.len() < 3 {
            push_warning(
                &mut summary,
                format!("Línea {} con menos de 3 columnas, se omite.", line_index + 1),
            );
            continue;
        }

        let (reading_raw, kanji_raw, meaning_raw, gloss_raw, uid_raw) = if values.len() >= 8 {
            (
                values[0],
                values[1],
                values[3],
                Some(values[4]),
                Some(values[7]),
            )
        } else {
            (values[0], values[1], values[2], None, None)
        };

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
            .map(|value| text::clean_display(value))
            .and_then(non_empty);
        let kanji = non_empty(kanji_text);

        let uid = uid_raw
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("{deck_name}::txt::{line_index}"));

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
        let tags = tags_column
            .and_then(|index| values.get(index))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();

        let new_item = NewItem {
            source_uid: uid.as_str(),
            kanji: kanji.as_deref(),
            reading: reading.as_str(),
            meaning: meaning.as_str(),
            meaning_plain: meaning_plain.as_str(),
            kanji_gloss: gloss.as_deref(),
            audio_file: None,
            lesson: deck_name.as_str(),
            category: "",
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

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let summary = import_apkg(path, &mut conn, dir.path()).expect("import");
        println!("{summary:#?}");
        assert_eq!(summary.notes, 1076);
        assert!(summary.items_inserted >= 1000);
        assert!(summary.audio_extracted >= 700);
        assert_eq!(summary.sources, 1076);
    }
}
