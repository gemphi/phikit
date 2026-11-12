// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of this module. It also holds the device and the host the others, and
//! the tests of the session, render with.


pub(crate) use std::cell::RefCell;
pub(crate) use std::collections::VecDeque;
pub(crate) use std::rc::Rc;
pub(crate) use std::sync::Arc;
pub(crate) use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) use crate::integrator::scheduler::test::{Rig, SMALL, background, rig};
pub(crate) use crate::integrator::*;
pub(crate) use crate::session::{BufferParam, RenderBuffer, Session};
pub(crate) use crate::util::ManualClock;

/// What the device and the host were asked to do, in order.
pub(crate) type Log = Rc<RefCell<Vec<String>>>;

pub(crate) fn note(log: &Log, line: impl Into<String>) {
    log.borrow_mut().push(line.into());
}

/// A device that traces nothing: it writes what it was asked into the log
/// and moves the clock on as if it had worked.
pub(crate) struct Device {
    pub(crate) log: Log,
    pub(crate) clock: Rc<ManualClock>,
    pub(crate) second_per_sample: f64,
    pub(crate) occupancy: f32,
    /// The open pixels each filter call reports; zero once it runs out.
    pub(crate) active: VecDeque<u32>,
    pub(crate) error: bool,
    /// Every slice of the big tile its buffer was reset to.
    pub(crate) allocated: Vec<BufferParam>,
    /// Every slice it was told to render into, at the path tracer's size.
    pub(crate) effective: Vec<BufferParam>,
}

impl Device {
    pub(crate) fn new(log: &Log, clock: &Rc<ManualClock>, second_per_sample: f64) -> Device {
        Device { log: log.clone(), clock: clock.clone(), second_per_sample, occupancy: 1.0, active: VecDeque::new(), error: false, allocated: Vec::new(), effective: Vec::new() }
    }
}

pub(crate) fn device(rig: &Rig, log: &Log, second_per_sample: f64) -> Device {
    Device::new(log, &rig.clock, second_per_sample)
}

impl PathTraceWork for Device {
    fn init_execution(&mut self) {
        note(&self.log, "init");
    }
    fn deinit_execution(&mut self) {
        note(&self.log, "deinit");
    }
    fn has_error(&self) -> bool {
        self.error
    }
    fn reset_render_buffer(&mut self, param: &BufferParam) {
        self.allocated.push(param.clone());
    }
    fn set_effective_buffer_param(&mut self, _big_tile: &BufferParam, slice: &BufferParam, _denoised_big_tile: &BufferParam, _denoised_slice: &BufferParam) {
        note(&self.log, "buffer");
        self.effective.push(slice.clone());
    }
    fn zero_render_buffer(&mut self) {
        note(&self.log, "zero");
    }
    fn copy_to_render_buffer(&mut self, _buffer: &mut RenderBuffer) {
        note(&self.log, "copy out");
    }
    fn copy_from_render_buffer(&mut self, _buffer: &RenderBuffer) {
        note(&self.log, "copy in");
    }
    fn render_samples(&mut self, start_sample: i32, sample_count: i32, _sample_offset: i32) -> RenderStatistic {
        note(&self.log, format!("trace {start_sample}+{sample_count}"));
        self.clock.advance(self.second_per_sample * f64::from(sample_count));
        RenderStatistic { occupancy: self.occupancy }
    }
    fn adaptive_sampling_converge_filter_count_active(&mut self, threshold: f32, reset: bool) -> u32 {
        note(&self.log, format!("filter {threshold} {reset}"));
        self.active.pop_front().unwrap_or(0)
    }
    fn cryptomatte_postprocess(&mut self) {
        note(&self.log, "cryptomatte");
    }
    fn denoise_volume_guiding_buffer(&mut self) {
        note(&self.log, "guiding");
    }
}

/// A host with a display, and a denoiser when `denoiser` says so. Its
/// denoiser takes a second.
pub(crate) struct Host {
    pub(crate) log: Log,
    pub(crate) clock: Rc<ManualClock>,
    pub(crate) cancelled: bool,
    pub(crate) denoiser: Option<bool>,
    pub(crate) display: bool,
    /// Raised as a first round starts, as a cancel from another thread
    /// would be.
    pub(crate) raise: Option<Arc<AtomicBool>>,
}

