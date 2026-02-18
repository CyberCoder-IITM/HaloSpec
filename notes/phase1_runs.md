Run 3 times without load:

# 1st RUN

# HaloSpec Benchmark Summary

mode steps s% fail avg median p95 min max stddev throughput score
fixed_1 10 100.0% 0 5752.2 ms 5331 ms 9463 ms 3948 ms 9463 ms 1451.0 ms 11.1 tok/s 10773.9
fixed_2 10 100.0% 0 3977.7 ms 3879 ms 5036 ms 3710 ms 5036 ms 358.3 ms 16.1 tok/s 6567.4
fixed_3 10 100.0% 0 4150.7 ms 4070 ms 5215 ms 3727 ms 5215 ms 416.0 ms 15.4 tok/s 6841.4
fixed_4 10 100.0% 0 3829.8 ms 3816 ms 4064 ms 3727 ms 4064 ms 95.4 ms 16.7 tok/s 5880.9
fixed_5 10 100.0% 0 3801.8 ms 3800 ms 4260 ms 3468 ms 4260 ms 247.8 ms 16.8 tok/s 5981.4
fixed_6 10 100.0% 0 3654.6 ms 3564 ms 4033 ms 3436 ms 4033 ms 196.8 ms 17.5 tok/s 5710.5
fixed_7 10 100.0% 0 3539.7 ms 3470 ms 3740 ms 3429 ms 3740 ms 106.4 ms 18.1 tok/s 5431.0
fixed_8 10 100.0% 0 3504.8 ms 3429 ms 3695 ms 3379 ms 3695 ms 115.3 ms 18.3 tok/s 5375.4
adaptive 15 100.0% 0 3498.1 ms 3434 ms 3731 ms 3393 ms 3731 ms 114.7 ms 18.3 tok/s 5386.5

[Adaptive Behavior] draft_length changes=2 | converged_at_step=7 (k=5)

Winner (lowest SLO-aware score): fixed_8 at 5375.4

2nd RUN

==============================
HaloSpec Benchmark Summary
==============================

mode steps s% fail avg median p95 min max stddev throughput score
fixed_1 10 100.0% 0 4403.3 ms 3685 ms 8556 ms 3624 ms 8556 ms 1460.5 ms 14.5 tok/s 8973.4
fixed_2 10 100.0% 0 3699.0 ms 3645 ms 4141 ms 3601 ms 4141 ms 151.7 ms 17.3 tok/s 5799.8
fixed_3 10 100.0% 0 3697.6 ms 3658 ms 4013 ms 3591 ms 4013 ms 114.9 ms 17.3 tok/s 5727.1
fixed_4 10 100.0% 0 3630.7 ms 3615 ms 3669 ms 3593 ms 3669 ms 26.9 ms 17.6 tok/s 5470.6
fixed_5 10 100.0% 0 3585.2 ms 3506 ms 3891 ms 3368 ms 3891 ms 204.0 ms 17.9 tok/s 5571.5
fixed_6 10 100.0% 0 3470.0 ms 3388 ms 3728 ms 3364 ms 3728 ms 120.1 ms 18.4 tok/s 5358.0
fixed_7 10 100.0% 0 3530.9 ms 3421 ms 3920 ms 3366 ms 3920 ms 169.1 ms 18.1 tok/s 5524.7
fixed_8 10 100.0% 0 4019.3 ms 3916 ms 4977 ms 3480 ms 4977 ms 400.4 ms 15.9 tok/s 6587.9
adaptive 15 100.0% 0 4017.5 ms 3996 ms 4642 ms 3647 ms 4642 ms 270.6 ms 15.9 tok/s 6392.6

[Adaptive Behavior] draft_length changes=4 | converged_at_step=9 (k=5)

Winner (lowest SLO-aware score): fixed_6 at 5358.0

3rd RUN

==============================
HaloSpec Benchmark Summary
==============================

mode steps s% fail avg median p95 min max stddev throughput score
fixed_1 10 100.0% 0 4632.7 ms 4723 ms 5533 ms 3691 ms 5533 ms 625.7 ms 13.8 tok/s 7524.3
fixed_2 10 100.0% 0 3722.5 ms 3469 ms 5227 ms 3419 ms 5227 ms 525.6 ms 17.2 tok/s 6441.1
fixed_3 10 100.0% 0 3466.9 ms 3423 ms 3775 ms 3403 ms 3775 ms 105.4 ms 18.5 tok/s 5375.5
fixed_4 10 100.0% 0 3451.7 ms 3452 ms 3484 ms 3414 ms 3484 ms 21.3 ms 18.5 tok/s 5198.0
fixed_5 10 100.0% 0 3433.4 ms 3435 ms 3471 ms 3395 ms 3471 ms 20.5 ms 18.6 tok/s 5173.0
fixed_6 10 100.0% 0 3453.6 ms 3420 ms 3542 ms 3412 ms 3542 ms 43.1 ms 18.5 tok/s 5233.2
fixed_7 10 100.0% 0 3560.6 ms 3460 ms 3915 ms 3430 ms 3915 ms 171.7 ms 18.0 tok/s 5552.4
fixed_8 10 100.0% 0 3519.2 ms 3507 ms 3611 ms 3456 ms 3611 ms 41.7 ms 18.2 tok/s 5333.0
adaptive 15 100.0% 0 3714.4 ms 3603 ms 4435 ms 3304 ms 4435 ms 287.1 ms 17.2 tok/s 5989.3

[Adaptive Behavior] draft_length changes=7 | converged_at_step=7 (k=5)

Winner (lowest SLO-aware score): fixed_5 at 5173.0

