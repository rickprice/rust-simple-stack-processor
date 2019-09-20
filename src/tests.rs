use super::*;

#[test]
fn test_execute_jr_forward() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::LDI(1),
        Opcode::LDI(2),
        Opcode::LDI(2), // Jump to location 6 with the JR statement, relative jump of 1
        Opcode::JR,
        Opcode::LDI(3),
        Opcode::LDI(4),
        Opcode::LDI(5),
        Opcode::RET,
    ]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 1, 2, 4, 5]);
}

#[test]
fn test_execute_jr_backward() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::LDI(1),
        Opcode::RET,
        Opcode::LDI(2),
        Opcode::LDI(-5), // Jump to the LDI(0)
        Opcode::JR,
    ]);

    // Execute the instructions
    sm.execute(3, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 2, 0, 1]);
}

#[test]
fn test_execute_jrz_forward() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::LDI(1),
        Opcode::LDI(2),
        Opcode::LDI(1), // This won't happen because TOS won't be zero...
        Opcode::LDI(2), // TOS for JRZ
        Opcode::JRZ,
        Opcode::LDI(3),
        Opcode::LDI(4),
        Opcode::LDI(5),
        Opcode::LDI(0),
        Opcode::LDI(2), // Relative Jump of 1
        Opcode::JRZ,    // Jump over the LDI(6)
        Opcode::LDI(6),
        Opcode::LDI(7),
        Opcode::LDI(8),
        Opcode::RET,
    ]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 1, 2, 3, 4, 5, 7, 8]);
}

#[test]
fn test_execute_jrz_backward() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::RET,
        Opcode::LDI(1),
        Opcode::LDI(2),
        Opcode::LDI(1),  // This won't happen because TOS won't be zero...
        Opcode::LDI(-2), // TOS for JRZ
        Opcode::JRZ,
        Opcode::LDI(3),
        Opcode::LDI(4),
        Opcode::LDI(5),
        Opcode::LDI(0),
        Opcode::LDI(-12), // Relative Jump to start of code
        Opcode::JRZ,      // Jump over the LDI(6)
        Opcode::LDI(6),
        Opcode::LDI(7),
        Opcode::LDI(8),
        Opcode::RET,
    ]);

    // Execute the instructions
    sm.execute(2, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 1, 2, 3, 4, 5, 0]);
}

#[test]
fn test_execute_jrnz_forward() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::LDI(1),
        Opcode::LDI(2),
        Opcode::LDI(0), // This won't happen because TOS is zero...
        Opcode::LDI(2), // TOS for JRZ
        Opcode::JRNZ,
        Opcode::LDI(3),
        Opcode::LDI(4),
        Opcode::LDI(5),
        Opcode::LDI(1),
        Opcode::LDI(2), // Relative Jump of 1
        Opcode::JRNZ,   // Jump over the LDI(6)
        Opcode::LDI(6),
        Opcode::LDI(7),
        Opcode::LDI(8),
        Opcode::RET,
    ]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 1, 2, 3, 4, 5, 7, 8]);
}

#[test]
fn test_execute_jrnz_backward() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::RET,
        Opcode::LDI(1),
        Opcode::LDI(2),
        Opcode::LDI(0),  // This won't happen because TOS is zero...
        Opcode::LDI(-2), // TOS for JRZ
        Opcode::JRNZ,
        Opcode::LDI(3),
        Opcode::LDI(4),
        Opcode::LDI(5),
        Opcode::LDI(1),
        Opcode::LDI(-12), // Relative Jump to start of code
        Opcode::JRNZ,     // Jump over the LDI(6)
        Opcode::LDI(6),
        Opcode::LDI(7),
        Opcode::LDI(8),
        Opcode::RET,
    ]);

    // Execute the instructions
    sm.execute(2, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 1, 2, 3, 4, 5, 0]);
}

#[test]
fn test_execute_cmpz_1() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[123, 321, 0]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::CMPZ, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![123_i64, 321, 0]);
}

#[test]
fn test_execute_cmpz_2() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[123, 321, 1]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::CMPZ, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![123_i64, 321, -1]);
}

#[test]
fn test_execute_cmpnz_1() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[123, 321, 0]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::CMPNZ, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![123_i64, 321, -1]);
}

#[test]
fn test_execute_cmpnz_2() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[123, 321, 1]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::CMPNZ, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![123_i64, 321, 0]);
}

