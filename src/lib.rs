use std::convert::TryFrom;
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum GasLimit {
    Unlimited,
    Limited(u64),
}

#[derive(Error, Debug)]
pub enum StackMachineError {
    #[error("Numeric overflow")]
    NumericOverflow(#[from] std::num::TryFromIntError),

    #[error("The internal number stack has underflowed (do you have too many POPs?)")]
    NumberStackUnderflow,

    #[error("The internal loop stack has underflowed (are you missing a loop start opcode?)")]
    LoopStackUnderflow,

    #[error(
        "The internal scratch stack has underflowed (do you have too many scratch stack POPs?)"
    )]
    ScratchStackUnderflow,

    #[error("Invalid Cell Operation (perhaps your parameters are incorrect?)")]
    InvalidCellOperation,

    #[error("Unhandled trap id: {unhandled_trap_id}")]
    UnhandledTrap { unhandled_trap_id: i64 },

    #[error("You used too much gas during execution (used {gas_used:?}, gas_limit {gas_limit:?}")]
    RanOutOfGas { gas_used: u64, gas_limit: GasLimit },

    #[error("Unknown StackMachineError")]
    UnknownError,
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
    GtR,
    RGt,
    RAt,
    GtR2,
    RGt2,
    RAt2,
    AND,
    NEWCELLS,
    MOVETOCELLS,
    MOVEFROMCELLS,
}

pub struct StackMachineState {
    pub number_stack: Vec<i64>,
    pub scratch_stack: Vec<i64>,
    return_stack: Vec<usize>,
    // current index, max_index
    loop_stack: Vec<(i64, i64)>,
    cells: Vec<i64>,
    pub opcodes: Vec<Opcode>,
    pc: usize,
    gas_used: u64,
}

