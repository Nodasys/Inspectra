//! Debugging and code injection module

use crate::error::Result;
use crate::memory::Memory;
use crate::types::Address;
use serde::{Deserialize, Serialize};

/// Breakpoint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakpointType {
    Software,
    Hardware,
}

/// Breakpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub address: Address,
    pub bp_type: BreakpointType,
    pub enabled: bool,
    pub original_byte: Option<u8>,
}

/// Register information (x64 example)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registers {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub rip: u64,
    pub rflags: u64,
}

/// Debugger interface
pub trait Debugger: Send + Sync {
    /// Set a breakpoint
    fn set_breakpoint(&mut self, address: Address, bp_type: BreakpointType) -> Result<()>;

    /// Remove a breakpoint
    fn remove_breakpoint(&mut self, address: Address) -> Result<()>;

    /// Continue execution
    fn continue_execution(&mut self) -> Result<()>;

    /// Step one instruction
    fn step(&mut self) -> Result<()>;

    /// Read registers
    fn read_registers(&self) -> Result<Registers>;

    /// Write registers
    fn write_registers(&mut self, registers: &Registers) -> Result<()>;

    /// Wait for debug event
    fn wait_for_event(&mut self) -> Result<DebugEvent>;
}

/// Debug event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugEvent {
    Breakpoint { address: Address },
    Exception { code: u32, address: Address },
    ProcessExit { exit_code: u32 },
    ThreadExit { exit_code: u32 },
}

/// Code patcher for inline assembly modifications
pub struct CodePatcher {
    memory: Box<dyn Memory>,
}

impl CodePatcher {
    /// Create a new code patcher
    pub fn new(memory: Box<dyn Memory>) -> Self {
        Self { memory }
    }

    /// NOP (No Operation) a region
    pub fn nop_region(&self, address: Address, size: usize) -> Result<Vec<u8>> {
        // Read original bytes
        let original = self.memory.read(address, size)?;

        // Write NOPs (0x90 on x86/x64)
        let nops = vec![0x90; size];
        self.memory.write(address, &nops)?;

        Ok(original)
    }

    /// Write a JMP (jump) instruction
    pub fn write_jmp(&self, from: Address, to: Address) -> Result<Vec<u8>> {
        // Read original bytes
        let original = self.memory.read(from, 5)?;

        // Calculate relative offset
        let offset = (to as i64 - from as i64 - 5) as i32;

        // Build JMP instruction (E9 XX XX XX XX)
        let mut jmp = vec![0xE9];
        jmp.extend_from_slice(&offset.to_le_bytes());

        self.memory.write(from, &jmp)?;

        Ok(original)
    }

    /// Restore original bytes
    pub fn restore(&self, address: Address, original: &[u8]) -> Result<()> {
        self.memory.write(address, original)?;
        Ok(())
    }

    /// Allocate memory for shellcode
    pub fn allocate_shellcode(&self, shellcode: &[u8]) -> Result<Address> {
        use crate::types::Protection;

        // Allocate executable memory
        let address = self.memory.allocate(
            shellcode.len(),
            Protection::new(true, true, true),
        )?;

        // Write shellcode
        self.memory.write(address, shellcode)?;

        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_creation() {
        let bp = Breakpoint {
            address: 0x1000,
            bp_type: BreakpointType::Software,
            enabled: true,
            original_byte: Some(0x55),
        };
        assert!(bp.enabled);
    }

    #[test]
    fn test_registers() {
        let regs = Registers {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            rip: 0x1000,
            rflags: 0,
        };
        assert_eq!(regs.rip, 0x1000);
    }
}
