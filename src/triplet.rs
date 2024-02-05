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