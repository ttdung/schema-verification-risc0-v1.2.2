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

use risc0_circuit_rv32im::prove::emu::addr::ByteAddr;
use risc0_zkvm_platform::syscall::reg_abi::REG_A3;

use super::{Syscall, SyscallContext, SyscallKind};

#[derive(Clone, Default)]
pub(crate) struct SysKeccak;

impl Syscall for SysKeccak {
    fn syscall(
        &mut self,
        _syscall: &str,
        ctx: &mut dyn SyscallContext,
        to_guest: &mut [u32],
    ) -> anyhow::Result<(u32, u32)> {
        let buf_ptr = ByteAddr(ctx.load_register(REG_A3));
        let from_guest = &ctx.load_region(buf_ptr, 25 * 8)?;
        let mut from_guest: [u64; 25] = bytemuck::cast_slice(from_guest).try_into()?;

        keccak::f1600(&mut from_guest);
        to_guest.clone_from_slice(bytemuck::cast_slice(&from_guest));

        let metric = &mut ctx.syscall_table().metrics.borrow_mut()[SyscallKind::Keccak];
        metric.count += 1;

        Ok((0, 0))
    }
}
