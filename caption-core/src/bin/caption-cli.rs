use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use caption_core::{
    extract_audio, group_words, load_wav_16k_mono, merge_tokens_to_words, render_ass, AssOptions,
    TranscribeOptions, WordsPerLine,
};
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(about = "Transcribe audio/video into word-timed ASS subtitles")]
struct Args {
    input: PathBuf,
    #[arg(long)]
    model: PathBuf,
    #[arg(long, value_enum)]
    words: WordsArg,
    #[arg(long)]
    highlight: bool,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(long, default_value = "ffmpeg")]
    ffmpeg: PathBuf,
    #[arg(long)]
    dump_tokens: Option<PathBuf>,
    #[arg(long)]
    dump_words: Option<PathBuf>,
    #[arg(long)]
    prompt: Option<String>,
    #[arg(long)]
    threads: Option<i32>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum WordsArg {
    #[value(name = "1")]
    One,
    #[value(name = "2")]
    Two,
    #[value(name = "3")]
    Three,
}

impl From<WordsArg> for WordsPerLine {
    fn from(value: WordsArg) -> Self {
        match value {
            WordsArg::One => Self::One,
            WordsArg::Two => Self::Two,
            WordsArg::Three => Self::Three,
        }
    }
}

struct TemporaryWav(PathBuf);

impl TemporaryWav {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self(std::env::temp_dir().join(format!("caption-maker-{}-{nonce}.wav", process::id())))
    }
}

impl Drop for TemporaryWav {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let temp_wav = TemporaryWav::new();
    extract_audio(&args.ffmpeg, &args.input, &temp_wav.0)?;
    let samples = load_wav_16k_mono(&temp_wav.0)?;
    let mut options = TranscribeOptions::new(args.model);
    options.initial_prompt = args.prompt;
    if let Some(threads) = args.threads {
        options.threads = threads;
    }
    let thread_count = options.threads;
    let audio_seconds = samples.len() as f64 / 16_000.0;
    let transcribe_started = Instant::now();
    let tokens = caption_core::transcribe(&samples, &options)?;
    let transcribe_seconds = transcribe_started.elapsed().as_secs_f64();
    if let Some(path) = args.dump_tokens {
        let mut file = File::create(path)?;
        serde_json::to_writer_pretty(&mut file, &tokens)?;
        file.write_all(b"\n")?;
    }

    let words = merge_tokens_to_words(&tokens);
    if let Some(path) = args.dump_words {
        let mut file = File::create(path)?;
        serde_json::to_writer_pretty(&mut file, &words)?;
        file.write_all(b"\n")?;
    }

    let groups = group_words(&words, args.words.into());
    let output_path = args
        .output
        .unwrap_or_else(|| args.input.with_extension("ass"));
    let ass_options = AssOptions {
        highlight: args.highlight,
        ..AssOptions::default()
    };
    let ass = render_ass(&groups, &ass_options);
    fs::write(&output_path, ass)?;

    println!(
        "tokens={} words={} groups={} output={}",
        tokens.len(),
        words.len(),
        groups.len(),
        output_path.display()
    );
    eprintln!(
        "audio={audio_seconds:.3}s transcribe={transcribe_seconds:.3}s rtf={:.3} threads={thread_count}",
        transcribe_seconds / audio_seconds
    );
    Ok(())
}

fn main() {
    if let Err(error) = run(Args::parse()) {
        eprintln!("caption-cli: {error}");
        process::exit(1);
    }
}
