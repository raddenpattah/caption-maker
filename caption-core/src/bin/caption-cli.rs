use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use caption_core::{
    build_burn_args, default_ffprobe_for, extract_audio, group_words, load_wav_16k_mono,
    merge_tokens_to_words, parse_ffprobe_json, parse_progress_line, percent, render_ass,
    render_ass_with_preset, setup_work_dir, AssOptions, PresetOptions, Quality, TranscribeOptions,
    VideoGeometry, VideoInfo, WordsPerLine,
};
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(about = "Transcribe audio/video into word-timed ASS subtitles")]
struct Args {
    /// Input media (boleh dihilangkan bila --words-json + --size diberikan).
    input: Option<PathBuf>,

    #[arg(long)]
    model: Option<PathBuf>,

    #[arg(long, value_enum)]
    words: Option<WordsArg>,

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

    #[arg(long, value_enum, default_value_t = FormatArg::Ass)]
    format: FormatArg,

    // -------- Tugas 4 --------
    #[arg(long)]
    list_presets: bool,

    #[arg(long)]
    preset: Option<String>,

    #[arg(long)]
    burn: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = QualityArg::Bagus)]
    quality: QualityArg,

    #[arg(long)]
    render_frame: Option<PathBuf>,

    #[arg(long)]
    at_ms: Option<u32>,

    #[arg(long)]
    words_json: Option<PathBuf>,

    /// Ukuran WxH untuk input non-video atau render frame.
    #[arg(long)]
    size: Option<String>,

    #[arg(long)]
    ffprobe: Option<PathBuf>,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FormatArg {
    #[value(name = "ass")]
    Ass,
    #[value(name = "srt")]
    Srt,
    #[value(name = "vtt")]
    Vtt,
    #[value(name = "txt")]
    Txt,
    #[value(name = "json")]
    Json,
}

