pub trait Error: core::fmt::Debug {
    /// Convert error to a generic OW error kind.
    ///
    /// By using this method, OW errors freely defined by HAL implementations
    /// can be converted to a set of generic OW errors upon which generic
    /// code can act.
    fn kind(&self) -> ErrorKind;
}

impl Error for core::convert::Infallible {
    #[inline]
    fn kind(&self) -> ErrorKind {
        match *self {}
    }
}

/// OW error kind.
///
/// This represents a common set of OW operation errors. HAL implementations are
/// free to define more specific or additional error types. However, by providing
/// a mapping to these common OW errors, generic code can still react to them.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[non_exhaustive]
pub enum ErrorKind {
    /// Bus error occurred.
    Bus,
    /// The family code of the device did not match the expected one.
    FamilyCodeMismatch,
    /// No presence pulse was detected.
    NoPresencePulseDetected,
    /// The CRC check failed.
    CrcMismatch,
    /// A different error occurred. The original error may contain more information.
    Other,
}

impl Error for ErrorKind {
    #[inline]
    fn kind(&self) -> ErrorKind {
        *self
    }
}

impl core::fmt::Display for ErrorKind {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bus => write!(f, "Bus error occurred"),
            Self::FamilyCodeMismatch => write!(f, "The family code of the device did not match the expected one"),
            Self::NoPresencePulseDetected => write!(f, "No presence pulse was detected"),
            Self::CrcMismatch => write!(f, "The CRC check failed"),
            Self::Other => write!(f, "A different error occurred. The original error may contain more information"),
        }
    }
}

/// OW error type trait.
///
/// This just defines the error type, to be used by the other traits.
pub trait ErrorType {
    /// Error type
    type Error: Error;
}

impl<T: ErrorType + ?Sized> ErrorType for &mut T {
    type Error = T::Error;
}