impl Default for StackMachineState {
    fn default() -> Self {
        StackMachineState {
            number_stack: Vec::new(),
            scratch_stack: Vec::new(),
            return_stack: Vec::new(),
            loop_stack: Vec::new(),
            cells: Vec::new(),
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

macro_rules! pop_number_stack {
    ($variable:ident) => {
        $variable
            .st
            .number_stack
            .pop()
            .ok_or(StackMachineError::NumberStackUnderflow)?
    };
}

macro_rules! push_number_stack {
    ($variable:ident,$expr:expr) => {
        $variable.st.number_stack.push($expr)
    };
}

macro_rules! pop_scratch_stack {
    ($variable:ident) => {
        $variable
            .st
            .scratch_stack
            .pop()
            .ok_or(StackMachineError::ScratchStackUnderflow)?
    };
}

macro_rules! push_scratch_stack {
    ($variable:ident,$expr:expr) => {
        $variable.st.scratch_stack.push($expr);
    };
}

macro_rules! last_scratch_stack {
    ($variable:ident) => {
        $variable
            .st
            .scratch_stack
            .last()
            .ok_or(StackMachineError::ScratchStackUnderflow)?
    };
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
                    self.st.pc = usize::try_from(pop_number_stack!(self)).unwrap();
                    pc_reset = true;
                }
                Opcode::JR => {
                    let new_offset = i64::try_from(self.st.pc)? + pop_number_stack!(self);
                    self.st.pc = usize::try_from(new_offset).unwrap();
                    pc_reset = true;
                }
                Opcode::CALL => {
                    self.st.return_stack.push(self.st.pc + 1);
                    self.st.pc = usize::try_from(pop_number_stack!(self))?;
                    pc_reset = true;
                }
                Opcode::CMPZ => {
                    let x = pop_number_stack!(self);
                    if x == 0 {
                        self.st.number_stack.push(-1);
                    } else {
                        self.st.number_stack.push(0);
                    }
                }
                Opcode::CMPNZ => {
                    let x = pop_number_stack!(self);
                    if x == 0 {
                        self.st.number_stack.push(0);
                    } else {
                        self.st.number_stack.push(-1);
                    }
                }
                Opcode::JRZ => {
                    let new_offset = i64::try_from(self.st.pc)? + pop_number_stack!(self);
                    let x = pop_number_stack!(self);
                    if x == 0 {
                        self.st.pc = usize::try_from(new_offset).unwrap();
                        pc_reset = true;
                    }
                }
                Opcode::JRNZ => {
                    let new_offset = i64::try_from(self.st.pc)? + pop_number_stack!(self);
                    let x = pop_number_stack!(self);
                    if x != 0 {
                        self.st.pc = usize::try_from(new_offset).unwrap();
                        pc_reset = true;
                    }
                }
                Opcode::LDI(x) => push_number_stack!(self, x),
                Opcode::DROP => {
                    let _ = pop_number_stack!(self);
                }
                Opcode::RET => {
                    match self.st.return_stack.pop() {
                        None => return Ok(()),
                        Some(oldpc) => self.st.pc = oldpc,
                    };
                    pc_reset = true;
                }
                Opcode::GtR => {
                    let x = pop_number_stack!(self);
                    push_scratch_stack!(self, x);
                }
                Opcode::RGt => {
                    let x = pop_scratch_stack!(self);
                    push_number_stack!(self, x);
                }
                Opcode::RAt => {
                    let x = last_scratch_stack!(self);
                    push_number_stack!(self, *x);
                }
                Opcode::GtR2 => {
                    let x = pop_number_stack!(self);
                    let y = pop_number_stack!(self);
                    push_scratch_stack!(self, y);
                    push_scratch_stack!(self, x);
                }
                Opcode::RGt2 => {
                    let x = pop_scratch_stack!(self);
                    let y = pop_scratch_stack!(self);
                    push_number_stack!(self, y);
                    push_number_stack!(self, x);
                }
                Opcode::RAt2 => {
                    let x = pop_scratch_stack!(self);
                    let y = pop_scratch_stack!(self);
                    push_scratch_stack!(self, y);
                    push_scratch_stack!(self, x);
                    push_number_stack!(self, y);
                    push_number_stack!(self, x);
                }
                Opcode::ADD => {
                    let x = pop_number_stack!(self);
                    let y = pop_number_stack!(self);
                    push_number_stack!(self, x + y);
                }
                Opcode::SUB => {
                    let x = pop_number_stack!(self);
                    let y = pop_number_stack!(self);
                    push_number_stack!(self, x - y);
                }
                Opcode::MUL => {
                    let x = pop_number_stack!(self);
                    let y = pop_number_stack!(self);
                    push_number_stack!(self, x * y);
                }
                Opcode::DIV => {
                    let x = pop_number_stack!(self);
                    let y = pop_number_stack!(self);
                    push_number_stack!(self, y / x);
                }
                Opcode::NOT => {
                    let x = pop_number_stack!(self);
                    push_number_stack!(
                        self,
                        match x {
                            0 => 1,
                            _ => 0,
                        }
                    );
                }
                Opcode::DUP => {
                    let x = pop_number_stack!(self);
                    push_number_stack!(self, x);
                    push_number_stack!(self, x);
                }
                Opcode::DUP2 => {
                    let x = pop_number_stack!(self);
                    let y = pop_number_stack!(self);
                    push_number_stack!(self, y);
                    push_number_stack!(self, x);
                    push_number_stack!(self, y);
                    push_number_stack!(self, x);
                }
                Opcode::OVER2 => {
                    let x4 = pop_number_stack!(self);
                    let x3 = pop_number_stack!(self);
                    let x2 = pop_number_stack!(self);
                    let x1 = pop_number_stack!(self);
                    push_number_stack!(self, x1);
                    push_number_stack!(self, x2);
                    push_number_stack!(self, x3);
                    push_number_stack!(self, x4);
                    push_number_stack!(self, x1);
                    push_number_stack!(self, x2);
                }
                Opcode::SWAP => {
                    let x = pop_number_stack!(self);
                    let y = pop_number_stack!(self);
                    push_number_stack!(self, x);
                    push_number_stack!(self, y);
                }
                Opcode::SWAP2 => {
                    let x4 = pop_number_stack!(self);
                    let x3 = pop_number_stack!(self);
                    let x2 = pop_number_stack!(self);
                    let x1 = pop_number_stack!(self);
                    push_number_stack!(self, x3);
                    push_number_stack!(self, x4);
                    push_number_stack!(self, x1);
                    push_number_stack!(self, x2);
                }
                Opcode::TRAP => {
                    let trap_id = pop_number_stack!(self);
                    for h in self.trap_handlers.iter_mut() {
                        if let TrapHandled::Handled = h.handle_trap(trap_id, &mut self.st)? {
                            return Ok(());
                        }
                    }
                    return Err(StackMachineError::UnhandledTrap {
                        unhandled_trap_id: trap_id,
                    });
                }
                Opcode::NOP => {}
                Opcode::PUSHLP => {
                    let current_index = pop_number_stack!(self);
                    let max_index = pop_number_stack!(self);
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
                    let increment = pop_number_stack!(self);

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
                Opcode::AND => {
                    let x = pop_number_stack!(self);
                    let y = pop_number_stack!(self);
                    push_number_stack!(self, x & y);
                }
                Opcode::NEWCELLS => {
                    let num_cells = usize::try_from(pop_number_stack!(self))
                        .map_err(|_| StackMachineError::InvalidCellOperation)?;
                    let newaddress = self.st.cells.len();
                    self.st
                        .cells
                        .resize_with(newaddress + num_cells, Default::default);
                }
                Opcode::MOVETOCELLS => {
                    let num_cells = usize::try_from(pop_number_stack!(self))
                        .map_err(|_| StackMachineError::InvalidCellOperation)?;
                    let address = usize::try_from(pop_number_stack!(self))
                        .map_err(|_| StackMachineError::InvalidCellOperation)?;
                    if num_cells < 1 || self.st.cells.len() < address + num_cells {
                        return Err(StackMachineError::InvalidCellOperation);
                    }
                    for i in address..address + num_cells {
                        self.st.cells[i] = pop_number_stack!(self);
                    }
                }
                Opcode::MOVEFROMCELLS => {
                    let num_cells = usize::try_from(pop_number_stack!(self))
                        .map_err(|_| StackMachineError::InvalidCellOperation)?;
                    let address = usize::try_from(pop_number_stack!(self))
                        .map_err(|_| StackMachineError::InvalidCellOperation)?;
                    if num_cells < 1 || self.st.cells.len() < address + num_cells {
                        return Err(StackMachineError::InvalidCellOperation);
                    }
                    for i in (address..address + num_cells).rev() {
                        push_number_stack!(self, self.st.cells[i]);
                    }
                }
            };
            if !pc_reset {
                self.st.pc += 1;
            }

            self.st.gas_used += 1;

            if let GasLimit::Limited(limit) = gas_limit {
                if self.st.gas_used > limit {
                    return Err(StackMachineError::RanOutOfGas {
                        gas_used: self.st.gas_used,
                        gas_limit: gas_limit,
                    });
                }
            }
        }
    }
}
