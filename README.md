# uvfits-rip

Quick-and-dirty pulling out uvfits data into a numpy .npy file.

```shell
uvfits-rip \
    -u hyp_cal.uvfits \
    -o hyp_cal.npy \
    --num-timesteps 52 \
    --num-baselines-per-timestep 7875 \
    --num-channels 768 \
    10 20
```
