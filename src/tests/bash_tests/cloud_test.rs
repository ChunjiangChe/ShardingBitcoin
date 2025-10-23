// #[cfg(test)]
// use chrono::{DateTime, Local};
#[cfg(test)]
use std::{time::{Duration, SystemTime, UNIX_EPOCH}, ops::Add};

#[test]
fn cloud_test() {
    let d1 = Duration::new(1695004834, 360119220);
    let d2 = Duration::new(1695004894, 449808621);

    let st1 = SystemTime::from(UNIX_EPOCH.add(d1));
    let st2 = SystemTime::from(UNIX_EPOCH.add(d2));

    let duration = st2.duration_since(st1).unwrap();
    println!("Duration: {:?}", duration);

}
