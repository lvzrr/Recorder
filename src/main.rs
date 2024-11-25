use cpal::{
    self, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, SampleRate, StreamConfig,
};
use crossterm;
use hound;
use std::{
    io::{self, Write},
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
        print!("Nombre del archivo (sin extension): ");
        io::stdout().flush().unwrap(); // Flush to ensure prompt is printed immediately

        let mut filename = String::new();
        io::stdin().read_line(&mut filename).unwrap();
        filename = filename.trim().to_string(); // Remove newline from the filename

        filename
    });

    // Wait for the user input thread to finish and get the filename
    let filename = user_input_thread.join().unwrap();

    let path: String = format!("/home/lvx/Uni/clases_aud/{}.wav", filename);
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
        buffer.extend_from_slice(data);
        let bar_levels = "▁▂▃▄▅▆▇▉";
        for batch in data.chunks(t_size as usize) {
            let mut bar_line = String::new(); // Will store the line of bars for printing

            // For each sample in the batch
            for sample in batch {
                // Scale the sample value to the bar range (0–7) based on its absolute value
                let scaled_sample = (sample.abs() * 7.0) as usize; // Normalize to the bar range
                let bar = bar_levels.chars().nth(scaled_sample).unwrap_or('▁'); // Get the corresponding bar
                bar_line.push(bar);
            }

            // Print the full line of bars
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
    println!("Recording audio for an hour...");
    std::thread::sleep(Duration::from_secs(3600));

    let record = audio_buffer.lock().unwrap();
    print!("{}\r", " ".repeat(t_size as usize));
    println!("Recorded {} samples.", &record.len());
    save_to_wav(&record, sample_rate.0, num_channels, path);
}
