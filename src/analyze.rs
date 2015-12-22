use dump::{JThreadState, JThreadInfo, JThreadDump};

use std::collections::BTreeMap;

pub fn by_state<'a>(dump: &'a JThreadDump) -> BTreeMap<JThreadState, Vec<&'a JThreadInfo<'a>>> {
    let mut m = BTreeMap::new();
    for th in dump.threads.iter() {
        if let Some(ref state) = th.state {
            if !m.contains_key(state) {
                m.insert(*state, Vec::<&'a JThreadInfo<'a>>::new());
            }

            m.get_mut(state).unwrap().push(&th);
        }
    }
    m
}

pub fn by_stacktrace<'a>(dump: &'a JThreadDump) -> BTreeMap<&'a str, Vec<&'a JThreadInfo<'a>>> {
    let mut m = BTreeMap::new();
    for th in dump.threads.iter() {
        if let Some(ref stacktrace) = th.stacktrace {
            if !m.contains_key(stacktrace) {
                m.insert(*stacktrace, Vec::<&'a JThreadInfo<'a>>::new());
            }

            m.get_mut(stacktrace).unwrap().push(&th);
        }
    }
    m
}
