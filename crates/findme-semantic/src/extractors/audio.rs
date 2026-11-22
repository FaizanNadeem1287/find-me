//! Multimodal audio metadata extraction and speech/transcription chunking pipeline
//!
//! Provides fast, pure-Rust header parsing and metadata decoding for common audio
//! formats (WAV, MP3, OGG Vorbis, FLAC) without external C libraries or FFmpeg dependencies.

use crate::error::SemanticError;
use crate::models::{DocumentChunk, DocumentMetadata, DocumentType, ExtractedDocument};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Maximum audio header bytes read during inspection (128 KB)
const HEADER_PROBE_SIZE: usize = 128 * 1024;

/// Extracted intermediate audio attributes
#[derive(Debug, Default, Clone)]
struct ParsedAudioInfo {
    codec: String,
    duration_secs: Option<u64>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    comments: Option<String>,
}

/// Formats seconds into standard timestamp notation `MM:SS` or `HH:MM:SS`
pub fn format_duration(total_secs: u64) -> String {
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{hours:02}:{mins:02}:{secs:02}")
    } else {
        format!("{mins:02}:{secs:02}")
    }
}

/// Extracts metadata and generates semantic time-sliced chunks from an audio file
pub fn extract_audio(path: &Path) -> Result<ExtractedDocument, SemanticError> {
    let mut file = File::open(path)?;
    let file_len = file.metadata()?.len();

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let mut header_buf = vec![0u8; (file_len as usize).min(HEADER_PROBE_SIZE)];
    file.read_exact(&mut header_buf)?;

    let mut info = ParsedAudioInfo::default();

    match ext.as_str() {
        "wav" => {
            parse_wav_header(&header_buf, file_len, &mut info);
        }
        "mp3" => {
            parse_mp3_header(&mut file, &header_buf, file_len, &mut info);
        }
        "ogg" => {
            parse_ogg_header(&mut file, &header_buf, file_len, &mut info);
        }
        "flac" => {
            parse_flac_header(&header_buf, file_len, &mut info);
        }
        _ => {
            info.codec = ext.to_uppercase();
        }
    }

    if info.codec.is_empty() {
        info.codec = ext.to_uppercase();
    }

    let duration_label = info
        .duration_secs
        .map_or_else(|| "Unknown length".to_string(), format_duration);

    let mut chunks = Vec::new();

    // 1. Primary Header & Tags Chunk
    let mut summary = format!(
        "Audio ({codec}) • Duration: {duration}",
        codec = info.codec,
        duration = duration_label
    );

    if let (Some(sr), Some(ch)) = (info.sample_rate, info.channels) {
        let channel_label = match ch {
            1 => "Mono",
            2 => "Stereo",
            n => &format!("{n} Channels"),
        };
        summary.push_str(&format!(" • {sr} Hz ({channel_label})"));
    }

    if let Some(ref title) = info.title {
        summary.push_str(&format!("\nTitle: {title}"));
    }
    if let Some(ref artist) = info.artist {
        summary.push_str(&format!("\nArtist: {artist}"));
    }
    if let Some(ref album) = info.album {
        summary.push_str(&format!("\nAlbum: {album}"));
    }
    if let Some(ref comments) = info.comments {
        summary.push_str(&format!("\nDescription: {comments}"));
    }

    chunks.push(DocumentChunk::new(0, "Audio Header & Tags", summary));

    // 2. Segmented Time-Sliced Chunks (30-second windows for timestamp jump-links)
    if let Some(total_secs) = info.duration_secs {
        if total_secs > 0 {
            let chunk_window = 30u64;
            let num_segments = total_secs.div_ceil(chunk_window).max(1);

            for seg_idx in 0..num_segments {
                let start_sec = seg_idx * chunk_window;
                let end_sec = ((seg_idx + 1) * chunk_window).min(total_secs);
                let time_marker = format!(
                    "[{start} - {end}]",
                    start = format_duration(start_sec),
                    end = format_duration(end_sec)
                );

                let seg_text = format!(
                    "Audio Segment {time_marker} • {codec} track '{name}'",
                    codec = info.codec,
                    name = info
                        .title
                        .as_deref()
                        .or_else(|| path.file_stem().and_then(|s| s.to_str()))
                        .unwrap_or("Audio")
                );

                chunks.push(DocumentChunk::new(chunks.len(), time_marker, seg_text));
            }
        }
    }

    let metadata = DocumentMetadata {
        title: info.title,
        author: info.artist,
        duration_secs: info.duration_secs,
        sample_rate: info.sample_rate,
        channels: info.channels,
        audio_codec: Some(info.codec),
        language_or_ext: Some(ext),
        ..Default::default()
    };

    Ok(ExtractedDocument::new(
        path.to_path_buf(),
        DocumentType::Audio,
        chunks,
        metadata,
    ))
}

