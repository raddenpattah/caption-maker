use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperProgressCallback,
};

use crate::{CaptionError, RawToken, TranscribeOptions};

type ProgressCallback = Arc<dyn Fn(i32) + Send + Sync>;

unsafe extern "C" fn report_progress(
    _context: *mut whisper_rs::WhisperSysContext,
    _state: *mut whisper_rs::WhisperSysState,
    progress: i32,
    user_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }
    let callback = unsafe { &*(user_data as *const Mutex<ProgressCallback>) };
    if let Ok(callback) = callback.lock() {
        callback(progress);
    }
}

unsafe extern "C" fn abort_if_requested(user_data: *mut c_void) -> bool {
    if user_data.is_null() {
        return false;
    }
    let abort_requested = unsafe { &*(user_data as *const AtomicBool) };
    abort_requested.load(Ordering::Acquire)
}

pub fn transcribe(
    samples: &[f32],
    options: &TranscribeOptions,
) -> Result<Vec<RawToken>, CaptionError> {
    if options.threads <= 0 {
        return Err(CaptionError::InvalidThreadCount);
    }
    let model_path = options
        .model_path
        .to_str()
        .ok_or_else(|| CaptionError::InvalidModelPath(options.model_path.clone()))?;
    let context = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())?;
    let mut state = context.create_state()?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    let progress_callback = options
        .on_progress
        .as_ref()
        .map(|callback| Arc::new(Mutex::new(callback.clone())));
    params.set_language(Some(&options.language));
    params.set_translate(false);
    params.set_token_timestamps(true);
    params.set_split_on_word(true);
    params.set_n_threads(options.threads);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);
    if let Some(prompt) = &options.initial_prompt {
        params.set_initial_prompt(prompt);
    }
    if let Some(progress_callback) = &progress_callback {
        let callback: WhisperProgressCallback = Some(report_progress);
        let callback_data = Arc::as_ptr(progress_callback) as *mut c_void;
        // The Arc remains alive until `state.full` returns; this callback only reports a value.
        unsafe {
            params.set_progress_callback(callback);
            params.set_progress_callback_user_data(callback_data);
        }
    }
    if let Some(on_abort) = &options.on_abort {
        let callback_data = Arc::as_ptr(on_abort) as *mut c_void;
        // `options` owns this Arc until `state.full` returns, keeping callback data valid.
        unsafe {
            params.set_abort_callback(Some(abort_if_requested));
            params.set_abort_callback_user_data(callback_data);
        }
    }

    state.full(params, samples)?;
    let mut tokens = Vec::new();
    let token_eot = context.token_eot();

    for segment_index in 0..state.full_n_segments()? {
        for token_index in 0..state.full_n_tokens(segment_index)? {
            let bytes = state.full_get_token_bytes(segment_index, token_index)?;
            let data = state.full_get_token_data(segment_index, token_index)?;
            tokens.push(RawToken {
                text: String::from_utf8_lossy(&bytes).into_owned(),
                bytes,
                t0_ms: data.t0.saturating_mul(10),
                t1_ms: data.t1.saturating_mul(10),
                id: data.id,
                p: data.p,
                is_special: data.id >= token_eot,
            });
        }
    }

    Ok(tokens)
}