#[test]
fn test_execute_call() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::LDI(5),
        Opcode::CALL,
        Opcode::LDI(1),
        Opcode::RET,
        Opcode::LDI(2),
        Opcode::LDI(10),
        Opcode::CALL,
        Opcode::LDI(3),
        Opcode::RET,
        Opcode::LDI(4),
        Opcode::LDI(15),
        Opcode::CALL,
        Opcode::LDI(5),
        Opcode::RET,
        Opcode::LDI(6),
        Opcode::LDI(20),
        Opcode::CALL,
        Opcode::LDI(7),
        Opcode::RET,
        Opcode::LDI(8),
        Opcode::LDI(25),
        Opcode::CALL,
        Opcode::LDI(9),
        Opcode::RET,
        Opcode::RET,
    ]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(
        sm.st.number_stack,
        vec![321, 39483, 0, 2, 4, 6, 8, 9, 7, 5, 3, 1]
    );
}

#[test]
fn test_execute_ldi() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::LDI(0), Opcode::LDI(1), Opcode::LDI(2), Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 1, 2]);
}

#[test]
fn test_execute_pop() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::LDI(1),
        Opcode::DROP,
        Opcode::LDI(2),
        Opcode::RET,
    ]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 2]);
}

#[test]
#[should_panic]
fn test_execute_pop_error() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::LDI(1),
        Opcode::DROP,
        Opcode::DROP,
        Opcode::DROP,
        Opcode::DROP,
        Opcode::DROP,
        Opcode::LDI(2),
        Opcode::RET,
    ]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();
}

#[test]
fn test_execute_swap() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::LDI(1),
        Opcode::SWAP,
        Opcode::LDI(2),
        Opcode::RET,
    ]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 1, 0, 2]);
}

#[test]
fn test_execute_add() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[123, 321]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[Opcode::ADD, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![444]);
}

#[test]
fn test_execute_sub() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 444]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[Opcode::SUB, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![123]);
}

#[test]
fn test_execute_mul() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 123]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[Opcode::MUL, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![39483]);
}

#[test]
fn test_execute_div() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[10, 5]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[Opcode::DIV, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![2]);
}

#[test]
fn test_execute_not_1() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 0]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[Opcode::NOT, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321_i64, 1]);
}

#[test]
fn test_execute_not_2() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 1]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[Opcode::NOT, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321_i64, 0]);
}

#[test]
fn test_execute_not_3() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 346780]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[Opcode::NOT, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321_i64, 0]);
}

#[test]
fn test_execute_dup() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[123, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[Opcode::DUP, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![123, 39483, 39483]);
}

#[test]
#[should_panic]
fn test_execute_run_out_of_gas() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Put the opcodes into the *memory*
    sm.st.opcodes.extend_from_slice(&[
        Opcode::LDI(0),
        Opcode::LDI(5),
        Opcode::CALL,
        Opcode::LDI(1),
        Opcode::RET,
        Opcode::LDI(2),
        Opcode::LDI(10),
        Opcode::CALL,
        Opcode::LDI(3),
        Opcode::RET,
        Opcode::LDI(4),
        Opcode::LDI(15),
        Opcode::CALL,
        Opcode::LDI(5),
        Opcode::RET,
        Opcode::LDI(6),
        Opcode::LDI(20),
        Opcode::CALL,
        Opcode::LDI(7),
        Opcode::RET,
        Opcode::LDI(8),
        Opcode::LDI(25),
        Opcode::CALL,
        Opcode::LDI(9),
        Opcode::RET,
        Opcode::RET,
    ]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(10)).unwrap();
}

#[test]
fn test_handle_trap_1() {
    let mut sm = StackMachine::default();

    sm.trap_handlers
        .push(Box::from(TrapHandler::new(100, |_trap_id, st| {
            st.number_stack
                .pop()
                .ok_or(StackMachineError::NumberStackUnderflow)?;
            st.number_stack.push(200);
            Ok(TrapHandled::Handled)
        })));

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[50_i64, 100]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::TRAP, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![200]);
}

#[test]
fn test_handle_trap_2() {
    let mut sm = StackMachine::default();

    sm.trap_handlers
        .push(Box::from(TrapHandler::new(-100, |_trap_id, st| {
            st.number_stack
                .pop()
                .ok_or(StackMachineError::NumberStackUnderflow)?;
            st.number_stack.push(-100);
            Ok(TrapHandled::Handled)
        })));
    sm.trap_handlers
        .push(Box::from(TrapHandler::new(100, |_trap_id, st| {
            st.number_stack
                .pop()
                .ok_or(StackMachineError::NumberStackUnderflow)?;
            st.number_stack.push(200);
            Ok(TrapHandled::Handled)
        })));
    sm.trap_handlers
        .push(Box::from(TrapHandler::new(-200, |_trap_id, st| {
            st.number_stack
                .pop()
                .ok_or(StackMachineError::NumberStackUnderflow)?;
            st.number_stack.push(-200);
            Ok(TrapHandled::Handled)
        })));

    // Populate the number stack, with a value (50), and the trap number (100)
    sm.st.number_stack.extend_from_slice(&[50_i64, 100]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::TRAP, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![200]);
}

