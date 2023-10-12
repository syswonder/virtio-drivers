//! Driver for VirtIO v9pfs devices.

use crate::hal::Hal;
use crate::queue::VirtQueue;
use crate::transport::Transport;
use bitflags::bitflags;
use core::result::Result;
use zerocopy::AsBytes;
use log;

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
        transport.begin_init(|features| {
            let features = Feature::from_bits_truncate(features);
            log::info!("device features: {:?}", features);
            // negotiate these flags only
            let supported_features = Feature::empty();
            (features & supported_features).bits()
        });

        match VirtQueue::new(&mut transport, QUEUE) {
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
        log::debug!("{:?}", request);
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
    struct Feature: u64 {
        const VIRTIO_9P_F_MOUNT_TAG = 1 << 0;
    }
}
