// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cmp::min;

use anyhow::{bail, Result};

use super::{
    addr::{ByteAddr, WordAddr},
    platform::*,
    rv32im::{DecodedInstruction, EmuContext, Emulator, Instruction, TrapCause},
};

pub trait Risc0Context {
    /// Get the program counter
    fn get_pc(&self) -> ByteAddr;

    /// Set the program counter
    fn set_pc(&mut self, addr: ByteAddr);

    /// Get the machine mode
    fn get_machine_mode(&self) -> u32;

    /// Set the machine mode
    fn set_machine_mode(&mut self, mode: u32);

    fn on_insn_start(&mut self, insn: &Instruction, decoded: &DecodedInstruction) -> Result<()>;

    fn on_insn_end(&mut self, insn: &Instruction, decoded: &DecodedInstruction) -> Result<()>;

    fn peek_u32(&mut self, addr: WordAddr) -> Result<u32>;

    fn load_register(&mut self, base: WordAddr, idx: usize) -> Result<u32> {
        self.load_u32(base + idx)
    }

    fn store_register(&mut self, base: WordAddr, idx: usize, word: u32) -> Result<()> {
        self.store_u32(base + idx, word)
    }

    fn load_u32(&mut self, addr: WordAddr) -> Result<u32>;

    fn store_u32(&mut self, addr: WordAddr, word: u32) -> Result<()>;

    fn on_ecall_cycle(
        &mut self,
        cur: CycleState,
        next: CycleState,
        s0: u32,
        s1: u32,
        s2: u32,
    ) -> Result<()>;

    fn on_terminate(&mut self, a0: u32, a1: u32);

    fn suspend(&mut self) -> Result<()> {
        // default no-op
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        // default no-op
        Ok(())
    }

    fn trap_rewind(&mut self) {
        // default no-op
    }

    fn trap(&mut self, _cause: TrapCause) {
        // default no-op
    }

    /// Record what was read during execution so we can replay
    fn host_read(&mut self, fd: u32, buf: &mut [u8]) -> Result<u32>;

    /// For writes, just pass through, record rlen only
    fn host_write(&mut self, fd: u32, buf: &[u8]) -> Result<u32>;
}

pub struct Risc0Machine<'a> {
    ctx: &'a mut dyn Risc0Context,
}

impl<'a> Risc0Machine<'a> {
    pub fn step(emu: &mut Emulator, ctx: &'a mut dyn Risc0Context) -> Result<()> {
        emu.step(&mut Risc0Machine { ctx })
    }

    pub fn suspend(ctx: &'a mut dyn Risc0Context) -> Result<()> {
        let mut this = Risc0Machine { ctx };
        this.store_memory(SUSPEND_PC_ADDR.waddr(), this.ctx.get_pc().0)?;
        this.store_memory(SUSPEND_MODE_ADDR.waddr(), this.ctx.get_machine_mode())?;
        this.ctx.suspend()
    }

    pub fn resume(ctx: &'a mut dyn Risc0Context) -> Result<()> {
        let mut this = Risc0Machine { ctx };
        let pc = ByteAddr(this.load_memory(SUSPEND_PC_ADDR.waddr())?);
        let machine_mode = this.load_memory(SUSPEND_MODE_ADDR.waddr())?;
        // tracing::debug!("resume(entry: {pc:?}, mode: {machine_mode})");
        this.ctx.set_pc(pc);
        this.ctx.set_machine_mode(machine_mode);
        this.ctx.resume()
    }

    fn is_machine_mode(&self) -> bool {
        self.ctx.get_machine_mode() != 0
    }

    fn next_pc(&mut self) {
        self.ctx.set_pc(self.ctx.get_pc() + WORD_SIZE);
    }

    fn machine_ecall(&mut self) -> Result<bool> {
        match self.load_register(REG_A7)? {
            HOST_ECALL_TERMINATE => self.ecall_terminate(),
            HOST_ECALL_READ => self.ecall_read(),
            HOST_ECALL_WRITE => self.ecall_write(),
            HOST_ECALL_POSEIDON2 => self.ecall_poseidon2(),
            _ => unimplemented!(),
        }
    }

    fn user_ecall(&mut self) -> Result<bool> {
        let dispatch_idx = self.load_register(REG_A7)?;
        if dispatch_idx >= SYSCALL_MAX {
            return self.trap(TrapCause::EnvironmentCallFromUserMode);
        }

        let dispatch_addr = ByteAddr(self.load_memory(ECALL_DISPATCH_ADDR.waddr() + dispatch_idx)?);
        if dispatch_addr.is_aligned() || dispatch_addr < KERNEL_START_ADDR {
            return self.trap(TrapCause::EnvironmentCallFromUserMode);
        }

        self.enter_trap(dispatch_addr)?;
        Ok(true)
    }

