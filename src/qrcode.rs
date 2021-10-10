use ::qrcode::{QrCode, types::QrError};
use ::image::{DynamicImage, ImageError, Luma};

#[derive(Debug)]
pub enum Error {
    QrError(QrError),
    ImageError(ImageError)
}
impl From<QrError> for Error {
    fn from(e: QrError) -> Self {
        Self::QrError(e)
    }
}
impl From<ImageError> for Error {
    fn from(e: ImageError) -> Self {
        Self::ImageError(e)
    }
}

pub fn qr<D: AsRef<[u8]>>(data: D) -> Result<DynamicImage, Error> {
    let code = QrCode::new(data)?;
    let image = code.render::<Luma<u8>>().build();
    Ok(DynamicImage::ImageLuma8(image))
}