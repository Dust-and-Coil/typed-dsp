import std/[math, options, unittest]
import typed_dsp

const
  Half = PeakAmplitude.init(0.5'f32).get()
  Quiet = LevelDbfs.init(-10.0'f32).get()
  Boost = GainDb.init(6.0'f32).get()
  Cut = GainDb.init(-6.0'f32).get()

func level(dbfs: float32): LevelDbfs =
  LevelDbfs.init(dbfs).get()

func gainDb(db: float32): GainDb =
  GainDb.init(db).get()

suite "typed_dsp":
  test "magnitudes must be finite and non-negative":
    for x in [NaN.float32, Inf.float32, (-Inf).float32, -0.25'f32]:
      check PeakAmplitude.init(x).isNone
      check LinearGain.init(x).isNone

  test "magnitudes may exceed full scale":
    check PeakAmplitude.init(1.5'f32).get().get() == 1.5'f32
    check LinearGain.init(4.0'f32).get().get() == 4.0'f32

  test "decibels reject NaN and positive infinity":
    for x in [NaN.float32, Inf.float32]:
      check LevelDbfs.init(x).isNone
      check GainDb.init(x).isNone

  test "decibels accept negative infinity as silence":
    check level((-Inf).float32) == LevelDbfs.silence()
    check GainDb.init((-Inf).float32).get() == GainDb.mute()

  test "constants are built at compile time":
    check Half.get() == 0.5'f32
    check Quiet.get() == -10.0'f32
    check Boost.get() == 6.0'f32

  test "a sample amplitude is its magnitude":
    check PeakAmplitude.fromSample(-0.5'f32).get().get() == 0.5'f32
    check PeakAmplitude.fromSample(0.0'f32).get().get() == 0.0'f32
    check PeakAmplitude.fromSample(1.5'f32).get().get() == 1.5'f32

  test "non-finite samples have no amplitude":
    for x in [NaN.float32, Inf.float32, (-Inf).float32]:
      check PeakAmplitude.fromSample(x).isNone

  test "the louder peak wins":
    check PeakAmplitude.silence().max(Half) == Half

  test "full scale is zero dBFS":
    check PeakAmplitude.init(1.0'f32).get().toDbfs() == level(0.0'f32)

  test "half amplitude is about minus six dBFS":
    check abs(Half.toDbfs().get() + 6.0206'f32) < 1.0e-3'f32

  test "silent amplitude is silent level and back":
    let silentLevel = PeakAmplitude.silence().toDbfs()
    check silentLevel == LevelDbfs.silence()
    check silentLevel.toAmplitude() == PeakAmplitude.silence()

  test "the loudest amplitude has a finite level":
    let dbfs = PeakAmplitude.init(high(float32)).get().toDbfs().get()
    check dbfs == dbfs
    check dbfs != Inf.float32
    check dbfs != (-Inf).float32

  test "the loudest level has a finite amplitude":
    check level(high(float32)).toAmplitude().get() == high(float32)

  test "the louder level wins":
    check LevelDbfs.silence().max(Quiet) == Quiet

  test "zero dB is exactly unity gain and back":
    check GainDb.zero().toLinear() == LinearGain.unity()
    check LinearGain.unity().toDb() == GainDb.zero()

  test "muting gain is silent multiplier and back":
    check GainDb.mute().toLinear() == LinearGain.silence()
    check LinearGain.silence().toDb() == GainDb.mute()

  test "any gain dB converts to a finite multiplier":
    check gainDb(high(float32)).toLinear().get() == high(float32)

  test "clamping a gain keeps it in range":
    check GainDb.mute().clamp(Cut, Boost) == Cut
    check gainDb(high(float32)).clamp(Cut, Boost) == Boost

  test "the difference of two levels is a gain":
    check level(-10.0'f32) - level(-4.0'f32) == Cut

  test "silence less silence is zero gain":
    check LevelDbfs.silence() - LevelDbfs.silence() == GainDb.zero()

  test "silence less any level is muting gain":
    check LevelDbfs.silence() - Quiet == GainDb.mute()

  test "any level less silence is the largest finite gain":
    check (Quiet - LevelDbfs.silence()).get() == high(float32)

  test "a gain moves a level":
    check Quiet + Boost == level(-4.0'f32)

  test "silence stays silent under any gain":
    check LevelDbfs.silence() + Boost == LevelDbfs.silence()
    check LevelDbfs.silence() + GainDb.mute() == LevelDbfs.silence()

  test "muting gain silences any level":
    check Quiet + GainDb.mute() == LevelDbfs.silence()

  test "gains add":
    check Boost + Cut == GainDb.zero()
    check Boost + Boost == gainDb(12.0'f32)

  test "muting gain absorbs any gain":
    check GainDb.mute() + Boost == GainDb.mute()
    check GainDb.mute() + GainDb.mute() == GainDb.mute()

  test "decibel arithmetic saturates at the largest finite value":
    let most = gainDb(high(float32))
    check (most + most).get() == high(float32)
    check (level(high(float32)) - level(-high(float32))).get() == high(float32)
    check (level(high(float32)) + most).get() == high(float32)
