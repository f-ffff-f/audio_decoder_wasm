use wasm_bindgen::prelude::*;
use hound::WavReader;

#[wasm_bindgen]
pub fn decode_wav(buffer: &[u8]) -> Result<Vec<f32>, JsValue> {
    let mut reader = WavReader::new(buffer).map_err(|e| JsValue::from_str(&format!("Failed to read WAV: {}", e)))?;
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();
    Ok(samples)
}