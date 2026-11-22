use findme_semantic::extractors::audio::{extract_audio, format_duration};
use findme_semantic::extractors::extract_content;
use findme_semantic::extractors::image::{
    calculate_aspect_ratio, calculate_orientation, compute_dhash, extract_image,
};
use findme_semantic::models::DocumentType;
use image::{ImageBuffer, Rgb};
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(0), "00:00");
    assert_eq!(format_duration(45), "00:45");
    assert_eq!(format_duration(65), "01:05");
    assert_eq!(format_duration(3600), "01:00:00");
    assert_eq!(format_duration(3665), "01:01:05");
}

#[test]
fn test_aspect_ratio_and_orientation() {
    assert_eq!(calculate_aspect_ratio(1920, 1080), "16:9");
    assert_eq!(calculate_aspect_ratio(800, 600), "4:3");
    assert_eq!(calculate_aspect_ratio(500, 500), "1:1");
    assert_eq!(calculate_orientation(1920, 1080), "Landscape");
    assert_eq!(calculate_orientation(1080, 1920), "Portrait");
    assert_eq!(calculate_orientation(500, 500), "Square");
}

#[test]
fn test_extract_wav_audio() {
    let mut file = NamedTempFile::with_suffix(".wav").expect("tempfile");

    // Construct valid PCM 16-bit Stereo 44.1kHz WAV with 44100 samples (1 second)
    let sample_rate: u32 = 44100;
    let channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let bytes_per_sample = (bits_per_sample / 8) * channels;
    let data_len = sample_rate * (bytes_per_sample as u32); // 1 second of audio data

    let mut wav_bytes = Vec::new();
    wav_bytes.extend_from_slice(b"RIFF");
    wav_bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav_bytes.extend_from_slice(b"WAVE");

    // fmt subchunk
    wav_bytes.extend_from_slice(b"fmt ");
    wav_bytes.extend_from_slice(&16u32.to_le_bytes()); // subchunk1 size
    wav_bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav_bytes.extend_from_slice(&channels.to_le_bytes());
    wav_bytes.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * (bytes_per_sample as u32);
    wav_bytes.extend_from_slice(&byte_rate.to_le_bytes());
    wav_bytes.extend_from_slice(&bytes_per_sample.to_le_bytes());
    wav_bytes.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data subchunk
    wav_bytes.extend_from_slice(b"data");
    wav_bytes.extend_from_slice(&data_len.to_le_bytes());
    wav_bytes.resize(wav_bytes.len() + (data_len as usize), 0);

    file.write_all(&wav_bytes).expect("write");

    let doc = extract_audio(file.path()).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::Audio);
    assert_eq!(doc.metadata.sample_rate, Some(44100));
    assert_eq!(doc.metadata.channels, Some(2));
    assert_eq!(doc.metadata.duration_secs, Some(1));
    assert!(doc.metadata.audio_codec.as_deref().unwrap().contains("WAV"));
    assert!(!doc.chunks.is_empty());
    assert!(doc.chunks[0].text.contains("PCM WAV"));
    assert!(doc.chunks[0].text.contains("44100 Hz (Stereo)"));
}

#[test]
fn test_extract_mp3_with_id3v2() {
    let mut file = NamedTempFile::with_suffix(".mp3").expect("tempfile");

    let mut mp3_bytes = Vec::new();
    // ID3v2.3 header
    mp3_bytes.extend_from_slice(b"ID3");
    mp3_bytes.push(3); // major version 2.3
    mp3_bytes.push(0); // revision
    mp3_bytes.push(0); // flags

    // Construct ID3v2 frames for TIT2 (Title) and TPE1 (Artist)
    let mut frames = Vec::new();
    // TIT2 frame
    let title = "Meeting Voice Note";
    frames.extend_from_slice(b"TIT2");
    frames.extend_from_slice(&((title.len() + 1) as u32).to_be_bytes());
    frames.extend_from_slice(&[0, 0]); // flags
    frames.push(0); // ISO-8859-1 encoding
    frames.extend_from_slice(title.as_bytes());

    // TPE1 frame
    let artist = "Faizan Nadeem";
    frames.extend_from_slice(b"TPE1");
    frames.extend_from_slice(&((artist.len() + 1) as u32).to_be_bytes());
    frames.extend_from_slice(&[0, 0]); // flags
    frames.push(0);
    frames.extend_from_slice(artist.as_bytes());

    let frame_len = frames.len();
    // Syncsafe integer for ID3v2 tag size
    let b1 = ((frame_len >> 21) & 0x7F) as u8;
    let b2 = ((frame_len >> 14) & 0x7F) as u8;
    let b3 = ((frame_len >> 7) & 0x7F) as u8;
    let b4 = (frame_len & 0x7F) as u8;
    mp3_bytes.extend_from_slice(&[b1, b2, b3, b4]);
    mp3_bytes.extend_from_slice(&frames);

    // Append mock MPEG Audio Frame Sync: 0xFF 0xFB (MPEG-1 Layer III, 128 kbps, 44.1 kHz, Stereo)
    mp3_bytes.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]);
    // Append padding bytes
    mp3_bytes.resize(mp3_bytes.len() + 4000, 0xAA);

    file.write_all(&mp3_bytes).expect("write");

    let doc = extract_audio(file.path()).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::Audio);
    assert_eq!(doc.metadata.title.as_deref(), Some("Meeting Voice Note"));
    assert_eq!(doc.metadata.author.as_deref(), Some("Faizan Nadeem"));
    assert!(doc.metadata.audio_codec.as_deref().unwrap().contains("MP3"));
    assert!(doc.chunks[0].text.contains("Title: Meeting Voice Note"));
    assert!(doc.chunks[0].text.contains("Artist: Faizan Nadeem"));
}

