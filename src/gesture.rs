use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use fnv::FnvHashMap;
use std::f64;
use std::time::Duration;
use std::thread;
use crate::unit::mm_to_px;
use crate::input::{DeviceEvent, FingerStatus, ButtonCode, ButtonStatus};
use crate::view::Event;
use crate::device::CURRENT_DEVICE;
use crate::geom::{Point, Vec2, Dir, DiagDir, Axis, nearest_segment_point, elbow};

pub const JITTER_TOLERANCE_MM: f32 = 6.0;
pub const HOLD_DELAY_SHORT: Duration = Duration::from_millis(666);
pub const HOLD_DELAY_LONG: Duration = Duration::from_millis(1333);

#[derive(Debug, Copy, Clone)]
pub enum GestureEvent {
    Tap(Point),
    MultiTap([Point; 2]),
    Swipe {
        dir: Dir,
        start: Point,
        end: Point,
    },
    MultiSwipe {
        dir: Dir,
        starts: [Point; 2],
        ends: [Point; 2],
    },
    Arrow {
        dir: Dir,
        start: Point,
        end: Point,
    },
    Corner {
        dir: DiagDir,
        start: Point,
        end: Point,
    },
    Pinch {
        axis: Axis,
        strength: u32,
        starts: [Point; 2],
        ends: [Point; 2],
    },
    Spread {
        axis: Axis,
        strength: u32,
        starts: [Point; 2],
        ends: [Point; 2],
    },
    Rotate {
        center: Point,
        quarter_turns: i8,
        angle: f32,
    },
    Cross(Point),
    HoldFingerShort(Point, i32),
    HoldFingerLong(Point, i32),
    HoldButtonShort(ButtonCode),
    HoldButtonLong(ButtonCode),
}

#[derive(Debug)]
pub struct TouchState {
    time: f64,
    held: bool,
    positions: Vec<Point>,
}

pub fn gesture_events(rx: Receiver<DeviceEvent>) -> Receiver<Event> {
    let (ty, ry) = mpsc::channel();
    thread::spawn(move || parse_gesture_events(&rx, &ty));
    ry
}