/// Parses standard RIFF/WAVE header and INFO tags
fn parse_wav_header(buf: &[u8], file_len: u64, info: &mut ParsedAudioInfo) {
    info.codec = "PCM WAV".to_string();
    if buf.len() < 44 || &buf[0..4] != b"RIFF" || &buf[8..12] != b"WAVE" {
        return;
    }

    let mut cursor = 12;
    let mut data_size: Option<u32> = None;

    while cursor + 8 <= buf.len() {
        let chunk_id = &buf[cursor..cursor + 4];
        let chunk_size = u32::from_le_bytes([
            buf[cursor + 4],
            buf[cursor + 5],
            buf[cursor + 6],
            buf[cursor + 7],
        ]);
        let data_start = cursor + 8;

        if chunk_id == b"fmt " && data_start + 14 <= buf.len() {
            let format_tag = u16::from_le_bytes([buf[data_start], buf[data_start + 1]]);
            info.channels = Some(u16::from_le_bytes([
                buf[data_start + 2],
                buf[data_start + 3],
            ]));
            info.sample_rate = Some(u32::from_le_bytes([
                buf[data_start + 4],
                buf[data_start + 5],
                buf[data_start + 6],
                buf[data_start + 7],
            ]));
            let byte_rate = u32::from_le_bytes([
                buf[data_start + 8],
                buf[data_start + 9],
                buf[data_start + 10],
                buf[data_start + 11],
            ]);

            info.codec = match format_tag {
                1 => "PCM WAV",
                3 => "IEEE Float WAV",
                6 => "A-Law WAV",
                7 => "μ-Law WAV",
                0xFFFE => "Extensible WAV",
                _ => "WAV",
            }
            .to_string();

            if byte_rate > 0 && data_size.is_some() {
                if let Some(ds) = data_size {
                    info.duration_secs = Some((ds as u64) / (byte_rate as u64));
                }
            }
        } else if chunk_id == b"data" {
            data_size = Some(chunk_size);
            if let (Some(sr), Some(ch)) = (info.sample_rate, info.channels) {
                let bytes_per_sample = 2u64; // Default 16-bit
                let denom = (sr as u64) * (ch as u64) * bytes_per_sample;
                let actual_data = (chunk_size as u64).min(file_len.saturating_sub(44));
                info.duration_secs = actual_data.checked_div(denom);
            }
        }

        // Advance to next chunk (padded to even 2-byte boundary)
        let pad = (chunk_size % 2) as usize;
        cursor = data_start + chunk_size as usize + pad;
    }
}

