use cpal::{
    self, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, SampleRate, StreamConfig,
};
use crossterm;
use hound;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process,
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
    u16,
};

fn show_stream_info(s: SampleRate, b: BufferSize, c: u16, n: String) -> () {
    // Extract the numeric value from `BufferSize`
    let buffer_size = match b {
        BufferSize::Fixed(size) => size as i32,
        BufferSize::Default => -1, // Represent default with a special value
    };
    println!(
        "Device: {} \n\tSample Rate: {} Hz\n\tBuffer Size: {}\n\tNum Channels: {}",
        n, s.0, buffer_size, c
    );
}

fn save_to_wav(audio_buffer: &Vec<f32>, sample_rate: u32, num_channels: u16, path: String) {
    let spec = hound::WavSpec {
        channels: num_channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let wav_file = path.as_str();
    let mut writer = hound::WavWriter::create(wav_file, spec).unwrap();

    for &sample in audio_buffer {
        let scaled_sample = (sample * i16::MAX as f32) as i16;
        writer.write_sample(scaled_sample).unwrap();
    }
    writer.finalize().unwrap();

    println!("Audio saved: {}", wav_file);
}

fn main() {
    let (t_size, _) = crossterm::terminal::size().unwrap();
    let user_input_thread = thread::spawn(|| {
        print!("File name (no extension): ");
        io::stdout().flush().unwrap(); // Flush to ensure prompt is printed immediately

        let mut filename = String::new();
        io::stdin().read_line(&mut filename).unwrap();
        filename = filename.trim().to_string(); // Remove newline from the filename

        filename
    });

    let filename = user_input_thread.join().unwrap();

    let time_input_thread = thread::spawn(|| {
        print!("Duration of recording (seconds): ");
        io::stdout().flush().unwrap(); // Flush to ensure prompt is printed immediately

        let mut secs = String::new();
        io::stdin().read_line(&mut secs).unwrap();
        secs = secs.trim().to_string(); // Remove newline from the filename

        secs
    });

    let secs: u64 = time_input_thread.join().unwrap().parse().unwrap();

    let path: String = format!("/home/lvx/Uni/clases_aud/{}.wav", filename);

    let def_input = default_host().default_input_device().unwrap();
    let config: StreamConfig = StreamConfig::from(def_input.default_input_config().unwrap());

    let devicename = def_input.name().unwrap();

    let sample_rate = config.sample_rate;
    let buffer_size = config.buffer_size;
    let num_channels = config.channels;

    show_stream_info(sample_rate, buffer_size, num_channels, devicename);

    let audio_buffer = Arc::new(Mutex::new(Vec::new()));

    let buffer_clone = Arc::clone(&audio_buffer);
    let data_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut buffer = buffer_clone.lock().unwrap();
        let amplification_factor = 1.5;
        buffer.extend(data.iter().map(|&sample| {
            let amplified_sample = sample * amplification_factor;
            // Clamp to avoid exceeding the range of f32
            amplified_sample.clamp(-1.0, 1.0)
        }));

        let bar_levels = "▁▂▃▄▅▆▇▉";
        for batch in data.chunks(t_size as usize) {
            let mut bar_line = String::new();

            for sample in batch {
                let scaled_sample = (sample.abs() * 7.0) as usize;
                let bar = bar_levels.chars().nth(scaled_sample).unwrap_or('▁');
                bar_line.push(bar);
            }

            print!("{}\r", bar_line);
        }
    };

    let error_callback = move |err| {
        eprintln!("An error occurred on the input stream: {}", err);
    };

    let stream = def_input
        .build_input_stream(
            &config,
            data_callback,
            error_callback,
            Some(Duration::from_secs(10)),
        )
        .unwrap();

    stream.play().unwrap();
    println!("Recording...");
    std::thread::sleep(Duration::from_secs(secs));

    let record = audio_buffer.lock().unwrap();
    print!("{}\r", " ".repeat(t_size as usize));
    println!("Recorded {} samples.", &record.len());
    save_to_wav(&record, sample_rate.0, num_channels, path.clone());

    //println!("Initializing whisper transcript");
    //
    //let command = format!("whisper {} --model small --language Spanish", path);
    //let transcript = process::Command::new("sh")
    //    .arg("-c")
    //    .arg(command)
    //    .output()
    //    .expect("Transcription failed");
    //
    //if !transcript.status.success() {
    //    eprintln!(
    //        "Whisper transcription error: {}",
    //        String::from_utf8_lossy(&transcript.stderr)
    //    );
    //    process::exit(1);
    //}
    //
    //let out = String::from_utf8_lossy(&transcript.stdout);
    //println!("{}", out);
    //
    //let transcript_path = format!("/home/lvx/Uni/clases_aud/{}_transcript.txt", filename);
    //fs::write(&transcript_path, &transcript.stdout).expect("Failed to save transcript file");
    //println!("Transcript saved to {}", transcript_path);
}
