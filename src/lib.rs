// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Code to handle reading from uvfits files.

use std::path::Path;

use mwalib::{fitsio::errors::check_status as fits_check_status, *};
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
    xx_and_yy: bool,
) -> Result<(), String> {
    let mut uvfits = fits_open!(&uvfits).map_err(|e| e.to_string())?;
    let _hdu = fits_open_hdu!(&mut uvfits, 0).map_err(|e| e.to_string())?;

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

    if xx_and_yy {
        // Throw away all but the first and second polarisations.
        let v = uvfits_vis
            .into_raw_vec()
            .chunks_exact(12)
            .flat_map(|c| [c[0], c[3]])
            .collect::<Vec<_>>();
        let a =
            Array4::from_shape_vec((num_timesteps, row_indices.len(), num_channels, 2), v).unwrap();
        write_npy(output, &a).map_err(|e| e.to_string())?;
    } else {
        // Throw away all but the first polarisation.
        let v = uvfits_vis
            .into_raw_vec()
            .chunks_exact(12)
            .map(|c| c[0])
            .collect::<Vec<_>>();
        let a =
            Array3::from_shape_vec((num_timesteps, row_indices.len(), num_channels), v).unwrap();
        write_npy(output, &a).map_err(|e| e.to_string())?;
    };

    Ok(())
}