#[test]
fn test_extract_png_image() {
    let file = NamedTempFile::with_suffix(".png").expect("tempfile");

    // Generate a 64x32 image with distinct pattern to verify dHash
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(64, 32);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = (x * 4) as u8;
        let g = (y * 8) as u8;
        let b = 128u8;
        *pixel = Rgb([r, g, b]);
    }

    img.save(file.path()).expect("save image");

    let doc = extract_image(file.path()).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::Image);
    assert_eq!(doc.metadata.image_width, Some(64));
    assert_eq!(doc.metadata.image_height, Some(32));
    assert_eq!(doc.metadata.aspect_ratio.as_deref(), Some("2:1"));
    assert!(doc.metadata.perceptual_hash.is_some());
    let phash = doc.metadata.perceptual_hash.as_ref().unwrap();
    assert_eq!(phash.len(), 16);

    assert_eq!(doc.chunks.len(), 1);
    assert!(doc.chunks[0].text.contains("64x32"));
    assert!(doc.chunks[0].text.contains("Landscape"));
    assert!(doc.chunks[0].text.contains("Perceptual Hash:"));
}

#[test]
fn test_dhash_consistency_and_invariance() {
    let img1: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(100, 100, Rgb([200, 100, 50]));
    let dynamic1 = image::DynamicImage::ImageRgb8(img1);
    let hash1 = compute_dhash(&dynamic1);

    // Resized version of identical image should yield identical perceptual hash
    let img2: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(50, 50, Rgb([200, 100, 50]));
    let dynamic2 = image::DynamicImage::ImageRgb8(img2);
    let hash2 = compute_dhash(&dynamic2);

    assert_eq!(hash1, hash2);
}

#[test]
fn test_extract_svg_image() {
    let mut file = NamedTempFile::with_suffix(".svg").expect("tempfile");
    writeln!(
        file,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="800px" height="600px" viewBox="0 0 800 600"><circle cx="400" cy="300" r="100" fill="blue"/></svg>"#
    )
    .expect("write");

    let doc = extract_image(file.path()).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::Image);
    assert_eq!(doc.metadata.image_width, Some(800));
    assert_eq!(doc.metadata.image_height, Some(600));
    assert_eq!(doc.metadata.aspect_ratio.as_deref(), Some("4:3"));
    assert_eq!(doc.chunks.len(), 1);
    assert!(doc.chunks[0].text.contains("800x600"));
}

#[test]
fn test_extract_content_multimodal_router() {
    // 1. WAV Routing
    let wav_file = NamedTempFile::with_suffix(".wav").expect("tempfile");
    {
        let mut f = File::create(wav_file.path()).expect("create");
        f.write_all(b"RIFF\x24\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00\x44\xac\x00\x00\x88\x58\x01\x00\x02\x00\x10\x00data\x00\x00\x00\x00")
            .expect("write");
    }
    let wav_doc = extract_content(wav_file.path()).expect("extract content");
    assert_eq!(wav_doc.doc_type, DocumentType::Audio);

    // 2. SVG Routing
    let svg_file = NamedTempFile::with_suffix(".svg").expect("tempfile");
    {
        let mut f = File::create(svg_file.path()).expect("create");
        f.write_all(b"<svg width=\"100\" height=\"100\"></svg>")
            .expect("write");
    }
    let svg_doc = extract_content(svg_file.path()).expect("extract content");
    assert_eq!(svg_doc.doc_type, DocumentType::Image);
}
