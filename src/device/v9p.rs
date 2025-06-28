//! Driver for VirtIO v9pfs devices.

use crate::hal::Hal;
use crate::queue::VirtQueue;
use crate::transport::Transport;
use bitflags::bitflags;
use core::result::Result;
use log::*;
use zerocopy::IntoBytes;

const UNDEFINED_ERROR: u8 = 0;
const QUEUE: u16 = 0;
const QUEUE_SIZE: usize = 16;
const V9P_MAX_QSIZE: u32 = 4096;
const V9P_MAX_PSIZE: u32 = 4096;

/// A virtio based 9pfs adapter.
///
/// It can transfer request and response between hypervisor and kernels.
pub struct VirtIO9p<H: Hal, T: Transport> {
    transport: T,
    queue: VirtQueue<H, QUEUE_SIZE>,
}

impl<H: Hal, T: Transport> VirtIO9p<H, T> {
    /// create a new virtio-9p device
    pub fn new(mut transport: T) -> Result<Self, u8> {
        let negotiated_features = transport.begin_init(Feature::VIRTIO_9P_F_MOUNT_TAG);

        match VirtQueue::new(
            &mut transport,
            QUEUE,
            negotiated_features.contains(Feature::RING_INDIRECT_DESC),
            negotiated_features.contains(Feature::RING_EVENT_IDX),
        ) {
            Ok(queue) => {
                transport.finish_init();
                Ok(VirtIO9p { transport, queue })
            }
            Err(_) => {
                transport.finish_init();
                Err(UNDEFINED_ERROR)
            }
        }
    }

    /// transmit request and get response in given buffer
    ///
    /// if Ok, it will return Ok with the length of response
    /// or it will return Err(0)
    pub fn request(&mut self, request: &[u8], response: &mut [u8]) -> Result<u32, u8> {
        trace!("{:?}", request);
        let enqueue_try = self.queue.add_notify_wait_pop(
            &[&request.as_bytes()],
            &mut [response.as_bytes_mut()],
            &mut self.transport,
        );
        match enqueue_try {
            Ok(length) => Ok(length),
            Err(_) => {
                log::error!("virtio-9p request fail!");
                Err(UNDEFINED_ERROR)
            }
        }
    }
}

bitflags! {
    #[derive(Debug)]
    struct Feature: u64 {
        const VIRTIO_9P_F_MOUNT_TAG = 1 << 0;
        const RING_INDIRECT_DESC    = 1 << 28;
        const RING_EVENT_IDX        = 1 << 29;
    }
}
