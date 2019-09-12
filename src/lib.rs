use std::convert::TryFrom;

#[cfg(test)]
mod tests;

pub enum GasLimit {
    Unlimited,
    Limited(u64),
}

#[derive(Debug)]
pub enum StackMachineError {
    UnkownError,
    NumberStackUnderflow,
    LoopStackUnderflow,
    UnhandledTrap,
    RanOutOfGas,
}

pub enum TrapHandled {
    Handled,
    NotHandled,
}

// Chain of Command Pattern
pub trait HandleTrap {
    fn handle_trap(
        &mut self,
        trap_id: i64,
        st: &mut StackMachineState,
    ) -> Result<TrapHandled, StackMachineError>;
}

pub struct TrapHandler<'a> {
    handled_trap: i64,
    to_run: Box<dyn Fn(i64, &mut StackMachineState) -> Result<TrapHandled, StackMachineError> + 'a>,
}

impl<'a> TrapHandler<'a> {
    pub fn new<C>(handled_trap: i64, f: C) -> TrapHandler<'a>
    where
        C: Fn(i64, &mut StackMachineState) -> Result<TrapHandled, StackMachineError> + 'a,
    {
        TrapHandler {
            handled_trap: handled_trap,
            to_run: Box::new(f),
        }
    }
}

impl<'a> HandleTrap for TrapHandler<'a> {
    fn handle_trap(
        &mut self,
        trap_number: i64,
        st: &mut StackMachineState,
    ) -> Result<TrapHandled, StackMachineError> {
        if trap_number == self.handled_trap {
            return (self.to_run)(self.handled_trap, st);
        }
        Ok(TrapHandled::NotHandled)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    JMP,
    JR,
    JRZ,
    JRNZ,
    CALL,
    CMPZ,
    CMPNZ,
    LDI(i64),
    POP,
    SWAP,
    RET,
    ADD,
    SUB,
    MUL,
    DIV,
    NOT,
    DUP,
    TRAP,
    NOP,
    PUSHLP,
    INCLP,
    ADDLP,
    GETLP,
    GETLP2,
    DROPLP,
    CMPLOOP,
}

pub struct StackMachineState {
    pub number_stack: Vec<i64>,
    return_stack: Vec<usize>,
    // current index, max_index
    loop_stack: Vec<(i64, i64)>,
    pub opcodes: Vec<Opcode>,
    pc: usize,
    gas_used: u64,
}

impl StackMachineState {
    pub fn new() -> StackMachineState {
        StackMachineState {
            number_stack: Vec::new(),
            return_stack: Vec::new(),
            loop_stack: Vec::new(),
            opcodes: Vec::new(),
            pc: 0,
            gas_used: 0,
        }
    }
}

impl StackMachineState {
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }
}

pub struct StackMachine {
    pub st: StackMachineState,
    pub trap_handlers: Vec<Box<dyn HandleTrap>>,
}

impl StackMachine {
    pub fn new() -> StackMachine {
        StackMachine {
            st: StackMachineState::new(),
            trap_handlers: Vec::new(),
        }
    }

