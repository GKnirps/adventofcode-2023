use std::cmp::Ordering;
use std::fmt;
use std::ops;

#[derive(Copy, Clone)]
pub struct Rational(i128, i128);

impl Rational {
    pub fn new(counter: i128, denominator: i128) -> Rational {
        if denominator == 0 {
            panic!("rational denominator must not be 0");
        }
        Rational(counter, denominator)
    }
    pub fn reduce(self) -> Self {
        if self.0 == 0 {
            Rational(0, 1)
        } else {
            let l = gcd(self.0, self.1);
            Rational(self.1.signum() * self.0 / l, self.1.abs() / l)
        }
    }
}

impl PartialEq<Self> for Rational {
    fn eq(&self, other: &Self) -> bool {
        let r1 = self.reduce();
        let r2 = other.reduce();
        r1.0 == r2.0 && r1.1 == r2.1
    }
}

impl Eq for Rational {}

impl PartialEq<i128> for Rational {
    fn eq(&self, other: &i128) -> bool {
        let reduced = self.reduce();
        reduced.1 == 1 && reduced.0 == *other
    }
}

impl Ord for Rational {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let Rational(lnum, lre) = *self;
        let Rational(rnum, rre) = *rhs;
        let lcm = lcm(lre, rre);
        (lnum * lcm / lre).cmp(&(rnum * lcm / rre))
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl ops::Add<Rational> for Rational {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let de = lcm(self.1, other.1);
        let num = self.0 * (de / self.1) + other.0 * (de / other.1);
        Rational(num, de)
    }
}

impl ops::Neg for Rational {
    type Output = Self;
    fn neg(self) -> Self {
        Rational(-self.0, self.1)
    }
}

impl ops::Sub for Rational {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

impl ops::Mul for Rational {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let num = self.0 * other.0;
        let de = self.1 * other.1;
        Rational(num, de).reduce()
    }
}

impl ops::Div for Rational {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        if other.0 == 0 {
            panic!("rational division by zero");
        }
        self * Rational(other.1, other.0)
    }
}

impl From<i128> for Rational {
    fn from(n: i128) -> Self {
        Rational(n, 1)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let r = self.reduce();
        if r.1 == 1 {
            write!(f, "{}", r.0)
        } else {
            write!(f, "{}/{}", r.0, r.1)
        }
    }
}

impl fmt::Debug for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    if a == 0 {
        return b.abs();
    }
    if b == 0 {
        return a.abs();
    }
    while {
        let h = a % b;
        a = b;
        b = h;
        b != 0
    } {}

    a.abs()
}

fn lcm(a: i128, b: i128) -> i128 {
    (a * b) / gcd(a, b)
}
