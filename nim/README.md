# Typed DSP for Nim

Strongly typed audio units for Nim DSP code. This is the Nim port of the
Rust `typed-dsp` crate in the repository root.

| Type | Represents | Silence |
| --- | --- | --- |
| `PeakAmplitude` | A signal's absolute amplitude, 1.0 at full scale | `0.0` |
| `LevelDbfs` | A signal level relative to digital full scale | negative infinity |
| `LinearGain` | A multiplier applied to a signal | `0.0` |
| `GainDb` | A gain in decibels | negative infinity |

## Why two decibel types

`LevelDbfs` is an absolute signal level. `GainDb` is a relative change in
level. The operators preserve that distinction:

```text
LevelDbfs - LevelDbfs -> GainDb
LevelDbfs + GainDb    -> LevelDbfs
GainDb    + GainDb    -> GainDb
```

Adding two levels does not compile.

All four types wrap `float32`, matching the Rust implementation. Their
representations are private. Linear values reject negative numbers and all
non-finite values; decibel values reject NaN and positive infinity while
accepting negative infinity as silence/mute.

```nim
import std/options
import typed_dsp

const Target = LevelDbfs.init(-10.0'f32).get()

let peak = PeakAmplitude.fromSample(-0.5'f32).get().toDbfs()
let correction = Target - peak

assert abs(correction.toLinear().get() - 0.63'f32) < 0.01'f32
```

Constructors return `Option[T]`, so invalid literals used with `.get()` in
a `const` fail during compile-time evaluation.

## Test

From this directory:

```sh
nimble test
```

## License

MIT, matching the Rust crate.
