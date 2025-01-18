use wasm_bindgen::prelude::*;
use js_sys::Float32Array;
use std::io::Cursor;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

#[wasm_bindgen]
pub fn decode_audio_data(data: &[u8]) -> Result<Float32Array, JsValue> {
    // 데이터를 복사하여 'static 수명 보장
    let data_owned = data.to_vec();
    let cursor = Cursor::new(data_owned);

    // MediaSourceStream 생성
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    // 파일 확장자 힌트 제공 (선택 사항)
    let mut hint = Hint::new();

    // 디코딩 옵션 설정
    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions::default();

    // 포맷 분석
    let probed = get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| JsValue::from_str(&format!("Unsupported format: {}", e)))?;

    let mut format = probed.format;

    // 첫 번째 오디오 트랙 찾기
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| JsValue::from_str("No supported audio tracks found"))?;

    let track_id = track.id;

    // 디코더 생성
    let dec_opts = DecoderOptions::default();
    let mut decoder = get_codecs()
        .make(&track.codec_params, &dec_opts)
        .map_err(|e| JsValue::from_str(&format!("Unsupported codec: {}", e)))?;

    // PCM 샘플 저장
    let mut samples: Vec<f32> = Vec::new();

    // 디코딩 루프
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::IoError(_)) => break, // 파일 끝
            Err(Error::ResetRequired) => unimplemented!(),
            Err(err) => return Err(JsValue::from_str(&format!("Error reading packet: {}", err))),
        };

        // 선택된 트랙의 패킷이 아니면 건너뜀
        if packet.track_id() != track_id {
            continue;
        }

        // 패킷 디코딩
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
                sample_buf.copy_interleaved_ref(decoded);
                samples.extend_from_slice(sample_buf.samples());
            }
            Err(Error::IoError(_)) => continue,    // 디코딩 오류 발생 시 건너뜀
            Err(Error::DecodeError(_)) => continue, // 잘못된 데이터 건너뜀
            Err(err) => return Err(JsValue::from_str(&format!("Decoding error: {}", err))),
        }
    }

    // Float32Array로 변환하여 반환
    Ok(Float32Array::from(samples.as_slice()))
}