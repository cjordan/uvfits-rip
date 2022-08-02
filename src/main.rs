// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use clap::Parser;
use uvfits_rip::dump_baselines;

#[derive(clap::Parser)]
struct Args {
    /// Path to the uvfits file to read
    #[clap(short, long)]
    uvfits: PathBuf,

    /// Path to the output npy file
    #[clap(short, long)]
    output: PathBuf,

    /// The number of timesteps in the uvfits file
    #[clap(long)]
    num_timesteps: usize,

    /// The number of baselines per timestep in the uvfits file
    #[clap(long)]
    num_baselines_per_timestep: usize,

    /// The number of channels per baseline in the uvfits file
    #[clap(long)]
    num_channels: usize,

    #[clap(name = "ROW INDICES")]
    row_indices: Vec<usize>,
}

fn main() {
    let args = Args::parse();

    if args.row_indices.is_empty() {
        eprintln!("No row indices given!");
        std::process::exit(1);
    }

    dump_baselines(
        args.uvfits,
        args.row_indices,
        args.output,
        args.num_timesteps,
        args.num_baselines_per_timestep,
        args.num_channels,
    )
    .expect("uh oh")
}
