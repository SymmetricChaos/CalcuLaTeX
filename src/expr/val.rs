use num::traits::Pow;
use std::convert::TryFrom;

use super::unit::Unit;

use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone)]
pub struct Val {
    pub num: f64,
    pub unit: Unit,
}

impl PartialEq for Val {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num && self.unit == other.unit
    }
}

impl Debug for Val {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Display for Val {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let out = format!(
            "{} {}",
            // lossy conversions since Val's Display isn't actually used
            self.unit.mult * self.num * 10f64.powi(self.unit.exp as i32),
            self.unit.to_string()
        );
        write!(f, "{}", out.trim())
    }
}

impl std::ops::Add<Val> for Val {
    type Output = Val;

    fn add(self, rhs: Val) -> Self::Output {
        if self.unit.desc == rhs.unit.desc {
            let (larger_exp, smaller_exp) = if rhs.unit.exp.abs() > self.unit.exp.abs() {
                (&rhs, &self)
            } else {
                (&self, &rhs)
            };

            let add_exp = larger_exp.unit.exp - smaller_exp.unit.exp;
            let mult_factor = smaller_exp.unit.mult / larger_exp.unit.mult;

            let mut num =
                larger_exp.num + (smaller_exp.num / 10f64.powi(add_exp as i32) * mult_factor);
            let mut exp = larger_exp.unit.exp;
            if num.abs() >= 10f64 {
                exp += num.log10() as i64;
                num /= 10f64.powi(num.log10() as i32);
            }

            Val {
                num,
                unit: Unit {
                    exp,
                    mult: larger_exp.unit.mult,
                    ..self.unit
                },
            }
            .clamp_num()
        } else {
            panic!("Can't add")
        }
    }
}

impl std::ops::Sub<Val> for Val {
    type Output = Val;

    fn sub(self, rhs: Val) -> Self::Output {
        if self.unit.desc == rhs.unit.desc {
            let larger_exp = if rhs.unit.exp.abs() > self.unit.exp.abs() {
                &rhs
            } else {
                &self
            };

            #[rustfmt::skip]
            let mut num =
                self.num / 10f64.powi((larger_exp.unit.exp - self.unit.exp) as i32) * self.unit.mult / larger_exp.unit.mult
                - rhs.num / 10f64.powi((larger_exp.unit.exp - rhs.unit.exp) as i32) * rhs.unit.mult / larger_exp.unit.mult;

            let mut exp = larger_exp.unit.exp;
            if num.abs() >= 10f64 {
                exp += num.log10() as i64;
                num /= 10f64.powi(num.log10() as i32);
            }

            Val {
                num,
                unit: Unit {
                    exp,
                    mult: larger_exp.unit.mult,
                    ..self.unit
                },
            }
            .clamp_num()
        } else {
            panic!("Can't sub")
        }
    }
}

impl std::ops::Mul<Val> for Val {
    type Output = Val;

    fn mul(self, rhs: Val) -> Self::Output {
        let mut new_num = self.num * rhs.num;
        let mut new_unit = self.unit * rhs.unit;

        if new_num.abs() >= 10f64 {
            new_unit.exp += new_num.log10() as i64;
            new_num = new_num.signum() * new_num / 10f64.powi(new_num.log10() as i32);
        }

        Val {
            num: new_num,
            unit: new_unit,
        }
        .clamp_num()
    }
}

impl std::ops::Div<Val> for Val {
    type Output = Val;

    fn div(self, rhs: Val) -> Self::Output {
        let mut new_num = self.num / rhs.num;
        let mut new_unit = self.unit / rhs.unit;

        if new_num.abs() >= 10f64 {
            new_unit.exp += new_num.log10() as i64;
            new_num = new_num.signum() * new_num / 10f64.powi(new_num.log10() as i32);
        }

        Val {
            num: new_num,
            unit: new_unit,
        }
        .clamp_num()
    }
}

impl Val {
    pub fn empty(val: f64) -> Self {
        Self {
            unit: Unit::empty(),
            num: val,
        }
    }

    pub fn with_unit(&self, unit: &Unit) -> Val {
        Val {
            num: self.num,
            unit: self.unit.clone() * unit.clone(),
        }
    }

    pub fn pow(&self, rhs: &Val) -> Val {
        if rhs.unit.desc.is_empty() || rhs.num.fract() == 0.0 {
            let p = rhs.num * 10f64.powi(rhs.unit.exp as i32);
            let unit = self.unit.pow(p as i64);
            Val {
                num: self.num.pow(p),
                unit,
            }
        } else {
            panic!()
        }
    }

    pub fn clamp_num(&self) -> Val {
        let num_log10 = self.num.log10() as i64;
        let mult_log10 = self.unit.mult.log10() as i64;

        let mut res = Val {
            num: self.num / 10f64.powi(num_log10 as i32),
            unit: Unit {
                mult: self.unit.mult / 10f64.powi(mult_log10 as i32),
                exp: self.unit.exp + num_log10 + mult_log10,
                desc: self.unit.desc.clone(),
            },
        };

        if res.num.abs() < 1.0 {
            let n = (res.num.signum() * 1.0f64 / res.num).floor();
            res.unit.exp -= 1 + n.log10() as i64;
            res.num *= 10f64.powi(n.log10() as i32 + 1);
        }

        res
    }
}

impl<V, U> TryFrom<(V, U)> for Val
where
    V: Into<f64>,
    U: Into<Unit>,
{
    type Error = &'static str;

    fn try_from((v, u): (V, U)) -> Result<Self, Self::Error> {
        Ok(Val {
            num: v.into(),
            unit: u.into(),
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::expr::{BaseUnit, Unit};
    use std::convert::TryInto;

    #[test]
    fn create_val() {
        let val: Val = (2.5, BaseUnit::Meter).try_into().unwrap();
        assert_eq!(val.to_string(), "2.5 m");
    }

    #[test]
    fn add_val_success() {
        let val1: Val = (0.9, BaseUnit::Meter).try_into().unwrap();
        let val2: Val = (0.1, BaseUnit::Meter).try_into().unwrap();
        assert_eq!((val1 + val2).to_string(), "1 m");
    }

    // #[test]
    // fn add_val_failure(){
    //     let val1: Val = (0.9,BaseUnit::Meter).try_into().unwrap();
    //     let val2: Val = (0.1,BaseUnit::Gram).try_into().unwrap();
    //     assert_eq!((val1 + val2).to_string(),"");
    // }
}
