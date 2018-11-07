use crate::platter::Platter;
use crate::error::{err, Error};

pub struct Program {
    registers: [Platter; 8],
    platters: Vec<Vec<Platter>>,
    free: Vec<usize>,
    finger: usize
}

impl Program {

    pub fn new() -> Program {
        Program {
            registers: [Platter::from(0); 8],
            platters: vec![vec![]],
            free: vec![],
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
        let platter = *program.get(self.finger)?;

        // advance finger
        self.finger += 1;

        // apply operator
        self.apply_operator(platter)
    }

    fn apply_operator(&mut self, op: Platter) -> Result<StepResult,Error> {

        let op_val = op.to_u32();
        let op_num = (op_val >> 28) & 15;

        let a = || ((op_val >> 6) & 7) as usize;
        let b = || ((op_val >> 3) & 7) as usize;
        let c = || (op_val & 7) as usize;

        match op_num {
            0 /* Conditional Move */ => {
                if self.registers[c()] != Platter::from(0) {
                    self.registers[a()] = self.registers[b()];
                }
            },
            1 /* Array Index */ => {
                let array = self.platters.get(self.registers[b()].to_pos())?;
                let val = array.get(self.registers[c()].to_pos())?;
                self.registers[a()] = *val;
            },
            2 /* Array Amendment */ => {
                let array = self.platters.get_mut(self.registers[a()].to_pos())?;
                let offset = self.registers[b()].to_pos();
                *array.get_mut(offset)? = self.registers[c()];
            },
            3 /* Addition */ => {
                self.registers[a()] = self.registers[b()].wrapping_add(self.registers[c()]);
            },
            4 /* Multiplication */ => {
                self.registers[a()] = self.registers[b()].wrapping_mul(self.registers[c()]);
            },
            5 /* Division */ => {
                let c_val = self.registers[c()];
                if c_val == Platter::from(0) {
                    return Err(err("divide by 0"));
                }
                self.registers[a()] = self.registers[b()] / c_val;
            },
            6 /* Not-And */ => {
                self.registers[a()] = !self.registers[b()] | !self.registers[c()];
            },
            7 /* Halt */ => {
                return Ok(StepResult::Halted)
            },
            8 /* Allocation */ => {
                let size = self.registers[c()].to_pos();
                let pos = if let Some(idx) = self.free.pop() {
                    self.platters[idx] = vec![Platter::from(0); size];
                    idx
                } else {
                    let idx = self.platters.len();
                    self.platters.push(vec![Platter::from(0); size]);
                    idx
                };
                self.registers[b()] = Platter::from(pos as u32);
            },
            9 /* Abandonment */ => {
                let idx = self.registers[c()].to_pos();
                *self.platters.get_mut(self.registers[c()].to_pos())? = vec![];
                self.free.push(idx);
            },
            10 /* Output */ => {
                return Ok(StepResult::Output{ ascii: self.registers[c()].to_u8() });
            },
            11 /* Input */ => {
                return Ok(StepResult::InputNeeded{ inputter: Inputter{ register: c() } });
            },
            12 /* LoadProgram */ => {
                let pos = self.registers[b()].to_pos();
                if pos != 0 {
                    self.platters[0] = self.platters.get(pos)?.clone();
                }
                self.finger = self.registers[c()].to_pos();
            },
            13 /* Orthography */ => {
                let a = (op_val >> 25) & 7;
                let value = op_val & 0b0000_000_1111111111111111111111111;
                self.registers[a as usize] = Platter::from(value);
            },
            _ /* invalid op */ => {
                return Err(err("Invalid op"))
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

