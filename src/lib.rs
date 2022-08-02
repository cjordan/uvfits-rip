// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Code to handle reading from uvfits files.

use std::path::Path;

use marlu::mwalib;
use mwalib::{
    fitsio::{errors::check_status as fits_check_status, hdu::FitsHdu, FitsFile},
    *,
};
use ndarray::prelude::*;
use ndarray_npy::write_npy;

/// Given the path to a uvfits file, read all of the first polarisations of all
/// channels of the selected rows to the output file. The number of
/// timesteps and baselines per timestep must be given.
pub fn dump_baselines<P: AsRef<Path>, P2: AsRef<Path>>(
    uvfits: P,
    row_indices: Vec<usize>,
    output: P2,
    num_timesteps: usize,
    num_baselines_per_timestep: usize,
    num_channels: usize,
) -> Result<(), String> {
    let mut uvfits = fits_open!(&uvfits).map_err(|e| e.to_string())?;
    let hdu = fits_open_hdu!(&mut uvfits, 0).map_err(|e| e.to_string())?;
    let i_bl = get_baseline_index(&mut uvfits, &hdu)?;
    println!("BASELINE index: {i_bl}");

    // Assume there are 4 polarisations.
    let mut uvfits_vis: Array3<f32> =
        Array3::zeros((num_timesteps, row_indices.len(), num_channels * 12));
    for (i_timestep, mut uvfits_vis) in uvfits_vis.outer_iter_mut().enumerate() {
        for (row, mut uvfits_vis) in row_indices.iter().zip(uvfits_vis.outer_iter_mut()) {
            unsafe {
                let mut status = 0;
                // ffgpve = fits_read_img_flt
                fitsio_sys::ffgpve(
                    uvfits.as_raw(), /* I - FITS file pointer                       */
                    (row + 1 + i_timestep * num_baselines_per_timestep)
                        .try_into()
                        .unwrap(), /* I - group to read (1 = 1st group)           */
                    1,               /* I - first vector element to read (1 = 1st)  */
                    uvfits_vis.len() as i64, /* I - number of values to read                */
                    0.0,             /* I - value for undefined pixels              */
                    uvfits_vis.as_mut_ptr(), /* O - array of values that are returned       */
                    &mut 0,          /* O - set to 1 if any values are null; else 0 */
                    &mut status,     /* IO - error status                           */
                );
                fits_check_status(status).map_err(|e| e.to_string())?;
            }
        }
    }

    // Throw away all but the first polarisation.
    write_npy(output, &uvfits_vis.slice(s![.., .., 0])).map_err(|e| e.to_string())?;

    Ok(())
}

fn get_baseline_index(uvfits: &mut FitsFile, hdu: &FitsHdu) -> Result<u8, String> {
    // Accumulate the "PTYPE" keys.
    let mut ptypes = Vec::with_capacity(12);
    for i in 1.. {
        let ptype: Option<String> = get_optional_fits_key!(uvfits, hdu, &format!("PTYPE{}", i))
            .map_err(|e| format!("cfitsio error: {e}"))?;
        match ptype {
            Some(ptype) => ptypes.push(ptype),

            // We've found the last PTYPE.
            None => break,
        }
    }

    // We only care about the baseline index.
    let mut baseline_index = None;

    for (i, key) in ptypes.into_iter().enumerate() {
        let ii = (i + 1) as u8;
        match key.as_ref() {
            "BASELINE" => {
                if baseline_index.is_none() {
                    baseline_index = Some(ii);
                    break;
                }
            }
            _ => (),
        }
    }

    baseline_index.ok_or(format!("Did not find PTYPE index for BASELINE"))
}
