use std::time::Instant;

enum ConnState {
    NEGOTIATE_TLS,
    CONNECTED,
    STREAMING,
}

struct ConnTrack {
    id: String,
    start_time: Instant,
    state: ConnState,
    // end_time: Instant,
}