impl From<FormatArg> for caption_core::Format {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Ass => Self::Ass,
            FormatArg::Srt => Self::Srt,
            FormatArg::Vtt => Self::Vtt,
            FormatArg::Txt => Self::Txt,
            FormatArg::Json => Self::Json,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum QualityArg {
    #[value(name = "cepat")]
    Cepat,
    #[value(name = "bagus")]
    Bagus,
}

impl From<QualityArg> for Quality {
    fn from(value: QualityArg) -> Self {
        match value {
            QualityArg::Cepat => Quality::Cepat,
            QualityArg::Bagus => Quality::Bagus,
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

fn parse_size(s: &str) -> Result<VideoGeometry, String> {
    let (w, h) = s
        .split_once(['x', 'X'])
        .ok_or_else(|| format!("--size harus 'WxH', dapat '{s}'"))?;
    let width = w
        .parse::<u32>()
        .map_err(|_| format!("lebar tidak valid: {w}"))?;
    let height = h
        .parse::<u32>()
        .map_err(|_| format!("tinggi tidak valid: {h}"))?;
    if width == 0 || height == 0 {
        return Err("dimensi harus > 0".into());
    }
    Ok(VideoGeometry { width, height })
}

fn load_words_json(path: &Path) -> Result<Vec<caption_core::Word>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let words: Vec<caption_core::Word> = serde_json::from_reader(BufReader::new(file))?;
    Ok(words)
}

fn ffprobe_path(args: &Args) -> PathBuf {
    args.ffprobe
        .clone()
        .unwrap_or_else(|| default_ffprobe_for(&args.ffmpeg))
}

fn run_probe(args: &Args, input: &Path) -> Result<VideoInfo, Box<dyn std::error::Error>> {
    let ffprobe = ffprobe_path(args);
    let output = Command::new(&ffprobe)
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_streams",
            "-show_format",
            input.as_os_str().to_string_lossy().as_ref(),
        ])
        .output()?;
    if !output.status.success() {
        return Err(format!("ffprobe gagal: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    let json = String::from_utf8_lossy(&output.stdout);
    Ok(parse_ffprobe_json(&json)?)
}

fn list_presets() -> Result<(), Box<dyn std::error::Error>> {
    let presets = caption_core::builtin_presets()?;
    for p in presets {
        println!("{}\t{}\t{}", p.id, p.name, p.category.as_str());
    }
    Ok(())
}

fn run_burn(
    args: &Args,
    input: &Path,
    ass: &str,
    info: &VideoInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    let out = args.burn.as_ref().ok_or("burn path wajib")?;
    // workdir di temp
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let work = std::env::temp_dir().join(format!("caption-burn-{}-{nonce}", process::id()));
    setup_work_dir(&work, ass)?;
    let ff_args = build_burn_args(input, &work, out, args.quality.into(), info);
    let mut child = Command::new(&args.ffmpeg)
        .args(&ff_args)
        .current_dir(&work)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut last_pct: u8 = 0;
        for line in reader.lines().map_while(Result::ok) {
            if let Some(p) = parse_progress_line(&line) {
                if let Some(us) = p.out_time_us {
                    let pct = percent(us, info.duration_s);
                    if pct > last_pct {
                        last_pct = pct;
                        eprintln!("progress {pct}%");
                    }
                }
            }
        }
    }
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("ffmpeg burn-in gagal: {status}").into());
    }
    let _ = fs::remove_dir_all(&work);
    println!("burn output={}", out.display());
    Ok(())
}

fn render_frame(
    args: &Args,
    groups_for_frame: &[caption_core::Group],
    geometry: VideoGeometry,
    preset: &caption_core::StylePreset,
) -> Result<(), Box<dyn std::error::Error>> {
    let out = args
        .render_frame
        .as_ref()
        .ok_or("render-frame path wajib")?;
    let at_ms = args.at_ms.ok_or("--at-ms wajib dengan --render-frame")?;
    let ass = render_ass_with_preset(
        groups_for_frame,
        preset,
        geometry,
        &PresetOptions::default(),
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let work = std::env::temp_dir().join(format!("caption-frame-{}-{nonce}", process::id()));
    setup_work_dir(&work, &ass)?;
    let secs = at_ms as f64 / 1000.0;
    let color = format!("color=c=0x202020:s={}x{}", geometry.width, geometry.height);
    let status = Command::new(&args.ffmpeg)
        .current_dir(&work)
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            &color,
            "-vf",
            "ass=sub.ass:fontsdir=fonts",
            "-frames:v",
            "1",
            "-ss",
            &secs.to_string(),
            out.to_str().ok_or("render-frame path bukan UTF-8")?,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()?;
    let _ = fs::remove_dir_all(&work);
    if !status.success() {
        return Err(format!("ffmpeg render-frame gagal: {status}").into());
    }
    println!("frame output={}", out.display());
    Ok(())
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Jalur early exit: list presets
    if args.list_presets {
        return list_presets();
    }

    // Validasi kombinasi input
    let preset = match &args.preset {
        Some(id) => Some(caption_core::find_preset(id)?),
        None => None,
    };

    if preset.is_some() && (args.highlight || args.words.is_some()) {
        eprintln!("peringatan: --preset mengabaikan --highlight/--words");
    }

    let input = args.input.clone();
    // `geometry` dihitung lazily hanya untuk jalur preset/burn/frame.
    let compute_geometry = |args: &Args,
                            input: &Option<PathBuf>|
     -> Result<VideoGeometry, Box<dyn std::error::Error>> {
        if let Some(size_str) = &args.size {
            return parse_size(size_str).map_err(Into::into);
        }
        if let Some(inp) = input {
            let info = run_probe(args, inp)?;
            if info.is_hdr {
                eprintln!("peringatan: sumber HDR terdeteksi; tone mapping tidak dilakukan");
            }
            return Ok(VideoGeometry {
                width: info.width,
                height: info.height,
            });
        }
        Err("butuh <input> atau --size".into())
    };

    // Dapatkan words
    let mut token_count: usize = 0;
    let words: Vec<caption_core::Word> = if let Some(p) = &args.words_json {
        load_words_json(p)?
    } else {
        let input = input.as_ref().ok_or("input wajib untuk transkripsi")?;
        let model = args.model.as_ref().ok_or("--model wajib")?;
        let temp_wav = TemporaryWav::new();
        extract_audio(&args.ffmpeg, input, &temp_wav.0)?;
        let samples = load_wav_16k_mono(&temp_wav.0)?;
        let mut options = TranscribeOptions::new(model.clone());
        options.initial_prompt = args.prompt.clone();
        if let Some(threads) = args.threads {
            options.threads = threads;
        }
        let thread_count = options.threads;
        let audio_seconds = samples.len() as f64 / 16_000.0;
        let t_start = Instant::now();
        let tokens = caption_core::transcribe(&samples, &options)?;
        token_count = tokens.len();
        let t_secs = t_start.elapsed().as_secs_f64();
        if let Some(path) = &args.dump_tokens {
            let mut f = File::create(path)?;
            serde_json::to_writer_pretty(&mut f, &tokens)?;
            f.write_all(b"\n")?;
        }
        let words = merge_tokens_to_words(&tokens);
        if let Some(path) = &args.dump_words {
            let mut f = File::create(path)?;
            serde_json::to_writer_pretty(&mut f, &words)?;
            f.write_all(b"\n")?;
        }
        eprintln!(
            "audio={audio_seconds:.3}s transcribe={t_secs:.3}s rtf={:.3} threads={thread_count}",
            t_secs / audio_seconds
        );
        words
    };

    // Render frame (jika diminta)
    if args.render_frame.is_some() {
        let preset = preset.clone().ok_or("--render-frame butuh --preset")?;
        let groups_for_frame = group_words(&words, preset.words_per_line_enum()?);
        let geometry = compute_geometry(&args, &input)?;
        return render_frame(&args, &groups_for_frame, geometry, &preset);
    }

    // Render ASS / burn
    if let Some(preset) = preset {
        let groups = group_words(&words, preset.words_per_line_enum()?);
        let geometry = compute_geometry(&args, &input)?;
        let ass = render_ass_with_preset(&groups, &preset, geometry, &PresetOptions::default());
        if let Some(burn_out) = &args.burn {
            let input = input.as_ref().ok_or("burn butuh <input>")?;
            let info = run_probe(&args, input)?;
            // Tulis .ass juga bila -o diberikan
            if let Some(out) = &args.output {
                fs::write(out, &ass)?;
            }
            run_burn(&args, input, &ass, &info)?;
            println!(
                "burn tokens={token_count} words={} groups={} output={}",
                words.len(),
                groups.len(),
                burn_out.display()
            );
        } else {
            let out = args.output.unwrap_or_else(|| PathBuf::from("out.ass"));
            fs::write(&out, ass)?;
            println!(
                "words={} groups={} output={}",
                words.len(),
                groups.len(),
                out.display()
            );
        }
        return Ok(());
    }

    // Jalur lama (byte-identik): --words wajib, --highlight, --format
    let words_per_line = args.words.ok_or("--words wajib tanpa --preset")?;
    let groups = group_words(&words, words_per_line.into());
    let default_ext = match args.format {
        FormatArg::Ass => "ass",
        FormatArg::Srt => "srt",
        FormatArg::Vtt => "vtt",
        FormatArg::Txt => "txt",
        FormatArg::Json => "json",
    };
    let input = input.as_ref().ok_or("input wajib")?;
    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| input.with_extension(default_ext));
    match caption_core::Format::from(args.format) {
        caption_core::Format::Ass => {
            let opts = AssOptions {
                highlight: args.highlight,
                ..AssOptions::default()
            };
            fs::write(&output_path, render_ass(&groups, &opts))?;
        }
        caption_core::Format::Srt => fs::write(&output_path, caption_core::to_srt(&groups))?,
        caption_core::Format::Vtt => fs::write(&output_path, caption_core::to_vtt(&groups))?,
        caption_core::Format::Txt => fs::write(&output_path, caption_core::to_txt(&groups))?,
        caption_core::Format::Json => fs::write(&output_path, caption_core::to_json(&groups)?)?,
    }
    println!(
        "tokens={token_count} words={} groups={} output={}",
        words.len(),
        groups.len(),
        output_path.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run(Args::parse()) {
        eprintln!("caption-cli: {error}");
        process::exit(1);
    }
}
