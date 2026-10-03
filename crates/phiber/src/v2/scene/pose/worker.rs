//! The poser: a thread that poses the frames the UI asks for — the shown
//! one first, then, while playing, the frames the playhead reaches next —
//! and a shelf the UI takes them from. Of what is queued it takes only the
//! newest ask, as the tracer's thread does, and between frames it looks
//! again, so a scrub or a jump moves it straight to the frame wanted now.
//!
//! The frame it finished last is kept whether or not it is still wanted. A
//! playhead that outruns the poser — a debug build, or a late tick stepping
//! several frames — never wants the frame just posed by the time it is
//! done, and the pass drew only a frame it asked for, so the character stood
//! still for as long as playback ran. The pass now draws the newest pose
//! there is while the one it wants is on its way, as Blender drops frames.

use super::BPosed;
use crate::phiphx::scene::SceneModel;
use crate::v2::part::time::step::play_range;
use crate::v2::state::val::{V_REVERSE, V_TRUE};
use crate::v2::state::{BState, CtxKey};
use std::cell::RefCell;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, Weak};

/// How many frames past the playhead the poser works while playing: the
/// next one, and one more for a tick that comes late and steps two.
const AHEAD: usize = 2;

thread_local! {
    /// The UI thread's poser, started the first time a rig moves.
    static POSER: RefCell<Option<BPoser>> = const { RefCell::new(None) };
}

/// What the UI wants posed: the shown frame, then the frames after it.
struct BPoseJob {
    model: Arc<SceneModel>,
    want: Vec<f32>,
}

/// The frames posed for one model and still wanted.
#[derive(Default)]
struct BPoseShelf {
    model: Weak<SceneModel>,
    list: Vec<BPosed>,
    /// The frame posed last, wanted or not.
    latest: Option<BPosed>,
}

/// The UI's side of the poser.
pub struct BPoser {
    tx: Sender<BPoseJob>,
    shelf: Arc<Mutex<BPoseShelf>>,
    /// The last ask sent, so a pass asking the same queues nothing.
    asked: Option<(Weak<SceneModel>, Vec<f32>)>,
    /// What the viewport drew last, drawn again until the wanted frame lands.
    last: Option<(Weak<SceneModel>, BPosed)>,
    /// The viewport draws another frame than the one wanted.
    waiting: bool,
}

impl BPoser {
    fn start() -> Self {
        let (tx, rx) = channel();
        let shelf = Arc::new(Mutex::new(BPoseShelf::default()));
        let theirs = Arc::clone(&shelf);
        let _ = std::thread::Builder::new().name("phiber-pose".into()).spawn(move || BPoseShelf::run(&theirs, &rx));
        Self { tx, shelf, asked: None, last: None, waiting: false }
    }

    /// The rigged meshes to draw at the shown frame: posed there when the
    /// poser has it, else the last drawn (see the module docs).
    pub fn shown(model: &Arc<SceneModel>, s: &BState) -> BPosed {
        POSER.with(|cell| {
            let mut poser = cell.borrow_mut();
            match BPosed::moving(model, super::motion::shown_frame(s)) {
                Some(frame) => poser.get_or_insert_with(Self::start).take(model, Self::want(frame, s)),
                None => {
                    if let Some(poser) = poser.as_mut() {
                        poser.waiting = false;
                    }
                    BPosed::default()
                }
            }
        })
    }

    /// The viewport draws another frame than the one it wants: the window
    /// draws again until that lands.
    pub fn waiting() -> bool {
        POSER.with(|cell| cell.borrow().as_ref().is_some_and(|p| p.waiting))
    }

    /// Whether `frame` of `model` is posed and waiting on the shelf.
    pub fn ready(model: &Arc<SceneModel>, frame: f32) -> bool {
        POSER.with(|cell| cell.borrow().as_ref().is_some_and(|p| p.shelf.lock().is_ok_and(|s| s.find(model, frame).is_some())))
    }

    fn take(&mut self, model: &Arc<SceneModel>, want: Vec<f32>) -> BPosed {
        let same = |w: &Weak<SceneModel>| std::ptr::eq(w.as_ptr(), Arc::as_ptr(model));
        let frame = want[0];
        let (ready, latest) = self.shelf.lock().ok().map_or((None, None), |s| (s.find(model, frame), s.latest(model)));
        if !self.asked.as_ref().is_some_and(|(m, w)| same(m) && *w == want) {
            if self.tx.send(BPoseJob { model: Arc::clone(model), want: want.clone() }).is_err() {
                // The thread is gone: pose here rather than wait forever.
                self.waiting = false;
                return BPosed::at(model, frame);
            }
            self.asked = Some((Arc::downgrade(model), want));
        }
        self.waiting = ready.is_none();
        match ready.or(latest) {
            Some(posed) => {
                self.last = Some((Arc::downgrade(model), posed.clone()));
                posed
            }
            None => self.last.as_ref().filter(|(m, _)| same(m)).map(|(_, p)| p.clone()).unwrap_or_default(),
        }
    }

    /// The shown frame, then — while playing — the frames the playhead
    /// reaches next, wrapping inside the range it plays.
    fn want(frame: f32, s: &BState) -> Vec<f32> {
        let step = match s.get(CtxKey::Playing) {
            Some(V_TRUE) => 1.0,
            Some(V_REVERSE) => -1.0,
            _ => return vec![frame],
        };
        let (start, end) = play_range(s);
        let mut next = frame;
        let mut want = vec![frame];
        for _ in 0..AHEAD {
            next += step;
            next = if next > end as f32 { start as f32 } else if next < start as f32 { end as f32 } else { next };
            want.push(next);
        }
        want
    }
}

impl BPoseShelf {
    /// The thread: the newest ask, the first frame it wants that is not on
    /// the shelf posed and shelved, and again, until the ask is met.
    fn run(shelf: &Mutex<Self>, rx: &Receiver<BPoseJob>) {
        while let Ok(first) = rx.recv() {
            let mut job = first;
            loop {
                job = rx.try_iter().last().unwrap_or(job);
                let Some(frame) = shelf.lock().ok().and_then(|mut s| s.next(&job)) else { break };
                let posed = BPosed::at(&job.model, frame);
                if let Ok(mut s) = shelf.lock() {
                    s.put(&job.model, posed);
                }
            }
        }
    }

    fn holds(&self, model: &Arc<SceneModel>) -> bool {
        std::ptr::eq(self.model.as_ptr(), Arc::as_ptr(model))
    }

    /// The frame posed last for `model`.
    fn latest(&self, model: &Arc<SceneModel>) -> Option<BPosed> {
        self.holds(model).then(|| self.latest.clone()).flatten()
    }

    fn find(&self, model: &Arc<SceneModel>, frame: f32) -> Option<BPosed> {
        self.holds(model).then(|| self.list.iter().find(|p| p.frame == Some(frame)).cloned()).flatten()
    }

    /// The first frame `job` wants that is not posed; what it no longer
    /// wants leaves the shelf.
    fn next(&mut self, job: &BPoseJob) -> Option<f32> {
        if !self.holds(&job.model) {
            *self = Self { model: Arc::downgrade(&job.model), list: Vec::new(), latest: None };
        }
        self.list.retain(|p| p.frame.is_some_and(|f| job.want.contains(&f)));
        job.want.iter().copied().find(|f| !self.list.iter().any(|p| p.frame == Some(*f)))
    }

    fn put(&mut self, model: &Arc<SceneModel>, posed: BPosed) {
        if self.holds(model) {
            self.latest = Some(posed.clone());
            self.list.push(posed);
        }
    }
}
