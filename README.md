# Arithmetic Coding

[![Latest Docs](https://docs.rs/arithmetic-coding/badge.svg)](https://docs.rs/arithmetic-coding/)
![Continuous integration](https://github.com/danieleades/arithmetic-coding/workflows/Continuous%20integration/badge.svg)
[![codecov](https://codecov.io/gh/danieleades/arithmetic-coding/branch/main/graph/badge.svg?token=1qITX2tR0J)](https://codecov.io/gh/danieleades/arithmetic-coding)


A symbolic [arithmetic coding](https://en.wikipedia.org/wiki/Arithmetic_coding) library.

Extending this library is as simple as implementing the `Model` trait for your own type, and then plugging it in the provided `Encoder`/`Decoder`. Supports both fixed-length and variable-length encoding, as well as both adaptive and non-adaptive models.

Take a look at the  [API docs](https://docs.rs/arithmetic-coding/) or the [examples](https://github.com/danieleades/arithmetic-coding/tree/main/examples).