impl Host {
    pub(crate) fn new(log: &Log, clock: &Rc<ManualClock>) -> Host {
        Host { log: log.clone(), clock: clock.clone(), cancelled: false, denoiser: None, display: true, raise: None }
    }
}

impl TraceHost for Host {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn reset_display(&mut self, _big_tile_param: &BufferParam, reset_rendering: bool) {
        note(&self.log, format!("reset {reset_rendering}"));
    }
    fn read_tile_buffer(&mut self) {
        note(&self.log, "read");
        if let Some(cancel) = &self.raise {
            cancel.store(true, Ordering::Relaxed);
        }
    }
    fn denoise(&mut self, sample_count: i32) -> Option<bool> {
        note(&self.log, format!("denoise {sample_count}"));
        self.clock.advance(1.0);
        self.denoiser
    }
    fn write_tile(&mut self, to_disk: bool) {
        note(
            &self.log,
            match to_disk {
                true => "write disk",
                false => "write host",
            },
        );
    }
    fn update_display(&mut self, show_denoised: bool, _sample_count: i32) -> bool {
        note(
            &self.log,
            match show_denoised {
                true => "display denoised",
                false => "display noisy",
            },
        );
        self.display
    }
    fn update_progress(&mut self, _work: &RenderWork) {
        note(&self.log, "progress");
    }
    fn finish_tile_file(&mut self) {
        note(&self.log, "finish");
    }
}

/// A scheduler, a path tracer over devices reset onto the rig's first
/// tile, and the host they report to.
pub(crate) struct Bench {
    pub(crate) rig: Rig,
    pub(crate) log: Log,
    pub(crate) trace: PathTrace<Device>,
    pub(crate) host: Host,
}

pub(crate) fn bench(device: Vec<Device>, rig: Rig, log: Log) -> Bench {
    let mut host = Host::new(&log, &rig.clock);
    let mut trace = PathTrace::new(device);
    trace.reset(&rig.buffer, &Session::tile_param(&rig.buffer, &rig.tile.current_tile()), &mut host, true);
    log.take();
    Bench { rig, log, trace, host }
}

/// A bench of one device that takes `second_per_sample`.
pub(crate) fn single(rig: Rig, second_per_sample: f64) -> Bench {
    let log = Log::default();
    bench(vec![device(&rig, &log, second_per_sample)], rig, log)
}

/// A bench of two devices.
pub(crate) fn pair(rig: Rig, second_per_sample: [f64; 2]) -> Bench {
    let log = Log::default();
    bench(second_per_sample.iter().map(|second| device(&rig, &log, *second)).collect(), rig, log)
}

impl Bench {
    /// Ask the scheduler for work and render it. The work is returned.
    pub(crate) fn round(&mut self) -> RenderWork {
        let work = self.rig.work();
        self.trace.render(&work, &mut self.rig.scheduler, &self.rig.tile, &mut self.host);
        work
    }

    /// The log since it was last taken.
    pub(crate) fn take(&mut self) -> Vec<String> {
        self.log.take()
    }
}

#[test]
fn the_frame_is_shared_evenly_between_the_devices_at_first() {
    let bench = pair(rig(background(), SMALL, 2), [0.5, 0.5]);
    let weight: Vec<f64> = bench.trace.balance().iter().map(|info| info.weight).collect();
    assert_eq!(weight, [0.5, 0.5]);
    assert_eq!(bench.trace.work().len(), 2);
    assert!(bench.trace.need_schedule_rebalance());
    assert!(!single(rig(background(), SMALL, 2), 0.5).trace.need_schedule_rebalance());
}

#[test]
fn a_cancel_is_the_instant_one_or_the_user_s() {
    let mut bench = single(rig(background(), SMALL, 2), 0.5);
    assert!(!bench.trace.is_cancel_requested(&bench.host));
    bench.host.cancelled = true;
    assert!(bench.trace.is_cancel_requested(&bench.host));
    bench.host.cancelled = false;
    bench.trace.cancel_handle().store(true, Ordering::Relaxed);
    assert!(bench.trace.is_cancel_requested(&bench.host));
    bench.trace.clear_cancel();
    assert!(!bench.trace.is_cancel_requested(&bench.host));
}
