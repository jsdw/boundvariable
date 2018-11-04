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
        let (a,b,c) = self.extract_standard_op_registers();

        use self::Operator::*;
        match op_num {
            0 => Some(ConditionalMove{a,b,c}),
            1 => Some(ArrayIndex{a,b,c}),
            2 => Some(ArrayAmendment{a,b,c}),
            3 => Some(Addition{a,b,c}),
            4 => Some(Multiplication{a,b,c}),
            5 => Some(Division{a,b,c}),
            6 => Some(NotAnd{a,b,c}),
            7 => Some(Halt),
            8 => Some(Allocation{b,c}),
            9 => Some(Abandonment{c}),
            10 => Some(Output{c}),
            11 => Some(Input{c}),
            12 => Some(LoadProgram{b,c}),
            13 => {
                let a = (val >> 25) & 7;
                let value = val & 0b0000_000_1111111111111111111111111;
                Some(Orthography{ a: a as usize, value })
            },
            _ => None
        }
    }
    pub fn extract_standard_op_registers(self) -> (usize, usize, usize) {
        let val = self.0;
        let c = val & 7;
        let b = (val >> 3) & 7;
        let a = (val >> 6) & 7;
        (a as usize, b as usize, c as usize)
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