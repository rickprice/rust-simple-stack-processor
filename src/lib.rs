use std::convert::TryFrom;
use thiserror::Error;

#[cfg(test)]
mod tests;

/// Gas limit for execution: unlimited or limited to a number of steps.
#[derive(Debug, Clone, PartialEq)]
pub enum GasLimit {
    Unlimited,
    Limited(u64),
}

/// Errors that can occur during stack machine execution.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum StackMachineError {
    #[error("Division by zero")]
    DivisionByZero { failing_opcode: Opcode },

    #[error("Numeric overflow")]
    NumericOverflow { failing_opcode: Opcode },

    #[error("TryFromInt error")]
    TryFromIntError(#[from] std::num::TryFromIntError),

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

/// Result of trap handling.
#[derive(Debug, Clone, PartialEq)]
pub enum TrapHandled {
    Handled,
    NotHandled,
}

/// Trait for trap handlers implementing the Chain of Command pattern.
pub trait HandleTrap {
    fn handle_trap(
        &mut self,
        trap_id: i64,
        st: &mut StackMachineState,
    ) -> Result<TrapHandled, StackMachineError>;
}

/// A trap handler that handles a specific trap id with a closure.
pub struct TrapHandler<'a> {
    handled_trap: i64,
    to_run: Box<dyn Fn(i64, &mut StackMachineState) -> Result<TrapHandled, StackMachineError> + 'a>,
}

impl<'a> TrapHandler<'a> {
    /// Create a new TrapHandler for a specific trap id.
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
            (self.to_run)(self.handled_trap, st)
        } else {
            Ok(TrapHandled::NotHandled)
        }
    }
}

/// Opcodes supported by the stack machine.
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

/// Internal state of the stack machine.
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
    /// Returns the amount of gas used so far.
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }
}

