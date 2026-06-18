use std::io::Result;
use std::io::{BufRead, BufReader};
use std::fs::{read_to_string, File};
use std::str::FromStr;

use ndarray::Array2;

//use rayon::prelude::*;

pub struct DaqEvent {
    pub number   : u32,
    pub time     : u64,
    pub sampling : f32,
    pub waveforms: Array2<f32>,
}

fn get_token<T: FromStr>(line: &str, index: usize, label: &str) -> T {
    line.trim()
        .split_whitespace()
        .nth(index)
        .and_then(|x| x.parse().ok())
        .expect(&format!("Could not parse {label}"))
}

fn get_channels(line: &str, channels: &Vec<usize>) -> impl Iterator<Item=f32> {
    line.trim()
        .split_whitespace()
        .skip(1)
        .enumerate()
        .filter(|(i,_)| channels.contains(i))
        .map(|t| t.1)
        .map(|token| token.parse().expect(&format!("Could not parse token: {token}")))
}

pub fn read_waveform_length(filename: &str) -> usize {
    let reader = BufReader::new(File::open(filename).expect("Could not open first file"));
    for line in reader.lines() {
        let line = line.expect("Failure reading file header");
        if line.starts_with("Samples") {
            return line.trim()
                       .split_whitespace()
                       .nth(1)
                       .and_then(|x| x.parse().ok())
                       .expect("Could not read waveform length");
        }
    }
    unreachable!();
}

pub fn read_event(chunk: &str, channels: &Vec<usize>) -> DaqEvent {
    let mut meta : Vec<&str> = chunk.split("\n").collect();
    let     waves            = meta.split_off(5);

    let nchannels = channels.len();
    let number    = get_token::<u32>  (&meta[0], 0, "event number");
    let time      = get_token::<u64>  (&meta[1], 1, "time stamp");
    let nsamples  = get_token::<usize>(&meta[2], 1, "number of samples");
    let sampling  = get_token::<f32>  (&meta[3], 3, "sampling time");
    let waveforms = Array2::from_shape_vec((nsamples, nchannels),
        waves.into_iter()
             .flat_map(|line| get_channels(&line, &channels))
             .collect::<Vec<f32>>()
    ).expect("Could not create waveform array");

    let waveforms = waveforms.reversed_axes();
    DaqEvent{number, time, sampling, waveforms}
}

pub fn read_daq_file(filename: &str, channels: &Vec<usize>) -> Result<Vec<DaqEvent>> {
    Ok(
        read_to_string(filename)?
            .split("Event n. ")
            .skip(1)
            // .par_bridge()
            // .into_par_iter()
            .map(|chunk| read_event(chunk, channels))
            .collect()
    )
}

#[cfg(test)]
mod tests {
    use super::{get_channels, get_token, read_daq_file, read_event, read_waveform_length};

    const TINY_DAQ: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/tiny.daq");
    const TINY_EVENT: &str = "\
7
Timestamp 123
Samples 2
Sampling time is 0.5
Header ignored
0 1.0 2.0 3.0
1 4.0 5.0 6.0
";

    #[test]
    fn gets_token_by_index() {
        assert_eq!(get_token::<u32>("Samples 2", 1, "number of samples"), 2);
    }

    #[test]
    fn gets_selected_channels_from_waveform_line() {
        let values = get_channels("0 1.0 2.0 3.0", &vec![0, 2]).collect::<Vec<_>>();

        assert_eq!(values, vec![1.0, 3.0]);
    }

    #[test]
    fn reads_waveform_length_from_header() {
        assert_eq!(read_waveform_length(TINY_DAQ), 2);
    }

    #[test]
    fn reads_event_chunk() {
        let event = read_event(TINY_EVENT, &vec![1]);

        assert_eq!(event.number, 7);
        assert_eq!(event.time, 123);
        assert_eq!(event.sampling, 0.5);
        assert_eq!(event.waveforms.shape(), &[1, 2]);
        assert_eq!(event.waveforms[[0, 0]], 2.0);
        assert_eq!(event.waveforms[[0, 1]], 5.0);
    }

    #[test]
    fn reads_selected_channels_from_event_file() {
        let events = read_daq_file(TINY_DAQ, &vec![0, 2]).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].number, 7);
        assert_eq!(events[0].time, 123);
        assert_eq!(events[0].sampling, 0.5);
        assert_eq!(events[0].waveforms.shape(), &[2, 2]);
        assert_eq!(events[0].waveforms[[0, 0]], 1.0);
        assert_eq!(events[0].waveforms[[0, 1]], 4.0);
        assert_eq!(events[0].waveforms[[1, 0]], 3.0);
        assert_eq!(events[0].waveforms[[1, 1]], 6.0);
    }
}