    pub fn execute(
        &mut self,
        starting_point: usize,
        gas_limit: GasLimit,
    ) -> Result<(), StackMachineError> {
        self.st.gas_used = 0;
        self.st.pc = starting_point;
        loop {
            let mut pc_reset = false;
            match self.st.opcodes[self.st.pc] {
                Opcode::JMP => {
                    self.st.pc = self
                        .st
                        .number_stack
                        .pop()
                        .map(|x| x as usize)
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    pc_reset = true;
                }
                Opcode::JR => {
                    let new_offset = self.st.pc as i128
                        + self
                            .st
                            .number_stack
                            .pop()
                            .ok_or(StackMachineError::NumberStackUnderflow)?
                            as i128;
                    self.st.pc = usize::try_from(new_offset).unwrap();
                    pc_reset = true;
                }
                Opcode::CALL => {
                    self.st.return_stack.push(self.st.pc + 1);
                    self.st.pc = self
                        .st
                        .number_stack
                        .pop()
                        .map(|x| x as usize)
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    pc_reset = true;
                }
                Opcode::CMPZ => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    if x == 0 {
                        self.st.number_stack.push(0);
                    } else {
                        self.st.number_stack.push(-1);
                    }
                }
                Opcode::CMPNZ => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    if x == 0 {
                        self.st.number_stack.push(-1);
                    } else {
                        self.st.number_stack.push(0);
                    }
                }
                Opcode::JRZ => {
                    let new_offset = self.st.pc as i128
                        + self
                            .st
                            .number_stack
                            .pop()
                            .ok_or(StackMachineError::NumberStackUnderflow)?
                            as i128;
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    if x == 0 {
                        self.st.pc = usize::try_from(new_offset).unwrap();
                        pc_reset = true;
                    }
                }
                Opcode::JRNZ => {
                    let new_offset = self.st.pc as i128
                        + self
                            .st
                            .number_stack
                            .pop()
                            .ok_or(StackMachineError::NumberStackUnderflow)?
                            as i128;
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    if x != 0 {
                        self.st.pc = usize::try_from(new_offset).unwrap();
                        pc_reset = true;
                    }
                }
                Opcode::LDI(x) => self.st.number_stack.push(x),
                Opcode::POP => {
                    let _ = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                }
                Opcode::RET => {
                    match self.st.return_stack.pop() {
                        None => return Ok(()),
                        Some(oldpc) => self.st.pc = oldpc,
                    };
                    pc_reset = true;
                }
                Opcode::ADD => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x + y);
                }
                Opcode::SUB => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x - y);
                }
                Opcode::MUL => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x * y);
                }
                Opcode::DIV => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x / y);
                }
                Opcode::NOT => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(!x);
                }
                Opcode::DUP => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x);
                    self.st.number_stack.push(x);
                }
                Opcode::SWAP => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x);
                    self.st.number_stack.push(y);
                }
                Opcode::TRAP => {
                    // We are going to say that TRAPs always have a numeric code on the number stack to define which TRAP is being called
                    let trap_id = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    for h in self.trap_handlers.iter_mut() {
                        if let TrapHandled::Handled = h.handle_trap(trap_id, &mut self.st)? {
                            return Ok(());
                        }
                    }
                    return Err(StackMachineError::UnhandledTrap);
                }
                Opcode::NOP => {}
                Opcode::PUSHLP => {
                    let current_index = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let max_index = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.loop_stack.push((current_index, max_index));
                }
                Opcode::INCLP => match self.st.loop_stack.last_mut() {
                    Some((current_index, _max_index)) => {
                        *current_index = *current_index + 1;
                    }
                    None => {
                        return Err(StackMachineError::LoopStackUnderflow);
                    }
                },
                Opcode::ADDLP => {
                    let increment = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;

                    match self.st.loop_stack.last_mut() {
                        Some((current_index, _max_index)) => {
                            *current_index = *current_index + increment;
                        }
                        None => {
                            return Err(StackMachineError::LoopStackUnderflow);
                        }
                    }
                }
                Opcode::GETLP => {
                    let (current_index, _max_index) = self
                        .st
                        .loop_stack
                        .last()
                        .ok_or(StackMachineError::LoopStackUnderflow)?;
                    self.st.number_stack.push(*current_index);
                }
                Opcode::GETLP2 => {
                    let (current_index, _max_index) = self
                        .st
                        .loop_stack
                        .get(self.st.loop_stack.len() - 2)
                        .ok_or(StackMachineError::LoopStackUnderflow)?;
                    self.st.number_stack.push(*current_index);
                }
                Opcode::DROPLP => {
                    let _x = self
                        .st
                        .loop_stack
                        .pop()
                        .ok_or(StackMachineError::LoopStackUnderflow)?;
                }
                Opcode::CMPLOOP => {
                    let (current_index, max_index) = self
                        .st
                        .loop_stack
                        .last()
                        .ok_or(StackMachineError::LoopStackUnderflow)?;
                    if *current_index >= *max_index {
                        self.st.number_stack.push(0);
                    } else {
                        self.st.number_stack.push(1);
                    }
                }
            };
            if pc_reset == false {
                self.st.pc += 1;
            }

            self.st.gas_used += 1;

            if let GasLimit::Limited(x) = gas_limit {
                if self.st.gas_used > x {
                    return Err(StackMachineError::RanOutOfGas);
                }
            }
        }
    }
}
