enum SessionState {
    Connecting,
    Connected,
    Disconnected,
}
trait ClientSession {
    fn id(&self) -> u64;
    fn connect(&self);
    fn pending_messages(&self) -> Vec<String>;
    fn disconnect(&self);
    fn state(&self) -> SessionState;
}

pub struct Session {
    id: u64,
}
