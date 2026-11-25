//! Page images for the window, rendered once and kept in `Vorschau/` beside the archive (FA-10).
//! A cached image is named by the hash of its original, so it is never stale.

use std::fs;
use std::path::{Path, PathBuf};

use crate::documents::FileType;
use crate::pages::{self, PageError};
use crate::pdf::Pdfium;

pub const FOLDER: &str = "Vorschau";
/// Widths the window may ask for. Anything else is rounded up to the next, so the cache stays small.
pub const WIDTHS: [u32; 3] = [160, 640, 1280];

pub fn folder_of(database: &Path) -> PathBuf {
    database
        .parent()
        .map_or_else(|| PathBuf::from(FOLDER), |dir| dir.join(FOLDER))
}

pub fn snap_width(width: u32) -> u32 {
    WIDTHS
        .into_iter()
        .find(|candidate| *candidate >= width)
        .unwrap_or(WIDTHS[WIDTHS.len() - 1])
}

/// The page as a PNG, from the cache or rendered and then cached.
pub fn page_png(
    cache: &Path,
    original: &Path,
    file_type: FileType,
    pdfium: Option<&Pdfium>,
    page: u32,
    width: u32,
) -> Result<Vec<u8>, PageError> {
    let width = snap_width(width);
    let hash = original
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or(PageError::Unreadable)?;
    let cached = cache.join(format!("{hash}-{page}-{width}.png"));
    if let Ok(bytes) = fs::read(&cached) {
        return Ok(bytes);
    }
    let bytes = fs::read(original).map_err(|_| PageError::Unreadable)?;
    let png = match file_type {
        FileType::Pdf => {
            pages::render_pdf_page(pdfium.ok_or(PageError::Unreadable)?, &bytes, page, width)?
        }
        _ if page == 1 => pages::render_image(&bytes, width)?,
        _ => return Err(PageError::NoSuchPage),
    };
    // A cache that cannot be written costs time, never the image.
    if fs::create_dir_all(cache).is_ok() {
        let partial = cached.with_extension("part");
        if fs::write(&partial, &png).is_ok() {
            let _ = fs::rename(&partial, &cached);
        }
    }
    Ok(png)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn a_width_is_rounded_up_to_one_of_three() {
        assert_eq!(snap_width(1), 160);
        assert_eq!(snap_width(160), 160);
        assert_eq!(snap_width(161), 640);
        assert_eq!(snap_width(5000), 1280);
    }

    #[test]
    fn a_page_is_rendered_once_and_then_read_from_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir
            .path()
            .join("ab")
            .join(format!("{}.pdf", "ab".repeat(32)));
        fs::create_dir_all(original.parent().unwrap()).unwrap();
        fs::write(&original, testing::pdf(&["Rechnung"])).unwrap();
        let cache = dir.path().join(FOLDER);
        let first = page_png(
            &cache,
            &original,
            FileType::Pdf,
            Some(testing::pdfium()),
            1,
            150,
        )
        .unwrap();
        assert!(
            cache
                .join(format!("{}-1-160.png", "ab".repeat(32)))
                .exists()
        );
        fs::remove_file(&original).unwrap();
        let again = page_png(
            &cache,
            &original,
            FileType::Pdf,
            Some(testing::pdfium()),
            1,
            150,
        )
        .unwrap();
        assert_eq!(
            first, again,
            "the second call reads the cache, the original is gone"
        );
    }

    #[test]
    fn an_image_has_one_page() {
        let dir = tempfile::tempdir().unwrap();
        let png =
            pages::render_pdf_page(testing::pdfium(), &testing::pdf(&["Foto"]), 1, 300).unwrap();
        let original = dir.path().join(format!("{}.png", "cd".repeat(32)));
        fs::write(&original, png).unwrap();
        let cache = dir.path().join(FOLDER);
        assert!(page_png(&cache, &original, FileType::Png, None, 1, 160).is_ok());
        assert_eq!(
            page_png(&cache, &original, FileType::Png, None, 2, 160),
            Err(PageError::NoSuchPage)
        );
    }
}
