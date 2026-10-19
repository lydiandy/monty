//! host await 的 VM 侧
//!
//! 宿主创建的 pending `ExternalFuture`、宿主边界的驱动入口与 `HostAwait` 结果
//! `embed.rs` 的 `drive` / `resolve_future` / `fail_future` 从这里进入

use super::{AwaitResult, FrameExit, VM};
use crate::{
    asyncio::ExternalFuture,
    exception_private::{ExcType, ExcTypeExt, RunResult},
    heap::{HeapData, HeapId},
    value::Value,
};

/// Outcome of [`VM::run_awaitable`] at the embedder's host boundary
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostAwait {
    /// The awaitable finished
    Done,
    /// Every task waits on a host future
    Suspended,
}

impl<'h> VM<'h> {
    /// Allocates a pending `ExternalFuture` for a host await and returns its
    /// call id with the awaitable value, without pushing it
    ///
    /// The host returns the value from the call that asked for it, awaiting it
    /// parks the VM until the host resolves the call
    pub(crate) fn new_host_future(&mut self) -> (u32, Value) {
        let call_id = self.allocate_call_id();
        let future_id = self
            .heap
            .allocate(HeapData::ExternalFuture(Box::new(ExternalFuture::new_pending(
                call_id, None,
            ))));
        self.scheduler.add_pending_external(call_id, future_id, self.heap);
        (call_id.raw(), Value::Ref(future_id))
    }

    /// Runs an awaitable value to completion, or until it waits on the host
    ///
    /// The value is owned and released here
    pub(crate) fn run_awaitable(&mut self, value: Value) -> RunResult<HostAwait> {
        self.push(value);
        match self.exec_get_awaitable()? {
            AwaitResult::ValueReady(value) => {
                value.drop_with(self);
                Ok(HostAwait::Done)
            }
            AwaitResult::FramePushed => self.run_to_host_exit(),
            AwaitResult::Yield(_) => Ok(HostAwait::Suspended),
        }
    }

    /// Runs a coroutine as the root of the main task, for a host-driven
    /// `spawn`
    ///
    /// The coroutine gets a frame of its own rather than one pushed over the
    /// parked placeholder, which no snapshot can serialize
    pub(crate) fn run_coroutine(&mut self, coroutine_id: HeapId) -> RunResult<HostAwait> {
        self.suspended_frames.clear();
        let result = self.init_task_from_coroutine(coroutine_id);
        self.heap.dec_ref(coroutine_id);
        result?;
        self.run_to_host_exit()
    }

    /// Whether a task can run beside the blocked main task
    pub(crate) fn can_spawn_task(&self) -> bool {
        !self.current_frame.is_parked || self.scheduler.main_task().is_some_and(|task| !task.frames.is_empty())
    }

    /// Runs a coroutine as a task beside the blocked main task, for a host
    /// `spawn` while another await is pending
    ///
    /// The coroutine is owned and released here
    pub(crate) fn run_spawned_task(&mut self, coroutine_id: HeapId) -> RunResult<HostAwait> {
        let Some(task_id) = self.scheduler.spawn(self.heap, coroutine_id, None) else {
            self.heap.dec_ref(coroutine_id);
            return Err(ExcType::cannot_reuse_already_awaited_coroutine().into());
        };
        self.heap.dec_ref(coroutine_id);
        self.activate_task(task_id)?;
        self.run_to_host_exit()
    }

    /// Runs the loop until it returns or every task waits on the host
    fn run_to_host_exit(&mut self) -> RunResult<HostAwait> {
        match self.run_external()? {
            FrameExit::Return(value) => {
                value.drop_with(self);
                Ok(HostAwait::Done)
            }
            FrameExit::ResolveFutures(_) => Ok(HostAwait::Suspended),
            exit => Err(self.unsupported_frame_exit("await", exit)),
        }
    }
}
