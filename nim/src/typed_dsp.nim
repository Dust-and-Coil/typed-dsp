## Strongly typed audio units for Nim DSP code.
##
## Public value types keep their float32 representation private. Construct
## them through the checked init functions below.

import std/[math, options]

type
  PeakAmplitude* = object
    value: float32
  LevelDbfs* = object
    value: float32
  LinearGain* = object
    value: float32
  GainDb* = object
    value: float32

func isMagnitude(x: float32): bool {.inline.} =
  x == x and x != Inf.float32 and x != (-Inf).float32 and x >= 0.0'f32

func isDecibels(x: float32): bool {.inline.} =
  x == x and x != Inf.float32

func init*(T: typedesc[PeakAmplitude], amplitude: float32): Option[PeakAmplitude] =
  if isMagnitude(amplitude): some(PeakAmplitude(value: amplitude))
  else: none(PeakAmplitude)

func fromSample*(T: typedesc[PeakAmplitude], sample: float32): Option[PeakAmplitude] =
  PeakAmplitude.init(abs(sample))

func init*(T: typedesc[LevelDbfs], dbfs: float32): Option[LevelDbfs] =
  if isDecibels(dbfs): some(LevelDbfs(value: dbfs))
  else: none(LevelDbfs)

func init*(T: typedesc[LinearGain], multiplier: float32): Option[LinearGain] =
  if isMagnitude(multiplier): some(LinearGain(value: multiplier))
  else: none(LinearGain)

func init*(T: typedesc[GainDb], db: float32): Option[GainDb] =
  if isDecibels(db): some(GainDb(value: db))
  else: none(GainDb)

func silence*(T: typedesc[PeakAmplitude]): PeakAmplitude = PeakAmplitude(value: 0.0'f32)
func silence*(T: typedesc[LevelDbfs]): LevelDbfs = LevelDbfs(value: (-Inf).float32)
func silence*(T: typedesc[LinearGain]): LinearGain = LinearGain(value: 0.0'f32)
func unity*(T: typedesc[LinearGain]): LinearGain = LinearGain(value: 1.0'f32)
func zero*(T: typedesc[GainDb]): GainDb = GainDb(value: 0.0'f32)
func mute*(T: typedesc[GainDb]): GainDb = GainDb(value: (-Inf).float32)

func get*(x: PeakAmplitude): float32 {.inline.} = x.value
func get*(x: LevelDbfs): float32 {.inline.} = x.value
func get*(x: LinearGain): float32 {.inline.} = x.value
func get*(x: GainDb): float32 {.inline.} = x.value

func `==`*(a, b: PeakAmplitude): bool {.inline.} = a.value == b.value
func `==`*(a, b: LevelDbfs): bool {.inline.} = a.value == b.value
func `==`*(a, b: LinearGain): bool {.inline.} = a.value == b.value
func `==`*(a, b: GainDb): bool {.inline.} = a.value == b.value

func `<`*(a, b: PeakAmplitude): bool {.inline.} = a.value < b.value
func `<`*(a, b: LevelDbfs): bool {.inline.} = a.value < b.value
func `<`*(a, b: LinearGain): bool {.inline.} = a.value < b.value
func `<`*(a, b: GainDb): bool {.inline.} = a.value < b.value

func `<=`*(a, b: PeakAmplitude): bool {.inline.} = a.value <= b.value
func `<=`*(a, b: LevelDbfs): bool {.inline.} = a.value <= b.value
func `<=`*(a, b: LinearGain): bool {.inline.} = a.value <= b.value
func `<=`*(a, b: GainDb): bool {.inline.} = a.value <= b.value

func decibelSum(a, b: float32): float32 {.inline.} =
  let sum = a + b
  if sum > high(float32): high(float32) else: sum

func decibelDifference(a, b: float32): float32 {.inline.} =
  if a == b:
    0.0'f32
  else:
    let difference = a - b
    if difference > high(float32): high(float32) else: difference

func amplitudeToDb(amplitude: float32): float32 {.inline.} =
  float32(20.0 * log10(float64(amplitude)))

func dbToAmplitude(db: float32): float32 =
  if db == (-Inf).float32:
    return 0.0'f32
  let maxAmplitude = float64(high(float32))
  let maxDb = 20.0 * log10(maxAmplitude)
  if float64(db) >= maxDb:
    return high(float32)
  float32(pow(10.0, float64(db) * 0.05))

func toDbfs*(x: PeakAmplitude): LevelDbfs =
  LevelDbfs(value: amplitudeToDb(x.value))

func toAmplitude*(x: LevelDbfs): PeakAmplitude =
  PeakAmplitude(value: dbToAmplitude(x.value))

func toDb*(x: LinearGain): GainDb =
  GainDb(value: amplitudeToDb(x.value))

func toLinear*(x: GainDb): LinearGain =
  LinearGain(value: dbToAmplitude(x.value))

func max*(a, b: PeakAmplitude): PeakAmplitude =
  if a.value >= b.value: a else: b

func max*(a, b: LevelDbfs): LevelDbfs =
  if a.value >= b.value: a else: b

func clamp*(x, minValue, maxValue: GainDb): GainDb =
  doAssert minValue.value <= maxValue.value, "minimum gain must not exceed maximum gain"
  if x.value < minValue.value: minValue
  elif x.value > maxValue.value: maxValue
  else: x

func `-`*(level, other: LevelDbfs): GainDb =
  GainDb(value: decibelDifference(level.value, other.value))

func `+`*(level: LevelDbfs, gain: GainDb): LevelDbfs =
  LevelDbfs(value: decibelSum(level.value, gain.value))

func `+`*(a, b: GainDb): GainDb =
  GainDb(value: decibelSum(a.value, b.value))
