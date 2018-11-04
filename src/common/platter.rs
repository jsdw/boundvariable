use crate::ops::Operator;
use derive_more::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, BitOr, Not, BitAnd)]
pub struct Platter(u32);

impl Platter {
    pub fn wrapping_mul(&self, other: Platter) -> Platter {
        Platter(self.0.wrapping_mul(other.0))
    }
    pub fn wrapping_add(&self, other: Platter) -> Platter {
        Platter(self.0.wrapping_add(other.0))
    }
    pub fn to_pos(&self) -> usize {
        self.0 as usize
    }
    pub fn to_u8(&self) -> u8 {
        self.0 as u8
    }
    pub fn to_operator(self) -> Option<Operator> {
        let val = self.0;
        let op_num = (val >> 28) & 15;

        let a = || ((val >> 6) & 7) as usize;
        let b = || ((val >> 3) & 7) as usize;
        let c = || (val & 7) as usize;

        use self::Operator::*;
        match op_num {
            0 => Some(ConditionalMove{a:a(), b:b(), c:c()}),
            1 => Some(ArrayIndex{a:a(), b:b(), c:c()}),
            2 => Some(ArrayAmendment{a:a(), b:b(), c:c()}),
            3 => Some(Addition{a:a(), b:b(), c:c()}),
            4 => Some(Multiplication{a:a(), b:b(), c:c()}),
            5 => Some(Division{a:a(), b:b(), c:c()}),
            6 => Some(NotAnd{a:a(), b:b(), c:c()}),
            7 => Some(Halt),
            8 => Some(Allocation{b:b(), c:c()}),
            9 => Some(Abandonment{c:c()}),
            10 => Some(Output{c:c()}),
            11 => Some(Input{c:c()}),
            12 => Some(LoadProgram{b:b(), c:c()}),
            13 => {
                let a = (val >> 25) & 7;
                let value = val & 0b0000_000_1111111111111111111111111;
                Some(Orthography{ a: a as usize, value })
            },
            _ => None
        }
    }
}

impl std::ops::Div for Platter {
    type Output = Platter;
    fn div(self, other: Platter) -> Platter {
        Platter(self.0 / other.0)
    }
}

impl std::convert::From<u32> for Platter {
    fn from(val: u32) -> Platter {
        Platter(val)
    }
}