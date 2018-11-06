use derive_more::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, BitOr, Not, BitAnd)]
#[repr(transparent)]
pub struct Platter(u32);

impl Platter {
    pub fn wrapping_mul(&self, other: Platter) -> Platter {
        Platter(self.0.wrapping_mul(other.0))
    }
    pub fn wrapping_add(&self, other: Platter) -> Platter {
        Platter(self.0.wrapping_add(other.0))
    }
    pub fn to_pos(self) -> usize {
        self.0 as usize
    }
    pub fn to_u8(self) -> u8 {
        self.0 as u8
    }
    pub fn to_u32(self) -> u32 {
        self.0
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