/// Parses MP3 ID3v2 metadata, frame sync, and ID3v1 trailer
fn parse_mp3_header(file: &mut File, buf: &[u8], file_len: u64, info: &mut ParsedAudioInfo) {
    info.codec = "MP3 (MPEG-1 Layer III)".to_string();

    // Check ID3v2 header
    let mut audio_offset = 0usize;
    if buf.len() >= 10 && &buf[0..3] == b"ID3" {
        let tag_size = ((buf[6] as usize & 0x7F) << 21)
            | ((buf[7] as usize & 0x7F) << 14)
            | ((buf[8] as usize & 0x7F) << 7)
            | (buf[9] as usize & 0x7F);
        audio_offset = 10 + tag_size;

        parse_id3v2_frames(&buf[10..(10 + tag_size).min(buf.len())], info);
    }

    // Read ID3v1 at end of file if tags are still missing
    if (info.title.is_none() || info.artist.is_none())
        && file_len >= 128
        && file.seek(SeekFrom::End(-128)).is_ok()
    {
        let mut id3v1 = [0u8; 128];
        if file.read_exact(&mut id3v1).is_ok() && &id3v1[0..3] == b"TAG" {
            if info.title.is_none() {
                let title = String::from_utf8_lossy(&id3v1[3..33]).trim().to_string();
                if !title.is_empty() {
                    info.title = Some(title);
                }
            }
            if info.artist.is_none() {
                let artist = String::from_utf8_lossy(&id3v1[33..63]).trim().to_string();
                if !artist.is_empty() {
                    info.artist = Some(artist);
                }
            }
            if info.album.is_none() {
                let album = String::from_utf8_lossy(&id3v1[63..93]).trim().to_string();
                if !album.is_empty() {
                    info.album = Some(album);
                }
            }
        }
    }

    // Scan for MPEG Audio Frame Header Sync (0xFF 0xEx)
    let probe_start = audio_offset.min(buf.len().saturating_sub(4));
    for i in probe_start..buf.len().saturating_sub(4) {
        if buf[i] == 0xFF && (buf[i + 1] & 0xE0) == 0xE0 {
            let layer = (buf[i + 1] >> 1) & 0x03;
            let bitrate_idx = (buf[i + 2] >> 4) & 0x0F;
            let sr_idx = (buf[i + 2] >> 2) & 0x03;
            let channel_mode = (buf[i + 3] >> 6) & 0x03;

            info.channels = Some(if channel_mode == 3 { 1 } else { 2 });

            let sample_rates = [44100, 48000, 32000, 0];
            if (sr_idx as usize) < 3 {
                info.sample_rate = Some(sample_rates[sr_idx as usize]);
            }

            let bitrates_layer3 = [
                0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0,
            ];

            if layer == 1 && (bitrate_idx as usize) < bitrates_layer3.len() {
                let kbps = bitrates_layer3[bitrate_idx as usize];
                if kbps > 0 {
                    let audio_bytes = file_len.saturating_sub(audio_offset as u64);
                    info.duration_secs = Some((audio_bytes * 8) / (kbps as u64 * 1000));
                }
            }
            break;
        }
    }
}

/// Parses ID3v2 text frames (TIT2, TPE1, TALB)
fn parse_id3v2_frames(data: &[u8], info: &mut ParsedAudioInfo) {
    let mut cursor = 0;
    while cursor + 10 <= data.len() {
        let frame_id = &data[cursor..cursor + 4];
        let frame_size = u32::from_be_bytes([
            data[cursor + 4],
            data[cursor + 5],
            data[cursor + 6],
            data[cursor + 7],
        ]) as usize;

        let frame_data_start = cursor + 10;
        if frame_size == 0 || frame_data_start + frame_size > data.len() {
            break;
        }

        let raw_text = &data[frame_data_start..frame_data_start + frame_size];
        let text = decode_id3_string(raw_text);

        if !text.is_empty() {
            match frame_id {
                b"TIT2" if info.title.is_none() => info.title = Some(text),
                b"TPE1" if info.artist.is_none() => info.artist = Some(text),
                b"TALB" if info.album.is_none() => info.album = Some(text),
                b"COMM" if info.comments.is_none() => info.comments = Some(text),
                _ => {}
            }
        }

        cursor = frame_data_start + frame_size;
    }
}

/// Decodes ID3 string respecting leading encoding byte (ISO-8859-1 or UTF-8)
fn decode_id3_string(raw: &[u8]) -> String {
    if raw.is_empty() {
        return String::new();
    }
    let slice = if raw[0] <= 3 && raw.len() > 1 {
        &raw[1..]
    } else {
        raw
    };
    String::from_utf8_lossy(slice)
        .trim_matches('\0')
        .trim()
        .to_string()
}

