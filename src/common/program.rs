use crate::platter::Platter;
use crate::ops::Operator;
use crate::error::{err, Error};

pub struct Program {
    registers: [Platter; 8],
    platters: Vec<Vec<Platter>>,
    finger: usize
}

impl Program {

    pub fn new() -> Program {
        Program {
            registers: [Platter::from(0); 8],
            platters: vec![vec![]],
            finger: 0
        }
    }

    pub fn instruction_index(&self) -> usize {
        self.finger
    }

    pub fn instruction_count(&self) -> usize {
        self.platters[0].len()
    }

    pub fn load_program(&mut self, scrolls: &[u8]) {
        let mut program_vec = vec![];
        for chunks in scrolls.chunks(4) {
            if let &[a,b,c,d] = chunks {
                let val = (a as u32) << 24
                        | (b as u32) << 16
                        | (c as u32) << 8
                        | (d as u32);
                program_vec.push(Platter::from(val));
            } else {
                continue;
            }
        }
        self.platters[0] = program_vec;
    }

    /// If a step asks for input, we are given back an Inputter, which cannot
    /// otherwise be created. We can pass this inputter here with some input
    /// to complete the action.
    pub fn provide_input(&mut self, inputter: Inputter, ascii: Option<u8>) {
        if let Some(val) = ascii {
            self.registers[inputter.register] = Platter::from(val as u32);
        } else {
            self.registers[inputter.register] = Platter::from(!0);
        }
    }

    pub fn step(&mut self) -> Result<StepResult,Error> {
        let program = &self.platters[0];

        // get operator
        let platter = program[self.finger];
        let op = platter.to_operator().ok_or("could not convert platter to operator")?;

        // advance finger if possible
        if self.finger < program.len() - 1 {
            self.finger += 1
        }

        // apply operator
        self.apply_operator(op)
    }

    fn apply_operator(&mut self, op: Operator) -> Result<StepResult,Error> {
        use self::Operator::*;
        match op {
            ConditionalMove{a,b,c} => {
                if self.registers[c] != Platter::from(0) {
                    self.registers[a] = self.registers[b];
                }
            },
            ArrayIndex{a,b,c} => {
                let array = self.platters.get(self.registers[b].to_pos())?;
                let val = array.get(self.registers[c].to_pos())?;
                self.registers[a] = *val;
            },
            ArrayAmendment{a,b,c} => {
                let array = self.platters.get_mut(self.registers[a].to_pos())?;
                let offset = self.registers[b].to_pos();
                *array.get_mut(offset)? = self.registers[c];
            },
            Addition{a,b,c} => {
                self.registers[a] = self.registers[b].wrapping_add(self.registers[c]);
            },
            Multiplication{a,b,c} => {
                self.registers[a] = self.registers[b].wrapping_mul(self.registers[c]);
            },
            Division{a,b,c} => {
                let c_val = self.registers[c];
                if c_val == Platter::from(0) {
                    return Err(err("divide by 0"));
                }
                self.registers[a] = self.registers[b] / c_val;
            },
            NotAnd{a,b,c} => {
                self.registers[a] = !self.registers[b] | !self.registers[c];
            },
            Halt => {
                return Ok(StepResult::Halted)
            },
            Allocation{b, c} => {
                let pos = self.platters.len();
                self.platters.push(vec![Platter::from(0); self.registers[c].to_pos()]);
                self.registers[b] = Platter::from(pos as u32);
            },
            Abandonment{c} => {
                *self.platters.get_mut(self.registers[c].to_pos())? = vec![];
            },
            Output{c} => {
                return Ok(StepResult::Output{ ascii: self.registers[c].to_u8() });
            },
            Input{c} => {
                return Ok(StepResult::InputNeeded{ inputter: Inputter{ register: c } });
            },
            LoadProgram{b,c} => {
                let pos = self.registers[b].to_pos();
                if pos != 0 {
                    self.platters[0] = self.platters.get(self.registers[b].to_pos())?.clone();
                }
                self.finger = self.registers[c].to_pos();
            },
            Orthography{ a, value } => {
                self.registers[a] = Platter::from(value);
            }
        }

        Ok(StepResult::Continue)
    }

}

/// If a step succeeds we get back a result which describes
/// anything that needs to happen.
pub enum StepResult {
    Halted,
    Output{ ascii: u8 },
    InputNeeded{ inputter: Inputter },
    Continue
}

/// If a step succeeds and asks for input, we get given back
/// this opaque struct which describes what needs to happen
/// with the input when it's passed back.
#[derive(Clone,Copy)]
pub struct Inputter {
    register: usize
}

