extern crate time;

use std::fmt;
use time::precise_time_ns;
use std::io::fs::File;
use std::mem;
use std::rt;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;


#[derive(Clone)]
struct TimeDB {
    times: Arc<Mutex<HashMap<&'static str, InvocationTracking>>>,
}

fn time_db() -> TimeDB {
    use std::sync::{Once, ONCE_INIT};
    static mut TIMEDB : *const TimeDB = 0 as *const TimeDB;
    static mut ONCE: Once = ONCE_INIT;
    unsafe {
        ONCE.call_once(|| {
            let timedb = TimeDB {
                times: Arc::new(Mutex::new(HashMap::new())),
            };
            TIMEDB = mem::transmute(box timedb);

            // Make sure to free it at exit
            rt::at_exit(|| {
                mem::transmute::<_, Box<TimeDB>>(TIMEDB);
                TIMEDB = 0 as *const _;
            });
        });
        (*TIMEDB).clone()
    }
}

#[derive(Copy)]
struct InvocationTracking {
    count: u64,
    total_time: u64,
}

impl fmt::Show for InvocationTracking {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let avg_time = match self.count == 0 {
            true => 0.0f64,
            false => self.total_time as f64 / self.count as f64,
        };
        write!(f, "{} | {} ns/call", self.count, avg_time)
    }
}


impl InvocationTracking {
    fn new(t: &TimeIt) -> InvocationTracking {
        let mut i = InvocationTracking {
            count: 0,
            total_time: 0,
        };
        i.add_time(t);
        return i;
    }
    fn add_time(&mut self, t: &TimeIt) {
        let dur = precise_time_ns() - t.start_time;
        self.count += 1;
        self.total_time += dur;
    }
}


impl TimeDB {
    fn add_time(&mut self, time: &TimeIt) {
        let ref mut m = self.times.lock().unwrap();
        if !m.contains_key(time.name) {
            m.insert(time.name, InvocationTracking::new(time));
        } else {
            m.get_mut(time.name).unwrap().add_time(time);
        }
    }
}

impl fmt::Show for TimeDB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ref m = self.times.lock().unwrap();
        for (name, res) in m.iter() {
            try!(writeln!(f, "{} => {}", name, res));
        }
        write!(f, "")
    }
}


pub struct TimeIt {
    start_time: u64,
    name: &'static str,
}

impl TimeIt {
    pub fn new(name: &'static str) -> TimeIt {
        TimeIt {
            start_time: precise_time_ns(),
            name: name,
        }
    }
}

impl Drop for TimeIt {
    fn drop(&mut self) {
        time_db().add_time(self);
    }
}

pub struct TimeFileSave {
    file_name: &'static str,
}

impl Drop for TimeFileSave {
    fn drop(&mut self) {
        match File::create(&Path::new(self.file_name)) {
            Err(_) => {}
            Ok(mut f) => {
                let m1 : TimeDB = time_db();
                let _ = write!(&mut f, "{}", m1);
            }
        };
    }
}

impl TimeFileSave {
    pub fn new(file_name: &'static str) -> TimeFileSave {
        TimeFileSave {
            file_name: file_name,
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate test;
    use TimeIt;
    use time_db;
    use TimeFileSave;
    
    #[test]
    #[ignore]
    fn test_times() {
        {
            time_3();
        }
        {
            TimeFileSave::new("test_result.txt");
        }
        let td = time_db();
        let m = td.times.lock().unwrap();
        assert_eq!(m.get("time_1").unwrap().count, 2);
        assert_eq!(m.get("time_3").unwrap().count, 1);
    }

    fn time_1() -> u64{
        let _ = TimeIt::new("time_1");
        let a = 3u64;
        return a * 3;
    }

    fn time_3() {
        TimeIt::new("time_3");
        time_1();
        time_1();
    }
}
