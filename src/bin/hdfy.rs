use std::io:: Result;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use std::fs::read_dir;

use clap::Parser;
use h5rio::{ArrayHdf5Writer, TableHdf5Writer, write_chunked_array};
use hdf5_metno as hdf5;
use ndarray::Array;

use quack::progress::MaybeProgressBar;
use quack::io::read_daq_file;
use quack::io::read_waveform_length;
use quack::io::hdf5_types::DaqEventMeta;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CLI {

    #[arg(short, long, required=true)]
    input: String,

    #[arg(short, long, required=true)]
    output: String,

    #[arg(short, long, num_args = 1.., required=true)]
    channels: Vec<usize>,

    #[arg(long, default_value_t=false)]
    overwrite: bool,

    #[arg(long, default_value_t=false)]
    batch: bool,

}

pub fn main() -> Result<()> {
    let timer = Instant::now();

    let args = CLI::parse();
    let input = Path::new(&args.input);

    if !input.exists() {
        panic!("{}", &format!("Input folder {input:?} does not exist."));
    }

    let output = Path::new(&args.output);
    if output.exists() && !args.overwrite {
        panic!("Output file already exists. Use --overwrite to overwrite it.");
    }

    let mut input_files = read_dir(input).unwrap()
                                         .map(|item| item.expect("Error while globbing input path").path())
                                         .filter(|path| path.is_file())
                                         .map(|path| path.to_str()
                                                        .expect("Could not parse file name")
                                                        .to_owned()
                                         )
                                         .collect::<Vec<_>>();
    input_files.sort();
    let n_files = input_files.len();

    if n_files == 0 {
        panic!("Input folder is empty.");
    }

    let ofile  = hdf5::File::create(&output.to_str().unwrap()).unwrap();
    let ofile  = Rc::new(ofile);

    let nsamples  = read_waveform_length(input_files.first().unwrap());
    let nchannels = args.channels.len();

    let evt_writer = TableHdf5Writer::<DaqEventMeta>::new(Rc::clone(&ofile), "/events"   , 1024                           ).unwrap();
    let  wf_writer = ArrayHdf5Writer::<         f32>::new(Rc::clone(&ofile), "/waveforms",  512, vec![nchannels, nsamples]).unwrap();

    println!("Initialization time: {:?}", timer.elapsed().as_secs_f64());

    let pb = MaybeProgressBar::new(n_files, args.batch);

    let timer = Instant::now();

    let mut sampling_time = 0f32;
    for (i, filename) in input_files.into_iter().enumerate() {
        let events = read_daq_file(&filename, &args.channels)?;
        if i==0 {
            sampling_time = events.first().unwrap().sampling;
        }
        for event in events {
            evt_writer.write(DaqEventMeta{event: event.number, timestamp: event.time})?;
             wf_writer.write(event.waveforms)?;
        }
        pb.inc(1);
    }
    evt_writer.flush()?;
     wf_writer.flush()?;

    pb.finish();

    let time = (0..nsamples).map(|k| k as f32 * sampling_time)
                            .collect::<Vec<_>>();
    write_chunked_array(Rc::clone(&ofile), "/time"    , vec![nsamples ], &Array::from_vec(time))?;
    write_chunked_array(Rc::clone(&ofile), "/channels", vec![nchannels], &Array::from_vec(args.channels))?;

    let exe_time = timer.elapsed().as_secs_f64();
    println!( "Execution time for {} files: {:.2} s => {:.1} files/s or {:.8} s/file"
            , n_files
            , exe_time
            , n_files as f64 / exe_time
            , exe_time / n_files as f64
            );
    Ok(())
}
