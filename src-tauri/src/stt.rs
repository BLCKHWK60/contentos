// Local speech-to-text: on-device transcription via parakeet-rs (ONNX) and
// (later) whisper-rs. Models are downloaded to the app-data dir, never bundled.
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Serialize, Clone)]
pub struct ModelInfo {
    id: String,
    engine: String,
    label: String,
    size_mb: u64,
    installed: bool,
}

struct ModelSpec {
    id: &'static str,
    engine: &'static str,
    label: &'static str,
    size_mb: u64,
    // (filename-on-disk, download-url)
    files: &'static [(&'static str, &'static str)],
}

// Known downloadable models. Parakeet TDT (int8) is the default — ~670 MB,
// self-contained int8 ONNX, fast on Apple-Silicon CPU. Whisper models use
// whisper.cpp (Metal) and are single GGML .bin files.
fn catalog() -> Vec<ModelSpec> {
    vec![
        ModelSpec {
            id: "parakeet-tdt-v3-int8",
            engine: "parakeet",
            label: "Parakeet TDT 0.6B · int8 (multilingual, fast)",
            size_mb: 670,
            files: &[
                (
                    "encoder-model.int8.onnx",
                    "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/encoder-model.int8.onnx",
                ),
                (
                    "decoder_joint-model.int8.onnx",
                    "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/decoder_joint-model.int8.onnx",
                ),
                (
                    "vocab.txt",
                    "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/vocab.txt",
                ),
            ],
        },
        ModelSpec {
            id: "whisper-base-en",
            engine: "whisper",
            label: "Whisper base.en (English, fast)",
            size_mb: 148,
            files: &[(
                "ggml-base.en.bin",
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin",
            )],
        },
        ModelSpec {
            id: "whisper-small",
            engine: "whisper",
            label: "Whisper small (multilingual)",
            size_mb: 488,
            files: &[(
                "ggml-small.bin",
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
            )],
        },
    ]
}

fn models_root(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("models")
}

fn model_dir(app: &AppHandle, id: &str) -> PathBuf {
    models_root(app).join(id)
}

fn spec_installed(app: &AppHandle, spec: &ModelSpec) -> bool {
    let dir = model_dir(app, spec.id);
    spec.files.iter().all(|(f, _)| dir.join(f).exists())
}

#[tauri::command]
pub fn list_models(app: AppHandle) -> Vec<ModelInfo> {
    catalog()
        .iter()
        .map(|s| ModelInfo {
            id: s.id.to_string(),
            engine: s.engine.to_string(),
            label: s.label.to_string(),
            size_mb: s.size_mb,
            installed: spec_installed(&app, s),
        })
        .collect()
}

fn download_file(app: &AppHandle, id: &str, url: &str, dest: &Path) -> Result<(), String> {
    let resp = ureq::get(url).call().map_err(|e| e.to_string())?;
    let total: u64 = resp
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let fname = dest
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut reader = resp.into_reader();
    let mut file = fs::File::create(dest).map_err(|e| e.to_string())?;
    let mut buf = [0u8; 131072];
    let mut received: u64 = 0;
    let mut last: u64 = 0;
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        received += n as u64;
        if received - last > 2_000_000 {
            last = received;
            let _ = app.emit(
                "stt-progress",
                serde_json::json!({"id": id, "file": fname, "received": received, "total": total, "done": false}),
            );
        }
    }
    let _ = app.emit(
        "stt-progress",
        serde_json::json!({"id": id, "file": fname, "received": received, "total": total, "done": false}),
    );
    Ok(())
}

fn download_blocking(app: AppHandle, id: String) -> Result<(), String> {
    let spec = catalog()
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| "unknown model".to_string())?;
    let dir = model_dir(&app, spec.id);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (fname, url) in spec.files {
        download_file(&app, &id, url, &dir.join(fname))?;
    }
    let _ = app.emit("stt-progress", serde_json::json!({"id": id, "done": true}));
    Ok(())
}

#[tauri::command]
pub async fn download_model(app: AppHandle, id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || download_blocking(app, id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn delete_model(app: AppHandle, id: String) -> Result<(), String> {
    let dir = model_dir(&app, &id);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn read_wav_f32(path: &str) -> Result<Vec<f32>, String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => reader
            .samples::<i16>()
            .map(|s| s.map(|v| v as f32 / 32768.0).unwrap_or(0.0))
            .collect(),
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.unwrap_or(0.0))
            .collect(),
    };
    Ok(samples)
}

fn transcribe_blocking(
    app: AppHandle,
    wav_path: String,
    engine: String,
    model_id: String,
) -> Result<String, String> {
    let dir = model_dir(&app, &model_id);
    if !dir.exists() {
        return Err("model not installed".to_string());
    }
    if engine == "parakeet" {
        use parakeet_rs::{ParakeetTDT, Transcriber};
        let mut p = ParakeetTDT::from_pretrained(&dir, None).map_err(|e| e.to_string())?;
        let result = p
            .transcribe_file(Path::new(&wav_path), None)
            .map_err(|e| e.to_string())?;
        Ok(result.text)
    } else if engine == "whisper" {
        use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
        let bin = fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .find(|p| p.extension().map(|x| x == "bin").unwrap_or(false))
            .ok_or_else(|| "no whisper .bin model found".to_string())?;
        let samples = read_wav_f32(&wav_path)?;
        let ctx = WhisperContext::new_with_params(
            bin.to_string_lossy().as_ref(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| e.to_string())?;
        let mut state = ctx.create_state().map_err(|e| e.to_string())?;
        let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        state.full(params, &samples).map_err(|e| e.to_string())?;
        let n = state.full_n_segments().map_err(|e| e.to_string())?;
        let mut text = String::new();
        for i in 0..n {
            if let Ok(s) = state.full_get_segment_text(i) {
                text.push_str(&s);
            }
        }
        Ok(text.trim().to_string())
    } else {
        Err(format!("engine '{}' not supported yet", engine))
    }
}

#[tauri::command]
pub async fn transcribe(
    app: AppHandle,
    wav_path: String,
    engine: String,
    model_id: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || transcribe_blocking(app, wav_path, engine, model_id))
        .await
        .map_err(|e| e.to_string())?
}
