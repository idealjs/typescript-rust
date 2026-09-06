#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::diagnostics::Message;
use crate::lsp::lsproto;

struct ProgressEvent {
    message: Message,
    text: String,
    finish: bool,
}

pub trait ProgressReporter: Send + Sync {
    fn is_done(&self) -> bool;
    fn localize(&self, msg: &Message, args: &[String]) -> String;
    fn create_work_done_progress(&self, token: &str);
    fn send_progress(&self, token: &str, value: lsproto::WorkDoneProgressBeginOrReportOrEnd);
}

pub struct ProjectLoadingProgress {
    reporter: Arc<dyn ProgressReporter>,
    tx: mpsc::Sender<ProgressEvent>,
    _handle: Option<thread::JoinHandle<()>>,
}

impl ProjectLoadingProgress {
    pub fn new(reporter: Arc<dyn ProgressReporter>, delay: Duration) -> Arc<Self> {
        let (tx, rx) = mpsc::channel::<ProgressEvent>();
        let reporter_clone = Arc::clone(&reporter);
        let handle = thread::spawn(move || {
            Self::run(rx, reporter_clone, delay);
        });
        Arc::new(ProjectLoadingProgress {
            reporter,
            tx,
            _handle: Some(handle),
        })
    }

    pub fn start(&self, message: Message, args: Vec<String>) {
        let text = self.reporter.localize(&message, &args);
        let _ = self.tx.send(ProgressEvent {
            message,
            text,
            finish: false,
        });
    }

    pub fn finish(&self, message: Message, args: Vec<String>) {
        let text = self.reporter.localize(&message, &args);
        let _ = self.tx.send(ProgressEvent {
            message,
            text,
            finish: true,
        });
    }

    fn run(
        rx: mpsc::Receiver<ProgressEvent>,
        reporter: Arc<dyn ProgressReporter>,
        delay: Duration,
    ) {
        let mut loading: HashMap<String, i32> = HashMap::new();
        let mut token = String::new();
        let mut token_id = 0i32;
        let mut begun = false;
        let mut delay_fired = delay.is_zero();

        for ev in &rx {
            if reporter.is_done() {
                break;
            }

            if !ev.finish {
                let count = loading.entry(ev.text.clone()).or_insert(0);
                *count += 1;
                if token.is_empty() {
                    token_id += 1;
                    token = format!("tsgo-loading-{}", token_id);
                    begun = false;
                    if delay.is_zero() {
                        delay_fired = true;
                        reporter.create_work_done_progress(&token);
                    } else {
                        delay_fired = false;

                    }
                }
                if delay_fired {
                    begun = Self::begin_or_report(&reporter, &token, &ev.text, begun);
                }
            } else {
                let count = loading.entry(ev.text.clone()).or_insert(0);
                if *count <= 1 {
                    loading.remove(&ev.text);
                } else {
                    *count -= 1;
                }
                if token.is_empty() {
                    continue;
                }
                if loading.is_empty() {
                    if begun {
                        reporter.send_progress(
                            &token,
                            lsproto::WorkDoneProgressBeginOrReportOrEnd {
                                end: Some(lsproto::WorkDoneProgressEnd::default()),
                                ..Default::default()
                            },
                        );
                    }
                    token.clear();
                } else if delay_fired {
                    if let Some(first) = loading.keys().next() {
                        reporter.send_progress(
                            &token,
                            lsproto::WorkDoneProgressBeginOrReportOrEnd {
                                report: Some(lsproto::WorkDoneProgressReport {
                                    message: Some(first.clone()),
                                }),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }
    }

    fn begin_or_report(
        reporter: &Arc<dyn ProgressReporter>,
        token: &str,
        text: &str,
        begun: bool,
    ) -> bool {
        if !begun {
            let title = "Loading".to_string();
            reporter.send_progress(
                token,
                lsproto::WorkDoneProgressBeginOrReportOrEnd {
                    begin: Some(lsproto::WorkDoneProgressBegin {
                        title,
                        message: Some(text.to_string()),
                    }),
                    ..Default::default()
                },
            );
        } else {
            reporter.send_progress(
                token,
                lsproto::WorkDoneProgressBeginOrReportOrEnd {
                    report: Some(lsproto::WorkDoneProgressReport {
                        message: Some(text.to_string()),
                    }),
                    ..Default::default()
                },
            );
        }
        true
    }
}