Run 3 times with load:
$env:HALOSPEC_LOAD="1"
cargo run
1st RUN

==============================
HaloSpec Benchmark Summary
==============================

mode steps s% fail avg median p95 min max stddev throughput score
fixed_1 10 100.0% 0 3646.8 ms 3484 ms 4129 ms 3432 ms 4129 ms 265.1 ms 17.5 tok/s 5764.3
fixed_2 10 100.0% 0 3629.7 ms 3575 ms 3993 ms 3420 ms 3993 ms 163.7 ms 17.6 tok/s 5658.9
fixed_3 10 100.0% 0 3464.4 ms 3452 ms 3515 ms 3418 ms 3515 ms 28.5 ms 18.5 tok/s 5227.6
fixed_4 10 100.0% 0 4178.8 ms 4147 ms 5466 ms 3515 ms 5466 ms 534.0 ms 15.3 tok/s 7018.6
fixed_5 10 100.0% 0 3995.1 ms 3880 ms 4527 ms 3662 ms 4527 ms 259.8 ms 16.0 tok/s 6310.6
fixed_6 10 100.0% 0 5183.7 ms 5110 ms 6382 ms 4478 ms 6382 ms 493.5 ms 12.3 tok/s 8473.4
fixed_7 10 100.0% 0 3933.7 ms 3851 ms 4668 ms 3585 ms 4668 ms 321.8 ms 16.3 tok/s 6332.1
fixed_8 10 100.0% 0 4339.3 ms 4289 ms 4757 ms 3711 ms 4757 ms 271.6 ms 14.7 tok/s 6772.1
adaptive 15 100.0% 0 3969.5 ms 3748 ms 5260 ms 3549 ms 5260 ms 439.8 ms 16.1 tok/s 6687.4

[Adaptive Behavior] draft_length changes=2 | converged_at_step=7 (k=5)

Winner (lowest SLO-aware score): fixed_3 at 5227.6

$env:HALOSPEC_LOAD="1"
cargo run
2nd RUN

==============================
HaloSpec Benchmark Summary
==============================

mode steps s% fail avg median p95 min max stddev throughput score
fixed_1 10 100.0% 0 3648.6 ms 3644 ms 3893 ms 3491 ms 3893 ms 119.6 ms 17.5 tok/s 5619.0
fixed_2 10 100.0% 0 3529.6 ms 3475 ms 3705 ms 3416 ms 3705 ms 107.6 ms 18.1 tok/s 5403.6
fixed_3 10 100.0% 0 3767.3 ms 3621 ms 4592 ms 3552 ms 4592 ms 315.9 ms 17.0 tok/s 6126.5
fixed_4 10 100.0% 0 3893.5 ms 3560 ms 6141 ms 3518 ms 6141 ms 775.5 ms 16.4 tok/s 7119.1
fixed_5 10 100.0% 0 3445.6 ms 3387 ms 3695 ms 3344 ms 3695 ms 115.4 ms 18.6 tok/s 5316.2
fixed_6 10 100.0% 0 3684.7 ms 3676 ms 3901 ms 3434 ms 3901 ms 123.1 ms 17.4 tok/s 5659.8
fixed_7 10 100.0% 0 3867.1 ms 3724 ms 4569 ms 3521 ms 4569 ms 332.7 ms 16.5 tok/s 6218.1
fixed_8 10 100.0% 0 3727.3 ms 3660 ms 4101 ms 3541 ms 4101 ms 148.9 ms 17.2 tok/s 5807.6
adaptive 15 100.0% 0 3850.1 ms 3735 ms 4587 ms 3525 ms 4587 ms 334.2 ms 16.6 tok/s 6210.4

[Adaptive Behavior] draft_length changes=9 | no convergence within run (k=5)

Winner (lowest SLO-aware score): fixed_5 at 5316.2

$env:HALOSPEC_LOAD="1"
cargo run
3rd RUN

==============================
HaloSpec Benchmark Summary
==============================

mode steps s% fail avg median p95 min max stddev throughput score
fixed_1 10 100.0% 0 4014.6 ms 3851 ms 4520 ms 3627 ms 4520 ms 306.7 ms 15.9 tok/s 6335.9
fixed_2 10 100.0% 0 3761.8 ms 3744 ms 4187 ms 3588 ms 4187 ms 172.5 ms 17.0 tok/s 5889.8
fixed_3 10 100.0% 0 3512.6 ms 3493 ms 3707 ms 3454 ms 3707 ms 70.4 ms 18.2 tok/s 5380.2
fixed_4 10 100.0% 0 3955.2 ms 3762 ms 5140 ms 3471 ms 5140 ms 558.1 ms 16.2 tok/s 6636.8
fixed_5 10 100.0% 0 3629.4 ms 3582 ms 3886 ms 3474 ms 3886 ms 131.2 ms 17.6 tok/s 5598.6
fixed_6 10 100.0% 0 3517.3 ms 3480 ms 3661 ms 3444 ms 3661 ms 68.5 ms 18.2 tok/s 5361.5
fixed_7 10 100.0% 0 3473.6 ms 3419 ms 3784 ms 3338 ms 3784 ms 133.8 ms 18.4 tok/s 5392.4
fixed_8 10 100.0% 0 3375.3 ms 3363 ms 3458 ms 3291 ms 3458 ms 47.6 ms 19.0 tok/s 5113.8
adaptive 15 100.0% 0 3489.1 ms 3451 ms 3813 ms 3393 ms 3813 ms 114.9 ms 18.3 tok/s 5418.6

[Adaptive Behavior] draft_length changes=2 | converged_at_step=7 (k=5)

Winner (lowest SLO-aware score): fixed_8 at 5113.8