/// The stack machine itself, holding state and trap handlers.
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
    fn pop_number_stack(&mut self) -> Result<i64, StackMachineError> {
        self.st.number_stack.pop().ok_or(StackMachineError::NumberStackUnderflow)
    }

    fn push_number_stack(&mut self, value: i64) {
        self.st.number_stack.push(value);
    }

    fn pop_scratch_stack(&mut self) -> Result<i64, StackMachineError> {
        self.st.scratch_stack.pop().ok_or(StackMachineError::ScratchStackUnderflow)
    }

    fn push_scratch_stack(&mut self, value: i64) {
        self.st.scratch_stack.push(value);
    }

    fn peek_scratch_stack(&self) -> Result<i64, StackMachineError> {
        self.st.scratch_stack.last().copied().ok_or(StackMachineError::ScratchStackUnderflow)
    }

    fn execute_binary_op<F>(&mut self, op: F, opcode: &Opcode) -> Result<(), StackMachineError>
    where
        F: FnOnce(i64, i64) -> Option<i64>,
    {
        let second = self.pop_number_stack()?;
        let first = self.pop_number_stack()?;
        let result = op(first, second).ok_or(StackMachineError::NumericOverflow {
            failing_opcode: opcode.clone(),
        })?;
        self.push_number_stack(result);
        Ok(())
    }

    fn execute_division(&mut self, opcode: &Opcode) -> Result<(), StackMachineError> {
        let divisor = self.pop_number_stack()?;
        let dividend = self.pop_number_stack()?;
        let result = dividend.checked_div(divisor).ok_or(StackMachineError::DivisionByZero {
            failing_opcode: opcode.clone(),
        })?;
        self.push_number_stack(result);
        Ok(())
    }

    fn execute_jump_relative(&mut self, condition: Option<bool>) -> Result<bool, StackMachineError> {
        let offset = self.pop_number_stack()?;
        let should_jump = if let Some(cond) = condition {
            let value = self.pop_number_stack()?;
            (value == 0) == cond
        } else {
            true
        };

        if should_jump {
            let new_offset = i64::try_from(self.st.pc)? + offset;
            self.st.pc = usize::try_from(new_offset)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
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
            let opcode = self
                .st
                .opcodes
                .get(self.st.pc)
                .ok_or(StackMachineError::UnknownError)?;
            match opcode {
                Opcode::JMP => {
                    let target = usize::try_from(self.pop_number_stack()?)?;
                    self.st.pc = target;
                    pc_reset = true;
                }
                Opcode::JR => {
                    pc_reset = self.execute_jump_relative(None)?;
                }
                Opcode::CALL => {
                    self.st.return_stack.push(self.st.pc + 1);
                    let target = usize::try_from(self.pop_number_stack()?)?;
                    self.st.pc = target;
                    pc_reset = true;
                }
                Opcode::CMPZ => {
                    let value = self.pop_number_stack()?;
                    self.push_number_stack(if value == 0 { -1 } else { 0 });
                }
                Opcode::CMPNZ => {
                    let value = self.pop_number_stack()?;
                    self.push_number_stack(if value == 0 { 0 } else { -1 });
                }
                Opcode::JRZ => {
                    pc_reset = self.execute_jump_relative(Some(true))?;
                }
                Opcode::JRNZ => {
                    pc_reset = self.execute_jump_relative(Some(false))?;
                }
                Opcode::LDI(immediate_value) => self.push_number_stack(*immediate_value),
                Opcode::DROP => {
                    let _ = self.pop_number_stack()?;
                }
                Opcode::RET => {
                    if let Some(return_address) = self.st.return_stack.pop() {
                        self.st.pc = return_address;
                        pc_reset = true;
                    } else {
                        return Ok(());
                    }
                }
                Opcode::GtR => {
                    let value = self.pop_number_stack()?;
                    self.push_scratch_stack(value);
                }
                Opcode::RGt => {
                    let value = self.pop_scratch_stack()?;
                    self.push_number_stack(value);
                }
                Opcode::RAt => {
                    let value = self.peek_scratch_stack()?;
                    self.push_number_stack(value);
                }
                Opcode::GtR2 => {
                    let x = self.pop_number_stack()?;
                    let y = self.pop_number_stack()?;
                    self.push_scratch_stack( y);
                    self.push_scratch_stack( x);
                }
                Opcode::RGt2 => {
                    let x = self.pop_scratch_stack()?;
                    let y = self.pop_scratch_stack()?;
                    self.push_number_stack(y);
                    self.push_number_stack(x);
                }
                Opcode::RAt2 => {
                    let x = self.pop_scratch_stack()?;
                    let y = self.pop_scratch_stack()?;
                    self.push_scratch_stack(y);
                    self.push_scratch_stack(x);
                    self.push_number_stack(y);
                    self.push_number_stack(x);
                }
                Opcode::ADD => {
                    self.execute_binary_op(|a, b| a.checked_add(b), &opcode.clone())?;
                }
                Opcode::SUB => {
                    self.execute_binary_op(|a, b| b.checked_sub(a), &opcode.clone())?;
                }
                Opcode::MUL => {
                    self.execute_binary_op(|a, b| a.checked_mul(b), &opcode.clone())?;
                }
                Opcode::DIV => {
                    self.execute_division(&opcode.clone())?;
                }
                Opcode::NOT => {
                    let x = self.pop_number_stack()?;
                    self.push_number_stack(if x == 0 { 1 } else { 0 });
                }
                Opcode::DUP => {
                    let x = self.pop_number_stack()?;
                    self.push_number_stack(x);
                    self.push_number_stack(x);
                }
                Opcode::DUP2 => {
                    let x = self.pop_number_stack()?;
                    let y = self.pop_number_stack()?;
                    self.push_number_stack(y);
                    self.push_number_stack(x);
                    self.push_number_stack(y);
                    self.push_number_stack(x);
                }
                Opcode::OVER2 => {
                    let x4 = self.pop_number_stack()?;
                    let x3 = self.pop_number_stack()?;
                    let x2 = self.pop_number_stack()?;
                    let x1 = self.pop_number_stack()?;
                    self.push_number_stack(x1);
                    self.push_number_stack(x2);
                    self.push_number_stack(x3);
                    self.push_number_stack(x4);
                    self.push_number_stack(x1);
                    self.push_number_stack(x2);
                }
                Opcode::SWAP => {
                    let x = self.pop_number_stack()?;
                    let y = self.pop_number_stack()?;
                    self.push_number_stack(x);
                    self.push_number_stack(y);
                }
                Opcode::SWAP2 => {
                    let x4 = self.pop_number_stack()?;
                    let x3 = self.pop_number_stack()?;
                    let x2 = self.pop_number_stack()?;
                    let x1 = self.pop_number_stack()?;
                    self.push_number_stack(x3);
                    self.push_number_stack(x4);
                    self.push_number_stack(x1);
                    self.push_number_stack(x2);
                }
                Opcode::TRAP => {
                    let trap_id = self.pop_number_stack()?;
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
                    let current_index = self.pop_number_stack()?;
                    let max_index = self.pop_number_stack()?;
                    self.st.loop_stack.push((current_index, max_index));
                }
                Opcode::INCLP => {
                    if let Some((current_index, _)) = self.st.loop_stack.last_mut() {
                        *current_index += 1;
                    } else {
                        return Err(StackMachineError::LoopStackUnderflow);
                    }
                }
                Opcode::ADDLP => {
                    let increment = self.pop_number_stack()?;
                    if let Some((current_index, _)) = self.st.loop_stack.last_mut() {
                        *current_index += increment;
                    } else {
                        return Err(StackMachineError::LoopStackUnderflow);
                    }
                }
                Opcode::GETLP => {
                    let (current_index, _) = self
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
                    let (current_index, _) = self
                        .st
                        .loop_stack
                        .get(self.st.loop_stack.len() - 2)
                        .ok_or(StackMachineError::LoopStackUnderflow)?;
                    self.st.number_stack.push(*current_index);
                }
                Opcode::DROPLP => {
                    self.st
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
                    self.st
                        .number_stack
                        .push(if *current_index >= *max_index { 1 } else { 0 });
                }
                Opcode::AND => {
                    let x = self.pop_number_stack()?;
                    let y = self.pop_number_stack()?;
                    self.push_number_stack(x & y);
                }
                Opcode::NEWCELLS => {
                    let num_cells = usize::try_from(self.pop_number_stack()?)
                        .map_err(|_| StackMachineError::InvalidCellOperation)?;
                    let newaddress = self.st.cells.len();
                    self.st
                        .cells
                        .resize_with(newaddress + num_cells, Default::default);
                }
                Opcode::MOVETOCELLS => {
                    let num_cells = usize::try_from(self.pop_number_stack()?)
                        .map_err(|_| StackMachineError::InvalidCellOperation)?;
                    let address = usize::try_from(self.pop_number_stack()?)
                        .map_err(|_| StackMachineError::InvalidCellOperation)?;
                    if num_cells < 1 || self.st.cells.len() < address + num_cells {
                        return Err(StackMachineError::InvalidCellOperation);
                    }
                    for i in address..address + num_cells {
                        self.st.cells[i] = self.pop_number_stack()?;
                    }
                }
                Opcode::MOVEFROMCELLS => {
                    let num_cells = usize::try_from(self.pop_number_stack()?)
                        .map_err(|_| StackMachineError::InvalidCellOperation)?;
                    let address = usize::try_from(self.pop_number_stack()?)
                        .map_err(|_| StackMachineError::InvalidCellOperation)?;
                    if num_cells < 1 || self.st.cells.len() < address + num_cells {
                        return Err(StackMachineError::InvalidCellOperation);
                    }
                    for i in (address..address + num_cells).rev() {
                        self.push_number_stack(self.st.cells[i]);
                    }
                }
            }
            if !pc_reset {
                self.st.pc += 1;
            }

            self.st.gas_used += 1;

            if let GasLimit::Limited(limit) = gas_limit {
                if self.st.gas_used > limit {
                    return Err(StackMachineError::RanOutOfGas {
                        gas_used: self.st.gas_used,
                        gas_limit,
                    });
                }
            }
        }
    }
}
