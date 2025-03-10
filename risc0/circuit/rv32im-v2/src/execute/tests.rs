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

use risc0_binfmt::ExitCode;
use test_log::test;

use super::{image::MemoryImage2, testutil, DEFAULT_SEGMENT_LIMIT_PO2, MAX_INSN_CYCLES};

// impl Syscall for BasicSyscall {
//     fn syscall(
//         &self,
//         syscall: &str,
//         ctx: &mut dyn SyscallContext,
//         guest_buf: &mut [u32],
//     ) -> Result<(u32, u32)> {
//         self.state.borrow_mut().syscall = syscall.to_string();
//         let buf_ptr = ByteAddr(ctx.peek_register(REG_A4)?);
//         let buf_len = ctx.peek_register(REG_A5)?;
//         self.state.borrow_mut().from_guest = ctx.peek_region(buf_ptr, buf_len)?;
//         let guest_buf_bytes: &mut [u8] = bytemuck::cast_slice_mut(guest_buf);
//         let into_guest = &self.state.borrow().into_guest;
//         guest_buf_bytes[..into_guest.len()].clone_from_slice(into_guest);
//         Ok((0, 0))
//     }
// }

#[test]
fn basic() {
    let program = testutil::basic();
    let expected_cycles = program.image.len();
    let mut image = MemoryImage2::new(program);
    let pre_image_id = *image.image_id();

    println!("image_id: {pre_image_id}");

    let result = testutil::execute(
        image,
        DEFAULT_SEGMENT_LIMIT_PO2,
        MAX_INSN_CYCLES,
        testutil::DEFAULT_SESSION_LIMIT,
        &testutil::NullSyscall,
        None,
    )
    .unwrap();

    let segments = result.segments;
    assert_eq!(segments.len(), 1);
    let segment = segments.first().unwrap();
    assert_eq!(segment.pre_digest, pre_image_id);
    assert_ne!(segment.post_digest, pre_image_id);
    assert!(segment.read_record.is_empty());
    assert!(segment.write_record.is_empty());
    assert_eq!(segment.user_cycles, expected_cycles as u32);
    assert_eq!(segment.exit_code, ExitCode::Halted(0));
}

#[test]
fn system_split() {
    let program = testutil::simple_loop(2000);
    let mut image = MemoryImage2::new(program);
    let pre_image_id = *image.image_id();

    let result = testutil::execute(
        image,
        testutil::MIN_CYCLES_PO2,
        100,
        testutil::DEFAULT_SESSION_LIMIT,
        &testutil::NullSyscall,
        None,
    )
    .unwrap();

    let segments = result.segments;
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].exit_code, ExitCode::SystemSplit);
    assert_eq!(segments[0].pre_digest, pre_image_id);
    assert_ne!(segments[0].post_digest, pre_image_id);
    assert!(segments[0].read_record.is_empty());
    assert!(segments[0].write_record.is_empty());
    assert_eq!(segments[1].exit_code, ExitCode::Halted(0));
    assert_eq!(segments[1].pre_digest, segments[0].post_digest);
}
