# Typed DSP

Strongly typed audio units for Rust DSP code.

| Type            | Represents                                         | Silence          |
| --------------- | -------------------------------------------------- | ---------------- |
| `PeakAmplitude` | A signal's absolute amplitude, 1.0 at full scale   | `0.0`            |
| `LevelDbfs`     | A signal level relative to digital full scale      | negative infinity |
| `LinearGain`    | A multiplier applied to a signal                   | `0.0`            |
| `GainDb`        | A gain in decibels                                 | negative infinity |

None of them can hold NaN or positive infinity, and the linear ones are
never negative. `new` checks a raw number; in a `const` an invalid value is a
build error.

```rust
use typed_dsp::{LevelDbfs, PeakAmplitude};

const TARGET: LevelDbfs = LevelDbfs::new(-10.0).unwrap();

let peak = PeakAmplitude::from_sample(-0.5).unwrap().to_dbfs();
let correction = TARGET - peak;
assert!((correction.to_linear().get() - 0.63).abs() < 0.01);
```

It's `no_std`, with float math from [libm](https://crates.io/crates/libm).

## License

Licensed under [MIT license](LICENSE-MIT).
