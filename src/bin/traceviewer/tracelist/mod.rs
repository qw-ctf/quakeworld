use quakeworld::trace::TraceEntry;

#[derive(Default)]
pub struct Tracelist<'a> {
    pub index: Vec<usize>,
    pub trace: TraceEntry,
    pub last_selected: usize,
    pub selected_trace: Option<&'a TraceEntry>,
}

impl<'a> Tracelist<'_> {
    pub fn new(trace: TraceEntry) -> Tracelist<'a> {
        Tracelist {
            trace,
            index: vec![0],
            last_selected: 0,
            selected_trace: None,
        }
    }

    // this will get the currently highlighted from the top of the stack
    pub fn trace_get_current_highlighted(&self) -> Option<&TraceEntry> {
        let mut t = &self.trace;
        for i in &self.index {
            if (*i as usize) < t.traces.len() {
                t = &t.traces[*i as usize];
            } else {
                return None;
            }
        }
        return Some(&t);
    }

    // this will get the currently selected one aka the one one back from the top of the stack
    pub fn trace_get_current_selected(&self) -> Option<&TraceEntry> {
        let mut t = &self.trace;
        if self.index.len() == 1 {
            return Some(t);
        }
        let index_len = self.index.len() - 1;
        for (index, value) in self.index.iter().enumerate() {
            if index_len == index {
                break;
            }
            if (*value as usize) < t.traces.len() {
                t = &t.traces[*value as usize];
            } else {
                return None;
            }
        }
        return Some(t);
    }

    // this will get the currently selected one aka the one one back from the top of the stack
    pub fn trace_set_current_selected(&mut self) {
        // let mut t = &self.trace;
        // if self.index.len() == 1 {
        //     self.selected_trace = Some(t);
        // }
        // let index_len = self.index.len() - 1;
        // for (index, value) in self.index.iter().enumerate() {
        //     if index_len == index {
        //         break;
        //     }
        //     if (*value as usize) < t.traces.len() {
        //         t = &t.traces[*value as usize];
        //     }
        // }
        // self.selected_trace = Some(t);
    }
    #[allow(dead_code)]
    pub fn trace_get_index(&self, index: i32) -> Option<&TraceEntry> {
        let mut t = &self.trace;
        let index_fetch = if index < 0 {
            let i = self.index.len() as i32 - index;
            if i < 0 {
                0 as usize
            } else if i >= self.index.len() as i32 {
                let index = self.index.len();
                if index > 0 {
                    index - 1
                } else {
                    index
                }
            } else {
                i as usize
            }
        } else {
            index as usize
        };
        let mut c = 0;
        if index_fetch == 0 {
            return Some(&self.trace);
        }
        for i in &self.index {
            if (*i as usize) < t.traces.len() {
                t = &t.traces[*i as usize];
            };
            if c == index_fetch {
                return Some(&t);
            }
            c += 1;
        }
        return None;
    }

    pub fn trace_enter(&mut self) {
        let t = self.trace_get_current_highlighted();
        match t {
            Some(t) => {
                if t.traces.len() > 0 {
                    self.index.push(0);
                }
            }
            None => {}
        }
    }

    pub fn trace_leave(&mut self) {
        if self.index.len() > 1 {
            self.index.pop();
        }
    }

    pub fn trace_move(&mut self, amount: i32) {
        let t = self.trace_get_current_selected();
        let trace: &TraceEntry;
        if let Some(t) = t {
            trace = &t;
        } else {
            return;
        }
        let current_index = self.index_current() as i32;
        let wanted_index = current_index + amount;
        let tl = trace.traces.len();
        let wanted_index = if wanted_index >= tl as i32 {
            0
        } else if wanted_index < 0 {
            tl - 1
        } else {
            wanted_index as usize
        };

        self.index.pop();
        self.index.push(wanted_index);
    }

    pub fn trace_top(&mut self) {
        self.index.pop();
        self.index.push(0);
    }

    pub fn trace_bottom(&mut self) {
        let t = self.trace_get_current_selected();
        let trace: &TraceEntry;
        if let Some(t) = t {
            trace = &t;
        } else {
            return;
        }
        let tl = trace.traces.len();
        let wanted_index = if tl > 0 { tl - 1 } else { 0 };
        self.index.pop();
        self.index.push(wanted_index);
    }

    pub fn trace_get_with_offset(&self, window: usize) -> (usize, usize) {
        let t = match self.trace_get_current_selected() {
            Some(t) => t,
            None => return (0, window),
        };
        let i = self.index_current();
        let tlen = t.traces.len();
        if window >= tlen {
            return (0, i);
        }
        let mut start = if (i as i32) - (window as i32) / 2 < 0 {
            0
        } else {
            i - window / 2
        };
        let mut offset = i - start;
        if start >= tlen - window - 1 {
            start = tlen - window - 1;
            offset = i - start - 1;
        }
        return (start, offset);
    }

    pub fn index_current(&self) -> usize {
        let mut index = self.index.len();
        if index > 0 {
            index -= 1;
        }
        return self.index[index];
    }

    pub(crate) fn set_last_selected(&mut self, index_offset: usize) {
        self.last_selected = index_offset;
    }

    pub(crate) fn get_last_selected(&self) -> usize {
        return self.last_selected;
    }
}
