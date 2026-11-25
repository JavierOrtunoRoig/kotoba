use anyhow::{anyhow, Context, Result};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct MediaEntry {
    pub name: String,
    pub size: u64,
    pub sha1: Option<String>,
}

pub fn is_zstd(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x28, 0xb5, 0x2f, 0xfd])
}

/// Lee el mapa de medios de un `.apkg`.
///
/// - Formato nuevo (Anki 2.1.50+): zstd + protobuf.
/// - Formato antiguo: JSON plano `{ "0": "nombre.mp3", ... }`.
pub fn parse_media_map(bytes: &[u8]) -> Result<Vec<MediaEntry>> {
    let decoded = if is_zstd(bytes) {
        zstd::stream::decode_all(bytes)?
    } else {
        bytes.to_vec()
    };

    if let Ok(entries) = parse_protobuf(&decoded) {
        if !entries.is_empty() {
            return Ok(entries);
        }
    }

    parse_legacy_json(&decoded)
}

/// Copia un audio a la carpeta de medios con nombre por hash.
///
/// Devuelve el nombre de archivo y si se ha escrito ahora (`true`) o ya existía (`false`).
/// Cuando hay un SHA-1 esperado (mapa de medios de un `.apkg`) se verifica antes de escribir.
pub fn store_bytes(
    bytes: &[u8],
    name: &str,
    expected_sha1: Option<&str>,
    media_dir: &Path,
) -> Result<Option<(String, bool)>> {
    let sha = hex::encode(Sha1::digest(bytes));
    if let Some(expected) = expected_sha1 {
        if expected != sha {
            return Err(anyhow!("sha1 no coincide para {name}"));
        }
    }

    let extension = Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("bin");
    let filename = format!("{sha}.{extension}");
    let target = media_dir.join(&filename);
    let is_new = !target.exists();
    if is_new {
        std::fs::write(&target, bytes)
            .with_context(|| format!("no se pudo escribir {}", target.display()))?;
    }
    Ok(Some((filename, is_new)))
}

fn parse_legacy_json(bytes: &[u8]) -> Result<Vec<MediaEntry>> {
    let map: HashMap<String, String> = serde_json::from_slice(bytes)?;
    let mut indexed: Vec<(usize, String)> = map
        .into_iter()
        .filter_map(|(key, name)| key.parse::<usize>().ok().map(|index| (index, name)))
        .collect();
    indexed.sort_by_key(|(index, _)| *index);
    Ok(indexed
        .into_iter()
        .map(|(_, name)| MediaEntry {
            name,
            size: 0,
            sha1: None,
        })
        .collect())
}

fn parse_protobuf(bytes: &[u8]) -> Result<Vec<MediaEntry>> {
    let mut entries = Vec::new();
    let mut pos = 0usize;

    while pos < bytes.len() {
        let tag = read_varint(bytes, &mut pos)?;
        if tag != 0x0a {
            return Err(anyhow!("campo inesperado en el mapa de medios: {tag}"));
        }
        let len = read_varint(bytes, &mut pos)? as usize;
        let end = pos
            .checked_add(len)
            .ok_or_else(|| anyhow!("longitud desbordada"))?;
        if end > bytes.len() {
            return Err(anyhow!("mapa de medios truncado"));
        }

        let mut entry = MediaEntry::default();
        let mut inner = pos;
        while inner < end {
            let field = read_varint(bytes, &mut inner)?;
            match field {
                0x0a => {
                    let name_len = read_varint(bytes, &mut inner)? as usize;
                    let name_end = inner + name_len;
                    if name_end > end {
                        return Err(anyhow!("nombre de medio truncado"));
                    }
                    entry.name = String::from_utf8_lossy(&bytes[inner..name_end]).into_owned();
                    inner = name_end;
                }
                0x10 => {
                    entry.size = read_varint(bytes, &mut inner)?;
                }
                0x1a => {
                    let sha_len = read_varint(bytes, &mut inner)? as usize;
                    let sha_end = inner + sha_len;
                    if sha_end > end {
                        return Err(anyhow!("sha1 truncado"));
                    }
                    entry.sha1 = Some(hex::encode(&bytes[inner..sha_end]));
                    inner = sha_end;
                }
                other => return Err(anyhow!("campo inesperado en entrada de medio: {other}")),
            }
        }

        if !entry.name.is_empty() {
            entries.push(entry);
        }
        pos = end;
    }

    Ok(entries)
}

fn read_varint(bytes: &[u8], pos: &mut usize) -> Result<u64> {
    let mut result = 0u64;
    let mut shift = 0u32;
    loop {
        let byte = *bytes
            .get(*pos)
            .ok_or_else(|| anyhow!("varint truncado"))?;
        *pos += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 64 {
            return Err(anyhow!("varint demasiado largo"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    #[test]
    fn parses_protobuf_media_map() {
        let name = b"01_Saludos.mp3";
        let sha = [0xabu8; 20];

        let mut entry = Vec::new();
        entry.push(0x0a);
        encode_varint(name.len() as u64, &mut entry);
        entry.extend_from_slice(name);
        entry.push(0x10);
        encode_varint(1234, &mut entry);
        entry.push(0x1a);
        encode_varint(sha.len() as u64, &mut entry);
        entry.extend_from_slice(&sha);

        let mut map = Vec::new();
        map.push(0x0a);
        encode_varint(entry.len() as u64, &mut map);
        map.extend_from_slice(&entry);

        let entries = parse_media_map(&map).expect("parse");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "01_Saludos.mp3");
        assert_eq!(entries[0].size, 1234);
        assert_eq!(entries[0].sha1.as_deref(), Some(hex::encode(sha).as_str()));
    }

    #[test]
    fn parses_legacy_json_media_map() {
        let json = br#"{ "0": "a.mp3", "1": "b.mp3" }"#;
        let entries = parse_media_map(json).expect("parse");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "a.mp3");
        assert_eq!(entries[1].name, "b.mp3");
    }
}
