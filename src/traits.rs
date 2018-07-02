///! Traits used in this quick conversion script

/// A trait that provides a factor for converting values into SI base unit values
pub trait SI {
    type Unit;
    /// Generate a factor that can convert a value of type T into a the base unit value
    fn si_factor(&self) -> f64;
    /// The representation of the SI base unit in the implementing type
    fn si_base() -> Self::Unit;
}