/// Parses Ogg container (Vorbis / Opus) headers and comments
fn parse_ogg_header(file: &mut File, buf: &[u8], file_len: u64, info: &mut ParsedAudioInfo) {
    info.codec = "OGG (Vorbis/Opus)".to_string();
    if buf.len() < 28 || &buf[0..4] != b"OggS" {
        return;
    }

    // Inspect Vorbis identification header (\x01vorbis)
    if let Some(pos) = find_subsequence(buf, b"\x01vorbis") {
        if pos + 16 <= buf.len() {
            info.channels = Some(buf[pos + 11] as u16);
            info.sample_rate = Some(u32::from_le_bytes([
                buf[pos + 12],
                buf[pos + 13],
                buf[pos + 14],
                buf[pos + 15],
            ]));
        }
    }

    // Inspect Vorbis comment header (\x03vorbis)
    if let Some(pos) = find_subsequence(buf, b"\x03vorbis") {
        parse_vorbis_comments(&buf[pos + 7..], info);
    }

    // Estimate duration via last page granule position
    if let Some(sr) = info.sample_rate {
        if sr > 0 && file_len > 4096 && file.seek(SeekFrom::End(-4096)).is_ok() {
            let mut tail = [0u8; 4096];
            if file.read_exact(&mut tail).is_ok() {
                if let Some(last_oggs) = rfind_subsequence(&tail, b"OggS") {
                    if last_oggs + 14 <= tail.len() {
                        let granule = u64::from_le_bytes([
                            tail[last_oggs + 6],
                            tail[last_oggs + 7],
                            tail[last_oggs + 8],
                            tail[last_oggs + 9],
                            tail[last_oggs + 10],
                            tail[last_oggs + 11],
                            tail[last_oggs + 12],
                            tail[last_oggs + 13],
                        ]);
                        if granule > 0 && granule != u64::MAX {
                            info.duration_secs = Some(granule / (sr as u64));
                        }
                    }
                }
            }
        }
    }
}

/// Parses Vorbis comment block (TITLE=, ARTIST=, ALBUM=)
fn parse_vorbis_comments(data: &[u8], info: &mut ParsedAudioInfo) {
    if data.len() < 8 {
        return;
    }
    let vendor_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut cursor = 4 + vendor_len;
    if cursor + 4 > data.len() {
        return;
    }

    let comment_count = u32::from_le_bytes([
        data[cursor],
        data[cursor + 1],
        data[cursor + 2],
        data[cursor + 3],
    ]) as usize;
    cursor += 4;

    for _ in 0..comment_count.min(100) {
        if cursor + 4 > data.len() {
            break;
        }
        let len = u32::from_le_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]) as usize;
        cursor += 4;

        if cursor + len > data.len() {
            break;
        }

        let comment = String::from_utf8_lossy(&data[cursor..cursor + len]);
        cursor += len;

        if let Some((key, val)) = comment.split_once('=') {
            match key.to_ascii_uppercase().as_str() {
                "TITLE" if info.title.is_none() => info.title = Some(val.trim().to_string()),
                "ARTIST" if info.artist.is_none() => info.artist = Some(val.trim().to_string()),
                "ALBUM" if info.album.is_none() => info.album = Some(val.trim().to_string()),
                "DESCRIPTION" | "COMMENT" if info.comments.is_none() => {
                    info.comments = Some(val.trim().to_string())
                }
                _ => {}
            }
        }
    }
}

/// Parses FLAC header and STREAMINFO block
fn parse_flac_header(buf: &[u8], _file_len: u64, info: &mut ParsedAudioInfo) {
    info.codec = "FLAC (Lossless)".to_string();
    if buf.len() < 42 || &buf[0..4] != b"fLaC" {
        return;
    }

    // STREAMINFO is always the first block (offset 4, data at 8, 34 bytes)
    let streaminfo = &buf[8..42];
    let sr_high = (streaminfo[10] as u32) << 12;
    let sr_mid = (streaminfo[11] as u32) << 4;
    let sr_low = (streaminfo[12] as u32) >> 4;
    let sample_rate = sr_high | sr_mid | sr_low;
    info.sample_rate = Some(sample_rate);

    let channels = (((streaminfo[12] >> 1) & 0x07) + 1) as u16;
    info.channels = Some(channels);

    let total_samples = (((streaminfo[13] as u64) & 0x0F) << 32)
        | ((streaminfo[14] as u64) << 24)
        | ((streaminfo[15] as u64) << 16)
        | ((streaminfo[16] as u64) << 8)
        | (streaminfo[17] as u64);

    if sample_rate > 0 && total_samples > 0 {
        info.duration_secs = Some(total_samples / (sample_rate as u64));
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn rfind_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}
