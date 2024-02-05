use embedded_hal::delay::DelayNs;

use crate::{commands, crc, OneWire};
use crate::address::Address;
use crate::triplet::Triplet;

pub struct DeviceSearch<'a, O, D>
{
    one_wire: &'a mut O,
    delay: &'a mut D,
    state: Option<SearchState>,
    finished: bool,
    search_command: u8,
}

impl<'a, O, D> DeviceSearch<'a, O, D>
    where O: OneWire,
          D: DelayNs,
{
    pub fn new(
        only_alarming: bool,
        one_wire: &'a mut O,
        delay: &'a mut D,
    ) -> impl Iterator<Item=Result<Address, O::Error>> + 'a
    {
        Self {
            one_wire,
            delay,
            state: None,
            finished: false,
            search_command: if only_alarming { commands::SEARCH_ALARM } else { commands::SEARCH_NORMAL },
        }
    }

    fn search(search_state: Option<&SearchState>, command: u8, one_wire: &mut O, delay: &mut D) -> Result<Option<(Address, SearchState)>, O::Error> {
        let first_time = search_state.is_none();
        let mut search_state = search_state.cloned().unwrap_or(SearchState::initial());

        // stop searching if there are no discrepancies left
        if search_state.discrepancies == 0 && !first_time {
            return Ok(None);
        }

        // reset one-wire bus
        let presence_pulse_detected = one_wire.reset(delay)?;
        if !presence_pulse_detected {
            return Ok(None);
        }

        // send search command
        one_wire.write_byte(command, delay)?;

        // do binary search for next device address
        for i in 0..64_u8 {
            // determine direction bit
            let dir_bit = if first_time {
                // first time searching, so always choose 0 in case of discrepancy
                false
            } else if i < search_state.last_discrepancy_index() {
                // follow same path as last time in case of discrepancy
                search_state.addr_bit(i)
            } else if i == search_state.last_discrepancy_index() {
                // at last discrepancy index, so now choose the other branch: 1
                true
            } else {
                // unknown path, so choose 0 in case of discrepancy
                false
            };

            // execute triplet
            let triplet = one_wire.triplet(dir_bit, delay)?;

            // update search state
            match triplet {
                Triplet::Discrepancy(dir_bit) => {
                    if first_time || i > search_state.last_discrepancy_index() {
                        // discrepancy found, so set bit in discrepancies bitflags
                        search_state.set_discrepancy(i);
                    } else if !first_time && i == search_state.last_discrepancy_index() {
                        // discrepancy found at last discrepancy index, so unset bit in discrepancies bitflags
                        search_state.unset_discrepancy(i);
                    }
                    search_state.set_addr_bit(i, dir_bit);
                }
                Triplet::AllMatch(bit) => {
                    // all devices have the same bit at this position
                    search_state.set_addr_bit(i, bit);
                }
                Triplet::NoDevicesFound => {
                    // no devices found, so stop searching
                    return Ok(None);
                }
            }
        }

        // TODO: do proper error handling
        crc::check_crc8(&search_state.address.to_le_bytes()).unwrap();

        Ok(Some((Address(search_state.address), search_state)))
    }
}

impl<'a, O, D> Iterator for DeviceSearch<'a, O, D>
    where O: OneWire,
          D: DelayNs,
{
    type Item = Result<Address, O::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let result = Self::search(self.state.as_ref(), self.search_command, self.one_wire, self.delay);
        match result {
            Ok(Some((address, search_state))) => {
                self.state = Some(search_state);
                Some(Ok(address))
            }
            Ok(None) => {
                self.state = None;
                self.finished = true;
                None
            }
            Err(err) => {
                self.state = None;
                self.finished = true;
                Some(Err(err))
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct SearchState {
    // the address of the last found device
    address: u64,

    // bitflags of discrepancies found
    discrepancies: u64,
}

impl SearchState {
    fn initial() -> SearchState {
        SearchState {
            address: 0,
            discrepancies: 0,
        }
    }

    pub fn addr_bit(&self, index: u8) -> bool {
        Self::get_bit(self.address, index)
    }

    pub fn set_addr_bit(&mut self, index: u8, bit: bool) {
        self.address = Self::set_bit(self.address, index, bit);
    }

    pub fn set_discrepancy(&mut self, index: u8) {
        self.discrepancies = Self::set_bit(self.discrepancies, index, true);
    }

    pub fn unset_discrepancy(&mut self, index: u8) {
        self.discrepancies = Self::set_bit(self.discrepancies, index, false);
    }

    pub fn last_discrepancy_index(&self) -> u8 {
        if self.discrepancies == 0 {
            return 0;
        }
        63 - self.discrepancies.leading_zeros() as u8
    }

    fn get_bit(data: u64, index: u8) -> bool {
        (data >> index) & 1 == 1
    }

    fn set_bit(data: u64, index: u8, value: bool) -> u64 {
        if value {
            data | (1 << index)
        } else {
            data & !(1 << index)
        }
    }
}