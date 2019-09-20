use std::convert::TryFrom;
use std::num::TryFromIntError;

#[cfg(test)]
mod tests;

pub enum GasLimit {
    Unlimited,
    Limited(u64),
}

#[derive(Debug)]
pub enum StackMachineError {
    UnkownError,
    NumericOverflow,
    NumberStackUnderflow,
    LoopStackUnderflow,
    UnhandledTrap,
    RanOutOfGas,
}

impl From<TryFromIntError> for StackMachineError {
    fn from(err: TryFromIntError) -> StackMachineError {
        match err {
            _ => StackMachineError::NumericOverflow,
        }
    }
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
            handled_trap,
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
    DROP,
    SWAP,
    SWAP2,
    RET,
    ADD,
    SUB,
    MUL,
    DIV,
    NOT,
    DUP,
    DUP2,
    TRAP,
    NOP,
    PUSHLP,
    INCLP,
    ADDLP,
    GETLP,
    GETLP2,
    DROPLP,
    CMPLOOP,
    OVER2,
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

impl Default for StackMachineState {
    fn default() -> Self {
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

impl Default for StackMachine {
    fn default() -> StackMachine {
        StackMachine {
            st: StackMachineState::default(),
            trap_handlers: Vec::new(),
        }
    }
}

impl StackMachine {
    /// JR(*) is relative from the JR(*) instruction,
    /// 0 would jump back onto the JR instruction
    /// -1 Would jump back to the instruction before the JR(*}) instruction
    /// 1 Would jump to the instruction after the JR(*) instruction
    ///
    /// TRAPs always have a numeric code on the number stack to define which TRAP is being called
    ///
    /// CMPLOOP
    /// pushes 1 on the stack if the loop counter is greater than or equal to the max
    /// pushes 0 on the stack if the loop counter is less than the max
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
                    let new_offset = i64::try_from(self.st.pc)?
                        + self
                            .st
                            .number_stack
                            .pop()
                            .ok_or(StackMachineError::NumberStackUnderflow)?;
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
                    let new_offset = i64::try_from(self.st.pc)?
                        + self
                            .st
                            .number_stack
                            .pop()
                            .ok_or(StackMachineError::NumberStackUnderflow)?;
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
                    let new_offset = i64::try_from(self.st.pc)?
                        + self
                            .st
                            .number_stack
                            .pop()
                            .ok_or(StackMachineError::NumberStackUnderflow)?;
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
                Opcode::DROP => {
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
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let x = self
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
                    self.st.number_stack.push(match x {
                        0 => 1,
                        _ => 0,
                    });
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
                Opcode::DUP2 => {
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
                    self.st.number_stack.push(y);
                    self.st.number_stack.push(x);
                    self.st.number_stack.push(y);
                    self.st.number_stack.push(x);
                }
                Opcode::OVER2 => {
                    let x4 = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let x3 = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let x2 = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let x1 = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x1);
                    self.st.number_stack.push(x2);
                    self.st.number_stack.push(x3);
                    self.st.number_stack.push(x4);
                    self.st.number_stack.push(x1);
                    self.st.number_stack.push(x2);
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
                Opcode::SWAP2 => {
                    let x4 = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let x3 = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let x2 = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let x1 = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x3);
                    self.st.number_stack.push(x4);
                    self.st.number_stack.push(x1);
                    self.st.number_stack.push(x2);
                }
                Opcode::TRAP => {
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
                        *current_index += 1;
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
                            *current_index += increment;
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
                    if self.st.loop_stack.len() < 2 {
                        return Err(StackMachineError::LoopStackUnderflow);
                    }
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
                        self.st.number_stack.push(1);
                    } else {
                        self.st.number_stack.push(0);
                    }
                }
            };
            if !pc_reset {
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
