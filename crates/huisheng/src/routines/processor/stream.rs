use std::sync::mpsc;

use cpal::{FromSample, I24, Sample, SizedSample, U24, traits::DeviceTrait};
use log::warn;

use crate::model::patch::Number;

pub fn build_stream(
    device: &cpal::Device,
    format: cpal::SampleFormat,
    config: cpal::StreamConfig,
    output_rx: mpsc::Receiver<[Number; 2]>,
) -> cpal::Stream {
    match format {
        cpal::SampleFormat::I8 => build_stream_inner::<i8>(device, config, output_rx),
        cpal::SampleFormat::I16 => build_stream_inner::<i16>(device, config, output_rx),
        cpal::SampleFormat::I24 => build_stream_inner::<I24>(device, config, output_rx),
        cpal::SampleFormat::I32 => build_stream_inner::<i32>(device, config, output_rx),
        cpal::SampleFormat::I64 => build_stream_inner::<i64>(device, config, output_rx),

        cpal::SampleFormat::U8 => build_stream_inner::<u8>(device, config, output_rx),
        cpal::SampleFormat::U16 => build_stream_inner::<u16>(device, config, output_rx),
        cpal::SampleFormat::U24 => build_stream_inner::<U24>(device, config, output_rx),
        cpal::SampleFormat::U32 => build_stream_inner::<u32>(device, config, output_rx),
        cpal::SampleFormat::U64 => build_stream_inner::<u64>(device, config, output_rx),

        cpal::SampleFormat::F32 => build_stream_inner::<f32>(device, config, output_rx),
        cpal::SampleFormat::F64 => build_stream_inner::<f64>(device, config, output_rx),

        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

fn build_stream_inner<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    output_rx: mpsc::Receiver<[Number; 2]>,
) -> cpal::Stream
where
    T: SizedSample + FromSample<Number>,
{
    device
        .build_output_stream(
            &config,
            move |output: &mut [T], _| {
                write_stream(output, &output_rx, config.channels as usize);
            },
            move |err| {
                // TODO: Handle errors properly
                warn!("{:?}", err);
            },
            None,
        )
        .unwrap()
}

fn write_stream<T>(output: &mut [T], output_rx: &mpsc::Receiver<[Number; 2]>, channels: usize)
where
    T: Sample + FromSample<Number>,
{
    for frame in output.chunks_mut(channels) {
        let [left_sample, right_sample] = output_rx.recv().unwrap();
        if channels == 2 {
            frame[0] = T::from_sample(left_sample);
            frame[1] = T::from_sample(right_sample);
        } else {
            let val = T::from_sample((left_sample + right_sample) / 2.);
            frame.iter_mut().for_each(|sample| {
                *sample = val;
            });
        }
    }
}
