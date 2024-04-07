/// A Triplet is used for the 1-Wire search algorithm.
/// It is a tuple of three bits, where the first bit is the bit read from the bus, the second bit is the complement
/// of the first bit, and the third bit is the direction bit.
/// The direction bit is the bit that is written to the bus and is derived from the first and second bits.
/// The direction bit is used to determine the path to take in the search algorithm.
pub enum Triplet {
    Discrepancy(bool),
    AllMatch(bool),
    NoDevicesFound,
}

impl Triplet {
    pub fn new(bit: bool, complement_bit: bool, direction_bit: bool) -> Triplet {
        match (bit, complement_bit) {
            (false, true) => Triplet::AllMatch(false),
            (true, false) => Triplet::AllMatch(true),
            (false, false) => Triplet::Discrepancy(direction_bit),
            (true, true) => Triplet::NoDevicesFound,
        }
    }
}