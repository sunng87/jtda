use std::convert::From;

use regex::Regex;

lazy_static! {
    static ref THREAD_STATE_MATCHER: Regex = Regex::new(r"State: (\S+)").unwrap();
    static ref THREAD_DATA_MATCHER: Regex = Regex::new("\"(.+?)\"( daemon)? prio=(\\d+) tid=(\\S+) nid=(\\S+)").unwrap();
}

#[derive(Debug, PartialEq)]
pub enum JThreadState {
    Waiting,
    TimedWaiting,
    Blocked,
    Runnable,
    New,
    Terminated
}

impl<'a> From<&'a str> for JThreadState {
    fn from(line: &str) -> Self {
        match THREAD_STATE_MATCHER.captures(line) {
            Some(caps) => {
                match caps.at(1).unwrap() {
                    "WAITING" => JThreadState::Waiting,
                    "TIMED_WAITING" => JThreadState::TimedWaiting,
                    "BLOCKED" => JThreadState::Blocked,
                    "RUNNABLE" => JThreadState::Runnable,
                    "NEW" => JThreadState::New,
                    "TERMINATED" => JThreadState::Terminated,
                    _ => panic!(format!("Invalid thread state: {}", line))
                }
            },
            None => {
                panic!(format!("Invalid thread state:: {}", line))
            }
        }
    }
}

#[test]
fn test_state_parser() {
    let s = "   java.lang.Thread.State: WAITING (parking)";
    assert_eq!(JThreadState::from(s), JThreadState::Waiting);
}

#[derive(Debug, PartialEq)]
pub struct JThreadInfo<'a> {
    name: &'a str,
    daemon: bool,
    priority: &'a str,
    thread_id: &'a str,
    native_id: &'a str,
    state: Option<JThreadState>,
    stacktrace: Option<&'a str>
}

impl<'a> From<&'a str> for JThreadInfo<'a> {
    fn from(lines: &'a str) -> Self {
        let mut tinfo = JThreadInfo {
            name: "",
            daemon: false,
            priority: "",
            thread_id: "",
            native_id: "",
            state: None,
            stacktrace: None
        };

        let mut ls = lines.trim().split("\n");

        // thread info line
        match ls.next() {
            Some(info_line) => {
                match THREAD_DATA_MATCHER.captures(info_line) {
                    Some(caps) => {
                        let len = caps.len();
                        tinfo.name = caps.at(1).unwrap();
                        tinfo.daemon = caps.at(2).is_some();
                        tinfo.priority = caps.at(len-3).unwrap();
                        tinfo.thread_id = caps.at(len-2).unwrap();
                        tinfo.native_id = caps.at(len-1).unwrap();
                    },
                    None => {
                        panic!(format!("Failed to parse thread info: {}", info_line))
                    }
                }
            },
            None => {
                panic!(format!("Invalid thread lines: {}", lines));
            }
        }

        // thread state
        if let Some(state_line) = ls.next() {
            tinfo.state = Some(JThreadState::from(state_line));
        }

        if let Some(stacktrace_line) = ls.next() {
            let index = lines.find(stacktrace_line).unwrap();
            tinfo.stacktrace = Some(&lines[index..]);
        }

        tinfo
    }
}

#[test]
fn test_thread_info_line() {
    let t = "\"async-dispatch-4\" daemon prio=10 tid=0x00007f49d5b4a800 nid=0x1b2 waiting on condition [0x00007f48cd498000]
   java.lang.Thread.State: WAITING (parking)
	at java.net.PlainSocketImpl.socketAccept(Native Method)
	at java.net.AbstractPlainSocketImpl.accept(AbstractPlainSocketImpl.java:398)
	at java.net.ServerSocket.implAccept(ServerSocket.java:530)
	at java.net.ServerSocket.accept(ServerSocket.java:498)
	at sun.rmi.transport.tcp.TCPTransport$AcceptLoop.executeAcceptLoop(TCPTransport.java:399)
	at sun.rmi.transport.tcp.TCPTransport$AcceptLoop.run(TCPTransport.java:371)
	at java.lang.Thread.run(Thread.java:745)";
    let expected = JThreadInfo {
        name: "async-dispatch-4",
        daemon: true,
        priority: "10",
        thread_id: "0x00007f49d5b4a800",
        native_id: "0x1b2",
        state: Some(JThreadState::Waiting),
        stacktrace: Some("	at java.net.PlainSocketImpl.socketAccept(Native Method)
	at java.net.AbstractPlainSocketImpl.accept(AbstractPlainSocketImpl.java:398)
	at java.net.ServerSocket.implAccept(ServerSocket.java:530)
	at java.net.ServerSocket.accept(ServerSocket.java:498)
	at sun.rmi.transport.tcp.TCPTransport$AcceptLoop.executeAcceptLoop(TCPTransport.java:399)
	at sun.rmi.transport.tcp.TCPTransport$AcceptLoop.run(TCPTransport.java:371)
	at java.lang.Thread.run(Thread.java:745)")
    };
    assert_eq!(JThreadInfo::from(t), expected);

    let t2 = "\"G1 Concurrent Refinement Thread#17\" prio=10 tid=0x00007f49d4045800 nid=0x59 runnable";
    let expected2 = JThreadInfo {
        name: "G1 Concurrent Refinement Thread#17",
        daemon: false,
        priority: "10",
        thread_id: "0x00007f49d4045800",
        native_id: "0x59",
        state: None,
        stacktrace: None
    };
    assert_eq!(JThreadInfo::from(t2), expected2);
}

#[derive(Debug, PartialEq)]
pub struct JThreadDump<'a> {
    timestamp: &'a str,
    jvm_info: &'a str,
    threads: Vec<JThreadInfo<'a>>
}

impl<'a> From<&'a str> for JThreadDump<'a> {
    fn from(lines: &'a str) -> Self {
        let ls = lines.trim().split("\n\n").collect::<Vec<&str>>();

        let (ts, jvm_info) = if let Some(info_line) = ls.get(0) {
            let mut ils = info_line.split("\n");
            (ils.next().unwrap(), ils.next().unwrap())
        } else {
            panic!("Invalid thread format.")
        };

        let threads = ls[1..ls.len()-1].iter().map(|s| JThreadInfo::from(*s)).collect();

        JThreadDump {
            timestamp: ts,
            jvm_info: jvm_info,
            threads: threads
        }
    }
}

#[test]
fn test_parsing_dump() {
    let s = "2015-12-20 12:23:06
Full thread dump Java HotSpot(TM) 64-Bit Server VM (24.80-b11 mixed mode):

\"Attach Listener\" daemon prio=10 tid=0x00007f48d801d800 nid=0x3c21 waiting on condition [0x0000000000000000]
   java.lang.Thread.State: RUNNABLE

JNI global references: 331";
    let t = JThreadDump::from(s);
    assert_eq!(t.timestamp, "2015-12-20 12:23:06");
    assert_eq!(t.jvm_info, "Full thread dump Java HotSpot(TM) 64-Bit Server VM (24.80-b11 mixed mode):");
    assert_eq!(t.threads.len(), 1);
}
