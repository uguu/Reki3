pub const ANNOUNCE_INTERVAL: i64 = 60;
pub const DROP_THRESHOLD: i64 = 3*ANNOUNCE_INTERVAL;
pub const REDIS_URL: &'static str = "redis://127.0.0.1:56002/";
pub const LISTEN_ADDR: &'static str = "0.0.0.0:9001";
pub const NUM_THREADS: usize = 10;

#[test]
fn check_announce_interval_test() {
    assert!(ANNOUNCE_INTERVAL > 0);
}

#[test]
fn check_drop_threshold_test() {
    assert!(DROP_THRESHOLD > ANNOUNCE_INTERVAL);
}