pub fn parse_gesture_events(rx: &Receiver<DeviceEvent>, ty: &Sender<Event>) {
    let contacts: Arc<Mutex<FnvHashMap<i32, TouchState>>> = Arc::new(Mutex::new(FnvHashMap::default()));
    let buttons: Arc<Mutex<FnvHashMap<ButtonCode, f64>>> = Arc::new(Mutex::new(FnvHashMap::default()));
    let segments: Arc<Mutex<Vec<Vec<Point>>>> = Arc::new(Mutex::new(Vec::new()));
    let jitter = mm_to_px(JITTER_TOLERANCE_MM, CURRENT_DEVICE.dpi);

    while let Ok(evt) = rx.recv() {
        ty.send(Event::Device(evt)).unwrap();
        match evt {
            DeviceEvent::Finger { status: FingerStatus::Down, position, id, time } => {
                let mut ct = contacts.lock().unwrap();
                ct.insert(id, TouchState { time, held: false, positions: vec![position] });
                let ty = ty.clone();
                let contacts = contacts.clone();
                let segments = segments.clone();
                thread::spawn(move || {
                    let mut held = false;
                    thread::sleep(HOLD_DELAY_SHORT);
                    {
                        let mut ct = contacts.lock().unwrap();
                        let sg = segments.lock().unwrap();
                        if ct.len() > 1 || !sg.is_empty() {
                            return;
                        }
                        if let Some(ts) = ct.get(&id) {
                            let tp = &ts.positions;
                            if (ts.time - time).abs() < f64::EPSILON && (tp[tp.len()-1] - position).length() < jitter
                                                                     && (tp[tp.len()/2] - position).length() < jitter {
                                held = true;
                                ty.send(Event::Gesture(GestureEvent::HoldFingerShort(position, id))).unwrap();
                            }
                        }
                        if held {
                            if let Some(ts) = ct.get_mut(&id) {
                                ts.held = true;
                            }
                        } else {
                            return;
                        }
                    }
                    thread::sleep(HOLD_DELAY_LONG - HOLD_DELAY_SHORT);
                    {
                        let mut ct = contacts.lock().unwrap();
                        let sg = segments.lock().unwrap();
                        if ct.len() > 1 || !sg.is_empty() {
                            return;
                        }
                        if let Some(ts) = ct.get_mut(&id) {
                            let tp = &ts.positions;
                            if (ts.time - time).abs() < f64::EPSILON && (tp[tp.len()-1] - position).length() < jitter
                                                                     && (tp[tp.len()/2] - position).length() < jitter {
                                ty.send(Event::Gesture(GestureEvent::HoldFingerLong(position, id))).unwrap();
                            }
                        }
                    }
                });
            },
            DeviceEvent::Finger { status: FingerStatus::Motion, position, id, .. } => {
                let mut ct = contacts.lock().unwrap();
                if let Some(ref mut ts) = ct.get_mut(&id) {
                    ts.positions.push(position);
                }
            },
            DeviceEvent::Finger { status: FingerStatus::Up, position, id, .. } => {
                let mut ct = contacts.lock().unwrap();
                let mut sg = segments.lock().unwrap();
                if let Some(mut ts) = ct.remove(&id) {
                    if !ts.held {
                        ts.positions.push(position);
                        sg.push(ts.positions);
                    }
                }
                if ct.is_empty() && !sg.is_empty() {
                    let len = sg.len();
                    if len == 1 {
                        ty.send(Event::Gesture(interpret_segment(&sg.pop().unwrap(), jitter))).unwrap();
                    } else if len == 2 {
                        let ge1 = interpret_segment(&sg.pop().unwrap(), jitter);
                        let ge2 = interpret_segment(&sg.pop().unwrap(), jitter);
                        match (ge1, ge2) {
                            (GestureEvent::Tap(c1), GestureEvent::Tap(c2)) => {
                                ty.send(Event::Gesture(GestureEvent::MultiTap([c1, c2]))).unwrap();
                            },
                            (GestureEvent::Swipe { dir: d1, start: s1, end: e1, .. },
                             GestureEvent::Swipe { dir: d2, start: s2, end: e2, .. }) if d1 == d2 => {
                                ty.send(Event::Gesture(GestureEvent::MultiSwipe {
                                    dir: d1,
                                    starts: [s1, s2],
                                    ends: [e1, e2],
                                })).unwrap();
                            },
                            (GestureEvent::Swipe { dir: d1, start: s1, end: e1, .. },
                             GestureEvent::Swipe { dir: d2, start: s2, end: e2, .. }) if d1 == d2.opposite() => {
                                let ds = (s2 - s1).length();
                                let de = (e2 - e1).length();
                                if ds > de {
                                    ty.send(Event::Gesture(GestureEvent::Pinch {
                                        axis: d1.axis(),
                                        starts: [s1, s2],
                                        ends: [e1, e2],
                                        strength: (ds - de) as u32,
                                    })).unwrap();
                                } else {
                                    ty.send(Event::Gesture(GestureEvent::Spread {
                                        axis: d1.axis(),
                                        starts: [s1, s2],
                                        ends: [e1, e2],
                                        strength: (de - ds) as u32,
                                    })).unwrap();
                                }
                            },
                            (GestureEvent::Arrow { dir: Dir::East, start: s1, end: e1 }, GestureEvent::Arrow { dir: Dir::West, start: s2, end: e2 }) |
                            (GestureEvent::Arrow { dir: Dir::West, start: s2, end: e2 }, GestureEvent::Arrow { dir: Dir::East, start: s1, end: e1 }) if s1.x < s2.x => {
                                ty.send(Event::Gesture(GestureEvent::Cross((s1+e1+s2+e2)/4))).unwrap();
                            },
                            (GestureEvent::Tap(c), GestureEvent::Swipe { start: s, end: e, .. }) |
                            (GestureEvent::Swipe { start: s, end: e, .. }, GestureEvent::Tap(c)) |
                            (GestureEvent::Tap(c), GestureEvent::Arrow { start: s, end: e, .. }) |
                            (GestureEvent::Arrow { start: s, end: e, .. }, GestureEvent::Tap(c)) |
                            (GestureEvent::Tap(c), GestureEvent::Corner { start: s, end: e, .. }) |
                            (GestureEvent::Corner { start: s, end: e, .. }, GestureEvent::Tap(c)) => {
                                // Angle are positive in the counter clockwise direction.
                                let angle = ((e - c).angle() - (s - c).angle()).to_degrees();
                                let quarter_turns = (angle / 90.0).round() as i8;
                                ty.send(Event::Gesture(GestureEvent::Rotate {
                                    angle,
                                    quarter_turns,
                                    center: c,
                                })).unwrap();
                            },
                            _ => (),
                        }
                    } else {
                        sg.clear();
                    }
                }
            },
            DeviceEvent::Button { status: ButtonStatus::Pressed, code, time } => {
                let mut bt = buttons.lock().unwrap();
                bt.insert(code, time);
                let ty = ty.clone();
                let buttons = buttons.clone();
                thread::spawn(move || {
                    thread::sleep(HOLD_DELAY_SHORT);
                    {
                        let bt = buttons.lock().unwrap();
                        if let Some(&initial_time) = bt.get(&code) {
                            if (initial_time - time).abs() < f64::EPSILON {
                                ty.send(Event::Gesture(GestureEvent::HoldButtonShort(code))).unwrap();
                            }
                        }
                    }
                    thread::sleep(HOLD_DELAY_LONG - HOLD_DELAY_SHORT);
                    {
                        let bt = buttons.lock().unwrap();
                        if let Some(&initial_time) = bt.get(&code) {
                            if (initial_time - time).abs() < f64::EPSILON {
                                ty.send(Event::Gesture(GestureEvent::HoldButtonLong(code))).unwrap();
                            }
                        }
                    }
                });
            },
            DeviceEvent::Button { status: ButtonStatus::Released, code, .. } => {
                let mut bt = buttons.lock().unwrap();
                bt.remove(&code);
            },
            _ => (),
        }
    }
}

fn interpret_segment(sp: &[Point], jitter: f32) -> GestureEvent {
    let a = sp[0];
    let b = sp[sp.len()-1];
    let ab = b - a;
    let d = ab.length();
    if d < jitter {
        GestureEvent::Tap(a)
    } else {
        let p = sp[elbow(sp)];
        let (n, p) = {
            let p: Vec2 = p.into();
            let (n, _) = nearest_segment_point(p, a.into(), b.into());
            (n, p)
        };
        let np = p - n;
        let ds = np.length();
        if ds > d / 5.0 {
            let g = (np.x as f32 / np.y as f32).abs();
            if g < 0.5 || g > 2.0 {
                GestureEvent::Arrow {
                    dir: np.dir(),
                    start: a,
                    end: b,
                }
            } else {
                GestureEvent::Corner {
                    dir: np.diag_dir(),
                    start: a,
                    end: b,
                }
            }
        } else {
            GestureEvent::Swipe {
                start: a,
                end: b,
                dir: ab.dir(),
            }
        }
    }
}
