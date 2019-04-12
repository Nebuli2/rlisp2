use std::{fmt, ops, str::FromStr};
use regex::Regex;

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

/// Represents a quaternion in the form a + b*i + c*j + d*k.
const QUAT_REGEX_STR_ABCD: &str = 
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-][0-9]+(\.[0-9]*)?)i([+-][0-9]+(\.[0-9]*)?)j([+-][0-9]+(\.[0-9]*)?)k";

/// Represents a quaternion in the form a + b*i.
const QUAT_REGEX_STR_AB: &str =
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-][0-9]+(\.[0-9]*)?)i";

/// Represents a quaternion in the form a + c*j.
const QUAT_REGEX_STR_AC: &str =
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-][0-9]+(\.[0-9]*)?)j";

/// Represents a quaternion in the form a + d*k.
const QUAT_REGEX_STR_AD: &str =
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-][0-9]+(\.[0-9]*)?)k";

/// Represents a quaternion in the form b*i + c*j.
const QUAT_REGEX_STR_BC: &str =
    r"([+-]?[0-9]+(\.[0-9]*)?)i([+-][0-9]+(\.[0-9]*)?)j";

/// Represents a quaternion in the form b*i + d*k.
const QUAT_REGEX_STR_BD: &str =
    r"([+-]?[0-9]+(\.[0-9]*)?)i([+-][0-9]+(\.[0-9]*)?)k";

/// Represents a quaternion in the form c*j + d*k.
const QUAT_REGEX_STR_CD: &str =
    r"([+-]?[0-9]+(\.[0-9]*)?)j([+-][0-9]+(\.[0-9]*)?)k";

/// Represents a quaternion in the form a + b*i + c*j.
const QUAT_REGEX_STR_ABC: &str =
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-][0-9]+(\.[0-9]*)?)i([+-][0-9]*(\.[0-9]*)?)j";

/// Represents a quaternion in the form a + b*i + d*k.
const QUAT_REGEX_STR_ABD: &str =
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-][0-9]+(\.[0-9]*)?)i([+-][0-9]*(\.[0-9]*)?)k";

/// Represents a quaternion in the form a + c*j + d*k.
const QUAT_REGEX_STR_ACD: &str =
    r"([+-]?[0-9]+(\.[0-9]*)?)([+-][0-9]+(\.[0-9]*)?)j([+-][0-9]+(\.[0-9]*)?)k";

/// Represents a quaternion in the form b*i + c*j + d*k.
const QUAT_REGEX_STR_BCD: &str =
  r"([+-]?[0-9]+(\.[0-9]*)?)i([+-][0-9]+(\.[0-9]*)?)j([+-][0-9]+(\.[0-9]*)?)k";

/// Represents a quaternion in the form b*i.
const QUAT_REGEX_STR_B: &str = r"([+-]?[0-9]+(\.[0-9]*)?)i";

/// Represents a quaternion in the form c*j.
const QUAT_REGEX_STR_C: &str = r"([+-]?[0-9]+(\.[0-9]*)?)j";

/// Represents a quaternion in the form d*k.
const QUAT_REGEX_STR_D: &str = r"([+-]?[0-9]+(\.[0-9]*)?)k";

lazy_static! {
    static ref QUAT_REGEX_ABCD: Regex = Regex::new(QUAT_REGEX_STR_ABCD)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_AB: Regex = Regex::new(QUAT_REGEX_STR_AB)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_AC: Regex = Regex::new(QUAT_REGEX_STR_AC)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_AD: Regex = Regex::new(QUAT_REGEX_STR_AD)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_BC: Regex = Regex::new(QUAT_REGEX_STR_BC)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_BD: Regex = Regex::new(QUAT_REGEX_STR_BD)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_CD: Regex = Regex::new(QUAT_REGEX_STR_CD)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_ABC: Regex = Regex::new(QUAT_REGEX_STR_ABC)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_ABD: Regex = Regex::new(QUAT_REGEX_STR_ABD)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_ACD: Regex = Regex::new(QUAT_REGEX_STR_ACD)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_BCD: Regex = Regex::new(QUAT_REGEX_STR_BCD)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_B: Regex = Regex::new(QUAT_REGEX_STR_B)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_C: Regex = Regex::new(QUAT_REGEX_STR_C)
        .expect("quaternion regex failed to compile");
    static ref QUAT_REGEX_D: Regex = Regex::new(QUAT_REGEX_STR_D)
        .expect("quaternion regex failed to compile");
}