    fn ecall_terminate(&mut self) -> Result<bool> {
        self.ctx
            .on_ecall_cycle(CycleState::MachineEcall, CycleState::Terminate, 0, 0, 0)?;
        let a0 = self.load_memory(USER_REGS_ADDR.waddr() + REG_A0)?;
        let a1 = self.load_memory(USER_REGS_ADDR.waddr() + REG_A1)?;
        self.ctx.on_terminate(a0, a1);
        self.ctx
            .on_ecall_cycle(CycleState::Terminate, CycleState::Suspend, 0, 0, 0)?;
        Ok(false)
    }

    fn ecall_read(&mut self) -> Result<bool> {
        self.ctx
            .on_ecall_cycle(CycleState::MachineEcall, CycleState::HostReadSetup, 0, 0, 0)?;
        let mut cur_state = CycleState::HostReadSetup;
        let fd = self.load_register(REG_A0)?;
        let mut ptr = ByteAddr(self.load_register(REG_A1)?);
        let len = self.load_register(REG_A2)?;
        if ptr + len < ptr {
            bail!("Invalid length in host read: {len}");
        }
        if len > MAX_IO_BYTES {
            bail!("Invalid length (too big) in host read: {len}");
        }
        let mut bytes = vec![0u8; len as usize];
        let mut rlen = self.ctx.host_read(fd, &mut bytes)?;
        self.store_register(REG_A0, rlen)?;
        if rlen == 0 {
            self.next_pc();
        }

        fn next_io_state(ptr: ByteAddr, rlen: u32) -> CycleState {
            if rlen == 0 {
                return CycleState::Decode;
            }
            if !ptr.is_aligned() || rlen < WORD_SIZE as u32 {
                return CycleState::HostReadBytes;
            }
            CycleState::HostReadWords
        }

        let next_state = next_io_state(ptr, rlen);
        self.ctx
            .on_ecall_cycle(cur_state, next_state, ptr.waddr().0, ptr.subaddr(), rlen)?;
        cur_state = next_state;

        let mut i = 0;

        while rlen > 0 && !ptr.is_aligned() {
            self.store_u8(ptr, bytes[i])?;
            ptr += 1u32;
            i += 1;
            rlen -= 1;
        }

        while rlen >= MAX_IO_WORDS {
            let words = min(rlen / MAX_IO_WORDS, MAX_IO_WORDS);
            for j in 0..MAX_IO_WORDS {
                if j < words {
                    let word = u32::from_le_bytes(bytes[i..i + WORD_SIZE].try_into()?);
                    self.store_memory(ptr.waddr(), word)?;
                } else {
                    self.store_memory(SAFE_WRITE_ADDR.waddr(), 0)?;
                }
                ptr += words;
                i += words as usize;
                rlen -= words;
            }

            if rlen == 0 {
                self.next_pc();
            }

            let next_state = next_io_state(ptr, rlen);
            self.ctx
                .on_ecall_cycle(cur_state, next_state, ptr.waddr().0, ptr.subaddr(), rlen)?;
            cur_state = next_state;
        }

        while rlen > 0 && !ptr.is_aligned() {
            self.store_u8(ptr, bytes[i])?;
            ptr += 1u32;
            i += 1;
            rlen -= 1;
        }

        // Ok(true)
        Ok(false)
    }

    fn ecall_write(&mut self) -> Result<bool> {
        self.ctx
            .on_ecall_cycle(CycleState::MachineEcall, CycleState::HostWrite, 0, 0, 0)?;
        let fd = self.load_register(REG_A0)?;
        let ptr = ByteAddr(self.load_register(REG_A1)?);
        let len = self.load_register(REG_A2)?;
        if ptr + len < ptr {
            bail!("Invalid length in host write: {len}");
        }
        if len > MAX_IO_BYTES {
            bail!("Invalid length (too big) in host write: {len}");
        }
        let bytes = self.peek(ptr, len as usize)?;
        let rlen = self.ctx.host_write(fd, &bytes)?;
        self.store_register(REG_A0, rlen)?;
        self.next_pc();
        self.ctx
            .on_ecall_cycle(CycleState::HostWrite, CycleState::Decode, 0, 0, 0)?;
        // Ok(true)
        Ok(false)
    }

    fn ecall_poseidon2(&mut self) -> Result<bool> {
        self.next_pc();
        self.ctx
            .on_ecall_cycle(CycleState::MachineEcall, CycleState::PoseidonEntry, 0, 0, 0)?;
        // Ok(true)
        Ok(false)
    }