#[test]
fn test_unhandled_trap_1() {
    let mut sm = StackMachine::default();

    // Populate the number stack, with a value (50), and the trap number (100)
    sm.st.number_stack.extend_from_slice(&[50_i64, 100]);

    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::TRAP, Opcode::RET]);

    // Execute the instructions
    match sm.execute(0, GasLimit::Limited(100)) {
        Err(StackMachineError::UnhandledTrap) => (),
        r => panic!("Incorrect error type returned {:?}", r),
    }
}

#[test]
fn test_execute_pushlp() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483, 0]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::PUSHLP, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321]);
    assert_eq!(sm.st.loop_stack, vec![(0, 39483)]);
}

#[test]
fn test_execute_inclp() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483, 0]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::PUSHLP, Opcode::INCLP, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321]);
    assert_eq!(sm.st.loop_stack, vec![(1, 39483)]);
}

#[test]
fn test_execute_addlp() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483, 0]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::PUSHLP, Opcode::ADDLP, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![]);
    assert_eq!(sm.st.loop_stack, vec![(321, 39483)]);
}

#[test]
fn test_execute_getlp() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Populate the loop stack
    sm.st
        .loop_stack
        .extend_from_slice(&[(3210, 0), (394836, 0)]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::GETLP, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 394836]);
    assert_eq!(sm.st.loop_stack, vec![(3210, 0), (394836, 0)]);
}

#[test]
fn test_execute_getlp_fail_1() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);

    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::GETLP, Opcode::RET]);

    // Execute the instructions
    assert_eq!(
        match sm.execute(0, GasLimit::Limited(100)) {
            Err(StackMachineError::LoopStackUnderflow) => 1,
            _ => 0,
        },
        1
    );
}

#[test]
fn test_execute_getlp2() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);
    // Populate the loop stack
    sm.st
        .loop_stack
        .extend_from_slice(&[(3210, 0), (394836, 0)]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::GETLP2, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39483, 3210]);
    assert_eq!(sm.st.loop_stack, vec![(3210, 0), (394836, 0)]);
}

#[test]
fn test_execute_getlp2_fail_2() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39483]);

    // Populate the loop stack
    sm.st.loop_stack.extend_from_slice(&[(3210, 0)]);

    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::GETLP2, Opcode::RET]);

    // Execute the instructions
    assert_eq!(
        match sm.execute(0, GasLimit::Limited(100)) {
            Err(StackMachineError::LoopStackUnderflow) => 1,
            _ => 0,
        },
        1
    );
}

#[test]
fn test_execute_cmpgelp_eq() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39583]);
    // Populate the loop stack
    sm.st
        .loop_stack
        .extend_from_slice(&[(3210, 0), (39483, 39483)]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::CMPLOOP, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39583, 1]);
    assert_eq!(sm.st.loop_stack, vec![(3210, 0), (39483, 39483)]);
}

#[test]
fn test_execute_cmpgelp_gt() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39583]);
    // Populate the loop stack
    sm.st
        .loop_stack
        .extend_from_slice(&[(3210, 0), (39484, 39483)]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::CMPLOOP, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39583, 1]);
    assert_eq!(sm.st.loop_stack, vec![(3210, 0), (39484, 39483)]);
}

#[test]
fn test_execute_cmpgelp_lt() {
    let mut sm = StackMachine::default();

    // Populate the number stack
    sm.st.number_stack.extend_from_slice(&[321, 39583]);
    // Populate the loop stack
    sm.st
        .loop_stack
        .extend_from_slice(&[(3210, 0), (39482, 39483)]);
    // Put the opcodes into the *memory*
    sm.st
        .opcodes
        .extend_from_slice(&[Opcode::CMPLOOP, Opcode::RET]);

    // Execute the instructions
    sm.execute(0, GasLimit::Limited(100)).unwrap();

    assert_eq!(sm.st.number_stack, vec![321, 39583, 0]);
    assert_eq!(sm.st.loop_stack, vec![(3210, 0), (39482, 39483)]);
}
