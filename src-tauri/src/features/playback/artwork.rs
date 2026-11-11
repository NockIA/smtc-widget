use std::sync::Mutex;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use windows::Graphics::Imaging::{
    BitmapAlphaMode, BitmapBounds, BitmapDecoder, BitmapEncoder, BitmapInterpolationMode, BitmapPixelFormat,
    BitmapTransform, ColorManagementMode, ExifOrientationMode,
};
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties as MediaProperties;
use windows::Storage::Streams::{
    DataReader, IRandomAccessStream, IRandomAccessStreamWithContentType, InMemoryRandomAccessStream,
};
use windows::core::Interface;

use crate::shared::smtc::{Session, active_session, source_id};

const DEFAULT_MIME: &str = "image/jpeg";

/// Dimension (en pixels) de la miniature carrée envoyée au front. Affichée en 30 px CSS,
/// elle reste nette jusqu'à 96 px à l'écran (par exemple 300 % de DPI avec le widget à taille normale).
const THUMBNAIL_SIZE_PX: u32 = 96;

/// Dernière pochette lue, avec la clé du morceau correspondant (évite de relire le flux).
static CACHE: Mutex<Option<(String, String)>> = Mutex::new(None);

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
fn open_artwork(props: &MediaProperties) -> windows::core::Result<Option<IRandomAccessStreamWithContentType>> {
    let Ok(thumbnail) = props.Thumbnail() else {
        log::debug!("aucune miniature fournie par la source");
        return Ok(None);
    };
    let stream = thumbnail.OpenReadAsync()?.join()?;
    Ok((stream.Size()? > 0).then_some(stream))
}

fn read_all(stream: &IRandomAccessStream) -> windows::core::Result<Vec<u8>> {
    let size = stream.Size()? as u32;
    stream.Seek(0)?;
    let reader = DataReader::CreateDataReader(stream)?;
    reader.LoadAsync(size)?.join()?;
    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes)?;
    Ok(bytes)
}

/// Les pochettes fournies par les sources sont souvent très grandes (plusieurs centaines de ko, voire des Mo),
/// alors que le widget les affiche en quelques dizaines de pixels : on les réduit pour alléger ce qui passe au front.
/// L'image est mise à l'échelle puis recadrée au centre en carré, et réencodée en JPEG.
fn shrink(source: &IRandomAccessStream) -> windows::core::Result<Vec<u8>> {
    source.Seek(0)?;
    let decoder = BitmapDecoder::CreateAsync(source)?.join()?;

    let (width, height) = (decoder.PixelWidth()?, decoder.PixelHeight()?);
    let scale = f64::from(THUMBNAIL_SIZE_PX) / f64::from(width.min(height).max(1));
    let scaled = |side: u32| ((f64::from(side) * scale).round() as u32).max(THUMBNAIL_SIZE_PX);
    let (scaled_width, scaled_height) = (scaled(width), scaled(height));

    let transform = BitmapTransform::new()?;
    transform.SetInterpolationMode(BitmapInterpolationMode::Fant)?;
    transform.SetScaledWidth(scaled_width)?;
    transform.SetScaledHeight(scaled_height)?;
    transform.SetBounds(BitmapBounds {
        X: (scaled_width - THUMBNAIL_SIZE_PX) / 2,
        Y: (scaled_height - THUMBNAIL_SIZE_PX) / 2,
        Width: THUMBNAIL_SIZE_PX,
        Height: THUMBNAIL_SIZE_PX,
    })?;
    let pixels = decoder
        .GetPixelDataTransformedAsync(
            BitmapPixelFormat::Bgra8,
            BitmapAlphaMode::Ignore,
            &transform,
            ExifOrientationMode::IgnoreExifOrientation,
            ColorManagementMode::DoNotColorManage,
        )?
        .join()?
        .DetachPixelData()?;

    let output = InMemoryRandomAccessStream::new()?;
    let encoder = BitmapEncoder::CreateAsync(BitmapEncoder::JpegEncoderId()?, &output)?.join()?;
    encoder.SetPixelData(
        BitmapPixelFormat::Bgra8,
        BitmapAlphaMode::Ignore,
        THUMBNAIL_SIZE_PX,
        THUMBNAIL_SIZE_PX,
        96.0,
        96.0,
        &pixels,
    )?;
    encoder.FlushAsync()?.join()?;
    read_all(&output.cast()?)
}

/// Miniature JPEG si l'image est décodable, sinon l'image d'origine.
fn to_data_url(artwork: &IRandomAccessStreamWithContentType) -> windows::core::Result<String> {
    let stream: IRandomAccessStream = artwork.cast()?;
    let (mime, bytes) = match shrink(&stream) {
        Ok(jpeg) => (DEFAULT_MIME.to_owned(), jpeg),
        Err(e) => {
            log::warn!("réduction de la pochette a échoué, image d'origine conservée : {e}");
            let mime = artwork.ContentType().map(|mime| mime.to_string()).unwrap_or_default();
            let mime = if mime.is_empty() { DEFAULT_MIME.to_owned() } else { mime };
            (mime, read_all(&stream)?)
        }
    };
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}

/// Pochette du morceau de la source choisie, en data URL.
/// `None` si le morceau n'en a pas (ou pas encore : certaines sources la fournissent après le titre).
pub fn get_album_art(selected: Option<&str>) -> windows::core::Result<Option<String>> {
    let Some(session) = active_session(selected)? else {
        return Ok(None);
    };

    let props = session.TryGetMediaPropertiesAsync()?.join()?;
    let key = cache_key(&session, &props)?;
    if let Some(art) = cached(&key) {
        return Ok(Some(art));
    }

    let Some(artwork) = open_artwork(&props)? else {
        return Ok(None);
    };
    let data_url = to_data_url(&artwork)?;

    *CACHE.lock().unwrap() = Some((key, data_url.clone()));
    Ok(Some(data_url))
}