#[derive(Debug)]
pub struct ParseQuatError;

impl FromStr for Quat {
    type Err = ParseQuatError;

    fn from_str(s: &str) -> Result<Quat, Self::Err> {
        if QUAT_REGEX_ABCD.is_match(s) {
            let caps = QUAT_REGEX_ABCD.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let b_str = caps.get(3).map_or("1", |m| m.as_str());
            let c_str = caps.get(5).map_or("1", |m| m.as_str());
            let d_str = caps.get(7).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_BCD.is_match(s) {
            let caps = QUAT_REGEX_BCD.captures(s).unwrap();
            let b_str = caps.get(1).map_or("1", |m| m.as_str());
            let c_str = caps.get(3).map_or("1", |m| m.as_str());
            let d_str = caps.get(5).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_BC.is_match(s) {
            let caps = QUAT_REGEX_BC.captures(s).unwrap();
            let b_str = caps.get(1).map_or("1", |m| m.as_str());
            let c_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_BD.is_match(s) {
            let caps = QUAT_REGEX_BD.captures(s).unwrap();
            let b_str = caps.get(1).map_or("1", |m| m.as_str());
            let d_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = 0.0;
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_CD.is_match(s) {
            let caps = QUAT_REGEX_CD.captures(s).unwrap();
            let c_str = caps.get(1).map_or("1", |m| m.as_str());
            let d_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = 0.0;
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_ABC.is_match(s) {
            let caps = QUAT_REGEX_ABC.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let b_str = caps.get(3).map_or("1", |m| m.as_str());
            let c_str = caps.get(5).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_ABD.is_match(s) {
            let caps = QUAT_REGEX_ABD.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let b_str = caps.get(3).map_or("1", |m| m.as_str());
            let d_str = caps.get(5).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = 0.0;
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_ACD.is_match(s) {
            let caps = QUAT_REGEX_ACD.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let c_str = caps.get(3).map_or("1", |m| m.as_str());
            let d_str = caps.get(5).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = 0.0;
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_AD.is_match(s) {
            let caps = QUAT_REGEX_AD.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let d_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = 0.0;
            let c = 0.0;
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_AC.is_match(s) {
            let caps = QUAT_REGEX_AC.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let c_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = 0.0;
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_AB.is_match(s) {
            let caps = QUAT_REGEX_AB.captures(s).unwrap();
            let a_str = caps.get(1).map_or("", |m| m.as_str());
            let b_str = caps.get(3).map_or("1", |m| m.as_str());

            let a = a_str.parse::<f64>().unwrap_or_default();
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = 0.0;
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_B.is_match(s) {
            let caps = QUAT_REGEX_B.captures(s).unwrap();
            let b_str = caps.get(1).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = b_str.parse::<f64>().unwrap_or_default();
            let c = 0.0;
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_C.is_match(s) {
            let caps = QUAT_REGEX_C.captures(s).unwrap();
            let c_str = caps.get(1).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = 0.0;
            let c = c_str.parse::<f64>().unwrap_or_default();
            let d = 0.0;
            Ok(Quat(a, b, c, d))
        } else if QUAT_REGEX_D.is_match(s) {
            let caps = QUAT_REGEX_D.captures(s).unwrap();
            let d_str = caps.get(1).map_or("1", |m| m.as_str());

            let a = 0.0;
            let b = 0.0;
            let c = 0.0;
            let d = d_str.parse::<f64>().unwrap_or_default();
            Ok(Quat(a, b, c, d))
        } else {
            Err(ParseQuatError)
        }
    }
}
