use anyhow::Result;

#[cfg(target_os = "windows")]
use std::{ptr, slice};
#[cfg(target_os = "windows")]
use windows::{
    Graphics::Imaging::{BitmapBufferAccessMode, BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Win32::System::WinRT::IMemoryBufferByteAccess,
    core::ComInterface,
};

pub async fn extract_text_from_image(bytes: &[u8]) -> Result<Option<String>> {
    #[cfg(target_os = "windows")]
    {
        let image = match image::load_from_memory(bytes) {
            Ok(image) => image,
            Err(_) => return Ok(None),
        };
        let rgba = image.to_rgba8();
        let width = rgba.width() as i32;
        let height = rgba.height() as i32;
        if width <= 0 || height <= 0 {
            return Ok(None);
        }

        let bitmap = SoftwareBitmap::Create(BitmapPixelFormat::Rgba8, width, height)?;
        {
            let buffer = bitmap.LockBuffer(BitmapBufferAccessMode::Write)?;
            let reference = buffer.CreateReference()?;
            let byte_access: IMemoryBufferByteAccess = reference.cast()?;
            let mut data = ptr::null_mut();
            let mut capacity = 0u32;
            unsafe {
                byte_access.GetBuffer(&mut data, &mut capacity)?;
            }
            let dest = unsafe { slice::from_raw_parts_mut(data, capacity as usize) };
            let src = rgba.as_raw();
            let copy_len = std::cmp::min(dest.len(), src.len());
            dest[..copy_len].copy_from_slice(&src[..copy_len]);
        }

        let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;
        let result = engine.RecognizeAsync(&bitmap)?.await?;
        let text = result.Text()?.to_string();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed.to_string()))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = bytes;
        Ok(None)
    }
}
