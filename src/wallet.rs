use qrcode_generator::QrCodeEcc;

/// Used to create to create the QR Code that is then shown to the user. The QR Code contains the pubkey that someone will receive funds to.
pub(crate) fn create_pubkey_qrcode(pubkey: &[u8]) -> Result<String> {
    let result = qrcode_generator::to_svg_to_string(pubkey, QrCodeEcc::Low, 1024, None).unwrap();

    Ok(result)
}