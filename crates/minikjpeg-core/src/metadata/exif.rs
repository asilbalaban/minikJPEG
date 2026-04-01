/// JPEG APP segment (EXIF/IPTC/XMP/ICC) çıkarma ve geri yazma.
///
/// JPEG APP segmentleri görüntü verisiyle birlikte dosyada saklanır.
/// Bu modül, piksel verisi değiştirilirken metadata'nın korunmasını sağlar.

/// JPEG APP segment bilgisi.
#[derive(Debug, Clone)]
pub struct AppSegment {
    /// APP marker (0xE0 = APP0, 0xE1 = APP1/EXIF, 0xE2 = APP2/ICC, vs.)
    pub marker: u8,
    /// Segment verisi (marker ve uzunluk alanı hariç)
    pub data: Vec<u8>,
}

/// JPEG dosyasındaki tüm APP segmentlerini çıkarır.
/// Piksel verisi (SOF, SOS, Huffman tabloları) dokunulmadan bırakılır.
pub fn extract_app_segments(jpeg_data: &[u8]) -> Vec<AppSegment> {
    let mut segments = Vec::new();
    let mut i = 0;

    // JPEG dosyası 0xFF 0xD8 (SOI) ile başlamalı
    if jpeg_data.len() < 4 || jpeg_data[0] != 0xFF || jpeg_data[1] != 0xD8 {
        return segments;
    }
    i += 2; // SOI'yi atla

    while i + 3 < jpeg_data.len() {
        if jpeg_data[i] != 0xFF {
            break;
        }

        let marker = jpeg_data[i + 1];

        // APP0-APP15 segmentleri (0xE0-0xEF)
        let is_app = (0xE0..=0xEF).contains(&marker);
        // SOS marker'a gelince görüntü verisi başlıyor
        let is_sos = marker == 0xDA;

        if is_sos {
            break;
        }

        // Segment uzunluğu (marker sonrası, uzunluk alanı dahil)
        let seg_len = if i + 3 < jpeg_data.len() {
            ((jpeg_data[i + 2] as usize) << 8) | jpeg_data[i + 3] as usize
        } else {
            break;
        };

        if seg_len < 2 || i + 2 + seg_len > jpeg_data.len() {
            break;
        }

        if is_app {
            let data = jpeg_data[i + 4..i + 2 + seg_len].to_vec();
            segments.push(AppSegment { marker, data });
        }

        i += 2 + seg_len;
    }

    segments
}

/// Sıkıştırılmış JPEG'e APP segmentlerini geri ekler.
/// Mevcut APP segmentleri varsa kaldırılır ve yenileri SOI'den hemen sonra eklenir.
pub fn inject_app_segments(jpeg_data: &[u8], segments: &[AppSegment]) -> Vec<u8> {
    if segments.is_empty() {
        return jpeg_data.to_vec();
    }

    // SOI'den sonra, varolan APP segmentlerini atla ve geri kalanını bul
    let payload_start = find_payload_start(jpeg_data);

    let mut result = Vec::with_capacity(jpeg_data.len() + segments.iter().map(|s| s.data.len() + 4).sum::<usize>());

    // SOI
    result.push(0xFF);
    result.push(0xD8);

    // APP segmentlerini ekle
    for seg in segments {
        let seg_len = (seg.data.len() + 2) as u16;
        result.push(0xFF);
        result.push(seg.marker);
        result.push((seg_len >> 8) as u8);
        result.push(seg_len as u8);
        result.extend_from_slice(&seg.data);
    }

    // Kalan JPEG verisi (APP olmayan marker'lar ve görüntü verisi)
    result.extend_from_slice(&jpeg_data[payload_start..]);

    result
}

/// SOI sonrası, APP olmayan ilk marker'ın başlangıç konumunu bulur.
fn find_payload_start(jpeg_data: &[u8]) -> usize {
    let mut i = 2; // SOI'yi atla

    while i + 3 < jpeg_data.len() {
        if jpeg_data[i] != 0xFF {
            return i;
        }

        let marker = jpeg_data[i + 1];
        let is_app = (0xE0..=0xEF).contains(&marker);

        if !is_app {
            return i;
        }

        if i + 3 >= jpeg_data.len() {
            return i;
        }

        let seg_len = ((jpeg_data[i + 2] as usize) << 8) | jpeg_data[i + 3] as usize;
        if seg_len < 2 {
            return i + 2;
        }
        i += 2 + seg_len;
    }

    i
}

/// EXIF Orientation değerini okur (varsa).
/// Dönüş değerleri: 1=normal, 3=180°, 6=90°CW, 8=90°CCW
pub fn read_orientation(segments: &[AppSegment]) -> Option<u16> {
    // APP1 segmentini bul (EXIF)
    let app1 = segments.iter().find(|s| s.marker == 0xE1)?;

    // "Exif\0\0" başlığını kontrol et
    if app1.data.len() < 14 || &app1.data[..6] != b"Exif\0\0" {
        return None;
    }

    let tiff = &app1.data[6..];
    if tiff.len() < 8 {
        return None;
    }

    // Byte order: "II" = little-endian, "MM" = big-endian
    let little_endian = &tiff[0..2] == b"II";

    let read_u16 = |data: &[u8], offset: usize| -> Option<u16> {
        if offset + 2 > data.len() {
            return None;
        }
        if little_endian {
            Some(u16::from_le_bytes([data[offset], data[offset + 1]]))
        } else {
            Some(u16::from_be_bytes([data[offset], data[offset + 1]]))
        }
    };

    let read_u32 = |data: &[u8], offset: usize| -> Option<u32> {
        if offset + 4 > data.len() {
            return None;
        }
        if little_endian {
            Some(u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]))
        } else {
            Some(u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]))
        }
    };

    // IFD0 offset
    let ifd0_offset = read_u32(tiff, 4)? as usize;
    if ifd0_offset + 2 > tiff.len() {
        return None;
    }

    let entry_count = read_u16(tiff, ifd0_offset)? as usize;

    for e in 0..entry_count {
        let entry_offset = ifd0_offset + 2 + e * 12;
        if entry_offset + 12 > tiff.len() {
            break;
        }
        let tag = read_u16(tiff, entry_offset)?;
        if tag == 0x0112 {
            // Orientation tag
            let value = read_u16(tiff, entry_offset + 8)?;
            return Some(value);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_and_extract_roundtrip() {
        // Minimal JPEG (SOI + EOI)
        let jpeg = vec![0xFF, 0xD8, 0xFF, 0xD9];
        let segments = vec![AppSegment {
            marker: 0xE1,
            data: b"Exif\0\0test".to_vec(),
        }];

        let injected = inject_app_segments(&jpeg, &segments);
        let extracted = extract_app_segments(&injected);

        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].marker, 0xE1);
        assert_eq!(&extracted[0].data, b"Exif\0\0test");
    }
}
