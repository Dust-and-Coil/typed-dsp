version       = "0.1.0"
author        = "Dust & Coil Audio Laboratories"
description   = "Strongly typed audio units: peak amplitude, dBFS level, linear gain, and gain in dB."
license       = "MIT"
srcDir        = "src"

requires "nim >= 1.6.0"

task test, "Run the test suite":
  exec "nim c -r --path:src tests/test_typed_dsp.nim"
