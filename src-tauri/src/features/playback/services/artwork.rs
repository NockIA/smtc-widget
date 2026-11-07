use std::io::Cursor;
use std::sync::Mutex;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use windows::Storage::Streams::DataReader;

use crate::shared::smtc::{resolve_session, smtc_manager, source_id};

const DEFAULT_MIME: &str = "image/jpeg";

/// Côté de la miniature envoyée au front : l'image est affichée à 30 px, 96 px suffit jusqu'à 300 % de DPI.
const THUMBNAIL_SIZE_PX: u32 = 96;

/// Réduit l'image en carré (recadrage centré) et la réencode en JPEG. `None` si le format est illisible.
fn shrink(bytes: &[u8]) -> Option<Vec<u8>> {
    let thumbnail = image::load_from_memory(bytes)
        .ok()?
        .resize_to_fill(THUMBNAIL_SIZE_PX, THUMBNAIL_SIZE_PX, FilterType::Triangle)
        .to_rgb8();

    let mut jpeg = Vec::new();
    DynamicImage::ImageRgb8(thumbnail)
        .write_to(&mut Cursor::new(&mut jpeg), ImageFormat::Jpeg)
        .ok()?;
    Some(jpeg)
}

/// Dernière pochette lue, avec la clé du morceau correspondant (évite de relire le flux).
static CACHE: Mutex<Option<(String, String)>> = Mutex::new(None);

/// Pochette du morceau de la source choisie, en data URL. `None` si le morceau n'en a pas
/// (ou pas encore : certaines sources la fournissent après le titre).
pub fn get_album_art(selected: Option<&str>) -> windows::core::Result<Option<String>> {
    let manager = smtc_manager()?;
    let Some(session) = resolve_session(&manager, selected) else {
        return Ok(None);
    };

    let props = session.TryGetMediaPropertiesAsync()?.join()?;
    let key = format!(
        "{}|{}|{}|{}",
        source_id(&session),
        props.Title()?,
        props.Artist()?,
        props.AlbumTitle()?
    );

    if let Some((cached_key, art)) = CACHE.lock().unwrap().as_ref()
        && *cached_key == key
    {
        return Ok(Some(art.clone()));
    }

    let Ok(thumbnail) = props.Thumbnail() else {
        return Ok(None);
    };
    let stream = thumbnail.OpenReadAsync()?.join()?;
    let size = stream.Size()? as u32;
    if size == 0 {
        return Ok(None);
    }

    let reader = DataReader::CreateDataReader(&stream)?;
    reader.LoadAsync(size)?.join()?;
    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes)?;

    // Si le format n'est pas décodable, on envoie l'image d'origine.
    let (mime, bytes) = match shrink(&bytes) {
        Some(jpeg) => (DEFAULT_MIME.to_string(), jpeg),
        None => {
            let mime = stream.ContentType()?.to_string();
            (
                if mime.is_empty() {
                    DEFAULT_MIME.to_string()
                } else {
                    mime
                },
                bytes,
            )
        }
    };
    let data_url = format!("data:{mime};base64,{}", STANDARD.encode(bytes));

    *CACHE.lock().unwrap() = Some((key, data_url.clone()));
    Ok(Some(data_url))
}
