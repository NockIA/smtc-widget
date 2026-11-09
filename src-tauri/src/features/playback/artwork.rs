use std::io::Cursor;
use std::sync::Mutex;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties as MediaProperties;
use windows::Storage::Streams::DataReader;

use crate::shared::smtc::{Session, active_session, source_id};

const DEFAULT_MIME: &str = "image/jpeg";

/// Côté de la miniature envoyée au front : l'image est affichée à 30 px, 96 px suffit jusqu'à 300 % de DPI.
const THUMBNAIL_SIZE_PX: u32 = 96;

/// Dernière pochette lue, avec la clé du morceau correspondant (évite de relire le flux).
static CACHE: Mutex<Option<(String, String)>> = Mutex::new(None);

/// Image telle que fournie par la source, avant réduction.
struct RawArtwork {
    bytes: Vec<u8>,
    mime: String,
}

fn cache_key(session: &Session, props: &MediaProperties) -> windows::core::Result<String> {
    Ok(format!(
        "{}|{}|{}|{}",
        source_id(session),
        props.Title()?,
        props.Artist()?,
        props.AlbumTitle()?
    ))
}

fn cached(key: &str) -> Option<String> {
    CACHE
        .lock()
        .unwrap()
        .as_ref()
        .filter(|(cached_key, _)| cached_key == key)
        .map(|(_, art)| art.clone())
}

/// `None` si la source ne fournit pas de miniature (cas courant, pas une erreur) ou si elle est vide.
fn read_artwork(props: &MediaProperties) -> windows::core::Result<Option<RawArtwork>> {
    let Ok(thumbnail) = props.Thumbnail() else {
        log::debug!("aucune miniature fournie par la source");
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

    let mime = stream.ContentType().map(|mime| mime.to_string()).unwrap_or_default();
    Ok(Some(RawArtwork { bytes, mime }))
}

/// Réduit l'image en carré (recadrage centré) et la réencode en JPEG. `None` si le format est illisible.
fn shrink(bytes: &[u8]) -> Option<Vec<u8>> {
    let thumbnail = image::load_from_memory(bytes)
        .inspect_err(|e| log::warn!("pochette illisible, image d'origine conservée : {e}"))
        .ok()?
        .resize_to_fill(THUMBNAIL_SIZE_PX, THUMBNAIL_SIZE_PX, FilterType::Triangle)
        .to_rgb8();

    let mut jpeg = Vec::new();
    DynamicImage::ImageRgb8(thumbnail)
        .write_to(&mut Cursor::new(&mut jpeg), ImageFormat::Jpeg)
        .inspect_err(|e| log::warn!("réencodage de la pochette a échoué : {e}"))
        .ok()?;
    Some(jpeg)
}

/// Miniature JPEG si l'image est décodable, sinon l'image d'origine.
fn to_data_url(artwork: RawArtwork) -> String {
    let (mime, bytes) = match shrink(&artwork.bytes) {
        Some(jpeg) => (DEFAULT_MIME.to_owned(), jpeg),
        None if artwork.mime.is_empty() => (DEFAULT_MIME.to_owned(), artwork.bytes),
        None => (artwork.mime, artwork.bytes),
    };
    format!("data:{mime};base64,{}", STANDARD.encode(bytes))
}

/// Pochette du morceau de la source choisie, en data URL. `None` si le morceau n'en a pas
/// (ou pas encore : certaines sources la fournissent après le titre).
pub fn get_album_art(selected: Option<&str>) -> windows::core::Result<Option<String>> {
    let Some(session) = active_session(selected)? else {
        return Ok(None);
    };

    let props = session.TryGetMediaPropertiesAsync()?.join()?;
    let key = cache_key(&session, &props)?;
    if let Some(art) = cached(&key) {
        return Ok(Some(art));
    }

    let Some(artwork) = read_artwork(&props)? else {
        return Ok(None);
    };
    let data_url = to_data_url(artwork);

    *CACHE.lock().unwrap() = Some((key, data_url.clone()));
    Ok(Some(data_url))
}
