pub trait Exp {
    type Output;

    /// Raises `e` to the power of this value.
    fn exp(self) -> Self::Output;
}

pub trait Ln {
    type Output;

    fn ln(self) -> Self::Output;
}

impl Exp for f64 {
    type Output = Self;

    fn exp(self) -> f64 {
        f64::exp(self)
    }
}

impl Ln for f64 {
    type Output = Self;

    fn ln(self) -> f64 {
        f64::ln(self)
    }
}

use quat::Quat;

impl Exp for Quat {
    type Output = Self;

    fn exp(self) -> Quat {
        self.exp()
    }
}
