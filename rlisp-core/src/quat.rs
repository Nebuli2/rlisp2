use std::{fmt, ops};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat(pub f64, pub f64, pub f64, pub f64);

impl fmt::Display for Quat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Quat(a, b, c, d) = self;
        if a.is_nan() || b.is_nan() || c.is_nan() || d.is_nan() {
            write!(f, "{}", std::f64::NAN)
        } else {
            let mut has_printed = false;
            if a != 0.0 {
                write!(f, "{}", a)?;
                has_printed = true;
            }
            if b != 0.0 {
                if has_printed {
                    write!(f, "{:+}", b)?;
                } else {
                    write!(f, "{}", b)?;
                    has_printed = true;
                }
                write!(f, "i")?;
            }
            if c != 0.0 {
                if has_printed {
                    write!(f, "{:+}", c)?;
                } else {
                    write!(f, "{}", c)?;
                    has_printed = true;
                }
                write!(f, "j")?;
            }
            if d != 0.0 {
                if has_printed {
                    write!(f, "{:+}", d)?;
                } else {
                    write!(f, "{}", d)?;
                    has_printed = true;
                }
                write!(f, "k")?;
            }
            if !has_printed {
                write!(f, "{}", 0.0)?;
            }

            Ok(())
        }
    }
}

impl Default for Quat {
    fn default() -> Quat {
        Quat(0.0, 0.0, 0.0, 0.0)
    }
}

impl From<f64> for Quat {
    fn from(n: f64) -> Quat {
        Quat(n, 0.0, 0.0, 0.0)
    }
}

impl Quat {
    fn norm(&self) -> f64 {
        let Quat(a, b, c, d) = self;
        f64::sqrt(a * a + b * b + c * c + d * d)
    }

    fn unit(&self) -> Quat {
        let mag = self.norm();
        (1.0 / mag) * (*self)
    }

    /// Produces the value of `e` raised to the power of the quaternion.
    pub fn exp(&self) -> Quat {
        let Quat(a, ..) = self;
        let norm = self.norm();
        f64::exp(*a)
            * (Quat(f64::cos(norm), 0.0, 0.0, 0.0)
                + self.unit() * f64::sin(norm))
    }

    pub fn ln(&self) -> Quat {
        let Quat(a, b, c, d) = self;
        let norm = self.norm();
        let vec_norm = f64::sqrt(b * b + c * c + d * d);
        f64::ln(norm) * (((1.0 / vec_norm) * (*self)) * f64::acos(a / norm))
    }

    pub fn pow(&self, exponent: Quat) -> Quat {
        (exponent * self.ln()).exp()
    }
}

impl ops::Add for Quat {
    type Output = Quat;

    fn add(self, addend: Quat) -> Quat {
        let Quat(a1, b1, c1, d1) = self;
        let Quat(a2, b2, c2, d2) = addend;
        Quat(a1 + a2, b1 + b2, c1 + c2, d1 + d2)
    }
}

// q1^q2
// ln(q1^q2) = q2*ln(q1)

impl ops::Mul for Quat {
    type Output = Quat;

    fn mul(self, multiplicand: Quat) -> Quat {
        let Quat(a1, b1, c1, d1) = self;
        let Quat(a2, b2, c2, d2) = multiplicand;
        let a = a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2;
        let b = a1 * b2 + b1 * a2 + c1 * d2 - d1 * c2;
        let c = a1 * c2 - b1 * d2 + c1 * a2 + d1 * b2;
        let d = a1 * d2 + b1 * c2 - c1 * b2 + d1 * a2;
        Quat(a, b, c, d)
    }
}

impl ops::Mul<Quat> for f64 {
    type Output = Quat;

    fn mul(self, multiplicand: Quat) -> Quat {
        let Quat(a, b, c, d) = multiplicand;
        Quat(self * a, self * b, self * c, self * d)
    }
}

impl ops::Mul<f64> for Quat {
    type Output = Quat;

    fn mul(self, multiplicand: f64) -> Quat {
        let Quat(a, b, c, d) = self;
        Quat(
            multiplicand * a,
            multiplicand * b,
            multiplicand * c,
            multiplicand * d,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add() {
        let q1 = Quat(5.0, 0.0, 0.0, 0.0);
        let q2 = Quat(3.0, 0.0, 0.0, 0.0);
        let q3 = q1 + q2;
        let should_be = Quat(8.0, 0.0, 0.0, 0.0);
        assert_eq!(q3, should_be);
    }

    #[test]
    fn identities() {
        let i = Quat(0.0, 1.0, 0.0, 0.0);
        let j = Quat(0.0, 0.0, 1.0, 0.0);
        let k = Quat(0.0, 0.0, 0.0, 1.0);

        let ii = i * i;
        let jj = j * j;
        let kk = k * k;
        let ijk = i * j * k;
        let neg_one = Quat(-1.0, 0.0, 0.0, 0.0);
        assert_eq!(neg_one, ii);
        assert_eq!(neg_one, jj);
        assert_eq!(neg_one, kk);
        assert_eq!(neg_one, ijk);
    }

    #[test]
    fn exponentiation() {
        // Test e^(i*pi) = -1
        let q = Quat(0.0, std::f64::consts::PI, 0.0, 0.0);
        let neg_one = Quat(-1.0, 0.0, 0.0, 0.0);
        let dif = q.exp() + (-1.0 * neg_one);
        let mag = dif.norm();
        assert!(mag < 0.0001);
    }

    #[test]
    fn power() {
        let i = Quat(std::f64::consts::E, 0.0, 0.0, 0.0);
        println!("ln(i) = {:?}", i.ln());
        assert!(false);
    }
}
