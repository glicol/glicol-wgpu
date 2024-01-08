use anyhow::Result;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SizedSample};

const RB_SIZE: usize = 200;
const BLOCK_SIZE: usize = 128;

pub fn run_audio<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    options: (Arc<Mutex<String>>, Arc<AtomicBool>),
    // options: (
    //     Arc<AtomicPtr<f32>>,
    //     Arc<AtomicPtr<f32>>,
    //     Arc<AtomicUsize>,
    //     Arc<AtomicPtr<f32>>,
    //     Arc<AtomicPtr<f32>>,
    //     Arc<AtomicUsize>,
    //     String,
    //     f32,
    //     Arc<AtomicU32>,
    // ),
) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    tracing::warn!("run_audio");
    let sr = config.sample_rate.0 as usize;
    let bpm = 120.;
    let mut engine = glicol::Engine::<BLOCK_SIZE>::new();
    // let mut code = String::from("o: sin 220");
    // let ptr = unsafe { code.as_bytes_mut().as_mut_ptr() };
    // let code_ptr = Arc::new(AtomicPtr::<u8>::new(ptr));
    // let code_len = Arc::new(AtomicUsize::new(code.len()));
    // let has_update = Arc::new(AtomicBool::new(true));
    let code = Arc::clone(&options.0);
    let code_clone = Arc::clone(&options.0);
    let has_update = Arc::clone(&options.1);

    // let _code_ptr = Arc::clone(&code_ptr);
    // let _code_len = Arc::clone(&code_len);
    // let _has_update = Arc::clone(&has_update);
    let channels = 2 as usize; //config.channels as usize;
    engine.set_sr(sr);
    engine.set_bpm(bpm);

    let mut prev_block: [glicol_synth::Buffer<BLOCK_SIZE>; 2] = [glicol_synth::Buffer::SILENT; 2];

    let ptr = prev_block.as_mut_ptr();
    let prev_block_ptr = Arc::new(AtomicPtr::<glicol_synth::Buffer<BLOCK_SIZE>>::new(ptr));
    let prev_block_len = Arc::new(AtomicUsize::new(prev_block.len()));

    let mut prev_block_pos: usize = BLOCK_SIZE;

    // let mut sample_clock = 0f32;
    // let mut next_value = move || {
    //     sample_clock = (sample_clock + 1.0) % sr as f32;
    //     (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sr as f32).sin()
    // };

    let err_fn = |err| tracing::error!("an error occurred on stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            // write_data(data, channels, &mut next_value);
            // tracing::warn!("data len: {}", data.len());
            if has_update.load(Ordering::Acquire) {
                // let ptr = _code_ptr.load(Ordering::Acquire);
                // let len = _code_len.load(Ordering::Acquire);
                // let encoded: &[u8] = unsafe { std::slice::from_raw_parts(ptr, len) };
                // let code = std::str::from_utf8(encoded.clone()).unwrap().to_owned();
                let code = code_clone.lock().unwrap().as_str().to_owned();
                engine.update_with_code(&code);
                has_update.store(false, Ordering::Release);
            };
            let block_step = data.len() / channels;
            let mut write_samples =
                |block: &[glicol_synth::Buffer<BLOCK_SIZE>], sample_i: usize, i: usize| {
                    for chan in 0..channels {
                        let value: T = T::from_sample(block[chan][i]);
                        data[sample_i * channels + chan] = value;
                    }
                };
            let ptr = prev_block_ptr.load(Ordering::Acquire);
            let len = prev_block_len.load(Ordering::Acquire);
            let prev_block: &mut [glicol_synth::Buffer<BLOCK_SIZE>] =
                unsafe { std::slice::from_raw_parts_mut(ptr, len) };

            let mut writes = 0;

            for i in prev_block_pos..BLOCK_SIZE {
                write_samples(prev_block, writes, i);
                writes += 1;
            }

            prev_block_pos = BLOCK_SIZE;
            while writes < block_step {
                let (block, _err_msg) = engine.next_block(vec![]);
                if writes + BLOCK_SIZE <= block_step {
                    for i in 0..BLOCK_SIZE {
                        write_samples(block, writes, i);
                        writes += 1;
                    }
                } else {
                    let e = block_step - writes;
                    for i in 0..e {
                        write_samples(block, writes, i);
                        writes += 1;
                    }
                    let mut i = 0;
                    for buffer in prev_block.iter_mut() {
                        buffer.copy_from_slice(&block[i]);
                        i += 1;
                    }
                    prev_block_pos = e;
                    break;
                }
            }
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_secs(3600));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