    fn enter_trap(&mut self, dispatch_addr: ByteAddr) -> Result<()> {
        if self.is_machine_mode() {
            bail!("Illegal trap in machine mode");
        }
        let pc = self.ctx.get_pc();
        self.store_memory(MEPC_ADDR.waddr(), pc.0)?;
        self.ctx.set_pc(dispatch_addr);
        self.ctx.set_machine_mode(1);
        Ok(())
    }

    fn peek(&mut self, ptr: ByteAddr, len: usize) -> Result<Vec<u8>> {
        let mut bytes = vec![0u8; len];
        for (i, byte) in bytes.iter_mut().enumerate().take(len) {
            *byte = self.peek_u8(ptr + i)?;
        }
        Ok(bytes)
    }

    fn peek_u8(&mut self, ptr: ByteAddr) -> Result<u8> {
        let word = self.ctx.peek_u32(ptr.waddr())?;
        let bytes = word.to_le_bytes();
        let offset = ptr.subaddr() as usize;
        Ok(bytes[offset])
    }

    fn store_u8(&mut self, addr: ByteAddr, byte: u8) -> Result<()> {
        let byte_offset = addr.subaddr() as usize;
        let word = self.load_memory(addr.waddr())?;
        let mut bytes = word.to_le_bytes();
        bytes[byte_offset] = byte;
        let word = u32::from_le_bytes(bytes);
        self.store_memory(addr.waddr(), word)
    }
}

impl<'a> EmuContext for Risc0Machine<'a> {
    fn ecall(&mut self) -> Result<bool> {
        if self.is_machine_mode() {
            self.machine_ecall()
        } else {
            self.user_ecall()
        }
    }

    fn mret(&mut self) -> Result<bool> {
        if !self.is_machine_mode() {
            bail!("Illegal mret in user mode");
        }
        let dispatch_addr = ByteAddr(self.load_memory(MEPC_ADDR.waddr())?);
        self.ctx.set_pc(dispatch_addr + WORD_SIZE);
        self.ctx.set_machine_mode(0);
        Ok(true)
    }

    fn trap(&mut self, cause: TrapCause) -> Result<bool> {
        self.ctx.trap_rewind();
        let dispatch_addr =
            ByteAddr(self.load_memory(TRAP_DISPATCH_ADDR.waddr() + cause.as_u32())?);
        if !dispatch_addr.is_aligned() || !is_kernel_memory(dispatch_addr) {
            bail!("Invalid trap address: {dispatch_addr:?}, cause: {cause:?}");
        }
        self.enter_trap(dispatch_addr)?;
        self.ctx.trap(cause);
        Ok(false)
    }

    fn on_insn_decoded(&mut self, insn: &Instruction, decoded: &DecodedInstruction) -> Result<()> {
        self.ctx.on_insn_start(insn, decoded)
    }

    fn on_normal_end(&mut self, insn: &Instruction, decoded: &DecodedInstruction) -> Result<()> {
        self.ctx.on_insn_end(insn, decoded)
    }

    fn get_pc(&self) -> ByteAddr {
        self.ctx.get_pc()
    }

    fn set_pc(&mut self, addr: ByteAddr) {
        self.ctx.set_pc(addr);
    }

    fn load_register(&mut self, idx: usize) -> Result<u32> {
        // tracing::trace!("load_reg: x{idx}");
        let base = if self.is_machine_mode() {
            MACHINE_REGS_ADDR.waddr()
        } else {
            USER_REGS_ADDR.waddr()
        };
        self.ctx.load_register(base, idx)
    }

    fn store_register(&mut self, idx: usize, word: u32) -> Result<()> {
        // tracing::trace!("store_reg: x{idx} <= {word:#010x}");
        let mut base = if self.is_machine_mode() {
            MACHINE_REGS_ADDR.waddr()
        } else {
            USER_REGS_ADDR.waddr()
        };

        // To avoid the use of a degree in the circuit, all writes to REG_ZERO
        // are shunted to a memory location that is never read from.
        if idx == REG_ZERO {
            base += REG_MAX * 2;
        }

        self.ctx.store_register(base, idx, word)
    }

    fn load_memory(&mut self, addr: WordAddr) -> Result<u32> {
        self.ctx.load_u32(addr)
    }

    fn store_memory(&mut self, addr: WordAddr, word: u32) -> Result<()> {
        self.ctx.store_u32(addr, word)
    }

    fn check_insn_load(&self, addr: ByteAddr) -> bool {
        !(addr < ZERO_PAGE_END_ADDR || (!self.is_machine_mode() && addr >= KERNEL_START_ADDR))
    }

    fn check_data_load(&self, addr: ByteAddr) -> bool {
        self.is_machine_mode() || is_user_memory(addr)
    }

    fn check_data_store(&self, addr: ByteAddr) -> bool {
        self.check_data_load(addr)
    }
}
