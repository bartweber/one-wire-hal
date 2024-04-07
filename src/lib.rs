//! A hardware abstraction layer (HAL) for the 1-Wire protocol.

#![no_std]
// #![warn(missing_docs)]

use embedded_hal::delay::DelayNs;
use crate::address::Address;
use crate::error::ErrorType;
use crate::triplet::Triplet;

pub mod error;
pub mod address;
pub mod commands;
pub mod triplet;

pub mod device_search;
pub mod crc;

/// HAL trait for the 1-Wire protocol.
pub trait OneWire: ErrorType {
    /// Resets the 1-Wire bus.
    /// Returns true if a device responded with a presence pulse.
    fn reset(&mut self, delay: &mut impl DelayNs) -> Result<bool, Self::Error>;

    /// Reads a single bit from the bus.
    fn read_bit(&mut self, delay: &mut impl DelayNs) -> Result<bool, Self::Error>;

    /// Reads a single byte from the bus.
    fn read_byte(&mut self, delay: &mut impl DelayNs) -> Result<u8, Self::Error> {
        let mut output: u8 = 0;
        for _ in 0..8 {
            output >>= 1;
            if self.read_bit(delay)? {
                output |= 0x80;
            }
        }
        Ok(output)
    }

    /// Reads multiple bytes from the bus.
    fn read_bytes(&mut self, output: &mut [u8], delay: &mut impl DelayNs) -> Result<(), Self::Error> {
        for i in 0..output.len() {
            output[i] = self.read_byte(delay)?;
        }
        Ok(())
    }

    /// Writes a single bit to the bus.
    fn write_bit(&mut self, bit: bool, delay: &mut impl DelayNs) -> Result<(), Self::Error>;

    /// Writes a single byte to the bus.
    fn write_byte(&mut self, mut value: u8, delay: &mut impl DelayNs) -> Result<(), Self::Error> {
        for _ in 0..8 {
            self.write_bit(value & 0x01 == 0x01, delay)?;
            value >>= 1;
        }
        Ok(())
    }

    /// Writes multiple bytes to the bus.
    fn write_bytes(&mut self, bytes: &[u8], delay: &mut impl DelayNs) -> Result<(), Self::Error> {
        for i in 0..bytes.len() {
            self.write_byte(bytes[i], delay)?;
        }
        Ok(())
    }

    /// Reads the ROM of the single device on the bus.
    /// This only works for a single device on the bus.
    fn read_address(&mut self, delay: &mut impl DelayNs) -> Result<Address, Self::Error> {
        self.write_byte(commands::READ_ROM, delay)?;
        let mut rom: [u8; 8] = [0; 8];
        for i in 0..8 {
            rom[i] = self.read_byte(delay)?;
        }
        Ok(Address(u64::from_le_bytes(rom)))
    }

    /// Address a specific device. All others will wait for a reset pulse.
    /// This should only be called after a reset, and should be immediately followed by another command.
    fn match_address(&mut self, address: &Address, delay: &mut impl DelayNs) -> Result<(), Self::Error> {
        self.write_byte(commands::MATCH_ROM, delay)?;
        self.write_bytes(&address.0.to_le_bytes(), delay)?;
        Ok(())
    }

    /// Address all devices on the bus simultaneously.
    /// This should only be called after a reset, and should be immediately followed by another command.
    fn skip_address(&mut self, delay: &mut impl DelayNs) -> Result<(), Self::Error> {
        self.write_byte(commands::SKIP_ROM, delay)?;
        Ok(())
    }

    /// Sends a reset, followed with either a SKIP_ROM or MATCH_ROM (with an address), and then the supplied command.
    /// This should be followed by any reading/writing, if needed by the command used.
    fn send_command(&mut self, command: u8, address: Option<&Address>, delay: &mut impl DelayNs) -> Result<(), Self::Error> {
        self.reset(delay)?;
        if let Some(address) = address {
            self.match_address(address, delay)?;
        } else {
            self.skip_address(delay)?;
        }
        self.write_byte(command, delay)?;
        Ok(())
    }

    /// Generates three time slots on the bus: two read slots and one write slot.
    /// The first two read slots (bit value and complement bit value)
    /// are used to determine the bit value.
    /// The write slot is used to set the direction bit.
    /// The direction bit is the bit that is written to the bus
    /// and is derived from the bit value and the complement bit value.
    /// | bit value | complement bit value | direction bit |
    /// |-----------|----------------------|---------------|
    /// | 0         | 1                    | 0             |
    /// | 1         | 0                    | 1             |
    /// | 0         | 0                    | dir_bit       |
    /// | 1         | 1                    | 1             |
    ///
    ///
    /// Returns the triplet
    fn triplet(&mut self, dir_bit: bool, delay: &mut impl DelayNs) -> Result<Triplet, Self::Error>;

    /// Returns an iterator that iterates over all device addresses on the bus.
    /// There is no requirement to immediately finish iterating all devices, but if devices are
    /// added; are removed or change their alarm state, the search may return an error or fail to find a device.
    /// Device addresses will always be returned in the same order (lowest to highest, Little Endian).
    fn devices<'a>(&'a mut self, delay: &'a mut impl DelayNs) -> impl Iterator<Item=Result<Address, Self::Error>> + 'a;
    // TODO: Please fix this default implementation.
    // {
    //     DeviceSearch::new(false, self, delay)
    // }

    // Returns an iterator that iterates over all alarming device addresses on the bus.
    // There is no requirement to immediately finish iterating all devices, but if devices are
    // added; are removed or change their alarm state, the search may return an error or fail to find a device.
    // Device addresses will always be returned in the same order (lowest to highest, Little Endian).
    fn alarming_devices<'a>(&'a mut self, delay: &'a mut impl DelayNs) -> impl Iterator<Item=Result<Address, Self::Error>> + 'a;
    // TODO: Please fix this default implementation.
    // {
    //     DeviceSearch::new(true, self, delay)
    // }
}
