# `one-wire-hal`

This crate provides a hardware abstraction layer (HAL) for 1-wire devices.

Most of the code in this crate is based on and inspired by the [OneWire](https://github.com/fuchsnj/one-wire-bus) crate
made by [japaric](https://github.com/fuchsnj).
Possibly this code would better be a pull request to the original crate, but it seems that the original crate is not
maintained anymore.

This project is a work in progress and is not yet ready for use.

## To Do

- [ ] Write tests for `DeviceSearch`.
- [ ] Fix `OneWire::devices` and `OneWire::devices` default implementation.
