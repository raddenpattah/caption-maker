use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::{CaptionError, RawToken, TranscribeOptions};

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
    if let Some(on_progress) = &options.on_progress {
        let on_progress = on_progress.clone();
        params.set_progress_callback_safe(move |progress| on_progress(progress));
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
