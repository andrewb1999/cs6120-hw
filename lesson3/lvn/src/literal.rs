use std::ops::{Add, Mul, Div, Sub};
use std::cmp::Ordering;

#[derive(Eq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Literal::Int(x) => {
                if let Literal::Int(y) = other {
                    x == y
                } else {
                    false
                }
            },
            Literal::Float(x) => {
                if let Literal::Float(y) = other {
                    x == y
                } else {
                    false
                }
            },
            Literal::Bool(x) => {
                if let Literal::Bool(y) = other {
                    x == y
                } else {
                    false
                }
            },
        }
    }
}

impl PartialOrd for Literal {
    fn partial_cmp(&self, other : &Self) -> Option<Ordering> {

    }   
}

impl Ord for Literal {
    fn cmp(&self, other: &Self) -> Ordering {

    }
}

impl Add for Literal {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match self {
            Literal::Int(x) => {
                if let Literal::Int(y) = other {
                    Literal::Int(x + y)
                } else {
                    panic!("Not same type")
                }
            },
            Literal::Float(x) => {
                if let Literal::Float(y) = other {
                    Literal::Float(x + y)
                } else {
                    panic!("Not same type")
                }

            },
            Literal::Bool(x) => {
                    panic!("Can't add bools")
            },
        }
    }
}

impl Mul for Literal {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        match self {
            Literal::Int(x) => {
                if let Literal::Int(y) = other {
                    Literal::Int(x * y)
                } else {
                    panic!("Not same type")
                }
            },
            Literal::Float(x) => {
                if let Literal::Float(y) = other {
                    Literal::Float(x * y)
                } else {
                    panic!("Not same type")
                }

            },
            Literal::Bool(x) => {
                    panic!("Can't mul bools")
            },
        }
    }
}

impl Div for Literal {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        match self {
            Literal::Int(x) => {
                if let Literal::Int(y) = other {
                    Literal::Int(x / y)
                } else {
                    panic!("Not same type")
                }
            },
            Literal::Float(x) => {
                if let Literal::Float(y) = other {
                    Literal::Float(x / y)
                } else {
                    panic!("Not same type")
                }

            },
            Literal::Bool(x) => {
                    panic!("Can't mul bools")
            },
        }
    }
}
